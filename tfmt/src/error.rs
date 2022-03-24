#![allow(missing_docs)]
use crate::token::{Token, TokenType};
use std::fmt;
use thiserror::Error;

// TODO Learn more about nested errors
//
// Like this?
//
// TFMTError
// -> TokenError
// -> LexerError
// ...
//
// Or just shove everything under TFMTError?

#[derive(Debug, PartialEq)]
pub struct ErrorContext {
    script: String,
    pub(crate) line_no: usize,
    pub(crate) col_no: usize,
}

impl ErrorContext {
    pub fn line_no(&self) -> usize {
        self.line_no
    }

    pub fn col_no(&self) -> usize {
        self.col_no
    }
}

impl ErrorContext {
    pub(crate) fn new(input_text: &str, line_no: usize, col_no: usize) -> Self {
        Self {
            script: String::from(input_text),
            line_no,
            col_no,
        }
    }

    pub(crate) fn from_token(script: &str, token: &Token) -> Self {
        Self {
            script: String::from(script),
            line_no: token.line_no(),
            col_no: token.col_no(),
        }
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n{}^\nError at line {}, col {}:\n",
            &self.script.lines().collect::<String>()[..self.line_no]
                .to_string(),
            " ".repeat(self.col_no),
            self.line_no,
            self.col_no
        )
    }
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [token] module.
pub enum TokenError {
    #[error(r#""{0}" is not a valid TokenType!"#)]
    /// Invalid [TokenType].
    InvalidTokenType(String),
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [lexer] module.
pub enum LexerError {
    /// [Lexer] exhausted text input stream.
    #[error(r#"{0}Lexer exhausted text input stream looking for "{1}"!"#)]
    ExhaustedText(ErrorContext, String),

    /// String contains forbidden grapheme.
    #[error(r#"{0}String contains forbidden grapheme "{1}"!"#)]
    ForbiddenGrapheme(ErrorContext, String),

    /// String contains carriage return.
    #[error("{0}Input contains carriage return character (\\r)!")]
    InputContainsCr(ErrorContext),

    /// Single line string contains newline.
    #[error(r#"{0}String contains newline character (\n)!"#)]
    NewlineInString(ErrorContext),

    /// Unexpected terminator at EOF.
    #[error(r#"{context}Input ends with "{found}", expected "{expected}""#)]
    WrongTerminatorAtEOF {
        context: ErrorContext,
        found: String,
        expected: String,
    },
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [lexer] module.
pub enum ParserError {
    /// Encountered group without expressions.
    #[error("{0}Encountered group without expressions!")]
    EmptyGroup(ErrorContext),

    /// Parameter default is neither int nor string.
    #[error("{0}Parameter default is neither int nor string but {1:?}")]
    InvalidDefault(ErrorContext, TokenType),

    /// Unexpected [TokenType].
    #[error("Expected {expected:?}, got {found:?}")]
    UnexpectedTokenType {
        context: ErrorContext,
        expected: TokenType,
        found: TokenType,
    },

    /// Unable to parse [TokenType].
    #[error("Unable to parse token type {0:?}!")]
    UnrecognizedToken(ErrorContext, TokenType),

    /// Iterator has run out of tokens.
    #[error("Iterator has run out of tokens!")]
    ExhaustedTokens,

    /// Maximum iteration depth exceeded!
    #[error("Maximum iteration depth, {0}, exceeded!")]
    MaxDepth(u64),

    #[error(transparent)]
    Lexer(#[from] LexerError),
}

/// Errors in [function] module.
#[derive(Error, Debug)]
pub enum FunctionError {
    /// Error from the [function] module.
    #[error(r#"{context}Wrong number of arguments ({found}) for function "{name}", expected {expected}!"#)]
    /// Wrong number of arguments for function
    WrongArguments {
        context: ErrorContext,
        name: String,
        expected: usize,
        found: usize,
    },

    #[error(r#"{0}Unknown function "{1}"!"#)]
    /// Wrong number of arguments for function
    UnknownFunction(ErrorContext, String),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseChar(#[from] std::char::ParseCharError),
}

#[derive(Error, Debug)]
/// Error from the [genastdot] module.
pub enum SemanticError {
    /// "Symbol never occurs in program."
    #[error(
        r#"Declared symbol "{symbol}" never occurs in script "{script}"!"#
    )]
    SymbolNotUsed { symbol: String, script: String },

    /// "Symbol is not declared in script."
    #[error(r#"Symbol "{symbol}" is not declared in script "{script}"!"#)]
    SymbolNotDeclared { symbol: String, script: String },
}

#[derive(Error, Debug)]
/// Error from the [interpreter] module.
pub enum InterpreterError {
    /// Invalid [TokenType].
    #[error("{context}Invalid TokenType in {name}: {invalid_type:?}!")]
    InvalidTokenType {
        context: ErrorContext,
        invalid_type: TokenType,
        name: &'static str,
    },

    /// Too many arguments for program.
    #[error(r#"Found {found} arguments, expected {expected}!"#)]
    TooManyArguments { found: usize, expected: usize },

    /// Argument is required in program.
    #[error(r#"Argument "{argument}" is required in script "{script}"!"#)]
    ArgumentRequired { script: String, argument: String },

    #[error(transparent)]
    Lexer(#[from] LexerError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    Semantic(#[from] SemanticError),

    #[error(transparent)]
    Function(#[from] FunctionError),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Error, Debug)]
/// Error from the [`Visualizer`] module.
pub enum ScriptError {
    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    Semantic(#[from] SemanticError),
}
