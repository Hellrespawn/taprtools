use anyhow::{anyhow, Result};
use tfmttools::tfmt::lexer::Lexer;
use tfmttools::tfmt::token::Token;
use tfmttools::tfmt::token::TokenType::*;

mod common;

fn file_test(filename: &str, reference: Option<Vec<Token>>) -> Result<()> {
    let input = common::get_script(filename)?;

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
        Token::new(1, 1, ID, Some("simple_input".to_string()))?,
        Token::new(1, 13, ParenthesisLeft, None)?,
        Token::new(1, 14, ParenthesisRight, None)?,
        Token::new(1, 16, CurlyBraceLeft, None)?,
        Token::new(2, 5, AngleBracketLeft, None)?,
        Token::new(2, 6, ID, Some("artist".to_string()))?,
        Token::new(2, 12, AngleBracketRight, None)?,
        Token::new(2, 14, String, Some("/".to_string()))?,
        Token::new(2, 18, AngleBracketLeft, None)?,
        Token::new(2, 19, ID, Some("title".to_string()))?,
        Token::new(2, 24, AngleBracketRight, None)?,
        Token::new(3, 1, CurlyBraceRight, None)?,
    ];

    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn typical_input() -> Result<()> {
    let reference = vec![
        Token::new(1, 1, ID, Some("typical_input".to_string()))?,
        Token::new(1, 14, ParenthesisLeft, None)?,
        Token::new(1, 15, ID, Some("folder".to_string()))?,
        Token::new(1, 21, Equals, None)?,
        Token::new(1, 22, String, Some("destination".to_string()))?,
        Token::new(1, 35, ParenthesisRight, None)?,
        Token::new(
            1,
            37,
            String,
            Some("This file is used to test tfmttools.".to_string()),
        )?,
        Token::new(2, 1, CurlyBraceLeft, None)?,
        Token::new(3, 5, Dollar, None)?,
        Token::new(3, 6, ParenthesisLeft, None)?,
        Token::new(3, 7, ID, Some("folder".to_string()))?,
        Token::new(3, 13, ParenthesisRight, None)?,
        Token::new(3, 15, String, Some("/".to_string()))?,
        Token::new(4, 5, AngleBracketLeft, None)?,
        Token::new(4, 6, ID, Some("albumartist".to_string()))?,
        Token::new(4, 17, AngleBracketRight, None)?,
        Token::new(4, 19, VerticalBar, None)?,
        Token::new(4, 21, AngleBracketLeft, None)?,
        Token::new(4, 22, ID, Some("artist".to_string()))?,
        Token::new(4, 28, AngleBracketRight, None)?,
        Token::new(5, 5, String, Some("/".to_string()))?,
        Token::new(7, 5, ParenthesisLeft, None)?,
        Token::new(8, 9, AngleBracketLeft, None)?,
        Token::new(8, 10, ID, Some("date".to_string()))?,
        Token::new(8, 14, AngleBracketRight, None)?,
        Token::new(8, 16, Ampersand, None)?,
        Token::new(8, 18, ParenthesisLeft, None)?,
        Token::new(
            8,
            21,
            Comment,
            Some(" This is a single-line comment".to_string()),
        )?,
        Token::new(9, 13, Dollar, None)?,
        Token::new(9, 14, ID, Some("year_from_date".to_string()))?,
        Token::new(9, 28, ParenthesisLeft, None)?,
        Token::new(9, 29, AngleBracketLeft, None)?,
        Token::new(9, 30, ID, Some("date".to_string()))?,
        Token::new(9, 34, AngleBracketRight, None)?,
        Token::new(9, 35, ParenthesisRight, None)?,
        Token::new(10, 13, AngleBracketLeft, None)?,
        Token::new(10, 14, ID, Some("albumsort".to_string()))?,
        Token::new(10, 23, AngleBracketRight, None)?,
        Token::new(10, 25, Ampersand, None)?,
        Token::new(10, 27, ParenthesisLeft, None)?,
        Token::new(10, 28, String, Some(".".to_string()))?,
        Token::new(10, 32, Dollar, None)?,
        Token::new(10, 33, ID, Some("num".to_string()))?,
        Token::new(10, 36, ParenthesisLeft, None)?,
        Token::new(10, 37, AngleBracketLeft, None)?,
        Token::new(10, 38, ID, Some("albumsort".to_string()))?,
        Token::new(10, 47, AngleBracketRight, None)?,
        Token::new(10, 48, Comma, None)?,
        Token::new(10, 50, Integer, Some("2".to_string()))?,
        Token::new(10, 51, ParenthesisRight, None)?,
        Token::new(10, 53, ParenthesisRight, None)?,
        Token::new(11, 13, String, Some(" - ".to_string()))?,
        Token::new(12, 9, ParenthesisRight, None)?,
        Token::new(13, 9, AngleBracketLeft, None)?,
        Token::new(13, 10, ID, Some("album".to_string()))?,
        Token::new(13, 15, AngleBracketRight, None)?,
        Token::new(14, 5, ParenthesisRight, None)?,
        Token::new(14, 7, DoubleAmpersand, None)?,
        Token::new(14, 10, String, Some("/".to_string()))?,
        Token::new(
            14,
            16,
            Comment,
            Some(
                " This is a multiline comment.\n    Here's the second line! "
                    .to_string(),
            ),
        )?,
        Token::new(16, 5, AngleBracketLeft, None)?,
        Token::new(16, 6, ID, Some("discnumber".to_string()))?,
        Token::new(16, 16, AngleBracketRight, None)?,
        Token::new(16, 18, QuestionMark, None)?,
        Token::new(16, 20, Dollar, None)?,
        Token::new(16, 21, ID, Some("num".to_string()))?,
        Token::new(16, 24, ParenthesisLeft, None)?,
        Token::new(16, 25, AngleBracketLeft, None)?,
        Token::new(16, 26, ID, Some("discnumber".to_string()))?,
        Token::new(16, 36, AngleBracketRight, None)?,
        Token::new(16, 37, Comma, None)?,
        Token::new(16, 39, Integer, Some("1".to_string()))?,
        Token::new(16, 40, ParenthesisRight, None)?,
        Token::new(16, 42, Colon, None)?,
        Token::new(16, 44, String, Some("".to_string()))?,
        Token::new(17, 5, AngleBracketLeft, None)?,
        Token::new(17, 6, ID, Some("tracknumber".to_string()))?,
        Token::new(17, 17, AngleBracketRight, None)?,
        Token::new(17, 19, Ampersand, None)?,
        Token::new(17, 21, ParenthesisLeft, None)?,
        Token::new(17, 22, Dollar, None)?,
        Token::new(17, 23, ID, Some("num".to_string()))?,
        Token::new(17, 26, ParenthesisLeft, None)?,
        Token::new(17, 27, AngleBracketLeft, None)?,
        Token::new(17, 28, ID, Some("tracknumber".to_string()))?,
        Token::new(17, 39, AngleBracketRight, None)?,
        Token::new(17, 40, Comma, None)?,
        Token::new(17, 42, Integer, Some("2".to_string()))?,
        Token::new(17, 43, ParenthesisRight, None)?,
        Token::new(17, 44, String, Some(" - ".to_string()))?,
        Token::new(17, 49, ParenthesisRight, None)?,
        Token::new(18, 5, Dollar, None)?,
        Token::new(18, 6, ID, Some("if".to_string()))?,
        Token::new(18, 8, ParenthesisLeft, None)?,
        Token::new(18, 9, AngleBracketLeft, None)?,
        Token::new(18, 10, ID, Some("albumartist".to_string()))?,
        Token::new(18, 21, AngleBracketRight, None)?,
        Token::new(18, 22, Comma, None)?,
        Token::new(18, 24, ParenthesisLeft, None)?,
        Token::new(18, 25, AngleBracketLeft, None)?,
        Token::new(18, 26, ID, Some("artist".to_string()))?,
        Token::new(18, 32, AngleBracketRight, None)?,
        Token::new(18, 33, String, Some(" - ".to_string()))?,
        Token::new(18, 38, ParenthesisRight, None)?,
        Token::new(18, 39, Comma, None)?,
        Token::new(18, 41, String, Some("".to_string()))?,
        Token::new(18, 43, ParenthesisRight, None)?,
        Token::new(19, 5, AngleBracketLeft, None)?,
        Token::new(19, 6, ID, Some("title".to_string()))?,
        Token::new(19, 11, AngleBracketRight, None)?,
        Token::new(20, 1, CurlyBraceRight, None)?,
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
