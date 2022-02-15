use super::error::TokenError;
use std::str::FromStr;

type Result<T> = std::result::Result<T, TokenError>;

/// Forbidden graphemes that are part of TFMT.
pub const FORBIDDEN_GRAPHEMES: [&str; 8] =
    ["<", ">", ":", "\"", "|", "?", "*", "~"];

/// Directory separators.
pub static DIRECTORY_SEPARATORS: [&str; 2] = ["/", "\\"];

/// Represents the type of token, and optionally it's value.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
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
    SlashBack,
    SlashForward,
    VerticalBar,

    DoubleAmpersand,
    DoubleAsterisk,
    DoubleVerticalBar,

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
    /// Maximum length of [`TokenType`] string representation.
    pub const LOOKAHEAD_DEPTH: usize = 2;

    /// Whether or not this [`TokenType`] is ignored by [`Parser`].
    pub fn is_ignored(&self) -> bool {
        matches!(self, &Self::Comment(..))
    }
}

/// TFMT Token
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    /// Type of token
    pub token_type: TokenType,
    /// Line number
    pub line_no: usize,
    /// Column number
    pub col_no: usize,
}

impl Token {
    /// Create a new [`Token`].
    pub fn new(token_type: TokenType, line_no: usize, col_no: usize) -> Self {
        Self {
            token_type,
            line_no,
            col_no,
        }
    }

    /// Attempt to create a new [`Token`], parsing a string as [`TokenType`].
    pub fn from_str<S: AsRef<str>>(
        token_type: &S,
        line_no: usize,
        col_no: usize,
    ) -> Result<Self> {
        Ok(Self {
            token_type: TokenType::from_str(token_type.as_ref())?,
            line_no,
            col_no,
        })
    }

    /// Gets value from [`TokenType::{Comment, ID, String}`], panicking if the
    /// the token is a different type.
    pub fn get_string_unchecked(&self) -> &str {
        match &self.token_type {
            TokenType::Comment(string)
            | TokenType::ID(string)
            | TokenType::String(string) => string.as_str(),
            token_type => panic!(
                "get_string_unchecked was called on TokenType {:?}!",
                token_type
            ),
        }
    }

    /// Gets value from [`TokenType::Integer`], panicking if the
    /// the token is a different type.
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
