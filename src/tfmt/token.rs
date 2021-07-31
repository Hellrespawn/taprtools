#![allow(non_camel_case_types)]
use crate::error::TokenError;
use bimap::BiMap;
use once_cell::sync::Lazy;

type Result<T> = std::result::Result<T, TokenError>;

/// Forbidden graphemes that are part of TFMT.
pub static FORBIDDEN_GRAPHEMES: [&str; 8] =
    ["<", ">", ":", "\"", "|", "?", "*", "~"];

/// Directory separators.
pub static DIRECTORY_SEPARATORS: [&str; 2] = ["/", "\\"];

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
    ID,
    Integer,
    String,

    EOF,
    Uninitialized,
}

impl TokenType {
    /// Ignored [TokenType]s
    pub const IGNORED: [TokenType; 1] = [TokenType::Comment];

    fn string_map() -> &'static BiMap<TokenType, &'static str> {
        static STRING_MAP: Lazy<BiMap<TokenType, &'static str>> =
            Lazy::new(|| {
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
                ttypes.insert(TokenType::ID, "ID");
                ttypes.insert(TokenType::Integer, "INTEGER");
                ttypes.insert(TokenType::String, "STRING");

                ttypes.insert(TokenType::EOF, "EOF");
                ttypes.insert(TokenType::Uninitialized, "UNINITIALIZED");

                ttypes
            });

        &STRING_MAP
    }

    /// A collection of strings that represent a [TokenType].
    pub fn reserved_strings() -> &'static [&'static str] {
        static RESERVED_STRINGS: Lazy<Vec<&'static str>> = Lazy::new(|| {
            let mut reserved = Vec::new();
            for (_, string) in TokenType::string_map().iter() {
                if !string.chars().all(|c| c.is_alphabetic()) {
                    reserved.push(*string)
                }
            }
            reserved.sort_by_key(|a| a.len());
            reserved.reverse();
            reserved
        });

        &RESERVED_STRINGS
    }

    // TODO? impl TryFrom<&str> for Token and From<Token> for &str?
    /// Get [TokenType] from string.
    pub fn from_string(string: &str) -> Result<TokenType> {
        TokenType::string_map()
            .get_by_right(string)
            .ok_or_else(|| TokenError::InvalidType(string.to_string()))
            .map(|tt| *tt)
    }

    /// Get string representation of self.[TokenType].
    pub fn as_str(&self) -> &str {
        // All TokenTypes should be in TokenType::string_map, so unwrap
        // should always be safe.
        let string = TokenType::string_map().get_by_left(self);

        debug_assert!(string.is_some());
        string.unwrap()
    }
}

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
        Self::new(line_no, col_no, TokenType::from_string(ttype_str)?, value)
    }

    /// Get unwrapped value from [Token]. Only use it after checking it's
    /// a [TokenType] with a value.
    pub fn get_value_unchecked(&self) -> &str {
        self.value.as_ref().expect(
            "Token values are checked at creation! Are you  trying to get the value from a token that isn't supposed to have one?",
        )
    }
}
