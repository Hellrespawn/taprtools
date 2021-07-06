use std::iter::Iterator;

use anyhow::Result;
use log::{debug, error, trace};
use unicode_segmentation::UnicodeSegmentation;

use super::token::{
    self, Token, TokenType, RESERVED_STRINGS, TOKEN_TYPE_STRING_MAP,
};
use crate::error::TFMTError;

pub struct Lexer {
    text: Vec<String>,
    index: usize,
    line_no: u64,
    col_no: u64,
    ended: bool,
}

impl Lexer {
    pub fn new(text: &str) -> Lexer {
        trace!("Creating lexer:\n{}", text);
        Lexer {
            text: UnicodeSegmentation::graphemes(text, true)
                .map(String::from)
                .collect(),
            index: 0,
            line_no: 1,
            col_no: 1,
            ended: false,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.line_no = 1;
        self.col_no = 1;
        self.ended = false;
        trace!("Resetting lexer:\n{}", self.text.join(""));
    }

    fn current_grapheme(&self) -> Result<&str> {
        match self.text.get(self.index) {
            Some(string) => Ok(string),
            None => Err(TFMTError::ExhaustedText.into()),
        }
    }

    fn current_string(&self, length: usize) -> Result<String> {
        let bound = std::cmp::min(self.text.len(), self.index + length);
        match self.text.get(self.index..bound) {
            Some(slice) => Ok(slice.join("")),
            None => Err(TFMTError::ExhaustedText.into()),
        }
    }

    fn test_current_string(&self, string: &str) -> bool {
        match self.current_string(string.len()) {
            Ok(current) => current == string,
            Err(_) => false,
        }
    }

    fn advance(&mut self) -> Result<()> {
        // Handle newline
        if self.current_grapheme()? == "\n" {
            self.line_no += 1;
            self.col_no = 1;
        } else {
            self.col_no += 1;
        }

        // Handle Index
        self.index += 1;

        Ok(())
    }

    fn advance_times(&mut self, times: u64) -> Result<()> {
        for _ in 1..=times {
            self.advance()?;
        }
        Ok(())
    }

    fn crawl(
        &mut self,
        terminators: Vec<&str>,
        discard_terminator: bool,
        terminate_on_eof: bool,
        skip_graphemes: u64,
    ) -> Result<String> {
        debug!("crawl(terminators: {:?}, discard_terminator: {}, terminate_on_eof: {}, skip_graphemes: {})",
        &terminators,
        &discard_terminator,
        &terminate_on_eof,
        &skip_graphemes,);

        self.advance_times(skip_graphemes)?;

        let mut string = String::new();

        'outer: loop {
            match self.current_grapheme() {
                Ok(grapheme) => {
                    for terminator in &terminators {
                        if self.test_current_string(terminator) {
                            if discard_terminator {
                                self.advance_times(terminator.len() as u64).expect("terminator somehow goes beyond text bounds!");
                            }
                            break 'outer;
                        }
                    }
                    string.push_str(grapheme)
                }
                Err(_) => {
                    if !terminate_on_eof {
                        let err_str = "Crawl reached EOF before terminator!";
                        error!("{}", err_str);
                        return Err(TFMTError::Crawler(err_str.into()).into());
                    } else {
                        break;
                    }
                }
            }
            self.advance()?;
        }
        trace!("Produced \"{}\"", string);
        Ok(string)
    }

