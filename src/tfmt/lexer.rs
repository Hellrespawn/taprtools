use log::{debug, trace};
use std::iter::Iterator;

use unicode_segmentation::UnicodeSegmentation;

use crate::error::LexerError;
use crate::tfmt::token::{self, Token, TokenType, RESERVED_CHARS, TOKEN_TYPES};

pub struct Lexer<'a> {
    text: Vec<&'a str>,
    index: usize,
    line_no: u32,
    char_no: u32,
    ended: bool,
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

    fn current_char(&self) -> Result<&str, LexerError> {
        match self.text.get(self.index) {
            Some(string) => Ok(&string),
            None => Err(LexerError::ExhaustedStream),
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

    fn advance(&mut self) -> Result<(), LexerError> {
        if let Ok(string) = self.current_char() {
            if string == "\n" {
                self.line_no += 1;
                self.char_no = 1;
            } else {
                self.char_no += 1;
            }
        } else {
            return Err(LexerError::ExhaustedStream);
        }

        self.index += 1;
        Ok(())
    }

    fn advance_times(&mut self, times: u32) -> Result<(), LexerError> {
        for _ in 1..=times {
            self.advance()?;
        }
        Ok(())
    }

    fn crawl(
        &mut self,
        terminators: Vec<String>,
        discard_terminator: bool,
        terminate_on_eof: bool,
        skip_chars: u32,
    ) -> Result<String, LexerError> {
        self.advance_times(skip_chars)?;

        let mut string = String::new();

        loop {
            let mut current_terminator = None;

            for terminator in terminators.iter() {
                if self.test_current_string(terminator) {
                    current_terminator = Some(terminator);
                    break;
                }
            }

            if let Some(terminator) = current_terminator {
                if discard_terminator {
                    // FIXME learn about casts
                    self.advance_times(terminator.len() as u32)?;
                };
                break;
            }

            match self.current_char() {
                Ok(char) => string.push_str(char),
                Err(_) => {
                    if !terminate_on_eof {
                        return Err(LexerError::Crawler(
                            "Crawl reached EOF before terminator!".to_owned(),
                        ));
                    } else {
                        break;
                    }
                }
            }
            self.advance()?;
        }

        Ok(string)
    }

    fn handle_string(&mut self, multiline: bool) -> Result<String, LexerError> {
        let quote = String::from(
            self.current_char()
                .expect("Checked in handle_bounded. Should never panic."),
        );

        let skip_chars = if multiline { 3 } else { 1 };

        let string = self.crawl(vec![quote], true, false, skip_chars)?;

        for char in &token::FORBIDDEN_CHARS {
            if string.contains(char) {
                return Err(LexerError::Lexer(format!(
                    "String contains forbidden char {:?}!",
                    char
                )));
            }
        }

        Ok(string)
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>, LexerError> {
        // Might panic here?
        let current_char = &self.current_char()?;

        let exp_string =
            "Should never panic, all TokenTypes are in TOKEN_TYPES.";
        let quotes = [
            TOKEN_TYPES
                .get_by_left(&TokenType::QUOTE_DOUBLE)
                .expect(exp_string),
            TOKEN_TYPES
                .get_by_left(&TokenType::QUOTE_SINGLE)
                .expect(exp_string),
        ];

        let single_line_comment =
            TOKEN_TYPES.get_by_left(&TokenType::HASH).expect(exp_string);
        let multiline_comment_start = TOKEN_TYPES
            .get_by_left(&TokenType::SLASH_ASTERISK)
            .expect(exp_string);
        let multiline_comment_end = TOKEN_TYPES
            .get_by_left(&TokenType::ASTERISK_SLASH)
            .expect(exp_string);

        if quotes.contains(&current_char) {
            let multiline =
                self.test_current_string(&format!("{0}{0}{0}", current_char));

            Ok(Some(Token::new(
                self.line_no,
                self.char_no,
                TokenType::STRING,
                Some(self.handle_string(multiline)?),
            )))
        } else if current_char == single_line_comment {
            Ok(Some(Token::new(
                self.line_no,
                self.char_no,
                TokenType::COMMENT,
                Some(self.crawl(
                    vec![String::from("\n")],
                    true,
                    true,
                    // FIXME Learn about casts.
                    single_line_comment.len() as u32,
                )?),
            )))
        } else if self.test_current_string(multiline_comment_start) {
            Ok(Some(Token::new(
                self.line_no,
                self.char_no,
                TokenType::COMMENT,
                Some(self.crawl(
                    vec![String::from(*multiline_comment_end)],
                    true,
                    false,
                    // FIXME Learn about casts.
                    multiline_comment_end.len() as u32,
                )?),
            )))
        } else {
            Ok(None)
        }
    }

    fn handle_reserved(&mut self) -> Result<Option<Token>, LexerError> {
        for chars in RESERVED_CHARS.iter() {
            if self.test_current_string(chars) {
                let token = Token::new_type_from_char(
                    self.line_no,
                    self.char_no,
                    chars,
                    None,
                )
                .expect("Use chars from TOKEN_TYPES, should always be safe.");
                // FIXME Learn about casts.
                self.advance_times(chars.len() as u32)?;
                return Ok(Some(token));
            }
        }

        Ok(None)
    }

    fn handle_misc_tokens(&mut self) -> Result<Token, LexerError> {
        let (line_no_start, char_no_start) = (self.line_no, self.char_no);

        let current_char = self.current_char()?;

        if current_char.chars().all(|c| c.is_alphabetic())
            && self.test_current_string(&(String::from(current_char) + ":\\"))
        {
            // Drive
            let token = Token::new(
                line_no_start,
                char_no_start,
                TokenType::DRIVE,
                self.current_string(3),
            );
            self.advance_times(3)?;

            Ok(token)
        } else {
            // ID
            let mut terminators: Vec<String> = Vec::new();
            for char in RESERVED_CHARS.iter() {
                terminators.push(String::from(*char));
            }
            terminators.push(String::from(" "));
            terminators.push(String::from("\t"));
            terminators.push(String::from("\n"));
            terminators.push(String::from("\r"));

            let value = self.crawl(terminators, false, true, 0)?;

            if value.starts_with(|c: char| c.is_alphabetic())
                && value.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                Ok(Token::new(
                    line_no_start,
                    char_no_start,
                    TokenType::ID,
                    Some(value),
                ))
            } else if value.chars().all(|c| c.is_numeric()) {
                Ok(Token::new(
                    line_no_start,
                    char_no_start,
                    TokenType::INTEGER,
                    Some(value),
                ))
            } else {
                Err(LexerError::Token(value))
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexerError> {
        trace!("Current char is {:?}", self.current_char());

        let token = {
            if self.current_char().is_err() {
                if self.ended {
                    None
                } else {
                    self.ended = true;
                    let token = Token::new(
                        self.line_no,
                        self.char_no,
                        TokenType::EOF,
                        None,
                    );

                    debug!("Returning token: {:#?}", token);
                    Some(token)
                }
            } else if self.current_char()?.chars().all(|c| c.is_whitespace()) {
                self.advance()?;
                return self.next_token();
            } else if let Some(token) = self.handle_bounded()? {
                Some(token)
            } else if let Some(token) = self.handle_reserved()? {
                Some(token)
            } else {
                Some(self.handle_misc_tokens()?)
            }
        };

        debug!("Returning token: {:#?}", token);
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DOUBLE_QUOTED_STRING: &str = "\"This is a double-quoted string\"";
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";
    static SINGLE_LINE_COMMENT: &str = "# This is a single line comment!\n";
    static MULTILINE_COMMENT: &str = "/* This is a \n multiline comment. */";
    static STRING_WITH_FORBIDDEN_CHARS: &str =
        "\"This \\ is / a string ~ with * forbidden chars.\"";

    fn slice_ends(string: &str, left: usize, right: usize) -> &str {
        &string[left..string.len() - right]
    }

    fn dequote(string: &str) -> &str {
        slice_ends(&string, 1, 1)
    }

    fn create_lexer(string: &str) -> Lexer {
        Lexer::new(&string)
    }

    fn run_lexer(string: &str, pop_eof: bool) -> Result<Vec<Token>, String> {
        let mut lex = Lexer::new(&string);

        let mut tokens: Vec<Token> = Vec::new();
        while let Some(token) = lex.next_token()? {
            tokens.push(token);
        }

        if pop_eof {
            tokens.pop();
        }

        Ok(tokens)
    }

    fn lexer_test(string: &str, reference: Vec<Token>) -> Result<(), String> {
        let tokens = run_lexer(string, true)?;

        assert_eq!(tokens, reference);

        Ok(())
    }

    mod lexer {
        use super::*;
        use std::fs;
        use std::path;

        fn file_test(filename: &str) -> Result<(), String> {
            let mut path = path::PathBuf::from(file!());
            for _ in 1..=3 {
                path.pop();
            }
            path.push("tests");
            path.push("files");
            path.push("config");
            path.push(filename);

            let input = fs::read_to_string(path)
                .expect(&format!("{} doesn't exist!", filename));

            run_lexer(&input, false).map(|_| ())
        }

        #[test]
        fn test_simple_input() -> Result<(), String> {
            file_test("simple_input.tfmt")
        }

        #[test]
        fn test_typical_input() -> Result<(), String> {
            file_test("typical_input.tfmt")
        }
    }

    mod handle_reserved {
        use super::*;

        fn reserved_test(
            string: &str,
            expected_type: TokenType,
        ) -> Result<(), String> {
            let mut lex = create_lexer(string);

            match lex.handle_reserved()? {
                Some(token) => {
                    if token.ttype == expected_type {
                        Ok(())
                    } else {
                        Err(format!(
                            "{} was parsed as {}, not {}!",
                            string,
                            // ttypes are always safe!
                            TOKEN_TYPES.get_by_left(&token.ttype).unwrap(),
                            TOKEN_TYPES.get_by_left(&expected_type).unwrap(),
                        ))
                    }
                }
                None => Err(format!("Unable to parse {} as Token!", string)),
            }
        }

        #[test]
        fn test_single_char() -> Result<(), String> {
            reserved_test("+", TokenType::PLUS)?;
            reserved_test("-", TokenType::HYPHEN)?;
            Ok(())
        }

        #[test]
        fn test_double_char() -> Result<(), String> {
            reserved_test("&&", TokenType::DOUBLE_AMPERSAND)?;
            reserved_test("||", TokenType::DOUBLE_VERTICAL_BAR)?;
            Ok(())
        }
    }

    mod handle_bounded {
        use super::*;

        #[test]
        fn test_double_quoted() -> Result<(), String> {
            let reference = vec![Token::new(
                1,
                1,
                TokenType::STRING,
                Some(String::from(dequote(DOUBLE_QUOTED_STRING))),
            )];
            lexer_test(DOUBLE_QUOTED_STRING, reference)
        }

        #[test]
        fn test_single_quoted() -> Result<(), String> {
            let reference = vec![Token::new(
                1,
                1,
                TokenType::STRING,
                Some(String::from(dequote(SINGLE_QUOTED_STRING))),
            )];
            lexer_test(SINGLE_QUOTED_STRING, reference)
        }

        #[test]
        fn test_string_with_forbidden_chars() -> Result<(), String> {
            match run_lexer(STRING_WITH_FORBIDDEN_CHARS, false) {
                Ok(tokens) => Err(format!("Lexer did not error on forbidden characters, returned {:?}", tokens)),
                Err(err) => {
                    if err.contains("forbidden char") {
                        Ok(())
                    } else {
                        Err(format!("Unrelated error {:?}!", err))
                    }
                }
            }
        }
    }

    mod handle_misc_tokens {
        use super::*;

        #[test]
        fn test_id() -> Result<(), String> {
            lexer_test(
                "id",
                vec![Token::new(1, 1, TokenType::ID, Some(String::from("id")))],
            )
        }

        #[test]
        fn test_integer() -> Result<(), String> {
            lexer_test(
                "1",
                vec![Token::new(
                    1,
                    1,
                    TokenType::INTEGER,
                    Some(String::from("1")),
                )],
            )
        }
    }

    mod crawler {
        use super::*;

        fn crawler_test(
            string: &String,
            reference: &str,
            terminators: Vec<String>,
            discard_terminator: bool,
            terminate_on_eof: bool,
            skip_chars: u32,
        ) -> Result<(), LexerError> {
            let mut lex = Lexer::new(&string);

            let output = lex.crawl(
                terminators,
                discard_terminator,
                terminate_on_eof,
                skip_chars,
            )?;

            assert_eq!(output.trim(), reference.trim());

            Ok(())
        }

        fn string_test(string: &str) -> Result<(), LexerError> {
            let string = String::from(string);
            let reference = dequote(&string);
            let terminators = vec![string.chars().next().unwrap().to_string()];

            crawler_test(&string, reference, terminators, true, false, 1)
        }

        #[test]
        fn test_double_quoted() -> Result<(), LexerError> {
            string_test(DOUBLE_QUOTED_STRING)
        }

        #[test]
        fn test_single_quoted() -> Result<(), LexerError> {
            string_test(SINGLE_QUOTED_STRING)
        }

        #[test]
        fn test_single_line_comment() -> Result<(), LexerError> {
            crawler_test(
                &String::from(SINGLE_LINE_COMMENT),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 0),
                vec![String::from("\n")],
                true,
                true,
                1,
            )?;

            crawler_test(
                &String::from(slice_ends(&SINGLE_LINE_COMMENT, 0, 1)),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 0),
                vec![String::from("\n")],
                true,
                true,
                1,
            )?;

            Ok(())
        }

        #[test]
        fn test_multiline_comment() -> Result<(), LexerError> {
            crawler_test(
                &String::from(MULTILINE_COMMENT),
                slice_ends(&MULTILINE_COMMENT, 2, 2),
                vec![String::from(
                    *TOKEN_TYPES
                        .get_by_left(&TokenType::ASTERISK_SLASH)
                        .unwrap(),
                )],
                true,
                false,
                2,
            )
        }
    }
}
