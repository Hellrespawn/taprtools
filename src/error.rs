use crate::tfmt::token::TokenType;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error from the [token] module.
pub enum TokenError {
    #[error("{0} is not a valid TokenType!")]
    /// Invalid [TokenType].
    InvalidType(String),

    #[error("TokenType {0:?} does not require a value, got {1}!")]
    /// [TokenType] does not require a value
    HasValue(TokenType, String),

    #[error("TokenType {0:?} requires a value!")]
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
    /// Wrapper for LexerError.
    Lexer {
        #[from]
        /// PLACEHOLDER
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

#[derive(Error, Debug)]
pub enum FunctionError {
    /// Error from the [function] module.
    #[error("Wrong number of arguments ({found}) for function {name}, expected {expected}!")]
    /// Wrong number of arguments for function
    WrongArguments {
        name: String,
        expected: usize,
        found: usize,
    },

    #[error("Unknown function {0}!")]
    /// Wrong number of arguments for function
    UnknownFunction(String),

    #[error("")]
    /// Wrapper for std::num::ParseIntError.
    ParseInt {
        #[from]
        /// PLACEHOLDER
        source: std::num::ParseIntError,
    },

    #[error("")]
    /// Wrapper for std::char::ParseCharError.
    ParseChar {
        #[from]
        /// PLACEHOLDER
        source: std::char::ParseCharError,
    },
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
    /// Wrapper for crate::error::ParserError.
    Parser {
        #[from]
        /// PLACEHOLDER
        source: ParserError,
    },

    #[error("")]
    /// Wrapper for crate::error::FunctionError.
    Function {
        #[from]
        /// PLACEHOLDER
        source: FunctionError,
    },

    #[error("")]
    /// Wrapper for std::num::ParseIntError.
    ParseInt {
        #[from]
        /// PLACEHOLDER
        source: std::num::ParseIntError,
    },

    /// Forbidden grapheme in ID.
    #[error("Encountered forbidden grapheme {0} in tag!")]
    TagForbidden(String),

    /// Directory separator in ID.
    #[error("Encountered directory separator {0} in tag!")]
    TagDirSep(String),
}
