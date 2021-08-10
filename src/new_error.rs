use crate::tfmt::new_token::TokenType;
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
    UnexpectedToken { expected: String, found: TokenType },

    #[error("Unable to parse token type {0:?}!")]
    UnrecognizedToken(TokenType),

    #[error("")]
    Lexer {
        #[from]
        source: LexerError,
    },
}
