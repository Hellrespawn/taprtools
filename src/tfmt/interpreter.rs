use super::ast::*;
use super::function::handle_function;
use super::semantic::{SemanticAnalyzer, SymbolTable};
use super::token::{
    Token, TokenType, DIRECTORY_SEPARATORS, FORBIDDEN_GRAPHEMES,
};
use super::visitor::Visitor;
use crate::error::InterpreterError;
use crate::file::audiofile::AudioFile;
use log::trace;

type Result<T> = std::result::Result<T, InterpreterError>;

/// Interprets an AST based on tags from and [AudioFile].
pub struct Interpreter<'a> {
    songs: &'a [Box<dyn AudioFile>],
    program: &'a Program,
    symbol_table: SymbolTable,
    index: usize,
}

impl<'a> Interpreter<'a> {
    /// Constructor
    pub fn new(
        program: &'a Program,
        arguments: &'a [&str],
        songs: &'a [Box<dyn AudioFile>],
    ) -> Result<Self> {
        let symbol_table = SemanticAnalyzer::analyze(program, arguments)?;

        Ok(Interpreter {
            songs,
            program,
            symbol_table,
            index: 0,
        })
    }

    /// Public function for interpreter.
    pub fn interpret(&mut self) -> Result<Vec<String>> {
        let mut paths = Vec::new();

        for i in 0..self.songs.len() {
            self.index = i;

            trace!("In:  \"{}\"", self.songs[i].path().to_string_lossy());
            let path =
                self.program.accept(self)? + "." + self.songs[i].extension();
            trace!("Out: \"{}\"", path);

            paths.push(path);
        }

        Ok(paths)
    }

    fn strip_leading_zeroes(number: &str) -> &str {
        let mut out = number;

        while out.starts_with('0') {
            out = &out[1..];
        }

        out
    }
}

impl<'a> Visitor<Result<String>> for Interpreter<'a> {
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
        block
            .expressions
            .iter()
            .map(|e| e.accept(self))
            .collect::<Result<Vec<String>>>()
            .map(|e| e.join(""))
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
        let name = start_token.get_value_unchecked();

        let arguments: Vec<String> = arguments
            .iter()
            .map(|a| a.accept(self))
            .collect::<Result<Vec<String>>>()?;

        Ok(handle_function(name, &arguments)?)
    }

    fn visit_integer(&mut self, integer: &Token) -> Result<String> {
        Ok(integer.get_value_unchecked().to_string())
    }

    fn visit_string(&mut self, string: &Token) -> Result<String> {
        Ok(string.get_value_unchecked().to_string())
    }

    fn visit_symbol(&mut self, symbol: &Token) -> Result<String> {
        let name = symbol.get_value_unchecked();
        // This is checked by SemanticAnalyzer, should be safe.
        Ok(self.symbol_table.get(name).unwrap().to_string())
    }

    fn visit_tag(&mut self, token: &Token) -> Result<String> {
        let tag_name = token.get_value_unchecked();

        let mut tag = match tag_name {
            // TODO Add less common tags from AudioFile
            "album" => self.songs[self.index].album(),
            "albumartist" | "album_artist" => {
                self.songs[self.index].album_artist()
            }
            "albumsort" | "album_sort" => self.songs[self.index].albumsort(),
            "artist" => self.songs[self.index].artist(),
            "duration" | "length" => self.songs[self.index].duration(),
            "disc" | "disk" | "discnumber" | "disknumber" | "disc_number"
            | "disk_number" => self.songs[self.index]
                .disc_number()
                .map(Self::strip_leading_zeroes),
            "genre" => self.songs[self.index].genre(),
            "title" | "name" => self.songs[self.index].title(),
            "track" | "tracknumber" | "track_number" => self.songs[self.index]
                .track_number()
                .map(Self::strip_leading_zeroes),
            "year" | "date" => self.songs[self.index].year(),
            _ => None,
        }
        .unwrap_or("")
        .to_string();

        // TODO? Add strict mode, which allows/denies/errors on forbidden
        // characters/directory separators.
        FORBIDDEN_GRAPHEMES
            .iter()
            .for_each(|g| tag = tag.replace(g, ""));
        DIRECTORY_SEPARATORS
            .iter()
            .for_each(|g| tag = tag.replace(g, ""));

        Ok(tag)
    }
}
