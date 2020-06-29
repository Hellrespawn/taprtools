use std::iter::Iterator;

use unicode_segmentation::UnicodeSegmentation;

use crate::tfmt::token::Token;

pub struct Lexer<'a> {
    text: Vec<&'a str>,
    index: usize,
    line_no: u32,
    char_no: u32,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.text.len() {
            return None;
        };

        Some(self.next_token())
    }
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Lexer {
            text: UnicodeSegmentation::graphemes(text, true)
                .collect::<Vec<&str>>(),
            index: 0,
            line_no: 0,
            char_no: 0,
        }
    }

    fn current_char(&self) -> &str {
        self.text[self.index]
    }

    fn current_string(&self, length: usize) -> String {
        self.text[self.index..self.index + length].join("")
    }

    fn test_current_string(&self, string: &str) -> bool {
        self.current_string(string.len()) == string
    }

    fn next_token(&mut self) -> Token {
        Token::new_type_from_char(0, 0, "&", None).unwrap()
    }
}
