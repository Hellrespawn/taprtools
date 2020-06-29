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

    fn advance(&mut self) -> Result<(), &'static str> {
        if let Some(string) = self.current_char() {
            if string == "\n" {
                self.line_no += 1;
                self.char_no = 1;
            } else {
                self.char_no += 1;
            }
        } else {
            return Err("Exhausted input!");
        }

        self.index += 1;
        Ok(())
    }

    fn crawl(
        &mut self,
        terminators: Vec<String>,
        discard_terminator: bool,
        terminate_on_eof: bool,
        skip_chars: u32,
    ) -> Result<String, &'static str> {
        for _ in 1..=skip_chars {
            self.advance()?;
        }

        let mut string = String::new();

        loop {
            match self.current_char() {
                Some(char) => {
                    if let Some(index) = terminators
                        .iter()
                        .position(|s| s == &String::from(char))
                    {
                        if discard_terminator {
                            for _ in 1..=terminators[index].len() {
                                self.advance()?;
                            }
                        };
                        break;
                    } else {
                        string.push_str(char);
                    }
                }
                None => {
                    if !terminate_on_eof {
                        return Err("Crawl reached EOF before terminator!");
                    } else {
                        break;
                    }
                }
            }
            self.advance()?;
        }

        Ok(string)
    }

    fn handle_string(
        &mut self,
        multiline: bool,
    ) -> Result<String, &'static str> {
        // Should never panic here.
        let quote = String::from(self.current_char().unwrap());

        let skip_chars = if multiline { 3 } else { 1 };

        self.crawl(vec![quote], true, false, skip_chars)
        // TODO Check for forbidden chars here.
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>, &'static str> {
        // Might panic here?
        let current_char = &self.current_char().unwrap();

        let quotes = [
            TOKEN_TYPES.get_by_left(&TokenType::QUOTE_DOUBLE).unwrap(),
            TOKEN_TYPES.get_by_left(&TokenType::QUOTE_SINGLE).unwrap(),
        ];

        if quotes.contains(&current_char) {
            let multiline =
                self.test_current_string(&format!("{0}{0}{0}", current_char));

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
            self.advance()?;
        }

        if let Some(token) = self.handle_bounded()? {
            return Ok(Some(token));
        }

        if let Some(token) = self.handle_reserved()? {
            return Ok(Some(token));
        }

        self.handle_misc_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DOUBLE_QUOTED_STRING: &str = "\"This is a double-quoted string\"";
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";

    // mod lexer {
    //     use super::*;

    //     fn lexer_test(string: &str, reference: Vec<Token>) -> Result<(), String> {
    //         let string = String::from(string);
    //         let lex = Lexer::new(&string);

    //         let tokens: Vec<Token> = lex.collect();

    //         if tokens != reference {
    //             Err(format!("Tokens don't match!\nExpected: {:?}\nFound: {:?}", reference, tokens))
    //         } else {
    //             Ok(())
    //         }
    //     }

    //     #[test]
    //     fn test_double_quoted() -> Result<(), String> {
    //             lexer_test(DOUBLE_QUOTED_STRING, vec!(Token::new(1, 1, TokenType::STRING, Some(String::from(DOUBLE_QUOTED_STRING)))))
    //     }

    //     #[test]
    //     fn test_single_quoted() -> Result<(), String> {
    //         lexer_test(SINGLE_QUOTED_STRING, vec!(Token::new(1, 1, TokenType::STRING, Some(String::from(SINGLE_QUOTED_STRING)))))
    //     }
    // }

    mod crawler {
        use super::*;

        fn crawler_test(
            string: &String,
            reference: &str,
            terminators: Vec<String>,
            discard_terminator: bool,
            terminate_on_eof: bool,
            skip_chars: u32,
        ) -> Result<(), String> {
            let mut lex = Lexer::new(&string);

            let output = lex.crawl(
                terminators,
                discard_terminator,
                terminate_on_eof,
                skip_chars,
            )?;

            assert_eq!(output, reference);

            Ok(())
        }

        fn string_test(string: &str) -> Result<(), String> {
            let string = String::from(string);
            let reference = &string[1..string.len() - 1];
            let terminators = vec![string.chars().next().unwrap().to_string()];

            crawler_test(&string, reference, terminators, true, false, 1)
        }

        #[test]
        fn test_double_quoted() -> Result<(), String> {
            string_test(DOUBLE_QUOTED_STRING)
        }

        #[test]
        fn test_single_quoted() -> Result<(), String> {
            string_test(SINGLE_QUOTED_STRING)
        }
    }
}
