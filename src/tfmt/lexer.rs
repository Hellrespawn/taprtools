use std::iter::Iterator;

use log::{error, trace};
use unicode_segmentation::UnicodeSegmentation;

use super::token::{
    self, Token, TokenType, RESERVED_STRINGS, TOKEN_TYPE_STRING_MAP,
};
use crate::error::TFMTError;

type Result<T> = std::result::Result<T, TFMTError>;

/// Lexer takes a string and returns [Token]s
pub struct Lexer {
    /// Text to analyze, separated into Unicode Graphemes
    text: Vec<String>,
    /// Current index into [text]
    index: usize,
    /// Current line number of [text]
    line_no: u64,
    /// Current column number of [text]
    col_no: u64,
    /// [Lexer] status
    ended: bool,
}

impl Lexer {
    pub fn new(text: &str) -> Result<Lexer> {
        if text.is_empty() {
            Err(TFMTError::Lexer(
                "Text provided to lexer was empty!".to_string(),
            ))
        } else {
            trace!("Creating lexer:\n{}", text);
            Ok(Lexer {
                text: UnicodeSegmentation::graphemes(text, true)
                    .map(String::from)
                    .collect(),
                index: 0,
                line_no: 1,
                col_no: 1,
                ended: false,
            })
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
            None => Err(TFMTError::ExhaustedText),
        }
    }

    fn current_string(&self, length: usize) -> Option<String> {
        let bound = std::cmp::min(self.text.len(), self.index + length);
        self.text.get(self.index..bound).map(|s| s.join(""))
    }

    fn test_current_string(&self, string: &str) -> bool {
        match self.current_string(string.len()) {
            Some(current) => current == string,
            None => false,
        }
    }

    fn advance(&mut self) -> Result<()> {
        // Handle lines/columns
        // FIXME Check for carriage return or \r\n?
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
    ) -> Result<String> {
        let mut string = String::new();

        'outer: loop {
            match self.current_grapheme() {
                Ok(grapheme) => {
                    for terminator in &terminators {
                        if self.test_current_string(terminator) {
                            if discard_terminator {
                                self.advance_times(terminator.len() as u64).expect("test_current_string(terminator) == true, so this should always succeed!");
                            }
                            break 'outer;
                        }
                    }

                    string += grapheme;
                    self.advance()?;
                }
                Err(err) => {
                    if !terminate_on_eof {
                        let err_str = format!("Crawl reached EOF before terminator! Original error: {}", err);
                        error!("{}", err_str);
                        return Err(TFMTError::Crawler(err_str));
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        trace!("crawl() produced \"{}\"", string);
        Ok(string)
    }

    fn handle_string(&mut self, multiline: bool) -> Result<String> {
        let quote_length = if multiline { 3 } else { 1 };
        let quote = self.current_grapheme()?.repeat(quote_length as usize);

        self.advance_times(quote_length).expect("Number of quotes is verified in previous function, this should never fail!");

        let string = self.crawl(vec![&quote], true, false)?;

        for grapheme in &token::FORBIDDEN_GRAPHEMES {
            if string.contains(grapheme) {
                return Err(TFMTError::Lexer(format!(
                    "String contains forbidden grapheme {:?}!",
                    grapheme
                )));
            }
        }

        Ok(string)
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>> {
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
            // FIXME Implemenet SlashSlash instead of this shit.
            .get_by_left(&TokenType::Hash)
            .expect(exp_string);
        let multiline_comment_start = TOKEN_TYPE_STRING_MAP
            .get_by_left(&TokenType::SlashAsterisk)
            .expect(exp_string);
        let multiline_comment_end = TOKEN_TYPE_STRING_MAP
            .get_by_left(&TokenType::AsteriskSlash)
            .expect(exp_string);

        let current_grapheme = &self.current_grapheme()?;

        let err_str =
            "Boundary length is verified above, this should never fail!";

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
            self.advance_times(single_line_comment.len() as u64)
                .expect(err_str);

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(vec!["\n"], true, true)?),
            )))
        } else if self.test_current_string(multiline_comment_start) {
            self.advance_times(multiline_comment_start.len() as u64)
                .expect(err_str);

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(vec![multiline_comment_end], true, false)?),
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

