#![allow(non_camel_case_types)]
use crate::error::TokenError;
use bimap::BiMap;
use lazy_static::lazy_static;

type Result<T> = std::result::Result<T, TokenError>;

/// Describes [Token] type.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
    QuoteDouble,
    QuoteSingle,
    SlashBack,
    SlashForward,
    VerticalBar,

    AsteriskSlash,
    DoubleAmpersand,
    DoubleAsterisk,
    DoubleVerticalBar,
    SlashAsterisk,

    Comment,
    Drive,
    ID,
    Integer,
    String,

    EOF,
    Uninitialized,
}

lazy_static! {
    static ref TOKEN_TYPE_STRING_MAP: BiMap<TokenType, &'static str> = {
        let mut ttypes = BiMap::new();
        ttypes.insert(TokenType::Ampersand, "&");
        ttypes.insert(TokenType::AngleBracketLeft, "<");
        ttypes.insert(TokenType::AngleBracketRight, ">");
        ttypes.insert(TokenType::Caret, "^");
        ttypes.insert(TokenType::Comma, ",");
        ttypes.insert(TokenType::CurlyBraceLeft, "{");
        ttypes.insert(TokenType::CurlyBraceRight, "}");
        ttypes.insert(TokenType::Colon, ":");
        ttypes.insert(TokenType::Dollar, "$");
        ttypes.insert(TokenType::Equals, "=");
        ttypes.insert(TokenType::Hash, "#");
        ttypes.insert(TokenType::Hyphen, "-");
        ttypes.insert(TokenType::Asterisk, "*");
        ttypes.insert(TokenType::ParenthesisLeft, "(");
        ttypes.insert(TokenType::ParenthesisRight, ")");
        ttypes.insert(TokenType::Percent, "%");
        ttypes.insert(TokenType::Plus, "+");
        ttypes.insert(TokenType::QuestionMark, "?");
        ttypes.insert(TokenType::QuoteDouble, "\"");
        ttypes.insert(TokenType::QuoteSingle, "'");
        ttypes.insert(TokenType::SlashBack, "\\");
        ttypes.insert(TokenType::SlashForward, "/");
        ttypes.insert(TokenType::VerticalBar, "|");

        ttypes.insert(TokenType::AsteriskSlash, "*/");
        ttypes.insert(TokenType::DoubleAmpersand, "&&");
        ttypes.insert(TokenType::DoubleAsterisk, "**");
        ttypes.insert(TokenType::DoubleVerticalBar, "||");
        ttypes.insert(TokenType::SlashAsterisk, "/*");

        ttypes.insert(TokenType::Comment, "COMMENT");
        ttypes.insert(TokenType::Drive, "DRIVE");
        ttypes.insert(TokenType::ID, "ID");
        ttypes.insert(TokenType::Integer, "INTEGER");
        ttypes.insert(TokenType::String, "STRING");

        ttypes.insert(TokenType::EOF, "EOF");
        ttypes.insert(TokenType::Uninitialized, "UNINITIALIZED");

        ttypes
    };
}

impl TokenType {
    /// Get [TokenType] from string.
    pub fn from_string(string: &str) -> Result<TokenType> {
        TOKEN_TYPE_STRING_MAP
            .get_by_right(string)
            .ok_or_else(|| TokenError::InvalidType(string.to_string()))
            .map(|tt| *tt)
    }

    /// Get string representation of self.[TokenType].
    pub fn grapheme(&self) -> &str {
        TOKEN_TYPE_STRING_MAP
            .get_by_left(self)
            .expect("fmt: All TokenTypes should be in TOKEN_TYPE_STRING_MAP")
    }
}

lazy_static! {
    /// Reserved strings.
    pub static ref RESERVED_STRINGS: Vec<&'static str> = {
        let mut reserved = Vec::new();
        for (_, string) in TOKEN_TYPE_STRING_MAP.iter() {
            if !string.chars().all(|c| c.is_alphabetic()) {
                reserved.push(*string)
            }
        }
        reserved.sort_by_key(|a| a.len());
        reserved.reverse();
        reserved
    };
}

/// Forbidden graphemes that are part of TFMT.
pub static FORBIDDEN_GRAPHEMES: [&str; 8] =
    ["<", ">", ":", "\"", "|", "?", "*", "~"];

pub static DIRECTORY_SEPARATORS: [&str; 2] = ["/", "\\"];

/// Ignored [TokenType]s
pub static IGNORED_TOKEN_TYPES: [TokenType; 1] = [TokenType::Comment];

/// [Token] is a lexical unit with an optional value.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub line_no: u64,
    pub col_no: u64,
    pub ttype: TokenType,
    value: Option<String>,
}

impl Token {
    /// Default constructor.
    pub fn new(
        line_no: u64,
        col_no: u64,
        ttype: TokenType,
        value: Option<String>,
    ) -> Result<Token> {
        match ttype {
            ttype
                if [
                    TokenType::Comment,
                    TokenType::Drive,
                    TokenType::ID,
                    TokenType::Integer,
                    TokenType::String,
                ]
                .contains(&ttype) =>
            {
                if value.is_none() {
                    return Err(TokenError::NoValue(ttype));
                }
            }
            ttype => {
                if let Some(value) = value {
                    return Err(TokenError::HasValue(ttype, value));
                }
            }
        }

        Ok(Token {
            line_no,
            col_no,
            ttype,
            value,
        })
    }

    /// Alternative constructor.
    pub fn new_type_from_string(
        line_no: u64,
        col_no: u64,
        ttype_str: &str,
        value: Option<String>,
    ) -> Result<Token> {
        Self::new(line_no, col_no, TokenType::from_string(&ttype_str)?, value)
    }

    pub fn get_value(&self) -> &str {
        self.value
            .as_ref()
            .expect("Token values should be checked at creation.")
    }
}
