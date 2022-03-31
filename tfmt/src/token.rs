use super::error::TokenError;
use std::str::FromStr;

type Result<T> = std::result::Result<T, TokenError>;

/// Represents the type of token, and optionally it's value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum TokenType {
    // Single character tokens
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
    ParenthesisLeft,
    ParenthesisRight,
    Percent,
    Plus,
    QuestionMark,
    SlashBack,
    SlashForward,

    // Single or double character tokens
    Ampersand,
    DoubleAmpersand,
    Asterisk,
    DoubleAsterisk,
    VerticalBar,
    DoubleVerticalBar,

    // Literals
    Comment,
    ID,
    Integer,
    String,

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
            s => return Err(TokenError::InvalidTokenType(s.to_string())),
        })
    }
}

impl TokenType {
    /// Maximum length of [`TokenType`] string representation.
    pub(crate) const LOOKAHEAD_DEPTH: usize = 2;

    /// Whether or not this [`TokenType`] is ignored by [`Parser`].
    pub(crate) fn is_ignored(self) -> bool {
        matches!(self, Self::Comment | Self::Uninitialized)
    }
}

/// TFMT Token
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    line_no: usize,
    col_no: usize,
    literal: Option<String>,
}

impl Token {
    // TODO? Check whether or not TokenType is a literal?
    /// Create a new [`Token`].
    pub(crate) fn new(
        token_type: TokenType,
        line_no: usize,
        col_no: usize,
    ) -> Self {
        Self {
            token_type,
            line_no,
            col_no,
            literal: None,
        }
    }

    /// Create a new [`Token`].
    pub(crate) fn with_literal(
        token_type: TokenType,
        line_no: usize,
        col_no: usize,
        literal: String,
    ) -> Self {
        Self {
            token_type,
            line_no,
            col_no,
            literal: Some(literal),
        }
    }

    pub(crate) fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub(crate) fn line_no(&self) -> usize {
        self.line_no
    }

    pub(crate) fn col_no(&self) -> usize {
        self.col_no
    }

    /// Get a reference to the token's literal.
    pub(crate) fn literal(&self) -> Option<&str> {
        self.literal.as_deref()
    }
}
