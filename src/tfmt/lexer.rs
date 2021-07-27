use std::convert::{TryFrom, TryInto};
use std::iter::Iterator;
use std::path::Path;
use std::str::FromStr;

use log::{error, trace};
use normalize_line_endings::normalized;
use unicode_segmentation::UnicodeSegmentation;

use super::token::{self, Token, TokenType};
use crate::error::LexerError;

type Result<T> = std::result::Result<T, LexerError>;
pub type LexerResult = std::result::Result<Token, LexerError>;

/// Lexer takes a string and returns [Token]s
pub struct Lexer {
    /// Text to analyze, separated into Unicode Graphemes
    text: Vec<String>,
    /// Current index into `text`
    index: usize,
    /// Current line number of `text`
    line_no: u64,
    /// Current column number of `text`
    col_no: u64,
    /// [Lexer] status
    ended: bool,
}

impl FromStr for Lexer {
    type Err = LexerError;
    fn from_str(text: &str) -> Result<Self> {
        let normalized_text: String = normalized(text.trim().chars()).collect();

        if normalized_text.is_empty() {
            Err(LexerError::Generic(
                "Text provided to lexer was empty!".to_string(),
            ))
        } else {
            trace!("Creating lexer:\n{}", normalized_text);

            Ok(Lexer {
                text: normalized_text
                    .graphemes(true)
                    .map(String::from)
                    .collect(),
                index: 0,
                line_no: 1,
                col_no: 1,
                ended: false,
            })
        }
    }
}

impl TryFrom<&Path> for Lexer {
    type Error = LexerError;

    fn try_from(path: &Path) -> Result<Self> {
        Lexer::from_str(match &std::fs::read_to_string(path) {
            Ok(string) => string,
            Err(err) => {
                return Err(LexerError::Generic(format!(
                    "Unable to read from path \"{:?}\": {}",
                    path, err
                )))
            }
        })
    }
}

impl Lexer {
    // pub fn reset(&mut self) {
    //     self.index = 0;
    //     self.line_no = 1;
    //     self.col_no = 1;
    //     self.ended = false;
    //     trace!("Resetting lexer:\n{}", self.text.join(""));
    // }

    /// Returns [UnicodeSegmentation::Grapheme] pointed to by [Lexer.index]
    fn current_grapheme(&self) -> Result<&str> {
        match self.text.get(self.index) {
            Some(string) => Ok(string),
            None => Err(LexerError::ExhaustedText),
        }
    }

    /// Returns `length`-character [String] from [Lexer.index]
    fn current_string(&self, length: usize) -> Option<String> {
        let bound = std::cmp::min(self.text.len(), self.index + length);
        self.text.get(self.index..bound).map(|s| s.join(""))
    }

    /// Returns [true] if [current_string(string.len())] matches string
    fn test_current_string(&self, string: &str) -> bool {
        match self.current_string(string.len()) {
            Some(current) => current == string,
            None => false,
        }
    }

