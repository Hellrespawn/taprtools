use super::error::{ErrorContext, LexerError};
use super::token::{Token, TokenType};
use crate::normalize_newlines;
use buffered_iterator::{buffered, BufferedIterator};
use log::{debug, trace};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

type Result<T> = std::result::Result<T, LexerError>;

/// Reads a string and returns a stream of [Token]s.
pub(crate) struct Lexer<'a> {
    /// Original text of Lexer.
    input_text: &'a str,
    buffer: BufferedIterator<Graphemes<'a>>,
    line_no: usize,
    col_no: usize,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = match self.buffer.peek() {
            Some(g) => g,
            None => return None,
        };

        if grapheme.chars().all(char::is_whitespace) {
            self.advance(1);
            self.next()
        } else {
            let option = self.handle_bounded().transpose().or_else(|| {
                self.handle_reserved()
                    .map(Ok)
                    .or_else(|| self.handle_misc().map(Ok))
            });
            trace!("{:#?}", option);
            option
        }
    }
}

impl<'a> Lexer<'a> {
    /// Attempt to create a [Lexer] from a string. Requires that the input
    /// does not contain carriage return characters (\r).
    pub(crate) fn new<S: AsRef<str>>(input_text: &'a S) -> Result<Self> {
        let input_text = input_text.as_ref();

        if let Some(cr_index) = input_text.find('\r') {
            #[allow(clippy::unnecessary_to_owned)]
            // Required to keep the reference in input_text
            let text_before_cr =
                normalize_newlines(&input_text[..cr_index].to_string());

            let newline_matches: Vec<usize> = text_before_cr
                .rmatch_indices('\n')
                .map(|(i, _)| i)
                .collect();

            let line_no = newline_matches.len();

            let col_no = if line_no > 0 {
                cr_index - newline_matches[0]
            } else {
                cr_index
            };

            return Err(LexerError::InputContainsCr(ErrorContext::new(
                input_text,
                1 + line_no,
                1 + col_no,
            )));
        }

        let lexer = Self {
            input_text,
            buffer: buffered(input_text.graphemes(true)),
            line_no: 1,
            col_no: 1,
        };

        debug!("Creating lexer:\n{}", input_text);
        Ok(lexer)
    }

    pub(crate) fn input_text(&self) -> &str {
        self.input_text
    }

    fn current_context(&self) -> ErrorContext {
        ErrorContext::new(self.input_text, self.line_no, self.col_no)
    }

    fn advance(&mut self, amount: usize) {
        for _ in 0..amount {
            let current_grapheme = self.buffer.next();

            if current_grapheme == Some("\n") {
                self.line_no += 1;
                self.col_no = 1;
            } else {
                self.col_no += 1;
            }
        }
    }

