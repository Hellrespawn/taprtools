use super::buffered_iterator::{Buffered, BufferedIterator};
use super::new_token::{self, Token, TokenType};
use crate::error::LexerError;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use log::{debug, trace};

type Result<T> = std::result::Result<T, LexerError>;

pub struct Lexer<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    //input_text: String,
    iterator: BufferedIterator<I>,
    current_grapheme: Option<&'a str>,
    line_no: u64,
    col_no: u64,
}

impl<'a, I> Iterator for Lexer<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = match self.current_grapheme {
            Some(g) => g,
            None => return None,
        };

        let option = if grapheme.chars().all(|c| c.is_whitespace()) {
            self.advance(1);
            self.next()
        } else if let Some(result) = self.handle_bounded().transpose() {
            Some(result)
        } else if let Some(token) = self.handle_reserved() {
            Some(Ok(token))
        } else if let Some(token) = self.handle_misc() {
            Some(Ok(token))
        } else {
            None
        };

        trace!("{:#?}", option);

        self.advance(1);
        option
    }
}

impl<'a> Lexer<'a, Graphemes<'a>> {
    pub fn new<S: AsRef<str>>(input_text: &S) -> Lexer<Graphemes> {
        let input_text = input_text.as_ref();
        let mut lexer = Lexer {
            //input_text: String::from(input_text),
            iterator: input_text.graphemes(true).buffered(),
            current_grapheme: None,
            line_no: 0,
            col_no: 0,
        };
        lexer.advance(1);
        debug!("Creating lexer:\n{}", input_text);
        lexer
    }
}

