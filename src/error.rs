use crate::tfmt::token::TokenType;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error from the [token] module.
pub enum TokenError {
    #[error("\"{0}\" is not a valid TokenType!")]
    /// Invalid [TokenType].
    InvalidType(String),

    #[error("TokenType::{0:?} does not require a value, got \"{1}\"!")]
    /// [TokenType] does not require a value
    HasValue(TokenType, String),

    #[error("TokenType::{0:?} requires a value!")]
    /// [TokenType] requires value.
    NoValue(TokenType),
}

#[derive(Error, Debug)]
/// Error from the [lexer] module.
pub enum LexerError {
    /// Non-specific error.
    #[error("Error in Lexer: {0}")]
    Generic(String),

    /// [Lexer] exhausted text input stream.
    #[error("Lexer exhausted text input stream!")]
    ExhaustedText,

    /// Crawl reached EOF before terminator.
    #[error("Crawl reached EOF before terminator! Original error:\n{0}")]
    CrawlerEOF(String),

    /// String contains forbidden grapheme.
    #[error("String contains forbidden grapheme \"{0}\"!")]
    ForbiddenGrapheme(String),

    /// Unable to convert string to [Token].
    #[error("Unable to convert \"{0}\" to Token!")]
    Tokenize(String),

    #[error("")]
    Token {
        #[from]
        source: TokenError,
    },

    #[error("")]
    TryFromInt {
        #[from]
        source: std::num::TryFromIntError,
    },
}

#[derive(Error, Debug)]
/// Error from the [parser] module.
pub enum ParserError {
    /// Non-specific error.
    #[error("Error in Parser: {0}")]
    Generic(String),

    /// Exhausted token stream looking for [TokenType].
    #[error("Exhausted token stream looking for {0:?}")]
    ExhaustedTokens(TokenType),

    /// Exhausted token stream without finding EOF.
    #[error("Exhausted token stream without finding EOF!")]
    ExpectedEOF,

    /// Unexpected TokenType.
    #[error("Expected {0:?}, got {1:?}")]
    UnexpectedToken(TokenType, TokenType),

    /// Unable to parse token type.
    #[error("Unable to parse token type {0:?}!")]
    UnrecognizedToken(TokenType),

    /// Maximum iteration depth exceeded.
    #[error("Maximum iteration depth {0} exceeded!")]
    MaxIteration(u64),

    /// Encountered group without expressions.
    #[error("Encountered group without expressions!")]
    EmptyGroup,

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
    #[error("Wrong number of arguments ({found}) for function \"{name}\", expected {expected}!")]
    /// Wrong number of arguments for function
    WrongArguments {
        name: String,
        expected: usize,
        found: usize,
    },

    #[error("Unknown function \"{0}\"!")]
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
    #[error("Symbol \"{0}\" never occurs in program \"{1}\"!")]
    SymbolNotUsed(String, String),

    /// Too many arguments for program.
    #[error("Too many arguments ({0}) for program \"{2}\", expected {1}!")]
    TooManyArguments(usize, usize, String),

    /// Argument is required in program.
    #[error("Argument \"{0}\" is required in program \"{1}\"!")]
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
    #[error("Encountered forbidden grapheme \"{0}\" in tag!")]
    TagForbidden(String),

    /// Directory separator in ID.
    #[error("Encountered directory separator \"{0}\" in tag!")]
    TagDirSep(String),
}