    fn crawl<P>(&mut self, predicate: P, discard: usize) -> Option<String>
    where
        P: Fn(&&str) -> bool,
    {
        if let Some(i) = self.buffer.findi(predicate) {
            let string = self.buffer.peekn(i).join("");
            trace!(r#"Crawl: {}"#, string);
            self.advance(i + discard);
            Some(string)
        } else {
            None
        }
    }

    fn handle_single_line_string(&mut self, quote: &str) -> Result<String> {
        let mut ctx = self.current_context();

        self.advance(1);

        match self.crawl(|s| *s == quote, 1) {
            None => Err(LexerError::ExhaustedText(ctx, quote.to_string())),
            Some(s) => {
                if let Some(i) = s.find('\n') {
                    ctx.col_no += i;
                    Err(LexerError::NewlineInString(ctx))
                } else {
                    Ok(s)
                }
            }
        }
    }

    fn handle_multiline_string(&mut self, quote: &str) -> Result<String> {
        let ctx = self.current_context();

        self.advance(3);

        let triple_quote = quote.repeat(3);

        let mut string = String::new();

        loop {
            match self.crawl(|s| *s == quote, 0) {
                None => {
                    return Err(LexerError::ExhaustedText(ctx, triple_quote))
                }
                Some(s) => string += &s,
            }

            let peek = self.buffer.peekn(3);

            if peek.len() != 3 {
                return Err(LexerError::WrongTerminatorAtEOF {
                    context: self.current_context(),
                    found: quote.to_string(),
                    expected: triple_quote,
                });
            } else if peek.join("") == triple_quote {
                break;
            }

            string += quote;
            self.advance(1);
        }

        self.advance(3);

        Ok(string)
    }

    fn handle_string(&mut self) -> Result<Token> {
        let ctx = self.current_context();

        // self.next() already checks for None, so this unwrap should be safe.
        debug_assert!(self.buffer.peek().is_some());

        let quote = *self.buffer.peek().unwrap();

        let string = if self.buffer.peekn(3).iter().all(|s| *s == quote) {
            self.handle_multiline_string(quote)
        } else {
            self.handle_single_line_string(quote)
        }?;

        for grapheme in string.graphemes(true) {
            if crate::FORBIDDEN_GRAPHEMES.contains(&grapheme) {
                return Err(LexerError::ForbiddenGrapheme(
                    ctx,
                    grapheme.to_string(),
                ));
            }
        }

        Ok(Token::new(
            TokenType::String(string),
            ctx.line_no,
            ctx.col_no,
        ))
    }

    fn handle_single_line_comment(&mut self) -> String {
        if let Some(s) = self.crawl(|s| *s == "\n", 0) {
            // Ends on newline
            s
        } else {
            // Ends on EOF, Skips final "/"
            let mut string = String::new();
            while let Some(grapheme) = self.buffer.peek() {
                string += grapheme;
                self.advance(1);
            }

            string
        }
    }

    fn handle_multiline_comment(&mut self) -> Result<String> {
        let mut string = String::new();

        let ctx = self.current_context();

        loop {
            match self.crawl(|s| *s == "*", 0) {
                None => {
                    return Err(LexerError::ExhaustedText(
                        ctx,
                        "*/".to_string(),
                    ))
                }
                Some(s) => string += &s,
            }

            let peek = self.buffer.peekn(2);

            if peek.len() != 2 {
                return Err(LexerError::WrongTerminatorAtEOF {
                    context: self.current_context(),
                    found: "*".to_string(),
                    expected: "*/".to_string(),
                });
            } else if peek == ["*", "/"] {
                break;
            }

            string += "*";
            self.advance(1);
        }

        self.advance(2);

        Ok(string)
    }

    fn handle_comment(&mut self) -> Result<Option<Token>> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        if self.buffer.peekn(2) == ["/", "/"] {
            self.advance(2);
            Ok(Some(Token::new(
                TokenType::Comment(self.handle_single_line_comment()),
                line_no,
                col_no,
            )))
        } else if self.buffer.peekn(2) == ["/", "*"] {
            self.advance(2);
            Ok(Some(Token::new(
                TokenType::Comment(self.handle_multiline_comment()?),
                line_no,
                col_no,
            )))
        } else {
            Ok(None)
        }
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>> {
        // self.next() already checks for None, so this unwrap should be safe.
        debug_assert!(self.buffer.peek().is_some());

        let current_grapheme = self.buffer.peek().unwrap();

        if ["\"", "'"].contains(current_grapheme) {
            Ok(Some(self.handle_string()?))
        } else if self.buffer.peekn(1) == ["/"] {
            self.handle_comment()
        } else {
            Ok(None)
        }
    }

    fn handle_reserved(&mut self) -> Option<Token> {
        for i in (0..TokenType::LOOKAHEAD_DEPTH).rev() {
            // self.next() already checks for None, so this unwrap should be safe.
            let string = self.buffer.peekn(i + 1).join("");

            if let Ok(t) = Token::from_str(&string, self.line_no, self.col_no) {
                self.advance(i + 1);
                return Some(t);
            }
        }

        None
    }

    fn handle_misc(&mut self) -> Option<Token> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        let string = self
            //.crawl(|s| [" ", "\t", "\n"].contains(s), 0)
            .crawl(|s| s.chars().any(|c| !(c.is_alphanumeric() || c == '_')), 0)
            .unwrap_or_else(|| {
                let mut string = String::new();
                while let Some(grapheme) = self.buffer.peek() {
                    string += grapheme;
                    self.advance(1);
                }
                string
            });

        if string.starts_with(char::is_alphabetic)
            && string.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            Some(Token::new(TokenType::ID(string), line_no, col_no))
        } else if string.chars().all(char::is_numeric) {
            // All chars are numeric, so should always be parsable.
            Some(Token::new(
                TokenType::Integer(string.parse().unwrap()),
                line_no,
                col_no,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{bail, Result};

    static DOUBLE_QUOTED_STRING: &str = r#""This is a double-quoted string""#;
    static SINGLE_QUOTED_STRING: &str = "'This is a single-quoted string'";
    static NEWLINE_IN_STRING: &str =
        "'This is a string \n with a newline in it!'";
    static MULTILINE_STRING: &str = "'''This is a \n multiline ' string'''";
    static STRING_WITH_FORBIDDEN_GRAPHEMES: &str =
        r#""This | is ? a string ~ with * forbidden graphemes.""#;
    static UNTERMINATED_STRING: &str = r#""This is an unterminated string"#;
    static UNTERMINATED_MULTILINE_STRING: &str =
        "'''This is an unterminated \n multiline string'";
    static SINGLE_LINE_COMMENT: &str = "// This is a single line comment!\n";
    static MULTILINE_COMMENT: &str = "/* This is a \n multiline * comment. */";
    static UNTERMINATED_COMMENT: &str =
        "/* This is an * unterminated \n comment";

    fn slice_ends(string: &str, left: usize, right: usize) -> &str {
        &string[left..string.len() - right]
    }

    fn dequote(string: &str) -> &str {
        slice_ends(string, 1, 1)
    }

    fn generic_test(
        input: &str,
        expected_type: &TokenType,
        name: &str,
        option: Option<Token>,
    ) -> Result<()> {
        match option {
            Some(token) => {
                assert_eq!(token.token_type(), expected_type,);
                Ok(())
            }
            None => {
                bail!(r#"{}: unable to parse "{}" as Token"#, name, input)
            }
        }
    }

    mod handle_reserved {
        use super::*;

        fn reserved_test(input: &str, expected_type: &TokenType) -> Result<()> {
            let mut lex = Lexer::new(&input)?;

            generic_test(
                input,
                expected_type,
                "reserved_test",
                lex.handle_reserved(),
            )
        }

        #[test]
        fn new_lexer_single_char_test() -> Result<()> {
            reserved_test("+", &TokenType::Plus)?;
            reserved_test("-", &TokenType::Hyphen)?;
            Ok(())
        }

        #[test]
        fn new_lexer_double_char_test() -> Result<()> {
            reserved_test("&&", &TokenType::DoubleAmpersand)?;
            reserved_test("||", &TokenType::DoubleVerticalBar)?;
            Ok(())
        }
    }

    mod handle_bounded {
        use super::*;

        fn bounded_test(input: &str, expected_type: &TokenType) -> Result<()> {
            let mut lex = Lexer::new(&input)?;

            generic_test(
                input,
                expected_type,
                "bounded_test",
                lex.handle_bounded()?,
            )
        }

        fn error_test(
            result: Result<()>,
            expected_error: &LexerError,
            name: &str,
        ) -> Result<()> {
            if let Err(err) = result {
                match err.downcast_ref::<LexerError>() {
                    Some(err) if err == expected_error => Ok(()),
                    Some(err) => {
                        bail!(
                            "Unexpected error in {}():\n{}\nExpected:\n{}",
                            name,
                            err,
                            expected_error
                        )
                    }
                    None => bail!("Error in downcasting error!"),
                }
            } else {
                bail!("{}() did not return an error as expected!", name)
            }
        }

        #[test]
        fn new_lexer_string_test() -> Result<()> {
            for string in &[SINGLE_QUOTED_STRING, DOUBLE_QUOTED_STRING] {
                bounded_test(
                    string,
                    &TokenType::String(dequote(string).to_string()),
                )?;
            }
            Ok(())
        }

        #[test]
        fn new_lexer_multiline_string_test() -> Result<()> {
            bounded_test(
                MULTILINE_STRING,
                &TokenType::String(
                    slice_ends(MULTILINE_STRING, 3, 3).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_newline_in_string_test() -> Result<()> {
            error_test(
                bounded_test(
                    NEWLINE_IN_STRING,
                    &TokenType::String(String::new()),
                ),
                &LexerError::NewlineInString(ErrorContext::new(
                    NEWLINE_IN_STRING,
                    1,
                    18,
                )),
                "newline_in_string_test",
            )
        }

        #[test]
        fn new_lexer_forbidden_graphemes_test() -> Result<()> {
            error_test(
                bounded_test(
                    STRING_WITH_FORBIDDEN_GRAPHEMES,
                    &TokenType::String(String::new()),
                ),
                &LexerError::ForbiddenGrapheme(
                    ErrorContext::new(STRING_WITH_FORBIDDEN_GRAPHEMES, 1, 1),
                    "|".to_string(),
                ),
                "forbidden_graphemes_test",
            )
        }

        #[test]
        fn new_lexer_unterminated_single_line_string_test() -> Result<()> {
            error_test(
                bounded_test(
                    UNTERMINATED_STRING,
                    &TokenType::String(String::new()),
                ),
                &LexerError::ExhaustedText(
                    ErrorContext::new(UNTERMINATED_STRING, 1, 1),
                    "\"".to_string(),
                ),
                "unterminated_single_line_string_test",
            )
        }

        #[test]
        fn new_lexer_unterminated_multiline_string_test() -> Result<()> {
            error_test(
                bounded_test(
                    UNTERMINATED_MULTILINE_STRING,
                    &TokenType::String(String::new()),
                ),
                &LexerError::WrongTerminatorAtEOF {
                    context: ErrorContext::new(
                        UNTERMINATED_MULTILINE_STRING,
                        2,
                        18,
                    ),
                    found: "'".to_string(),
                    expected: "'''".to_string(),
                },
                "unterminated_multiline_string_test",
            )?;

            error_test(
                bounded_test(
                    &(UNTERMINATED_MULTILINE_STRING.to_string() + "abcd"),
                    &TokenType::String(String::new()),
                ),
                &LexerError::ExhaustedText(
                    ErrorContext::new(
                        UNTERMINATED_MULTILINE_STRING.to_string() + "abcd",
                        1,
                        1,
                    ),
                    "'''".to_string(),
                ),
                "unterminated_multiline_string_test",
            )
        }

        #[test]
        fn new_lexer_single_line_comment_eof_test() -> Result<()> {
            bounded_test(
                slice_ends(SINGLE_LINE_COMMENT, 0, 1),
                &TokenType::Comment(
                    slice_ends(SINGLE_LINE_COMMENT, 2, 1).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_single_line_comment_newline_test() -> Result<()> {
            bounded_test(
                SINGLE_LINE_COMMENT,
                &TokenType::Comment(
                    slice_ends(SINGLE_LINE_COMMENT, 2, 1).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_multiline_comment_test() -> Result<()> {
            bounded_test(
                MULTILINE_COMMENT,
                &TokenType::Comment(
                    slice_ends(MULTILINE_COMMENT, 2, 2).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_unterminated_comment_test() -> Result<()> {
            error_test(
                bounded_test(
                    UNTERMINATED_COMMENT,
                    &TokenType::Comment(String::new()),
                ),
                &LexerError::ExhaustedText(
                    ErrorContext::new(UNTERMINATED_COMMENT, 1, 3),
                    "*/".to_string(),
                ),
                "unterminated_comment_test",
            )
        }
    }

    mod handle_misc_tokens {
        use super::*;

        fn misc_test(input: &str, expected_type: &TokenType) -> Result<()> {
            let mut lex = Lexer::new(&input)?;

            generic_test(input, expected_type, "misc_test", lex.handle_misc())
        }
        #[test]
        fn new_lexer_id_test() -> Result<()> {
            misc_test("id", &TokenType::ID("id".to_string()))
        }

        #[test]
        fn new_lexer_integer_test() -> Result<()> {
            misc_test("1", &TokenType::Integer(1))
        }
    }
}
