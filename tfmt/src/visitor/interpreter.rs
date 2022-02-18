#[allow(clippy::wildcard_imports)]
use crate::ast::node::*;
use crate::error::{ErrorContext, InterpreterError};
use crate::script::Script;
use crate::tags::Tags;
use crate::token::{Token, TokenType};
use crate::visitor::Visitor;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::trace;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, InterpreterError>;
type SymbolTable = HashMap<String, String>;

/// Interprets an `[AST](ast::Program)` based on tags from an [`AudioFile`].
#[derive(Debug)]
pub struct Interpreter {
    script: Script,
    symbol_table: SymbolTable,
}

impl Interpreter {
    /// Create new interpreter
    pub fn new(script: Script, arguments: Vec<String>) -> Result<Self> {
        let symbol_table =
            Interpreter::construct_symbol_table(&script, arguments)?;

        Ok(Self {
            script,
            symbol_table,
        })
    }

    /// Public function for interpreter.
    pub fn interpret(&mut self, audio_file: &dyn Tags) -> Result<String> {
        let string = crate::normalize_separators(&self.script.accept_visitor(
            &mut IntpVisitor {
                input_text: self.script.input_text(),
                symbol_table: &self.symbol_table,
                audio_file,
            },
        )?);

        trace!(r#"Out: "{}""#, string);

        Ok(string)
    }

    fn construct_symbol_table(
        script: &Script,
        arguments: Vec<String>,
    ) -> Result<SymbolTable> {
        let amount_of_arguments = arguments.len();
        let mut symbol_table = HashMap::new();

        for pair in script.parameters().iter().zip_longest(arguments) {
            match pair {
                Both(param, arg) => {
                    // parameter and arg present
                    symbol_table.insert(param.name().to_string(), arg);
                }
                Left(param) => {
                    // parameter and no arg present
                    if let Some(default) = param.default() {
                        // default present
                        symbol_table.insert(
                            param.name().to_string(),
                            default.to_string(),
                        );
                    } else {
                        // default not present
                        return Err(InterpreterError::ArgumentRequired(
                            param.name().to_string(),
                        ));
                    }
                }
                Right(_) => {
                    // no parameter but arg present
                    return Err(InterpreterError::TooManyArguments {
                        found: amount_of_arguments,
                        expected: script.parameters().len(),
                    });
                }
            }
        }

        Ok(symbol_table)
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
    input_text: &'a str,
    symbol_table: &'a SymbolTable,
    audio_file: &'a dyn Tags,
}

impl<'a> Visitor<Result<String>> for IntpVisitor<'a> {
    fn visit_program(&mut self, program: &Program) -> Result<String> {
        program.block().accept(self)
    }

    fn visit_parameters(&mut self, _: &Parameters) -> Result<String> {
        Ok("".to_string())
    }

    fn visit_parameter(&mut self, _: &Parameter) -> Result<String> {
        Ok("".to_string())
    }

    fn visit_block(&mut self, block: &Block) -> Result<String> {
        block
            .expressions()
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
        Ok(match &token.token_type() {
            TokenType::VerticalBar => {
                if l.is_empty() {
                    r
                } else {
                    l
                }
            }
            TokenType::DoubleVerticalBar => {
                if l.is_empty() {
                    r
                } else {
                    format!("{}{}", l, r)
                }
            }
            TokenType::Ampersand => {
                if l.is_empty() {
                    l
                } else {
                    r
                }
            }
            TokenType::DoubleAmpersand => {
                if l.is_empty() {
                    l
                } else {
                    format!("{}{}", l, r)
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
                return Err(InterpreterError::InvalidTokenType {
                    context: ErrorContext::from_token(self.input_text, token),
                    invalid_type: (*other).clone(),
                    name: "BinaryOp",
                })
            }
        })
    }

    fn visit_unaryop(
        &mut self,
        token: &Token,
        operand: &Expression,
    ) -> Result<String> {
        let o = operand.accept(self)?;
        Ok(match token.token_type() {
            TokenType::Plus => o,
            TokenType::Hyphen => (-o.parse::<i64>()?).to_string(),
            other => {
                return Err(InterpreterError::InvalidTokenType {
                    context: ErrorContext::from_token(self.input_text, token),
                    invalid_type: other.clone(),
                    name: "UnaryOp",
                })
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
        let arguments: Vec<String> = arguments
            .iter()
            .map(|a| a.accept(self))
            .collect::<Result<Vec<String>>>()?;

        Ok(crate::function::handle_function(
            &self.input_text,
            start_token,
            &arguments,
        )?)
    }

    fn visit_integer(&mut self, integer: &Token) -> Result<String> {
        Ok(integer.get_int_unchecked().to_string())
    }

    fn visit_string(&mut self, string: &Token) -> Result<String> {
        Ok(string.get_string_unchecked().to_string())
    }

    fn visit_symbol(&mut self, symbol: &Token) -> Result<String> {
        let name = symbol.get_string_unchecked();
        debug_assert!(self.symbol_table.get(name).is_some());

        // FIXME Check unwrap
        Ok(self.symbol_table.get(name).unwrap().clone())
    }

    fn visit_tag(&mut self, token: &Token) -> Result<String> {
        let tag_name = token.get_string_unchecked();

        let audio_file = self.audio_file;

        let mut tag = match tag_name {
            // TODO? Parse less common tags.
            "album" => audio_file.album(),
            "albumartist" | "album_artist" => audio_file.album_artist(),
            "albumsort" | "album_sort" => audio_file.albumsort(),
            "artist" => audio_file.artist(),
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
        crate::FORBIDDEN_GRAPHEMES
            .iter()
            .for_each(|g| tag = tag.replace(g, ""));

        crate::DIRECTORY_SEPARATORS
            .iter()
            .for_each(|g| tag = tag.replace(g, ""));

        Ok(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{bail, Result};
    use std::path::MAIN_SEPARATOR;

    #[derive(Debug)]
    struct MockTags;

    impl Tags for MockTags {
        fn album(&self) -> Option<&str> {
            Some("album")
        }

        fn album_artist(&self) -> Option<&str> {
            Some("album_artist")
        }

        fn albumsort(&self) -> Option<&str> {
            Some("9")
        }

        fn artist(&self) -> Option<&str> {
            Some("artist")
        }

        fn genre(&self) -> Option<&str> {
            Some("genre")
        }

        fn raw_disc_number(&self) -> Option<&str> {
            Some("8/9")
        }

        fn raw_track_number(&self) -> Option<&str> {
            Some("98/99")
        }

        fn title(&self) -> Option<&str> {
            Some("title")
        }

        fn year(&self) -> Option<&str> {
            Some("9999")
        }
    }

    #[cfg(unix)]
    const TYPICAL_INPUT: &str =
        include_str!("../../testdata/typical_input.tfmt");

    #[cfg(windows)]
    const TYPICAL_INPUT: &str =
        include_str!("..\\..\\testdata\\typical_input.tfmt");

    fn expected_output() -> String {
        "argument\\album_artist\\9999.09 - album\\898 - artist - title"
            .replace('\\', &MAIN_SEPARATOR.to_string())
            .replace('/', &MAIN_SEPARATOR.to_string())
    }

    #[test]
    fn test_full() -> Result<()> {
        let script = Script::new(TYPICAL_INPUT)?;
        let mut interpreter =
            Interpreter::new(script, vec!["argument".to_string()])?;

        let file = MockTags;

        let output = interpreter.interpret(&file)?;

        assert_eq!(output, expected_output());

        Ok(())
    }

    #[test]
    fn test_too_few_arguments() -> Result<()> {
        // Remove default argument
        let script =
            Script::new(TYPICAL_INPUT.replace("=\"destination\"", ""))?;

        match Interpreter::new(script, vec![]) {
            Ok(out) => {
                bail!("Expected InterpreterError::ArgumentRequired(\"folder\"), got Ok({out:?})")
            }
            Err(err) => match err {
                InterpreterError::ArgumentRequired(param) => {
                    assert_eq!(param, "folder".to_string());
                    Ok(())
                }
                other => {
                    bail!("Expected InterpreterError::ArgumentRequired(\"folder\"), got Err({other})")
                }
            },
        }
    }

    #[test]
    fn test_too_much_arguments() -> Result<()> {
        let script = Script::new(TYPICAL_INPUT)?;

        let result = Interpreter::new(
            script,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );

        match result {
            Ok(out) => {
                bail!("Expected TooManyArgument(3, 1), got Ok({out:?})")
            }
            Err(err) => match err {
                InterpreterError::TooManyArguments { found, expected } => {
                    assert_eq!(found, 3);
                    assert_eq!(expected, 1);
                    Ok(())
                }
                other => {
                    bail!("Expected Expected TooManyArgument(3, 1), got Err({other})")
                }
            },
        }
    }
}