    fn handle_string(&mut self, multiline: bool) -> Result<String> {
        let quote = self
            .current_grapheme()
            .expect("Checked in handle_bounded. Should never panic.")
            .to_string();

        let skip_graphemes = if multiline { 3 } else { 1 };

        let string =
            self.crawl(vec![quote.as_ref()], true, false, skip_graphemes)?;

        for grapheme in &token::FORBIDDEN_GRAPHEMES {
            if string.contains(grapheme) {
                return Err(TFMTError::Lexer(format!(
                    "String contains forbidden grapheme {:?}!",
                    grapheme
                ))
                .into());
            }
        }

        Ok(string)
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>> {
        let current_grapheme = &self.current_grapheme()?;

        let exp_string =
            "Should never panic, all TokenTypes are in TOKEN_TYPE_STRING_MAP.";
        let quotes = [
            TOKEN_TYPE_STRING_MAP
                .get_by_left(&TokenType::QuoteDouble)
                .expect(exp_string),
            TOKEN_TYPE_STRING_MAP
                .get_by_left(&TokenType::QuoteSingle)
                .expect(exp_string),
        ];

        let single_line_comment = TOKEN_TYPE_STRING_MAP
            .get_by_left(&TokenType::Hash)
            .expect(exp_string);
        let multiline_comment_start = TOKEN_TYPE_STRING_MAP
            .get_by_left(&TokenType::SlashAsterisk)
            .expect(exp_string);
        let multiline_comment_end = TOKEN_TYPE_STRING_MAP
            .get_by_left(&TokenType::AsteriskSlash)
            .expect(exp_string);

        if quotes.contains(&current_grapheme) {
            let multiline = self
                .test_current_string(&format!("{0}{0}{0}", current_grapheme));

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::String,
                Some(self.handle_string(multiline)?),
            )))
        } else if current_grapheme == single_line_comment {
            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(
                    vec!["\n"],
                    true,
                    true,
                    single_line_comment.len() as u64,
                )?),
            )))
        } else if self.test_current_string(multiline_comment_start) {
            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(
                    vec![multiline_comment_end],
                    true,
                    false,
                    multiline_comment_end.len() as u64,
                )?),
            )))
        } else {
            Ok(None)
        }
    }

    fn handle_reserved(&mut self) -> Result<Option<Token>> {
        for string in token::RESERVED_STRINGS.iter() {
            if self.test_current_string(string) {
                let token = Token::new_type_from_string(
                    self.line_no,
                    self.col_no,
                    string,
                    None,
                )
                .expect("Uses string from TOKEN_TYPE_STRING_MAP, should always be safe.");
                self.advance_times(string.len() as u64)?;
                return Ok(Some(token));
            }
        }

        Ok(None)
    }

    fn handle_misc_tokens(&mut self) -> Result<Token> {
        let (line_no_start, col_no_start) = (self.line_no, self.col_no);

        let current_grapheme = self.current_grapheme()?;

        if current_grapheme.chars().all(|c| c.is_alphabetic())
            && self.test_current_string(&format!("{}:\\", current_grapheme))
        {
            // Drive
            let token = Token::new(
                line_no_start,
                col_no_start,
                TokenType::Drive,
                Some(
                    self.current_string(3)
                        .expect("Tested above, should not panic here."),
                ),
            );
            self.advance_times(3)?;

            Ok(token)
        } else {
            // ID
            let mut terminators: Vec<&str> = Vec::new();
            for string in RESERVED_STRINGS.iter() {
                terminators.push(*string);
            }
            terminators.push(" ");
            terminators.push("\t");
            terminators.push("\n");
            terminators.push("\r");

            let value = self.crawl(terminators, false, true, 0)?;

            if value.starts_with(|c: char| c.is_alphabetic())
                && value.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                Ok(Token::new(
                    line_no_start,
                    col_no_start,
                    TokenType::ID,
                    Some(value),
                ))
            } else if value.chars().all(|c| c.is_numeric()) {
                Ok(Token::new(
                    line_no_start,
                    col_no_start,
                    TokenType::Integer,
                    Some(value),
                ))
            } else {
                Err(TFMTError::Tokenize(value).into())
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        let token = {
            match self.current_grapheme() {
                Err(err) => {
                    if self.ended {
                        return Err(err);
                    } else {
                        self.ended = true;
                        Token::new(
                            self.line_no,
                            self.col_no,
                            TokenType::EOF,
                            None,
                        )
                    }
                }
                Ok(current_grapheme) => {
                    if current_grapheme.chars().all(|c| c.is_whitespace()) {
                        self.advance()?;
                        self.next_token()?
                    } else if let Some(token) = self.handle_bounded()? {
                        token
                    } else if let Some(token) = self.handle_reserved()? {
                        token
                    } else {
                        self.handle_misc_tokens()?
                    }
                }
            }
        };

        // if let Some(token) = &token {
        //     trace!("{:?}", token);
        // }
        Ok(token)
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            None
        } else {
            Some(self.next_token())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    static DOUBLE_QUOTED_STRING: &str = "\"This is a double-quoted string\"";
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";
    static SINGLE_LINE_COMMENT: &str = "# This is a single line comment!\n";
    static MULTILINE_COMMENT: &str = "/* This is a \n multiline comment. */";
    static STRING_WITH_FORBIDDEN_GRAPHEMES: &str =
        "\"This \\ is / a string ~ with * forbidden graphemes.\"";

    fn slice_ends(string: &str, left: usize, right: usize) -> &str {
        &string[left..string.len() - right]
    }

    fn dequote(string: &str) -> &str {
        slice_ends(&string, 1, 1)
    }

    fn create_lexer(string: &str) -> Lexer {
        Lexer::new(&string)
    }

    fn run_lexer(string: &str, pop_eof: bool) -> Result<Vec<Token>> {
        let mut lex = Lexer::new(&string);

        let mut tokens: Vec<Token> = Vec::new();
        while let Ok(token) = lex.next_token() {
            tokens.push(token);
        }

        if pop_eof {
            tokens.pop();
        }

        Ok(tokens)
    }

    fn lexer_test(string: &str, reference: Vec<Token>) -> Result<()> {
        let tokens = run_lexer(string, true)?;

        assert_eq!(tokens, reference);

        Ok(())
    }

    mod lexer {
        use super::*;
        use std::fs;
        use std::path;

        fn file_test(filename: &str) -> Result<()> {
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
        fn test_simple_input() -> Result<()> {
            file_test("simple_input.tfmt")
        }

        #[test]
        fn test_typical_input() -> Result<()> {
            file_test("typical_input.tfmt")
        }
    }

    mod handle_reserved {
        use super::*;

        fn reserved_test(string: &str, expected_type: TokenType) -> Result<()> {
            let mut lex = create_lexer(string);

            match lex.handle_reserved()? {
                Some(token) => {
                    if token.ttype == expected_type {
                        Ok(())
                    } else {
                        Err(anyhow!(
                            "{} was parsed as {}, not {}!",
                            string,
                            // ttypes are always safe!
                            TOKEN_TYPE_STRING_MAP
                                .get_by_left(&token.ttype)
                                .unwrap(),
                            TOKEN_TYPE_STRING_MAP
                                .get_by_left(&expected_type)
                                .unwrap(),
                        ))
                    }
                }
                None => Err(anyhow!("Unable to parse {} as Token!", string)),
            }
        }

        #[test]
        fn test_single_char_string() -> Result<()> {
            reserved_test("+", TokenType::Plus)?;
            reserved_test("-", TokenType::Hyphen)?;
            Ok(())
        }

        #[test]
        fn test_double_char_string() -> Result<()> {
            reserved_test("&&", TokenType::DoubleAmpersand)?;
            reserved_test("||", TokenType::DoubleVerticalBar)?;
            Ok(())
        }
    }

    mod handle_bounded {
        use super::*;

        #[test]
        fn test_double_quoted() -> Result<()> {
            let reference = vec![Token::new(
                1,
                1,
                TokenType::String,
                Some(String::from(dequote(DOUBLE_QUOTED_STRING))),
            )];
            lexer_test(DOUBLE_QUOTED_STRING, reference)
        }

        #[test]
        fn test_single_quoted() -> Result<()> {
            let reference = vec![Token::new(
                1,
                1,
                TokenType::String,
                Some(String::from(dequote(SINGLE_QUOTED_STRING))),
            )];
            lexer_test(SINGLE_QUOTED_STRING, reference)
        }

        #[test]
        fn test_string_with_forbidden_graphemes() -> Result<()> {
            match run_lexer(STRING_WITH_FORBIDDEN_GRAPHEMES, false) {
                Ok(tokens) => Err(anyhow!("Lexer did not error on forbidden characters, returned {:?}", tokens)),
                Err(err) => {
                    if err.to_string().contains("forbidden grapheme") {
                        Ok(())
                    } else {
                        Err(anyhow!("Unrelated error {:?}!", err))
                    }
                }
            }
        }
    }

    mod handle_misc_tokens {
        use super::*;

        #[test]
        fn test_id() -> Result<()> {
            lexer_test(
                "id",
                vec![Token::new(1, 1, TokenType::ID, Some(String::from("id")))],
            )
        }

        #[test]
        fn test_drive() -> Result<()> {
            lexer_test(
                "D:\\",
                vec![Token::new(
                    1,
                    1,
                    TokenType::Drive,
                    Some(String::from("D:\\")),
                )],
            )
        }

        #[test]
        fn test_integer() -> Result<()> {
            lexer_test(
                "1",
                vec![Token::new(
                    1,
                    1,
                    TokenType::Integer,
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
            terminators: Vec<&str>,
            discard_terminator: bool,
            terminate_on_eof: bool,
            skip_graphemes: u64,
        ) -> Result<()> {
            let mut lex = Lexer::new(&string);

            let output = lex.crawl(
                terminators,
                discard_terminator,
                terminate_on_eof,
                skip_graphemes,
            )?;

            assert_eq!(output.trim(), reference.trim());

            Ok(())
        }

        fn string_test(string: &str) -> Result<()> {
            let string = String::from(string);
            let reference = dequote(&string);
            let terminator = string.chars().next().unwrap().to_string();

            crawler_test(
                &string,
                reference,
                vec![terminator.as_ref()],
                true,
                false,
                1,
            )
        }

        #[test]
        fn test_double_quoted() -> Result<()> {
            string_test(DOUBLE_QUOTED_STRING)
        }

        #[test]
        fn test_single_quoted() -> Result<()> {
            string_test(SINGLE_QUOTED_STRING)
        }

        #[test]
        fn test_single_line_comment() -> Result<()> {
            crawler_test(
                &String::from(SINGLE_LINE_COMMENT),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 0),
                vec!["\n"],
                true,
                true,
                1,
            )?;

            crawler_test(
                &String::from(slice_ends(&SINGLE_LINE_COMMENT, 0, 1)),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 0),
                vec!["\n"],
                true,
                true,
                1,
            )?;

            Ok(())
        }

        #[test]
        fn test_multiline_comment() -> Result<()> {
            crawler_test(
                &String::from(MULTILINE_COMMENT),
                slice_ends(&MULTILINE_COMMENT, 2, 2),
                vec![*TOKEN_TYPE_STRING_MAP
                    .get_by_left(&TokenType::AsteriskSlash)
                    .unwrap()],
                true,
                false,
                2,
            )
        }
    }
}
