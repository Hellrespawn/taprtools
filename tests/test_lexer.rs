use anyhow::Result;
use tfmttools::helpers::normalize_newlines;
use tfmttools::tfmt::lexer::Lexer;
use tfmttools::tfmt::token::Token;
use tfmttools::tfmt::token::TokenType::*;

mod common;

fn file_test(filename: &str, reference: Option<&[Token]>) -> Result<()> {
    let input = common::get_script(filename)?;

    if let Some(tokens) = reference {
        lexer_test(&input, tokens)
    } else {
        run_lexer(&input)?;
        Ok(())
    }
}

fn lexer_test(string: &str, reference: &[Token]) -> Result<()> {
    let tokens = run_lexer(string)?;

    println!("{:#?}", tokens);

    for (t1, t2) in tokens.iter().zip(reference) {
        assert_eq!(t1, t2)
    }

    assert_eq!(tokens, reference);

    Ok(())
}

fn run_lexer(string: &str) -> Result<Vec<Token>> {
    let string = normalize_newlines(&string);
    let lex = Lexer::new(&string)?;

    let tokens = lex.collect::<Result<Vec<Token>, _>>()?;

    Ok(tokens)
}

#[test]
fn new_lexer_simple_input_test() -> Result<()> {
    let reference = &[
        Token::new(ID("simple_input".to_string()), 1, 1),
        Token::new(ParenthesisLeft, 1, 13),
        Token::new(ParenthesisRight, 1, 14),
        Token::new(CurlyBraceLeft, 1, 16),
        Token::new(AngleBracketLeft, 2, 5),
        Token::new(ID("artist".to_string()), 2, 6),
        Token::new(AngleBracketRight, 2, 12),
        Token::new(String("/".to_string()), 2, 14),
        Token::new(AngleBracketLeft, 2, 18),
        Token::new(ID("title".to_string()), 2, 19),
        Token::new(AngleBracketRight, 2, 24),
        Token::new(CurlyBraceRight, 3, 1),
    ];

    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn new_lexer_typical_input_test() -> Result<()> {
    let reference = &[
        Token::new(ID("typical_input".to_string()), 1, 1),
        Token::new(ParenthesisLeft, 1, 14),
        Token::new(ID("folder".to_string()), 1, 15),
        Token::new(Equals, 1, 21),
        Token::new(String("destination".to_string()), 1, 22),
        Token::new(ParenthesisRight, 1, 35),
        Token::new(
            String("This file is used to test tfmttools.".to_string()),
            1,
            37,
        ),
        Token::new(CurlyBraceLeft, 2, 1),
        Token::new(Dollar, 3, 5),
        Token::new(ParenthesisLeft, 3, 6),
        Token::new(ID("folder".to_string()), 3, 7),
        Token::new(ParenthesisRight, 3, 13),
        Token::new(String("/".to_string()), 3, 15),
        Token::new(AngleBracketLeft, 4, 5),
        Token::new(ID("albumartist".to_string()), 4, 6),
        Token::new(AngleBracketRight, 4, 17),
        Token::new(VerticalBar, 4, 19),
        Token::new(AngleBracketLeft, 4, 21),
        Token::new(ID("artist".to_string()), 4, 22),
        Token::new(AngleBracketRight, 4, 28),
        Token::new(String("/".to_string()), 5, 5),
        Token::new(ParenthesisLeft, 7, 5),
        Token::new(AngleBracketLeft, 8, 9),
        Token::new(ID("date".to_string()), 8, 10),
        Token::new(AngleBracketRight, 8, 14),
        Token::new(Ampersand, 8, 16),
        Token::new(ParenthesisLeft, 8, 18),
        Token::new(
            Comment(" This is a single-line comment".to_string()),
            8,
            20,
        ),
        Token::new(Dollar, 9, 13),
        Token::new(ID("year_from_date".to_string()), 9, 14),
        Token::new(ParenthesisLeft, 9, 28),
        Token::new(AngleBracketLeft, 9, 29),
        Token::new(ID("date".to_string()), 9, 30),
        Token::new(AngleBracketRight, 9, 34),
        Token::new(ParenthesisRight, 9, 35),
        Token::new(AngleBracketLeft, 10, 13),
        Token::new(ID("albumsort".to_string()), 10, 14),
        Token::new(AngleBracketRight, 10, 23),
        Token::new(Ampersand, 10, 25),
        Token::new(ParenthesisLeft, 10, 27),
        Token::new(String(".".to_string()), 10, 28),
        Token::new(Dollar, 10, 32),
        Token::new(ID("num".to_string()), 10, 33),
        Token::new(ParenthesisLeft, 10, 36),
        Token::new(AngleBracketLeft, 10, 37),
        Token::new(ID("albumsort".to_string()), 10, 38),
        Token::new(AngleBracketRight, 10, 47),
        Token::new(Comma, 10, 48),
        Token::new(Integer(2), 10, 50),
        Token::new(ParenthesisRight, 10, 51),
        Token::new(ParenthesisRight, 10, 53),
        Token::new(String(" - ".to_string()), 11, 13),
        Token::new(ParenthesisRight, 12, 9),
        Token::new(AngleBracketLeft, 13, 9),
        Token::new(ID("album".to_string()), 13, 10),
        Token::new(AngleBracketRight, 13, 15),
        Token::new(ParenthesisRight, 14, 5),
        Token::new(DoubleAmpersand, 14, 7),
        Token::new(String("/".to_string()), 14, 10),
        Token::new(
            Comment(
                " This is a multiline comment.\n    Here's the second line! "
                    .to_string(),
            ),
            14,
            14,
        ),
        Token::new(AngleBracketLeft, 16, 5),
        Token::new(ID("discnumber".to_string()), 16, 6),
        Token::new(AngleBracketRight, 16, 16),
        Token::new(QuestionMark, 16, 18),
        Token::new(Dollar, 16, 20),
        Token::new(ID("num".to_string()), 16, 21),
        Token::new(ParenthesisLeft, 16, 24),
        Token::new(AngleBracketLeft, 16, 25),
        Token::new(ID("discnumber".to_string()), 16, 26),
        Token::new(AngleBracketRight, 16, 36),
        Token::new(Comma, 16, 37),
        Token::new(Integer(1), 16, 39),
        Token::new(ParenthesisRight, 16, 40),
        Token::new(Colon, 16, 42),
        Token::new(String("".to_string()), 16, 44),
        Token::new(AngleBracketLeft, 17, 5),
        Token::new(ID("tracknumber".to_string()), 17, 6),
        Token::new(AngleBracketRight, 17, 17),
        Token::new(Ampersand, 17, 19),
        Token::new(ParenthesisLeft, 17, 21),
        Token::new(Dollar, 17, 22),
        Token::new(ID("num".to_string()), 17, 23),
        Token::new(ParenthesisLeft, 17, 26),
        Token::new(AngleBracketLeft, 17, 27),
        Token::new(ID("tracknumber".to_string()), 17, 28),
        Token::new(AngleBracketRight, 17, 39),
        Token::new(Comma, 17, 40),
        Token::new(Integer(2), 17, 42),
        Token::new(ParenthesisRight, 17, 43),
        Token::new(String(" - ".to_string()), 17, 44),
        Token::new(ParenthesisRight, 17, 49),
        Token::new(Dollar, 18, 5),
        Token::new(ID("if".to_string()), 18, 6),
        Token::new(ParenthesisLeft, 18, 8),
        Token::new(AngleBracketLeft, 18, 9),
        Token::new(ID("albumartist".to_string()), 18, 10),
        Token::new(AngleBracketRight, 18, 21),
        Token::new(Comma, 18, 22),
        Token::new(ParenthesisLeft, 18, 24),
        Token::new(AngleBracketLeft, 18, 25),
        Token::new(ID("artist".to_string()), 18, 26),
        Token::new(AngleBracketRight, 18, 32),
        Token::new(String(" - ".to_string()), 18, 33),
        Token::new(ParenthesisRight, 18, 38),
        Token::new(Comma, 18, 39),
        Token::new(String("".to_string()), 18, 41),
        Token::new(ParenthesisRight, 18, 43),
        Token::new(AngleBracketLeft, 19, 5),
        Token::new(ID("title".to_string()), 19, 6),
        Token::new(AngleBracketRight, 19, 11),
        Token::new(CurlyBraceRight, 20, 1),
    ];

    file_test("typical_input.tfmt", Some(reference))
}

// TODO Write test with weird unicode characters.
