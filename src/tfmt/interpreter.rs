use super::ast::*;
use super::token::{Token, TokenType};
use super::visitor::Visitor;
use crate::error::InterpreterError;
use crate::file::audiofile::AudioFile;

type Result<T> = std::result::Result<T, InterpreterError>;

pub struct Interpreter {
    song: Box<dyn AudioFile>,
}

impl Interpreter {
    pub fn interpret(
        program: &Program,
        song: Box<dyn AudioFile>,
    ) -> Result<String> {
        let mut intp = Self { song };

        program.accept(&mut intp)
    }
}

impl Visitor<Result<String>> for Interpreter {
    fn visit_program(&mut self, program: &Program) -> Result<String> {
        program.block.accept(self)
    }
    fn visit_parameters(&mut self, _: &Parameters) -> Result<String> {
        Ok("".to_string())
    }
    fn visit_parameter(&mut self, _: &Parameter) -> Result<String> {
        Ok("".to_string())
    }
    fn visit_block(&mut self, block: &Block) -> Result<String> {
         let mut string = if let Some(drive) = &block.drive {
            drive.accept(self)?
        } else {
            "".to_string()
        };

        string += &block.expressions
        .iter()
        .map(|e| e.accept(self))
        .collect::<Result<Vec<String>>>()
        .map(|e| e.join(""))?;

        Ok(string)
    }
    fn visit_driveletter(
        &mut self,
        driveletter: &DriveLetter,
    ) -> Result<String> {
        if let Some(value) = driveletter.token.value.as_ref() {
            Ok(value.to_string())
        } else {
            Err(InterpreterError::TokenWithoutValue(TokenType::Drive))
        }
    }

    fn visit_ternaryop(
        &mut self,
        condition: &Expression,
        true_expr: &Expression,
        false_expr: &Expression,
    ) -> Result<String> {
        if condition.accept(self)?.is_empty() {
            false_expr.accept(self)
        } else {
            true_expr.accept(self)
        }
    }

    fn visit_binaryop(
        &mut self,
        left: &Expression,
        token: &Token,
        right: &Expression,
    ) -> Result<String> {
        let l = left.accept(self)?;
        let r = right.accept(self)?;
        Ok(match token.ttype {
            TokenType::VerticalBar => {
                if !l.is_empty() {
                    l
                } else {
                    r
                }
            }
            TokenType::DoubleVerticalBar => {
                if !l.is_empty() {
                    format!("{}{}", l, r)
                } else {
                    r
                }
            }
            TokenType::Ampersand => {
                if !l.is_empty() {
                    r
                } else {
                    l
                }
            }
            TokenType::DoubleAmpersand => {
                if !l.is_empty() {
                    format!("{}{}", l, r)
                } else {
                    l
                }
            }
            // FIXME number parsing can fail!
            TokenType::Plus => {
                (l.parse::<i64>()? + r.parse::<i64>()?).to_string()
            }
            TokenType::Hyphen => {
                (l.parse::<i64>()? - r.parse::<i64>()?).to_string()
            }
            TokenType::Asterisk => {
                (l.parse::<i64>()? * r.parse::<i64>()?).to_string()
            }
            TokenType::SlashForward => {
                (l.parse::<i64>()? / r.parse::<i64>()?).to_string()
            }
            TokenType::Percent => {
                (l.parse::<i64>()? % r.parse::<i64>()?).to_string()
            }
            TokenType::DoubleAsterisk | TokenType::Caret => {
                l.parse::<i64>()?.pow(r.parse::<u32>()?).to_string()
            }
            other => {
                return Err(InterpreterError::InvalidTokenType(
                    other, "BinaryOp",
                ))
            }
        })
    }

    fn visit_unaryop(
        &mut self,
        token: &Token,
        operand: &Expression,
    ) -> Result<String> {
        let o = operand.accept(self)?;
        Ok(match token.ttype {
            TokenType::Plus => o,
            TokenType::Hyphen => (o.parse::<i64>()? * -1).to_string(),
            other => {
                return Err(InterpreterError::InvalidTokenType(
                    other, "UnaryOp",
                ))
            }
        })
    }

    fn visit_group(&mut self, expressions: &[Expression]) -> Result<String> {
            expressions
            .iter()
            .map(|e| e.accept(self))
            .collect::<Result<Vec<String>>>()
            .and_then(|e| Ok(e.join("")))
    }

    fn visit_function(
        &mut self,
        start_token: &Token,
        arguments: &[Expression],
    ) -> Result<String> {
        Ok("".to_string())
    }

    fn visit_integer(&mut self, integer: &Token) -> Result<String> {
        // TODO? Error if not number?
        if let Some(value) = integer.value.as_ref() {
            Ok(value.to_string())
        } else {
            Err(InterpreterError::TokenWithoutValue(TokenType::Integer))
        }
    }

    fn visit_string(&mut self, string: &Token) -> Result<String> {
        if let Some(value) = string.value.as_ref() {
            Ok(value.to_string())
        } else {
            // FIXME Are empty strings valid string?
            Err(InterpreterError::TokenWithoutValue(TokenType::String))
        }
    }

    fn visit_substitution(&mut self, substitution: &Token) -> Result<String> {
        if let Some(value) = substitution.value.as_ref() {
            // FIXME Get from symbol table
            Ok(value.to_string())
        } else {
            Err(InterpreterError::TokenWithoutValue(TokenType::String))
        }
    }

    fn visit_tag(&mut self, token: &Token) -> Result<String> {
        let tag_name = if let Some(value) = token.value.as_ref() {
            value
        } else {
            return Err(InterpreterError::TokenWithoutValue(TokenType::Integer))
        };

        let out = match tag_name.as_str() {
            //TODO Find a better way to do this.
            "album" => self.song.album().map(String::from),
            "albumartist" | "album_artist" => self.song.album_artist().map(String::from),
            "albumsort" | "album_sort" => self.song.albumsort().map(String::from),
            "artist" => self.song.artist().map(String::from),
            "duration" | "length" => self.song.duration().map(|n| n.to_string()),
            "disc" | "disk" | "discnumber" | "disknumber" | "disc_number" | "disk_number" => self.song.disc_number().map(|n| n.to_string()),
            "genre" => self.song.genre().map(String::from),
            "title" | "name" => self.song.title().map(String::from),
            "track" | "tracknumber" | "track_number" => self.song.track_number().map(|n| n.to_string()),
            "year" | "date" => self.song.year().map(|n| n.to_string()),
            _ => None
        };

        Ok(out.unwrap_or("".to_string()))
    }
}
