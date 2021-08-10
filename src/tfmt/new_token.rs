use crate::new_error::TokenError;
use std::str::FromStr;

type Result<T> = std::result::Result<T, TokenError>;

/// Forbidden graphemes that are part of TFMT.
pub const FORBIDDEN_GRAPHEMES: [&str; 8] =
    ["<", ">", ":", "\"", "|", "?", "*", "~"];

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Ampersand,
    AngleBracketLeft,
    AngleBracketRight,
    Caret,
    Comma,
    CurlyBraceLeft,
    CurlyBraceRight,
    Colon,
    Dollar,
    Equals,
    Hash,
    Hyphen,
    Asterisk,
    ParenthesisLeft,
    ParenthesisRight,
    Percent,
    Plus,
    QuestionMark,
    //QuoteDouble,
    //QuoteSingle,
    SlashBack,
    SlashForward,
    VerticalBar,

    //AsteriskSlash,
    DoubleAmpersand,
    DoubleAsterisk,
    DoubleVerticalBar,
    //SlashAsterisk,
    Comment(String),
    ID(String),
    Integer(i64),
    String(String),

    Uninitialized,
}

impl FromStr for TokenType {
    type Err = TokenError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "&" => Self::Ampersand,
            "<" => Self::AngleBracketLeft,
            ">" => Self::AngleBracketRight,
            "^" => Self::Caret,
            "," => Self::Comma,
            "{" => Self::CurlyBraceLeft,
            "}" => Self::CurlyBraceRight,
            ":" => Self::Colon,
            "$" => Self::Dollar,
            "=" => Self::Equals,
            "#" => Self::Hash,
            "-" => Self::Hyphen,
            "*" => Self::Asterisk,
            "(" => Self::ParenthesisLeft,
            ")" => Self::ParenthesisRight,
            "%" => Self::Percent,
            "+" => Self::Plus,
            "?" => Self::QuestionMark,
            "\\" => Self::SlashBack,
            "/" => Self::SlashForward,
            "|" => Self::VerticalBar,
            //
            "&&" => Self::DoubleAmpersand,
            "**" => Self::DoubleAsterisk,
            "||" => Self::DoubleVerticalBar,
            s => return Err(TokenError::InvalidType(s.to_string())),
        })
    }
}

impl TokenType {
    pub const LOOKAHEAD_DEPTH: usize = 2;

    pub fn is_ignored(&self) -> bool {
        matches!(self, Self::Comment(..))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line_no: u64,
    pub col_no: u64,
}

impl Token {
    pub fn new(token_type: TokenType, line_no: u64, col_no: u64) -> Self {
        Token {
            token_type,
            line_no,
            col_no,
        }
    }

    pub fn from_str<S: AsRef<str>>(
        token_type: &S,
        line_no: u64,
        col_no: u64,
    ) -> Result<Self> {
        Ok(Token {
            token_type: TokenType::from_str(token_type.as_ref())?,
            line_no,
            col_no,
        })
    }

    pub fn get_string_unchecked(&self) -> String {
        match &self.token_type {
            TokenType::Comment(string)
            | TokenType::ID(string)
            | TokenType::String(string) => string.clone(),
            token_type => panic!(
                "get_string_unchecked was called on TokenType {:?}!",
                token_type
            ),
        }
    }

    pub fn get_int_unchecked(&self) -> i64 {
        match &self.token_type {
            TokenType::Integer(int) => *int,
            token_type => panic!(
                "get_int_unchecked was called on TokenType {:?}!",
                token_type
            ),
        }
    }
}
