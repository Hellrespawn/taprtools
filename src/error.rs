use crate::tfmt::token::TokenType;
use thiserror::Error;

// TODO Expand tfmt error handling.

#[derive(Error, Debug, PartialEq)]
/// Error from the [token] module.
pub enum TokenError {
    #[error(r#""{0}" is not a valid TokenType!"#)]
    /// Invalid [TokenType].
    InvalidType(String),
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [lexer] module.
pub enum LexerError {
    /// [Lexer] exhausted text input stream.
    #[error(r#"Lexer exhausted text input stream looking for "{0}"!"#)]
    ExhaustedText(String),

    /// String contains forbidden grapheme.
    #[error(r#"String contains forbidden grapheme "{0}"!"#)]
    ForbiddenGrapheme(String),

    /// String contains forbidden grapheme.
    #[error(r#"String contains newline character: "{0}""#)]
    NewlineInString(String),

    /// String contains forbidden grapheme.
    #[error(r#"Input ends with "{found}", expected "{expected}""#)]
    WrongTerminatorAtEOF { found: String, expected: String },
}

#[derive(Error, Debug, PartialEq)]
/// Error from the [lexer] module.
pub enum ParserError {
    #[error("{0}")]
    Generic(String),

    #[error("Encountered group without expressions!")]
    EmptyGroup,

    #[error("Maximum iteration depth {0}, exceeded!")]
    MaxIteration(u64),

    #[error("Expected {expected:?}, got {found:?}")]
    UnexpectedToken {
        expected: TokenType,
        found: TokenType,
    },

    #[error("Unable to parse token type {0:?}!")]
    UnrecognizedToken(TokenType),

    #[error("")]
    Lexer {
        #[from]
        source: LexerError,
    },
}

#[derive(Error, Debug)]
/// Error from the [genastdot] module.
pub enum DotError {
    /// Unable to run dot! Is GraphViz installed and is it in PATH?
    #[error("Unable to run dot! Is GraphViz installed and is it in PATH?")]
    CantRun,
}

/// Errors in [function] module.
#[derive(Error, Debug)]
pub enum FunctionError {
    /// Error from the [function] module.
    #[error(r#"Wrong number of arguments ({found}) for function "{name}", expected {expected}!"#)]
    /// Wrong number of arguments for function
    WrongArguments {
        name: String,
        expected: usize,
        found: usize,
    },

    #[error(r#"Unknown function "{0}"!"#)]
    /// Wrong number of arguments for function
    UnknownFunction(String),

    #[error("")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },

    #[error("")]
    ParseChar {
        #[from]
        source: std::char::ParseCharError,
    },
}

#[derive(Error, Debug)]
/// Error from the [genastdot] module.
pub enum SemanticError {
    /// "Symbol never occurs in program."
    #[error(r#"Symbol "{0}" never occurs in program "{1}"!"#)]
    SymbolNotUsed(String, String),

    /// Too many arguments for program.
    #[error(r#"Too many arguments ({found}) for program "{name}", expected {expected}!"#)]
    TooManyArguments {
        found: usize,
        expected: usize,
        name: String,
    },

    /// Argument is required in program.
    #[error(r#"Argument "{0}" is required in program "{1}"!"#)]
    ArgumentRequired(String, String),
}

#[derive(Error, Debug)]
/// Error from the [parser] module.
pub enum InterpreterError {
    /// Non-specific error.
    #[error("Error in Interpreter: {0}")]
    Generic(String),

    /// Invalid [TokenType].
    #[error("Invalid TokenType in {0:?}: {1}!")]
    InvalidTokenType(TokenType, &'static str),

    #[error("")]
    Parser {
        #[from]
        source: ParserError,
    },

    #[error("")]
    Semantic {
        #[from]
        source: SemanticError,
    },

    #[error("")]
    Function {
        #[from]
        source: FunctionError,
    },

    #[error("")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },

    /// Forbidden grapheme in ID.
    #[error(r#"Encountered forbidden grapheme "{0}" in tag!"#)]
    TagForbidden(String),

    /// Directory separator in ID.
    #[error(r#"Encountered directory separator "{0}" in tag!"#)]
    TagDirSep(String),
}
