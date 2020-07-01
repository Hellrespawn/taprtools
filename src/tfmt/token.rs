#![allow(non_camel_case_types)]

use bimap::BiMap;
use lazy_static::lazy_static;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum TokenType {
    AMPERSAND,
    ANGLE_BRACKET_LEFT,
    ANGLE_BRACKET_RIGHT,
    CARET,
    COMMA,
    CURLY_BRACE_LEFT,
    CURLY_BRACE_RIGHT,
    COLON,
    DOLLAR,
    EQUALS,
    HASH,
    HYPHEN,
    ASTERISK,
    PARENTHESIS_LEFT,
    PARENTHESIS_RIGHT,
    PERCENT,
    PLUS,
    QUESTION_MARK,
    QUOTE_DOUBLE,
    QUOTE_SINGLE,
    SLASH_BACK,
    SLASH_FORWARD,
    VERTICAL_BAR,

    ASTERISK_SLASH,
    DOUBLE_AMPERSAND,
    DOUBLE_ASTERISK,
    DOUBLE_VERTICAL_BAR,
    SLASH_ASTERISK,

    COMMENT,
    DRIVE,
    EOF,
    ID,
    INTEGER,
    STRING,
}

lazy_static! {
    pub static ref TOKEN_TYPE_STRING_MAP: BiMap<TokenType, &'static str> = {
        let mut ttypes = BiMap::new();
        ttypes.insert(TokenType::AMPERSAND, "&");
        ttypes.insert(TokenType::ANGLE_BRACKET_LEFT, "<");
        ttypes.insert(TokenType::ANGLE_BRACKET_RIGHT, ">");
        ttypes.insert(TokenType::CARET, "^");
        ttypes.insert(TokenType::COMMA, ",");
        ttypes.insert(TokenType::CURLY_BRACE_LEFT, "{");
        ttypes.insert(TokenType::CURLY_BRACE_RIGHT, "}");
        ttypes.insert(TokenType::COLON, ":");
        ttypes.insert(TokenType::DOLLAR, "$");
        ttypes.insert(TokenType::EQUALS, "=");
        ttypes.insert(TokenType::HASH, "#");
        ttypes.insert(TokenType::HYPHEN, "-");
        ttypes.insert(TokenType::ASTERISK, "*");
        ttypes.insert(TokenType::PARENTHESIS_LEFT, "(");
        ttypes.insert(TokenType::PARENTHESIS_RIGHT, ")");
        ttypes.insert(TokenType::PERCENT, "%");
        ttypes.insert(TokenType::PLUS, "+");
        ttypes.insert(TokenType::QUESTION_MARK, "?");
        ttypes.insert(TokenType::QUOTE_DOUBLE, "\"");
        ttypes.insert(TokenType::QUOTE_SINGLE, "'");
        ttypes.insert(TokenType::SLASH_BACK, "\\");
        ttypes.insert(TokenType::SLASH_FORWARD, "/");
        ttypes.insert(TokenType::VERTICAL_BAR, "|");

        ttypes.insert(TokenType::ASTERISK_SLASH, "*/");
        ttypes.insert(TokenType::DOUBLE_AMPERSAND, "&&");
        ttypes.insert(TokenType::DOUBLE_ASTERISK, "**");
        ttypes.insert(TokenType::DOUBLE_VERTICAL_BAR, "||");
        ttypes.insert(TokenType::SLASH_ASTERISK, "/*");

        ttypes.insert(TokenType::COMMENT, "COMMENT");
        ttypes.insert(TokenType::DRIVE, "DRIVE");
        ttypes.insert(TokenType::EOF, "EOF");
        ttypes.insert(TokenType::ID, "ID");
        ttypes.insert(TokenType::INTEGER, "INTEGER");
        ttypes.insert(TokenType::STRING, "STRING");
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
        reserved.sort_by(|a, b| a.len().cmp(&b.len()));
        reserved.reverse();
        reserved
    };
}

// FIXME Filter dir separator based on platform. Currently manually removed.
pub static FORBIDDEN_GRAPHEMES: [&str; 9] =
    ["\\", "<", ">", ":", "\"", "|", "?", "*", "~"];

#[derive(Debug, PartialEq)]
pub struct Token {
    line_no: u32,
    col_no: u32,
    pub ttype: TokenType,
    pub value: Option<String>,
}

impl Token {
    pub fn new(
        line_no: u32,
        col_no: u32,
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
        line_no: u32,
        col_no: u32,
        ttype_char: &str,
        value: Option<String>,
    ) -> Result<Token, String> {
        if let Some(ttype) = TOKEN_TYPE_STRING_MAP.get_by_right(&ttype_char) {
            Ok(Token {
                line_no,
                col_no,
                ttype: *ttype,
                value,
            })
        } else {
            Err(format!("Invalid character {} for token!", ttype_char))
        }
    }
}