            let value = self.crawl(terminators, false, true)?;

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
                Err(TFMTError::Tokenize(value))
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        let grapheme = match self.current_grapheme() {
            Ok(grapheme) => grapheme,
            Err(TFMTError::ExhaustedText) => {
                if self.ended {
                    return Err(TFMTError::ExhaustedText);
                } else {
                    self.ended = true;
                    return Ok(Token::new(
                        self.line_no,
                        self.col_no,
                        TokenType::EOF,
                        None,
                    ));
                }
            }
            Err(err) => return Err(err),
        };

        if grapheme.chars().all(|c| c.is_whitespace()) {
            self.advance()?;
            self.next_token()
        } else if let Some(result) = self.handle_bounded().transpose() {
            result
        } else if let Some(result) = self.handle_reserved().transpose() {
            result
        } else {
            self.handle_misc_tokens()
        }
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
    use anyhow::{anyhow, Result};

    static DOUBLE_QUOTED_STRING: &str = "\"This is a double-quoted string\"";
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";
    static MULTILINE_STRING: &str = "'''This is a \n multiline string'''";
    static STRING_WITH_FORBIDDEN_GRAPHEMES: &str =
        "\"This \\ is / a string ~ with * forbidden graphemes.\"";
    static UNTERMINATED_STRING: &str = "\"This is an unterminated string";
    static UNTERMINATED_MULTILINE_STRING: &str =
        "'''This is an unterminated \n multiline string'";
    static SINGLE_LINE_COMMENT: &str = "# This is a single line comment!\n";
    static MULTILINE_COMMENT: &str = "/* This is a \n multiline comment. */";
    static UNTERMINATED_COMMENT: &str = "/* This is an unterminated comment";

    fn slice_ends(string: &str, left: usize, right: usize) -> &str {
        &string[left..string.len() - right]
    }

    fn dequote(string: &str) -> &str {
        slice_ends(&string, 1, 1)
    }

    fn create_lexer(input: &str) -> Result<Lexer, TFMTError> {
        Ok(Lexer::new(&input)?)
    }

    mod handle_reserved {
        use super::*;

        fn reserved_test(input: &str, expected_type: TokenType) -> Result<()> {
            let mut lex = create_lexer(input)?;

            match lex.handle_reserved()? {
                Some(token) => {
                    assert_eq!(
                        token.ttype, expected_type,
                        "reserved: got {:?}\texpected {:?}",
                        token.ttype, expected_type
                    );
                    Ok(())
                }
                None => {
                    Err(anyhow!("reserved: unable to parse {} as Token", input))
                }
            }
        }

        #[test]
        fn single_char() -> Result<()> {
            reserved_test("+", TokenType::Plus)?;
            reserved_test("-", TokenType::Hyphen)?;
            Ok(())
        }

        #[test]
        fn double_char() -> Result<()> {
            reserved_test("&&", TokenType::DoubleAmpersand)?;
            reserved_test("||", TokenType::DoubleVerticalBar)?;
            Ok(())
        }
    }

    mod handle_bounded {
        use super::*;

        fn bounded_test(
            input: &str,
            expected_type: TokenType,
            expected_value: &str,
        ) -> Result<()> {
            let mut lex = create_lexer(input)?;

            match lex.handle_bounded()? {
                Some(token) => {
                    assert_eq!(
                        token.ttype, expected_type,
                        "bounded_type: got {:?}, expected {:?}",
                        token.ttype, expected_type
                    );
                    if let Some(value) = token.value {
                        assert_eq!(
                            value, expected_value,
                            "bounded_value: got {:?}, expected {:?}",
                            value, expected_value
                        );
                        Ok(())
                    } else {
                        Err(anyhow!(
                            "bounded_value: no value, expected {:?}",
                            expected_value
                        ))
                    }
                }
                None => {
                    Err(anyhow!("bounded: unable to parse {} as Token", input))
                }
            }
        }

