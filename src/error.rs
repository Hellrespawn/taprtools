use std::convert::From;
use std::error::Error;
use std::fmt;

use crate::tfmt::token::TokenType;

#[derive(Debug)]
pub enum LexerError {
    Lexer(String),
    Crawler(String),
    Token(String),
    ExhaustedStream,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // String already impls `Display`, so we defer to
            // the implementations.
            LexerError::Lexer(err) => write!(f, "Lexer error: {}", err),
            LexerError::Crawler(err) => write!(f, "Crawler error: {}", err),
            LexerError::Token(char) => {
                write!(f, "Unable to convert to Token: {:?}", char)
            }
            LexerError::ExhaustedStream => write!(f, "Exhausted input stream!"),
        }
    }
}

impl Error for LexerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<LexerError> for String {
    fn from(error: LexerError) -> Self {
        format!("{}", error)
    }
}

#[derive(Debug)]
pub enum ParserError {
    Parser(String),
    UnexpectedToken(TokenType, TokenType),
    ExhaustedStream(TokenType),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // String already impls `Display`, so we defer to
            // the implementations.
            ParserError::Parser(err) => write!(f, "Parser error: {}", err),
            ParserError::UnexpectedToken(wanted, found) => {
                write!(f, "Expected {:?}, got {:?}", wanted, found)
            }
            ParserError::ExhaustedStream(ttype) => write!(
                f,
                "Exhausted token stream while searching for {:?}!",
                ttype
            ),
        }
    }
}

impl Error for ParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<ParserError> for String {
    fn from(error: ParserError) -> Self {
        format!("{}", error)
    }
}
