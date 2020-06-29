use std::iter::Iterator;

use unicode_segmentation::UnicodeSegmentation;

use crate::tfmt::token::{Token, TokenType, TOKEN_TYPES};

pub struct Lexer<'a> {
    text: Vec<&'a str>,
    index: usize,
    line_no: u32,
    char_no: u32,
    ended: bool,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(opt) => opt,
            Err(err) => panic!(err),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Lexer {
            text: UnicodeSegmentation::graphemes(text, true)
                .collect::<Vec<&str>>(),
            index: 0,
            line_no: 1,
            char_no: 1,
            ended: false,
        }
    }

    fn current_char(&self) -> Option<&str> {
        match self.text.get(self.index) {
            Some(string) => Some(&string),
            None => None,
        }
    }

    fn current_string(&self, length: usize) -> Option<String> {
        match self.text.get(self.index..self.index + length) {
            Some(slice) => Some(slice.join("")),
            None => None,
        }
    }

    fn test_current_string(&self, string: &str) -> bool {
        match self.current_string(string.len()) {
            Some(current) => current == string,
            None => false,
        }
    }

    fn advance(&mut self) {
        match self.current_char() {
            Some(string) => {
                if string == "\n" {
                    self.line_no += 1;
                    self.char_no = 1;
                } else {
                    self.char_no += 1;
                }
            }
            None => (),
        }

        self.index += 1;
    }

    fn crawl(
        &self,
        terminators: Vec<&str>,
        discard_terminator: bool,
        terminate_on_eof: bool,
        skip_chars: u32,
    ) -> Result<String, &'static str> {
        // TODO Implement crawl
        Ok(String::from("Return from crawl"))
    }

    fn handle_string(&self, multiline: bool) -> Result<String, &'static str> {
        // Should never panic here.
        let quote = &self.current_char().unwrap();

        let skip_chars = if multiline { 3 } else { 1 };

        self.crawl(vec![quote], true, false, skip_chars)
        // TODO Check for forbidden chars here.
    }

    fn handle_bounded(&self) -> Result<Option<Token>, &'static str> {
        // Might panic here?
        let current_char = &self.current_char().unwrap();

        let quotes = [
            TOKEN_TYPES.get_by_left(&TokenType::QUOTE_DOUBLE).unwrap(),
            TOKEN_TYPES.get_by_left(&TokenType::QUOTE_SINGLE).unwrap(),
        ];

        if quotes.contains(&current_char) {
            let multiline = if self
                .test_current_string(&format!("{0}{0}{0}", current_char))
            {
                true
            } else {
                false
            };

            Ok(Some(Token::new(
                self.line_no,
                self.char_no,
                TokenType::STRING,
                Some(self.handle_string(multiline)?),
            )))

        // TODO Implement comments
        } else {
            Ok(None)
        }
    }

    fn handle_reserved(&self) -> Result<Option<Token>, &'static str> {
        Ok(None)
    }

    fn handle_misc_tokens(&self) -> Result<Option<Token>, &'static str> {
        Ok(None)
    }

    fn next_token(&mut self) -> Result<Option<Token>, &'static str> {
        if self.current_char().is_none() {
            if self.ended {
                return Ok(None);
            } else {
                self.ended = true;
                return Ok(Some(Token::new(
                    self.line_no,
                    self.char_no,
                    TokenType::EOF,
                    None,
                )));
            }
        }

        while self
            .current_char()
            .unwrap()
            .chars()
            .all(|c| c.is_whitespace())
        {
            self.advance();
        }

        for handler in &[Lexer::handle_bounded, Lexer::handle_reserved] {
            match handler(self)? {
                Some(token) => return Ok(Some(token)),
                None => (),
            }
        }

        self.handle_misc_tokens()
    }
}
