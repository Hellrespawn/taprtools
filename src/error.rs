use crate::tfmt::token::TokenType;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error from the [token] module.
pub enum TokenError {
    #[error("{0} is not a valid TokenType!")]
    /// Invalid [TokenType].
    InvalidType(String),
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
/// Error from the [parser] module.
pub enum InterpreterError {
    /// Non-specific error.
    #[error("Error in Interpreter: {0}")]
    Generic(String),

    #[error("Invalid TokenType in {0:?}: {1}!")]
    InvalidTokenType(TokenType, &'static str),

    #[error("TokenType {0:?} requires value!")]
    TokenWithoutValue(TokenType),

    #[error("")]
    /// Wrapper for crate::error::ParserError.
    Parser {
        #[from]
        /// PLACEHOLDER
        source: ParserError,
    },

    #[error("")]
    /// Wrapper for std::num::ParseIntError.
    ParseInt {
        #[from]
        /// PLACEHOLDER
        source: std::num::ParseIntError,
    },
}
