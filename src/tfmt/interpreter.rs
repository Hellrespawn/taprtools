use super::ast::*;
use super::function::handle_function;
use super::token::{
    Token, TokenType, DIRECTORY_SEPARATORS, FORBIDDEN_GRAPHEMES,
};
use super::visitor::Visitor;
use crate::error::InterpreterError;
use crate::file::audiofile::AudioFile;

type Result<T> = std::result::Result<T, InterpreterError>;

/// Interprets an AST based on tags from and [AudioFile].
pub struct Interpreter {
    song: Box<dyn AudioFile>,
}

impl Interpreter {
    /// Constructor
    pub fn new(song: Box<dyn AudioFile>) -> Self {
        Interpreter { song }
    }

    /// Public function for interpreter.
    pub fn interpret(&mut self, program: &Program) -> Result<String> {
        program.accept(self)
    }

    fn strip_leading_zeroes(number: &str) -> &str {
        let mut out = number;

        while out.starts_with('0') {
            out = &out[1..];
        }

        out
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

        string += &block
            .expressions
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
        Ok(driveletter.token.get_value().to_string())
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
            TokenType::Hyphen => (-o.parse::<i64>()?).to_string(),
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
            .map(|e| e.join(""))
    }

    fn visit_function(
        &mut self,
        start_token: &Token,
        arguments: &[Expression],
    ) -> Result<String> {
        let name = start_token.get_value();

        let arguments: Vec<String> = arguments
            .iter()
            .map(|a| a.accept(self))
            .collect::<Result<Vec<String>>>()?;

        Ok(handle_function(name, &arguments)?)
    }

    fn visit_integer(&mut self, integer: &Token) -> Result<String> {
        Ok(integer.get_value().to_string())
    }

    fn visit_string(&mut self, string: &Token) -> Result<String> {
        Ok(string.get_value().to_string())
    }

    fn visit_substitution(&mut self, substitution: &Token) -> Result<String> {
        Ok(substitution.get_value().to_string())
    }

    fn visit_tag(&mut self, token: &Token) -> Result<String> {
        let tag_name = token.get_value();

        let mut tag = match tag_name {
            // FIXME complete this.
            "album" => self.song.album(),
            "albumartist" | "album_artist" => self.song.album_artist(),
            "albumsort" | "album_sort" => self.song.albumsort(),
            "artist" => self.song.artist(),
            "duration" | "length" => self.song.duration(),
            "disc" | "disk" | "discnumber" | "disknumber" | "disc_number"
            | "disk_number" => {
                self.song.disc_number().map(Self::strip_leading_zeroes)
            }
            "genre" => self.song.genre(),
            "title" | "name" => self.song.title(),
            "track" | "tracknumber" | "track_number" => {
                self.song.track_number().map(Self::strip_leading_zeroes)
            }
            "year" | "date" => self.song.year(),
            _ => None,
        }
        .unwrap_or("")
        .to_string();

        // TODO? Use map or something?
        // FIXME Add strict mode?
        for grapheme in FORBIDDEN_GRAPHEMES {
            tag = tag.replace(grapheme, "");
            // if tag.contains(grapheme) {
            //     return Err(InterpreterError::TagForbidden(
            //         grapheme.to_string(),
            //     ));
            // }
        }

        for grapheme in DIRECTORY_SEPARATORS {
            tag = tag.replace(grapheme, "");
            // if tag.contains(grapheme) {
            //     return Err(InterpreterError::TagDirSep(grapheme.to_string()));
            // }
        }

        Ok(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::mp3::MP3;
    use crate::tfmt::token::Token;
    use anyhow::Result;
    use std::path::PathBuf;

    fn get_song() -> Result<Box<dyn AudioFile>> {
        Ok(MP3::read_from_path(&PathBuf::from(
            "testdata/music/Under Siege - Amon Amarth.mp3",
        ))?)
    }

    /// Test handling of leading zeroes.
    #[test]
    fn test_visit_tag() -> Result<()> {
        let mut intp = Interpreter { song: get_song()? };

        let token = Token::new_type_from_string(
            1,
            1,
            "STRING",
            Some("tracknumber".to_string()),
        )?;

        assert_eq!(intp.visit_tag(&token)?, "5");

        Ok(())
    }
}
