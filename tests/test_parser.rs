use anyhow::Result;

use tfmttools::tfmt::ast::{self, Expression};
use tfmttools::tfmt::lexer::Lexer;
use tfmttools::tfmt::parser::Parser;
use tfmttools::tfmt::token::{Token, TokenType};

mod common;

fn file_test(filename: &str, reference: Option<ast::Program>) -> Result<()> {
    let input = common::get_script(filename)?;

    let mut parser = Parser::<Lexer>::from_string(&input)?;

    let root = parser.parse()?;

    if let Some(reference) = reference {
        assert_eq!(root, reference)
    }

    Ok(())
}

#[test]
fn simple_input() -> Result<()> {
    let reference = ast::Program {
        name: Token::new(
            1,
            1,
            TokenType::ID,
            Some("simple_input".to_string()),
        )?,
        parameters: ast::Parameters {
            parameters: Vec::new(),
        },
        description: None,
        block: ast::Block {
            drive: None,
            expressions: vec![
                Expression::Tag {
                    start_token: Token::new(
                        2,
                        5,
                        TokenType::AngleBracketLeft,
                        None,
                    )?,
                    token: Token::new(
                        2,
                        6,
                        TokenType::ID,
                        Some("artist".to_string()),
                    )?,
                },
                Expression::StringNode(Token::new(
                    2,
                    14,
                    TokenType::String,
                    Some("/".to_string()),
                )?),
                Expression::Tag {
                    start_token: Token::new(
                        2,
                        18,
                        TokenType::AngleBracketLeft,
                        None,
                    )?,
                    token: Token::new(
                        2,
                        19,
                        TokenType::ID,
                        Some("title".to_string()),
                    )?,
                },
            ],
        },
    };
    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn typical_input() -> Result<()> {
    let reference = ast::Program {
        name: Token::new(1, 1, TokenType::ID, Some("typical_input".to_string(),))?,
        parameters: ast::Parameters {
            parameters: vec![
                ast::Parameter {
                    token: Token::new(1, 15, TokenType::ID, Some("folder".to_string(),))?,
                    default: Some(
                        Token::new(1, 22, TokenType::String, Some("destination".to_string(),))?,
                    ),
                },
            ],
        },
        description: Some(
            Token::new(1, 37, TokenType::String, Some("This file is used to test tfmttools.".to_string(),))?,
        ),
        block: ast::Block {
            drive: None,
            expressions: vec![
                Expression::Substitution(
                    Token::new(3, 7, TokenType::ID, Some("folder".to_string(),))?,
                ),
                Expression::StringNode(
                    Token::new(3, 15, TokenType::String, Some("/".to_string(),))?,
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token::new(4, 5, TokenType::AngleBracketLeft, None)?,
                        token: Token::new(4, 6, TokenType::ID, Some("albumartist".to_string(),))?,
                    }),
                    token: Token::new(4, 19, TokenType::VerticalBar, None)?,
                    right: Box::new(Expression::Tag {
                        start_token: Token::new(4, 21, TokenType::AngleBracketLeft, None)?,
                        token: Token::new(4, 22, TokenType::ID, Some("artist".to_string(),))?,
                    }),
                },
                Expression::StringNode(
                    Token::new(5, 5, TokenType::String, Some("/".to_string(),))?,
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::BinaryOp {
                                left: Box::new(Expression::Tag {
                                    start_token: Token::new(8, 9, TokenType::AngleBracketLeft, None)?,
                                    token: Token::new(8, 10, TokenType::ID, Some("date".to_string(),))?,
                                }),
                                token: Token::new(8, 16, TokenType::Ampersand, None)?,
                                right: Box::new(Expression::Group {
                                    expressions: vec![
                                        Expression::Function {
                                            start_token: Token::new(9, 14, TokenType::ID, Some("year_from_date".to_string(),))?,
                                            arguments: vec![
                                                Expression::Tag {
                                                    start_token: Token::new(9, 29, TokenType::AngleBracketLeft, None)?,
                                                    token: Token::new(9, 30, TokenType::ID, Some("date".to_string(),))?,
                                                },
                                            ],
                                            end_token: Token::new(9, 35, TokenType::ParenthesisRight, None)?,
                                        },
                                        Expression::BinaryOp {
                                            left: Box::new(Expression::Tag {
                                                start_token: Token::new(10, 13, TokenType::AngleBracketLeft, None)?,
                                                token: Token::new(10, 14, TokenType::ID, Some("albumsort".to_string(),))?,
                                            }),
                                            token: Token::new(10, 25, TokenType::Ampersand, None)?,
                                            right: Box::new(Expression::Group {
                                                expressions: vec![
                                                    Expression::StringNode(
                                                        Token::new(10, 28, TokenType::String, Some(".".to_string(),))?,
                                                    ),
                                                    Expression::Function {
                                                        start_token: Token::new(10, 33, TokenType::ID, Some("num".to_string(),))?,
                                                        arguments: vec![
                                                            Expression::Tag {
                                                                start_token: Token::new(10, 37, TokenType::AngleBracketLeft, None)?,
                                                                token: Token::new(10, 38, TokenType::ID, Some("albumsort".to_string(),))?,
                                                            },
                                                            Expression::IntegerNode(
                                                                Token::new(10, 50, TokenType::Integer, Some("2".to_string(),))?,
                                                            ),
                                                        ],
                                                        end_token: Token::new(10, 51, TokenType::ParenthesisRight, None)?,
                                                    },
                                                ],
                                            }),
                                        },
                                        Expression::StringNode(
                                            Token::new(11, 13, TokenType::String, Some(" - ".to_string(),))?,
                                        ),
                                    ],
                                }),
                            },
                            Expression::Tag {
                                start_token: Token::new(13, 9, TokenType::AngleBracketLeft, None)?,
                                token: Token::new(13, 10, TokenType::ID, Some("album".to_string(),))?,
                            },
                        ],
                    }),
                    token: Token::new(14, 7, TokenType::DoubleAmpersand, None)?,
                    right: Box::new(Expression::StringNode(
                        Token::new(14, 10, TokenType::String, Some("/".to_string(),))?,
                    )),
                },
                Expression::TernaryOp {
                    condition: Box::new(Expression::Tag {
                        start_token: Token::new(16, 5, TokenType::AngleBracketLeft, None)?,
                        token: Token::new(16, 6, TokenType::ID, Some("discnumber".to_string(),))?,
                    }),
                    true_expr: Box::new(Expression::Function {
                        start_token: Token::new(16, 21, TokenType::ID, Some("num".to_string(),))?,
                        arguments: vec![
                            Expression::Tag {
                                start_token: Token::new(16, 25, TokenType::AngleBracketLeft, None)?,
                                token: Token::new(16, 26, TokenType::ID, Some("discnumber".to_string(),))?,
                            },
                            Expression::IntegerNode(
                                Token::new(16, 39, TokenType::Integer, Some("1".to_string(),))?,
                            ),
                        ],
                        end_token: Token::new(16, 40, TokenType::ParenthesisRight, None)?,
                    }),
                    false_expr: Box::new(Expression::StringNode(
                        Token::new(16, 44, TokenType::String, Some("".to_string(),))?,
                    )),
                },
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token::new(17, 5, TokenType::AngleBracketLeft, None)?,
                        token: Token::new(17, 6, TokenType::ID, Some("tracknumber".to_string(),))?,
                    }),
                    token: Token::new(17, 19, TokenType::Ampersand, None)?,
                    right: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::Function {
                                start_token: Token::new(17, 23, TokenType::ID, Some("num".to_string(),))?,
                                arguments: vec![
                                    Expression::Tag {
                                        start_token: Token::new(17, 27, TokenType::AngleBracketLeft, None)?,
                                        token: Token::new(17, 28, TokenType::ID, Some("tracknumber".to_string(),))?,
                                    },
                                    Expression::IntegerNode(
                                        Token::new(17, 42, TokenType::Integer, Some("2".to_string(),))?,
                                    ),
                                ],
                                end_token: Token::new(17, 43, TokenType::ParenthesisRight, None)?,
                            },
                            Expression::StringNode(
                                Token::new(17, 44, TokenType::String, Some(" - ".to_string(),))?,
                            ),
                        ],
                    }),
                },
                Expression::Function {
                    start_token: Token::new(18, 6, TokenType::ID, Some("if".to_string(),))?,
                    arguments: vec![
                        Expression::Tag {
                            start_token: Token::new(18, 9, TokenType::AngleBracketLeft, None)?,
                            token: Token::new(18, 10, TokenType::ID, Some("albumartist".to_string(),))?,
                        },
                        Expression::Group {
                            expressions: vec![
                                Expression::Tag {
                                    start_token: Token::new(18, 25, TokenType::AngleBracketLeft, None)?,
                                    token: Token::new(18, 26, TokenType::ID, Some("artist".to_string(),))?,
                                },
                                Expression::StringNode(
                                    Token::new(18, 33, TokenType::String, Some(" - ".to_string(),))?,
                                ),
                            ],
                        },
                        Expression::StringNode(
                            Token::new(18, 41, TokenType::String, Some("".to_string(),))?,
                        ),
                    ],
                    end_token: Token::new(18, 43, TokenType::ParenthesisRight, None)?,
                },
                Expression::Tag {
                    start_token: Token::new(19, 5, TokenType::AngleBracketLeft, None)?,
                    token: Token::new(19, 6, TokenType::ID, Some("title".to_string(),))?,
                },
            ],
        },
    };
    file_test("typical_input.tfmt", Some(reference))
}
