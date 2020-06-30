use std::convert::From;
use std::error::Error;
use std::fmt;

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

// impl From<String> for LexerError {
//     fn from(string: String) -> Self {
//         LexerError::Lexer(string)
//     }
// }

impl From<LexerError> for String {
    fn from(error: LexerError) -> Self {
        format!("{}", error)
    }
}