        #[test]
        fn string() -> Result<()> {
            for string in &[SINGLE_QUOTED_STRING, DOUBLE_QUOTED_STRING] {
                bounded_test(string, TokenType::String, dequote(string))?;
            }
            Ok(())
        }

        #[test]
        fn multiline_string() -> Result<()> {
            bounded_test(
                MULTILINE_STRING,
                TokenType::String,
                slice_ends(MULTILINE_STRING, 3, 3),
            )
        }

        #[test]
        fn forbidden_graphemes() -> Result<()> {
            match bounded_test(STRING_WITH_FORBIDDEN_GRAPHEMES, TokenType::String, "") {
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

        #[test]
        fn unterminated_string() -> Result<()> {
            for string in &[UNTERMINATED_STRING, UNTERMINATED_MULTILINE_STRING]
            {
                if let Err(err) = bounded_test(string, TokenType::String, "") {
                    if !err
                        .to_string()
                        .contains("reached EOF before terminator")
                    {
                        return  Err(anyhow!("unterminated string {} did not return expected error!", string));
                    }
                }
            }

            Ok(())
        }

        #[test]
        fn single_line_comment() -> Result<()> {
            bounded_test(
                SINGLE_LINE_COMMENT,
                TokenType::Comment,
                slice_ends(SINGLE_LINE_COMMENT, 1, 1),
            )
        }

        #[test]
        fn multiline_comment() -> Result<()> {
            bounded_test(
                MULTILINE_COMMENT,
                TokenType::Comment,
                slice_ends(MULTILINE_COMMENT, 2, 2),
            )
        }

        #[test]
        fn unterminated_comment() -> Result<()> {
            if let Err(err) =
                bounded_test(UNTERMINATED_COMMENT, TokenType::Comment, "")
            {
                if !err.to_string().contains("reached EOF before terminator") {
                    return  Err(anyhow!("unterminated_comment {} did not return expected error!", UNTERMINATED_COMMENT));
                }
            }

            Ok(())
        }
    }

    // mod handle_misc_tokens {
    //     use super::*;

    //     #[test]
    //     fn test_id() -> Result<()> {
    //         lexer_test(
    //             "id",
    //             vec![Token::new(1, 1, TokenType::ID, Some(String::from("id")))],
    //         )
    //     }

    //     #[test]
    //     fn test_drive() -> Result<()> {
    //         lexer_test(
    //             "D:\\",
    //             vec![Token::new(
    //                 1,
    //                 1,
    //                 TokenType::Drive,
    //                 Some(String::from("D:\\")),
    //             )],
    //         )
    //     }

    //     #[test]
    //     fn test_integer() -> Result<()> {
    //         lexer_test(
    //             "1",
    //             vec![Token::new(
    //                 1,
    //                 1,
    //                 TokenType::Integer,
    //                 Some(String::from("1")),
    //             )],
    //         )
    //     }
    // }

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
            let mut lex = Lexer::new(&string)?;

            lex.advance_times(skip_graphemes)?;

            let output =
                lex.crawl(terminators, discard_terminator, terminate_on_eof)?;

            let output = output.trim();
            let reference = reference.trim();

            assert_eq!(
                output.trim(),
                reference.trim(),
                "crawler: got {}, expected {}",
                output.trim(),
                reference.trim()
            );

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
        fn string() -> Result<()> {
            string_test(SINGLE_QUOTED_STRING)?;
            string_test(DOUBLE_QUOTED_STRING)?;
            Ok(())
        }

        #[test]
        fn single_line_comment() -> Result<()> {
            // Terminate on \n
            crawler_test(
                &String::from(SINGLE_LINE_COMMENT),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 0),
                vec!["\n"],
                true,
                true,
                1,
            )?;

            // Terminate on EOF
            crawler_test(
                &String::from(SINGLE_LINE_COMMENT),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 1),
                vec!["\n"],
                true,
                true,
                1,
            )?;

            Ok(())
        }

        #[test]
        fn multiline_comment() -> Result<()> {
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