impl<'a, I> Lexer<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    fn advance(&mut self, amount: usize) {
        for _ in 0..amount {
            self.current_grapheme = self.iterator.next();

            // FIXME Normalize newlines!
            if self.current_grapheme.unwrap_or("") == "\n" {
                self.line_no = 0;
                self.col_no += 1;
            } else {
                self.col_no += 1;
            }
        }
    }

    fn lookahead(&mut self, amount: usize) -> &[&str] {
        self.iterator.peekn(amount)
    }

    // fn test_lookahead(&mut self, string: &str) -> bool {
    //     self.iterator.peekn(string.len()).join("") == string
    // }

    pub fn crawl<P>(&mut self, predicate: P, discard: usize) -> Option<String>
    where
        P: Fn(&I::Item) -> bool,
    {
        if let Some(i) = self.iterator.findi(predicate) {
            let string = self.lookahead(i).join("");
            self.advance(i + discard);
            trace!(r#"Crawl: {}"#, string);
            Some(string)
        } else {
            None
        }
    }

    fn handle_single_line_string(&mut self, quote: &str) -> Result<String> {
        match self.crawl(|s| *s == quote, 1) {
            None => {
                return Err(LexerError::Generic(
                    "Input ran out looking for quote!".to_string(),
                ))
            }
            Some(s) => {
                // FIXME Normalize newlines!
                if s.contains("\n") {
                    return Err(LexerError::Generic(
                        "Encountered newline in string!".to_string(),
                    ));
                }

                Ok(s)
            }
        }
    }

    fn handle_multiline_string(&mut self, quote: &str) -> Result<String> {
        self.advance(2);

        let mut string = String::new();

        loop {
            match self.crawl(|s| *s == quote, 0) {
                None => {
                    return Err(LexerError::Generic(
                        "Input ran out looking for triple quote!".to_string(),
                    ))
                }
                Some(s) => string += &s,
            }

            let peek = self.lookahead(3);

            if peek.len() != 3 {
                return Err(LexerError::Generic(
                    "Input ends with single quote!".to_string(),
                ));
            } else {
                if peek.join("") == quote.repeat(3) {
                    break;
                } else {
                    string += quote;
                    self.advance(1)
                }
            }
        }

        Ok(string)
    }

    fn handle_string(&mut self, quote: &str) -> Result<Token> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        let string = if self.lookahead(2).iter().all(|s| *s == quote) {
            self.handle_multiline_string(quote)
        } else {
            self.handle_single_line_string(quote)
        }?;

        if string
            .graphemes(true)
            .any(|s| new_token::FORBIDDEN_GRAPHEMES.contains(&s))
        {
            return Err(LexerError::Generic(
                "Forbidden grapheme in string!".to_string(),
            ));
        }

        Ok(Token::new(TokenType::String(string), line_no, col_no))
    }

    fn handle_single_line_comment(&mut self) -> Result<Token> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        self.advance(1);

        // FIXME Normalize newlines!
        match self.crawl(|s| *s == "\n", 0) {
            // Ends on newline
            Some(s) => Ok(Token::new(TokenType::Comment(s), line_no, col_no)),
            // Ends on EOF
            None => {
                // Skips final "/"
                self.advance(1);
                let mut string = String::new();
                while let Some(grapheme) = self.current_grapheme {
                    string += grapheme;
                    self.advance(1)
                }

                Ok(Token::new(TokenType::Comment(string), line_no, col_no))
            }
        }
    }

    fn handle_multiline_comment(&mut self) -> Result<Token> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        self.advance(1);

        let mut string = String::new();

        while self.lookahead(2) != ["*", "/"] {
            match self.crawl(|s| *s == "*", 0) {
                None => {
                    return Err(LexerError::Generic(
                        "Unterminated multiline comment!".to_string(),
                    ))
                }
                Some(s) => string += &s,
            }
        }

        self.advance(2);

        Ok(Token::new(TokenType::Comment(string), line_no, col_no))
    }

    fn handle_comment(&mut self) -> Result<Option<Token>> {
        if self.lookahead(1) == ["/"] {
            Ok(Some(self.handle_single_line_comment()?))
        } else if self.lookahead(1) == ["*"] {
            Ok(Some(self.handle_multiline_comment()?))
        } else {
            Ok(None)
        }
    }

    fn handle_bounded(&mut self) -> Result<Option<Token>> {
        // self.next() already checks for None, so this unwrap should be safe.
        debug_assert!(self.current_grapheme.is_some());

        let current_grapheme = self.current_grapheme.unwrap();

        if ["\"", "'"].contains(&current_grapheme) {
            // String
            Ok(Some(self.handle_string(current_grapheme)?))
        } else if current_grapheme == "/" {
            Ok(self.handle_comment()?)
        } else {
            Ok(None)
        }
    }

    fn handle_reserved(&mut self) -> Option<Token> {
        let mut token = None;
        // TODO? Better descending range?
        for i in (0..new_token::LOOKAHEAD).rev() {
            // self.next() already checks for None, so this unwrap should be safe.
            let string: String = self.current_grapheme.unwrap().to_string() + &self.lookahead(i).join("");
            trace!("Checking reserved chars for {}", string);

            let result = Token::from_str(
                &string,
                self.line_no,
                self.col_no,
            );
            if let Ok(t) = result {
                token = Some(t);
                self.advance(i);
                break;
            }
        }

        token
    }

    fn handle_misc(&mut self) -> Option<Token> {
        let (line_no, col_no) = (self.line_no, self.col_no);

        // FIXME Normalize newlines!
        let string = self
        //.crawl(|s| [" ", "\t", "\n"].contains(s), 0)
        .crawl(|s| s.chars().any(|c| !(c.is_alphanumeric() || c == '_')), 0)
            .unwrap_or_else(|| {
                let mut string = String::new();
                while let Some(grapheme) = self.current_grapheme {
                    string += grapheme;
                    self.advance(1);
                }
                string
            });

        if string.starts_with(|c: char| c.is_alphabetic())
            && string.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            Some(Token::new(TokenType::ID(string), line_no, col_no))
        } else if string.chars().all(|c| c.is_numeric()) {
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
    static MULTILINE_STRING: &str = "'''This is a \n multiline string'''";
    static STRING_WITH_FORBIDDEN_GRAPHEMES: &str =
        r#""This | is ? a string ~ with * forbidden graphemes.""#;
    static UNTERMINATED_STRING: &str = r#""This is an unterminated string"#;
    static UNTERMINATED_MULTILINE_STRING: &str =
        "'''This is an unterminated \n multiline string'";
    static SINGLE_LINE_COMMENT: &str = "// This is a single line comment!\n";
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
            let mut lex = Lexer::new(&input);

            match lex.handle_reserved() {
                Some(token) => {
                    assert_eq!(
                        token.token_type, expected_type,
                        "reserved: got {:?}\texpected {:?}",
                        token.token_type, expected_type
                    );
                    Ok(())
                }
                None => {
                    bail!("reserved: unable to parse {} as Token", input)
                }
            }
        }

        #[test]
        fn new_lexer_single_char_test() -> Result<()> {
            reserved_test("+", TokenType::Plus)?;
            reserved_test("-", TokenType::Hyphen)?;
            Ok(())
        }

        #[test]
        fn new_lexer_double_char_test() -> Result<()> {
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
        ) -> Result<Token> {
            let mut lex = Lexer::new(&input);

            match lex.handle_bounded()? {
                Some(token) => {
                    assert_eq!(
                        token.token_type,
                        expected_type,
                        // "bounded_type: got {:?}, expected {:?}",
                        // token.token_type, expected_type
                    );

                    Ok(token)
                }
                None => {
                    bail!("bounded: unable to parse {} as Token", input)
                }
            }
        }

        #[test]
        fn new_lexer_string_test() -> Result<()> {
            for string in &[SINGLE_QUOTED_STRING, DOUBLE_QUOTED_STRING] {
                bounded_test(
                    string,
                    TokenType::String(dequote(string).to_string()),
                )?;
            }
            Ok(())
        }

        #[test]
        fn new_lexer_multiline_string_test() -> Result<()> {
            bounded_test(
                MULTILINE_STRING,
                TokenType::String(
                    slice_ends(MULTILINE_STRING, 3, 3).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_newline_in_string_test() -> Result<()> {
            if let Err(err) = bounded_test(
                NEWLINE_IN_STRING,
                TokenType::String(String::new()),
            ) {
                if !err.to_string().contains("Encountered newline in string") {
                    bail!(
                        "unterminated string {} did not return expected error!",
                        NEWLINE_IN_STRING
                    );
                }
            }

            Ok(())
        }

        #[test]
        fn new_lexer_forbidden_graphemes_test() -> Result<()> {
            match bounded_test(STRING_WITH_FORBIDDEN_GRAPHEMES, TokenType::String(String::new())) {
                Ok(token) => bail!("Lexer did not error on forbidden characters, returned {:?}", token),
                Err(err) => {
                    if err.to_string().contains("Forbidden grapheme") {
                        Ok(())
                    } else {
                        bail!("Unrelated error {:?}!", err)
                    }
                }
            }
        }

        #[test]
        fn new_lexer_unterminated_single_line_string_test() -> Result<()> {
            if let Err(err) = bounded_test(
                UNTERMINATED_STRING,
                TokenType::String(String::new()),
            ) {
                if !err.to_string().contains("Input ran out looking for quote")
                {
                    bail!(
                        "unterminated string {} did not return expected error!",
                        UNTERMINATED_STRING
                    );
                }
            }

            Ok(())
        }

        #[test]
        fn new_lexer_unterminated_multiline_string_test() -> Result<()> {
            if let Err(err) = bounded_test(
                UNTERMINATED_MULTILINE_STRING,
                TokenType::String(String::new()),
            ) {
                if !err.to_string().contains("Input ends with single quote") {
                    bail!(
                        "unterminated string {} did not return expected error!",
                        UNTERMINATED_MULTILINE_STRING
                    );
                }
            }

            if let Err(err) = bounded_test(
                &(UNTERMINATED_MULTILINE_STRING.to_string() + "abcd"),
                TokenType::String(String::new()),
            ) {
                if !err.to_string().contains("Input ran out looking for triple quote") {
                    bail!(
                        "unterminated string {} did not return expected error!",
                        UNTERMINATED_MULTILINE_STRING
                    );
                }
            }

            Ok(())
        }

        #[test]
        fn new_lexer_single_line_comment_eof_test() -> Result<()> {
            bounded_test(
                slice_ends(SINGLE_LINE_COMMENT, 0, 1),
                TokenType::Comment(
                    slice_ends(SINGLE_LINE_COMMENT, 2, 1).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_single_line_comment_newline_test() -> Result<()> {
            bounded_test(
                SINGLE_LINE_COMMENT,
                TokenType::Comment(
                    slice_ends(SINGLE_LINE_COMMENT, 2, 1).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_multiline_comment_test() -> Result<()> {
            bounded_test(
                MULTILINE_COMMENT,
                TokenType::Comment(
                    slice_ends(MULTILINE_COMMENT, 2, 2).to_string(),
                ),
            )?;
            Ok(())
        }

        #[test]
        fn new_lexer_unterminated_comment_test() -> Result<()> {
            if let Err(err) = bounded_test(
                UNTERMINATED_COMMENT,
                TokenType::Comment(String::new()),
            ) {
                if !err.to_string().contains("Unterminated multiline comment") {
                    bail!("unterminated_comment {} did not return expected error!", UNTERMINATED_COMMENT);
                }
            }

            Ok(())
        }
    }

    mod handle_misc_tokens {
        use super::*;

        fn misc_test(input: &str, expected_type: TokenType) -> Result<()> {
            let mut lex = Lexer::new(&input);

            match lex.handle_misc() {
                Some(token) => {
                    assert_eq!(
                        token.token_type, expected_type,
                        "misc_type: got {:?}, expected {:?}",
                        token.token_type, expected_type
                    );

                    Ok(())
                }
                None => {
                    bail!("misc: unable to parse {} as Token", input)
                }
            }
        }
        #[test]
        fn new_lexer_id_test() -> Result<()> {
            misc_test("id", TokenType::ID("id".to_string()))
        }

        #[test]
        fn new_lexer_integer_test() -> Result<()> {
            misc_test("1", TokenType::Integer(1))
        }
    }
}
