use std::convert::From;
use std::error::Error;
use std::fmt;

use crate::tfmt::token::TokenType;

#[derive(Debug, PartialEq)]
pub enum TFMTError {
    External(String),
    Lexer(String),
    Crawler(String),
    Tokenize(String),
    Parser(String),
    UnexpectedToken(TokenType, TokenType),
    ExhaustedTokens(TokenType),
    UnrecognizedToken(TokenType),
    ExhaustedText,
    ExpectedValue,
    EmptyGroup,
    GenAstDot,
}

impl fmt::Display for TFMTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // String already impls `Display`, so we defer to
            // the implementations.
            TFMTError::External(err) => write!(f, "External error: {}", err),
            TFMTError::Lexer(err) => write!(f, "Lexer error: {}", err),
            TFMTError::Crawler(err) => write!(f, "Crawler error: {}", err),
            TFMTError::Tokenize(char) => {
                write!(f, "Unable to convert to Token: {:?}", char)
            }
            TFMTError::Parser(err) => write!(f, "Parser error: {}", err),
            TFMTError::UnexpectedToken(wanted, found) => {
                write!(f, "Expected {:?}, got {:?}", wanted, found)
            }
            TFMTError::ExhaustedTokens(ttype) => write!(
                f,
                "Exhausted token stream while searching for {:?}!",
                ttype
            ),
            TFMTError::UnrecognizedToken(ttype) => {
                write!(f, "Unable to parse token type {:?}!", ttype)
            }
            TFMTError::ExhaustedText => {
                write!(f, "Exhausted text input stream!")
            }
            TFMTError::ExpectedValue => {
                write!(f, "Expected token to have value!")
            }
            TFMTError::EmptyGroup => {
                write!(f, "Encountered group without expressions!")
            }
            TFMTError::GenAstDot => write!(
                f,
                "Unable to run dot! Is GraphViz installed and is it in PATH?"
            ),
        }
    }
}

impl Error for TFMTError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl From<TFMTError> for String {
    fn from(error: TFMTError) -> Self {
        format!("{}", error)
    }
}
