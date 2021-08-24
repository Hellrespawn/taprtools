use crate::file::audio_file::AudioFile;
use crate::helpers;
use crate::tfmt::ast::node::*;
use crate::tfmt::ast::{Parser, Visitor};
use crate::tfmt::error::InterpreterError;
use crate::tfmt::function::handle_function;
use crate::tfmt::token::{
    Token, TokenType, DIRECTORY_SEPARATORS, FORBIDDEN_GRAPHEMES,
};
use crate::tfmt::visitors::{SemanticAnalyzer, SymbolTable};
use log::trace;

type Result<T> = std::result::Result<T, InterpreterError>;

/// Interprets an [AST](ast::Program) based on tags from an [AudioFile].
pub struct Interpreter {
    program: Program,
    symbol_table: SymbolTable,
}

impl Interpreter {
    pub fn new<S: AsRef<str>>(
        input_text: &S,
        arguments: &[&str],
    ) -> Result<Self> {
        let program = Parser::new(input_text)?.parse()?;
        let symbol_table = SemanticAnalyzer::analyze(&program, arguments)?;

        Ok(Self {
            program,
            symbol_table,
        })
    }

    /// Public function for interpreter.
    pub fn interpret(&mut self, audio_file: &dyn AudioFile) -> Result<String> {
        trace!(r#"In:  "{}""#, audio_file.path().display());

        let string = format!(
            "{}.{}",
            helpers::normalize_separators(&self.program.accept(
                &mut IntpVisitor {
                    symbol_table: &self.symbol_table,
                    audio_file
                }
            )?),
            audio_file.extension()
        );

        trace!(r#"Out: "{}""#, string);

        Ok(string)
    }

    fn strip_leading_zeroes(number: &str) -> &str {
        let mut out = number;

        while out.starts_with('0') {
            out = &out[1..];
        }

        out
    }
}

struct IntpVisitor<'a> {
    symbol_table: &'a SymbolTable,
    audio_file: &'a dyn AudioFile,
}

impl<'a> Visitor<Result<String>> for IntpVisitor<'a> {
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
        Ok(match &token.token_type {
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
                    other.clone(),
                    "BinaryOp",
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
        Ok(match &token.token_type {
            TokenType::Plus => o,
            TokenType::Hyphen => (-o.parse::<i64>()?).to_string(),
            other => {
                return Err(InterpreterError::InvalidTokenType(
                    other.clone(),
                    "UnaryOp",
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
        let name = start_token.get_string_unchecked();

        let arguments: Vec<String> = arguments
            .iter()
            .map(|a| a.accept(self))
            .collect::<Result<Vec<String>>>()?;

        Ok(handle_function(name, &arguments)?)
    }

    fn visit_integer(&mut self, integer: &Token) -> Result<String> {
        Ok(integer.get_int_unchecked().to_string())
    }

    fn visit_string(&mut self, string: &Token) -> Result<String> {
        Ok(string.get_string_unchecked().to_string())
    }

    fn visit_symbol(&mut self, symbol: &Token) -> Result<String> {
        let name = symbol.get_string_unchecked();
        // This is checked by SemanticAnalyzer, should be safe.
        debug_assert!(self.symbol_table.get(name).is_some());
        Ok(self.symbol_table.get(name).unwrap().to_string())
    }

    fn visit_tag(&mut self, token: &Token) -> Result<String> {
        let tag_name = token.get_string_unchecked();

        let audio_file = self.audio_file;

        let mut tag = match tag_name {
            // TODO Add less common tags from AudioFile
            "album" => audio_file.album(),
            "albumartist" | "album_artist" => audio_file.album_artist(),
            "albumsort" | "album_sort" => audio_file.albumsort(),
            "artist" => audio_file.artist(),
            "duration" | "length" => audio_file.duration(),
            "disc" | "disk" | "discnumber" | "disknumber" | "disc_number"
            | "disk_number" => audio_file
                .disc_number()
                .map(Interpreter::strip_leading_zeroes),
            "genre" => audio_file.genre(),
            "title" | "name" => audio_file.title(),
            "track" | "tracknumber" | "track_number" => audio_file
                .track_number()
                .map(Interpreter::strip_leading_zeroes),
            "year" | "date" => audio_file.year(),
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