    /// Advances [Lexer.index] and handles [line_no] and [col_no]
    fn advance(&mut self) -> Result<()> {
        // Handle lines/columns
        // Newlines are normalized in Lexer::from_str()
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

    /// Call [Lexer::advance] `times` times
    fn advance_times(&mut self, times: u64) -> Result<()> {
        for _ in 0..times {
            self.advance()?;
        }
        Ok(())
    }

    /// Crawl [Lexer.text] until a designated terminator is reached
    ///
    /// # Arguments:
    ///
    /// * `terminators` - list of strings to stop on
    /// * `discard_terminator` - whether to advance past found terminator
    /// * `terminate_on_eof` - stop crawl() on EOF, else return Error
    fn crawl(
        &mut self,
        terminators: &[&str],
        discard_terminator: bool,
        terminate_on_eof: bool,
    ) -> Result<String> {
        let mut string = String::new();

        'outer: loop {
            match self.current_grapheme() {
                Ok(grapheme) => {
                    for terminator in terminators {
                        if self.test_current_string(terminator) {
                            if discard_terminator {
                                // "test_current_string(terminator) == true,
                                // so unwrap() should always succeed!"
                                self.advance_times(
                                    terminator.len().try_into()?,
                                )
                                .unwrap();
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
                        return Err(LexerError::CrawlerEOF(err_str));
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        trace!("crawl() produced \"{}\"", string);
        Ok(string)
    }

    /// Prepare [Lexer::crawl] for reading a string.
    fn handle_string(&mut self, multiline: bool) -> Result<String> {
        let quote_length = if multiline { 3 } else { 1 };
        let quote = self.current_grapheme()?.repeat(quote_length.try_into()?);

        // Number of quotes is verified in Lexer::handle_bounded,
        // so unwrap should never fail.
        self.advance_times(quote_length).unwrap();

        let string = self.crawl(&[quote.as_ref()], true, false)?;

        for grapheme in &token::FORBIDDEN_GRAPHEMES {
            if string.contains(grapheme) {
                return Err(LexerError::ForbiddenGrapheme(
                    grapheme.to_string(),
                ));
            }
        }

        Ok(string)
    }

    /// Handle bounded [Token]s such as strings and comments
    fn handle_bounded(&mut self) -> Result<Option<Token>> {
        let quotes = [
            TokenType::QuoteDouble.as_str(),
            TokenType::QuoteSingle.as_str(),
        ];

        let single_line_comment = TokenType::Hash.as_str();
        let multiline_comment_start = TokenType::SlashAsterisk.as_str();
        let multiline_comment_end = TokenType::AsteriskSlash.as_str();

        let current_grapheme = &self.current_grapheme()?;

        // advance_times amount is always based on current_grapheme length,
        // so unwrap should never fail.

        if quotes.contains(&current_grapheme) {
            let multiline = self
                .test_current_string(&format!("{0}{0}{0}", current_grapheme));

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::String,
                Some(self.handle_string(multiline)?),
            )?))
        } else if current_grapheme == &single_line_comment {
            self.advance_times(single_line_comment.len().try_into()?)
                .unwrap();

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(&["\n"], true, true)?),
            )?))
        } else if self.test_current_string(multiline_comment_start) {
            self.advance_times(multiline_comment_start.len().try_into()?)
                .unwrap();

            Ok(Some(Token::new(
                self.line_no,
                self.col_no,
                TokenType::Comment,
                Some(self.crawl(&[multiline_comment_end], true, false)?),
            )?))
        } else {
            Ok(None)
        }
    }

    /// Handle [Token]s involving reserved strings
    fn handle_reserved(&mut self) -> Result<Option<Token>> {
        for string in TokenType::reserved_strings() {
            if self.test_current_string(string) {
                let token = Token::new_type_from_string(
                    self.line_no,
                    self.col_no,
                    string,
                    None,
                )
                // Uses string from TokenType::string_map(), unwrap should
                // always be safe.
                .unwrap();
                self.advance_times(string.len().try_into()?)?;
                return Ok(Some(token));
            }
        }

        Ok(None)
    }

    /// Handle remaining [Token]s
    fn handle_misc_tokens(&mut self) -> Result<Token> {
        let (line_no_start, col_no_start) = (self.line_no, self.col_no);

        let mut terminators: Vec<&str> =
            TokenType::reserved_strings().iter().copied().collect();

        terminators.push(" ");
        terminators.push("\t");
        terminators.push("\n");
        terminators.push("\r");

        let value = self.crawl(&terminators, false, true)?;

        if value.starts_with(|c: char| c.is_alphabetic())
            && value.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            Ok(Token::new(
                line_no_start,
                col_no_start,
                TokenType::ID,
                Some(value),
            )?)
        } else if value.chars().all(|c| c.is_numeric()) {
            Ok(Token::new(
                line_no_start,
                col_no_start,
                TokenType::Integer,
                Some(value),
            )?)
        } else {
            Err(LexerError::Tokenize(value))
        }
    }

    /// Return next [Token], if any
    pub fn next_token(&mut self) -> LexerResult {
        let grapheme = match self.current_grapheme() {
            Ok(grapheme) => grapheme,
            Err(LexerError::ExhaustedText) => {
                if self.ended {
                    return Err(LexerError::ExhaustedText);
                } else {
                    self.ended = true;
                    return Ok(Token::new(
                        self.line_no,
                        self.col_no,
                        TokenType::EOF,
                        None,
                    )?);
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
    use anyhow::{bail, Result};

    use std::str::FromStr;

    static DOUBLE_QUOTED_STRING: &str = "\"This is a double-quoted string\"";
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";
    static MULTILINE_STRING: &str = "'''This is a \n multiline string'''";
    static STRING_WITH_FORBIDDEN_GRAPHEMES: &str =
        "\"This | is ? a string ~ with * forbidden graphemes.\"";
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

    mod handle_reserved {
        use super::*;

        fn reserved_test(input: &str, expected_type: TokenType) -> Result<()> {
            let mut lex = Lexer::from_str(input)?;

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
                    bail!("reserved: unable to parse {} as Token", input)
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
            let mut lex = Lexer::from_str(input)?;

            match lex.handle_bounded()? {
                Some(token) => {
                    assert_eq!(
                        token.ttype, expected_type,
                        "bounded_type: got {:?}, expected {:?}",
                        token.ttype, expected_type
                    );

                    assert_eq!(
                        token.get_value_unchecked(),
                        expected_value,
                        "bounded_value: got {:?}, expected {:?}",
                        token.get_value_unchecked(),
                        expected_value
                    );

                    Ok(())
                }
                None => {
                    bail!("bounded: unable to parse {} as Token", input)
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
                Ok(tokens) => bail!("Lexer did not error on forbidden characters, returned {:?}", tokens),
                Err(err) => {
                    if err.to_string().contains("forbidden grapheme") {
                        Ok(())
                    } else {
                        bail!("Unrelated error {:?}!", err)
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
                        bail!("unterminated string {} did not return expected error!", string);
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
                    bail!("unterminated_comment {} did not return expected error!", UNTERMINATED_COMMENT);
                }
            }

            Ok(())
        }
    }

    mod handle_misc_tokens {
        use super::*;

        fn misc_test(
            input: &str,
            expected_type: TokenType,
            expected_value: Option<&str>,
        ) -> Result<()> {
            let mut lex = Lexer::from_str(input)?;

            let expected_value = expected_value.map(String::from);

            match lex.handle_misc_tokens() {
                Ok(token) => {
                    assert_eq!(
                        token.ttype, expected_type,
                        "misc_type: got {:?}, expected {:?}",
                        token.ttype, expected_type
                    );

                    if let Some(expected_value) = expected_value {
                        assert_eq!(
                            token.get_value_unchecked(),
                            expected_value,
                            "misc_value: got {:?}, expected {:?}",
                            token.get_value_unchecked(),
                            expected_value
                        );
                    }

                    Ok(())
                }
                Err(LexerError::Tokenize(_)) => {
                    bail!("misc: unable to parse {} as Token", input)
                }
                Err(err) => bail!(
                    "misc: unexpected error with input {}: {}",
                    input,
                    err
                ),
            }
        }
        #[test]
        fn test_id() -> Result<()> {
            misc_test("id", TokenType::ID, Some("id"))
        }

        #[test]
        fn test_integer() -> Result<()> {
            misc_test("1", TokenType::Integer, Some("1"))
        }
    }

    mod crawler {
        use super::*;

        fn crawler_test(
            string: &str,
            reference: &str,
            terminators: &[&str],
            discard_terminator: bool,
            terminate_on_eof: bool,
            skip_graphemes: u64,
        ) -> Result<()> {
            let mut lex = Lexer::from_str(&string)?;

            lex.advance_times(skip_graphemes)?;

            let output =
                lex.crawl(&terminators, discard_terminator, terminate_on_eof)?;

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
                &[terminator.as_ref()],
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
                &["\n"],
                true,
                true,
                1,
            )?;

            // Terminate on EOF
            crawler_test(
                &String::from(SINGLE_LINE_COMMENT),
                slice_ends(&SINGLE_LINE_COMMENT, 1, 1),
                &["\n"],
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
                &[TokenType::AsteriskSlash.as_str()],
                true,
                false,
                2,
            )
        }
    }
}
