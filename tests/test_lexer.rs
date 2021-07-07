use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use tfmttools::tfmt::lexer::Lexer;
use tfmttools::tfmt::token::Token;
use tfmttools::tfmt::token::TokenType::*;

mod common;

fn file_test(filename: &str, reference: Option<Vec<Token>>) -> Result<()> {
    let mut path = PathBuf::from(file!());
    for _ in 1..=3 {
        path.pop();
    }
    path.push("tests");
    path.push("files");
    path.push("config");
    path.push(filename);

    let input = fs::read_to_string(path)
        .expect(&format!("{} doesn't exist!", filename));

    if let Some(tokens) = reference {
        lexer_test(&input, tokens)
    } else {
        run_lexer(&input, true)?;
        Ok(())
    }
}

fn lexer_test(string: &str, reference: Vec<Token>) -> Result<()> {
    let tokens = run_lexer(string, true)?;

    assert_eq!(tokens, reference);

    Ok(())
}

fn run_lexer(string: &str, pop_eof: bool) -> Result<Vec<Token>> {
    let lex = create_lexer(string)?;

    let mut tokens = lex.collect::<Result<Vec<Token>, _>>()?;

    if pop_eof {
        tokens.pop();
    }

    Ok(tokens)
}

fn create_lexer(string: &str) -> Result<Lexer> {
    Ok(Lexer::new(string)?)
}

