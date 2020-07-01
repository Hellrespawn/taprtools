use super::lexer::Lexer;
use super::token::{self, Token, TokenType};
use crate::error::ParserError;
use std::error::Error;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    depth: u32,
    current_token: Option<Token>,
    previous_token: Option<Token>,
}

impl<'a> Parser<'a> {
    // Constructors
    pub fn from_lexer(lexer: Lexer) -> Parser {
        Parser {
            lexer,
            depth: 0,
            current_token: None,
            previous_token: None,
        }
    }

    pub fn from_string(string: &str) -> Parser {
        Parser {
            lexer: Lexer::new(string),
            depth: 0,
            current_token: None,
            previous_token: None,
        }
    }

    fn _advance(&mut self, ignore: bool) -> Result<(), Box<dyn Error>> {
        if !ignore {
            self.previous_token = self.current_token.take()
        }

        self.current_token = match self.lexer.next_token()? {
            Some(token) => Some(token),
            None => None,
        };

        if let Some(token) = self.current_token.as_ref() {
            if token::IGNORED.contains(&token.ttype()) {
                self._advance(true)?;
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), Box<dyn Error>> {
        self._advance(false)
    }

    fn consume(
        &mut self,
        expected_ttype: TokenType,
    ) -> Result<(), Box<dyn Error>> {
        let current_ttype = self.current_token.as_ref().unwrap().ttype();

        if current_ttype == TokenType::EOF {
            return Err(Box::new(ParserError::ExhaustedStream(current_ttype)));
        }

        self.advance()?;

        if current_ttype != expected_ttype {
            return Err(Box::new(ParserError::UnexpectedToken(
                expected_ttype,
                current_ttype,
            )));
        }

        Ok(())
    }
}
