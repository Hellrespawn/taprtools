#![allow(non_camel_case_types)]

use anyhow::{anyhow, Result};
use bimap::BiMap;
use lazy_static::lazy_static;

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
    pub static ref TOKEN_TYPE_STRING_MAP: BiMap<TokenType, &'static str> = {
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

        ttypes
    };
}

lazy_static! {
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

// FIXME Filter dir separator based on platform. Currently manually removed.
pub static FORBIDDEN_GRAPHEMES: [&str; 9] =
    ["\\", "<", ">", ":", "\"", "|", "?", "*", "~"];

pub static IGNORED: [TokenType; 1] = [TokenType::Comment];

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub line_no: u64,
    pub col_no: u64,
    pub ttype: TokenType,
    pub value: Option<String>,
}

impl Token {
    pub fn new(
        line_no: u64,
        col_no: u64,
        ttype: TokenType,
        value: Option<String>,
    ) -> Token {
        Token {
            line_no,
            col_no,
            ttype,
            value,
        }
    }

    pub fn new_type_from_string(
        line_no: u64,
        col_no: u64,
        ttype_char: &str,
        value: Option<String>,
    ) -> Result<Token> {
        if let Some(ttype) = TOKEN_TYPE_STRING_MAP.get_by_right(&ttype_char) {
            Ok(Token {
                line_no,
                col_no,
                ttype: *ttype,
                value,
            })
        } else {
            Err(anyhow!("Invalid character {} for token!", ttype_char))
        }
    }
}