#[test]
fn simple_input() -> Result<()> {
    let reference = vec![
        Token {
            line_no: 1,
            col_no: 1,
            ttype: ID,
            value: Some("simple_input".to_string()),
        },
        Token {
            line_no: 1,
            col_no: 13,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 1,
            col_no: 14,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 1,
            col_no: 16,
            ttype: CurlyBraceLeft,
            value: None,
        },
        Token {
            line_no: 2,
            col_no: 5,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 2,
            col_no: 6,
            ttype: ID,
            value: Some("artist".to_string()),
        },
        Token {
            line_no: 2,
            col_no: 12,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 2,
            col_no: 14,
            ttype: String,
            value: Some("/".to_string()),
        },
        Token {
            line_no: 2,
            col_no: 18,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 2,
            col_no: 19,
            ttype: ID,
            value: Some("title".to_string()),
        },
        Token {
            line_no: 2,
            col_no: 24,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 3,
            col_no: 1,
            ttype: CurlyBraceRight,
            value: None,
        },
    ];

    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn typical_input() -> Result<()> {
    let reference = vec![
        Token {
            line_no: 1,
            col_no: 1,
            ttype: ID,
            value: Some("typical_input".to_string()),
        },
        Token {
            line_no: 1,
            col_no: 14,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 1,
            col_no: 15,
            ttype: ID,
            value: Some("folder".to_string()),
        },
        Token {
            line_no: 1,
            col_no: 21,
            ttype: Equals,
            value: None,
        },
        Token {
            line_no: 1,
            col_no: 22,
            ttype: String,
            value: Some("destination".to_string()),
        },
        Token {
            line_no: 1,
            col_no: 35,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 1,
            col_no: 37,
            ttype: String,
            value: Some("This file is used to test tfmttools.".to_string()),
        },
        Token {
            line_no: 2,
            col_no: 1,
            ttype: CurlyBraceLeft,
            value: None,
        },
        Token {
            line_no: 3,
            col_no: 5,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 3,
            col_no: 6,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 3,
            col_no: 7,
            ttype: ID,
            value: Some("folder".to_string()),
        },
        Token {
            line_no: 3,
            col_no: 13,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 3,
            col_no: 15,
            ttype: String,
            value: Some("/".to_string()),
        },
        Token {
            line_no: 4,
            col_no: 5,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 4,
            col_no: 6,
            ttype: ID,
            value: Some("albumartist".to_string()),
        },
        Token {
            line_no: 4,
            col_no: 17,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 4,
            col_no: 19,
            ttype: VerticalBar,
            value: None,
        },
        Token {
            line_no: 4,
            col_no: 21,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 4,
            col_no: 22,
            ttype: ID,
            value: Some("artist".to_string()),
        },
        Token {
            line_no: 4,
            col_no: 28,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 5,
            col_no: 5,
            ttype: String,
            value: Some("/".to_string()),
        },
        Token {
            line_no: 7,
            col_no: 5,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 8,
            col_no: 9,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 8,
            col_no: 10,
            ttype: ID,
            value: Some("date".to_string()),
        },
        Token {
            line_no: 8,
            col_no: 14,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 8,
            col_no: 16,
            ttype: Ampersand,
            value: None,
        },
        Token {
            line_no: 8,
            col_no: 18,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 8,
            col_no: 21,
            ttype: Comment,
            value: Some(" This is a single-line comment".to_string()),
        },
        Token {
            line_no: 9,
            col_no: 13,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 9,
            col_no: 14,
            ttype: ID,
            value: Some("year_from_date".to_string()),
        },
        Token {
            line_no: 9,
            col_no: 28,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 9,
            col_no: 29,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 9,
            col_no: 30,
            ttype: ID,
            value: Some("date".to_string()),
        },
        Token {
            line_no: 9,
            col_no: 34,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 9,
            col_no: 35,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 13,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 14,
            ttype: ID,
            value: Some("albumsort".to_string()),
        },
        Token {
            line_no: 10,
            col_no: 23,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 25,
            ttype: Ampersand,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 27,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 28,
            ttype: String,
            value: Some(".".to_string()),
        },
        Token {
            line_no: 10,
            col_no: 32,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 33,
            ttype: ID,
            value: Some("num".to_string()),
        },
        Token {
            line_no: 10,
            col_no: 36,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 37,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 38,
            ttype: ID,
            value: Some("albumsort".to_string()),
        },
        Token {
            line_no: 10,
            col_no: 47,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 48,
            ttype: Comma,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 50,
            ttype: Integer,
            value: Some("2".to_string()),
        },
        Token {
            line_no: 10,
            col_no: 51,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 10,
            col_no: 53,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 11,
            col_no: 13,
            ttype: String,
            value: Some(" - ".to_string()),
        },
        Token {
            line_no: 12,
            col_no: 9,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 13,
            col_no: 9,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 13,
            col_no: 10,
            ttype: ID,
            value: Some("album".to_string()),
        },
        Token {
            line_no: 13,
            col_no: 15,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 14,
            col_no: 5,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 14,
            col_no: 7,
            ttype: DoubleAmpersand,
            value: None,
        },
        Token {
            line_no: 14,
            col_no: 10,
            ttype: String,
            value: Some("/".to_string()),
        },
        Token {
            line_no: 14,
            col_no: 16,
            ttype: Comment,
            value: Some(
                " This is a multiline comment.\n    Here's the second line! "
                    .to_string(),
            ),
        },
        Token {
            line_no: 16,
            col_no: 5,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 6,
            ttype: ID,
            value: Some("discnumber".to_string()),
        },
        Token {
            line_no: 16,
            col_no: 16,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 18,
            ttype: QuestionMark,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 20,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 21,
            ttype: ID,
            value: Some("num".to_string()),
        },
        Token {
            line_no: 16,
            col_no: 24,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 25,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 26,
            ttype: ID,
            value: Some("discnumber".to_string()),
        },
        Token {
            line_no: 16,
            col_no: 36,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 37,
            ttype: Comma,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 39,
            ttype: Integer,
            value: Some("1".to_string()),
        },
        Token {
            line_no: 16,
            col_no: 40,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 42,
            ttype: Colon,
            value: None,
        },
        Token {
            line_no: 16,
            col_no: 44,
            ttype: String,
            value: Some("".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 5,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 6,
            ttype: ID,
            value: Some("tracknumber".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 17,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 19,
            ttype: Ampersand,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 21,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 22,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 23,
            ttype: ID,
            value: Some("num".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 26,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 27,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 28,
            ttype: ID,
            value: Some("tracknumber".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 39,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 40,
            ttype: Comma,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 42,
            ttype: Integer,
            value: Some("2".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 43,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 17,
            col_no: 44,
            ttype: String,
            value: Some(" - ".to_string()),
        },
        Token {
            line_no: 17,
            col_no: 49,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 5,
            ttype: Dollar,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 6,
            ttype: ID,
            value: Some("if".to_string()),
        },
        Token {
            line_no: 18,
            col_no: 8,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 9,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 10,
            ttype: ID,
            value: Some("albumartist".to_string()),
        },
        Token {
            line_no: 18,
            col_no: 21,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 22,
            ttype: Comma,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 24,
            ttype: ParenthesisLeft,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 25,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 26,
            ttype: ID,
            value: Some("artist".to_string()),
        },
        Token {
            line_no: 18,
            col_no: 32,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 33,
            ttype: String,
            value: Some(" - ".to_string()),
        },
        Token {
            line_no: 18,
            col_no: 38,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 39,
            ttype: Comma,
            value: None,
        },
        Token {
            line_no: 18,
            col_no: 41,
            ttype: String,
            value: Some("".to_string()),
        },
        Token {
            line_no: 18,
            col_no: 43,
            ttype: ParenthesisRight,
            value: None,
        },
        Token {
            line_no: 19,
            col_no: 5,
            ttype: AngleBracketLeft,
            value: None,
        },
        Token {
            line_no: 19,
            col_no: 6,
            ttype: ID,
            value: Some("title".to_string()),
        },
        Token {
            line_no: 19,
            col_no: 11,
            ttype: AngleBracketRight,
            value: None,
        },
        Token {
            line_no: 20,
            col_no: 1,
            ttype: CurlyBraceRight,
            value: None,
        },
    ];

    file_test("typical_input.tfmt", Some(reference))
}

#[test]
fn empty_text() -> Result<()> {
    match Lexer::new("") {
        Ok(_) => Err(anyhow!("Lexer should fail with empty text!")),
        Err(_) => Ok(()),
    }
}
