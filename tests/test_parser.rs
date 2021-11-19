use anyhow::Result;
use tfmttools::tfmt::ast::node::{self, Expression};
use tfmttools::tfmt::ast::Parser;
use tfmttools::tfmt::token::{Token, TokenType};
mod common;

fn file_test(filename: &str, reference: Option<node::Program>) -> Result<()> {
    let input = common::get_script(filename)?;

    let mut parser = Parser::new(&input)?;

    let program = parser.parse()?;

    if let Some(reference) = reference {
        assert_eq!(program, reference)
    }

    Ok(())
}

#[test]
fn parser_simple_input_test() -> Result<()> {
    let reference = node::Program {
        name: Token::new(TokenType::ID("simple_input".to_string()), 1, 1),
        parameters: node::Parameters {
            parameters: Vec::new(),
        },
        description: None,
        block: node::Block {
            expressions: vec![
                Expression::Tag {
                    start_token: Token::new(TokenType::AngleBracketLeft, 2, 5),
                    token: Token::new(
                        TokenType::ID("artist".to_string()),
                        2,
                        6,
                    ),
                },
                Expression::StringNode(Token::new(
                    TokenType::String("/".to_string()),
                    2,
                    14,
                )),
                Expression::Tag {
                    start_token: Token::new(TokenType::AngleBracketLeft, 2, 18),
                    token: Token::new(
                        TokenType::ID("title".to_string()),
                        2,
                        19,
                    ),
                },
            ],
        },
    };
    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn parser_typical_input_test() -> Result<()> {
    let reference = node::Program {
        name: Token::new( TokenType::ID("typical_input".to_string()), 1, 1),
        parameters: node::Parameters {
            parameters: vec![
                node::Parameter {
                    token: Token::new( TokenType::ID("folder".to_string()), 1, 15),
                    default: Some(
                        Token::new( TokenType::String("destination".to_string()), 1, 22),
                    ),
                },
            ],
        },
        description: Some(
            Token::new( TokenType::String("This file is used to test tfmttools.".to_string()), 1, 37),
        ),
        block: node::Block {
            expressions: vec![
                Expression::Symbol(
                    Token::new( TokenType::ID("folder".to_string()), 3, 7),
                ),
                Expression::StringNode(
                    Token::new( TokenType::String("/".to_string()), 3, 15),
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token::new( TokenType::AngleBracketLeft, 4, 5),
                        token: Token::new( TokenType::ID("albumartist".to_string()), 4, 6),
                    }),
                    operator: Token::new( TokenType::VerticalBar, 4, 19),
                    right: Box::new(Expression::Tag {
                        start_token: Token::new( TokenType::AngleBracketLeft, 4, 21),
                        token: Token::new( TokenType::ID("artist".to_string()), 4, 22),
                    }),
                },
                Expression::StringNode(
                    Token::new( TokenType::String("/".to_string()), 5, 5),
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::BinaryOp {
                                left: Box::new(Expression::Tag {
                                    start_token: Token::new( TokenType::AngleBracketLeft, 8, 9),
                                    token: Token::new( TokenType::ID("date".to_string()), 8, 10),
                                }),
                                operator: Token::new( TokenType::Ampersand, 8, 16),
                                right: Box::new(Expression::Group {
                                    expressions: vec![
                                        Expression::Function {
                                            start_token: Token::new( TokenType::ID("year_from_date".to_string()), 9, 14),
                                            arguments: vec![
                                                Expression::Tag {
                                                    start_token: Token::new( TokenType::AngleBracketLeft, 9, 29),
                                                    token: Token::new( TokenType::ID("date".to_string()), 9, 30),
                                                },
                                            ],
                                            end_token: Token::new( TokenType::ParenthesisRight, 9, 35),
                                        },
                                        Expression::BinaryOp {
                                            left: Box::new(Expression::Tag {
                                                start_token: Token::new( TokenType::AngleBracketLeft, 10, 13),
                                                token: Token::new( TokenType::ID("albumsort".to_string()), 10, 14),
                                            }),
                                            operator: Token::new( TokenType::Ampersand, 10, 25),
                                            right: Box::new(Expression::Group {
                                                expressions: vec![
                                                    Expression::StringNode(
                                                        Token::new( TokenType::String(".".to_string()), 10, 28),
                                                    ),
                                                    Expression::Function {
                                                        start_token: Token::new( TokenType::ID("num".to_string()), 10, 33),
                                                        arguments: vec![
                                                            Expression::Tag {
                                                                start_token: Token::new( TokenType::AngleBracketLeft, 10, 37),
                                                                token: Token::new( TokenType::ID("albumsort".to_string()), 10, 38),
                                                            },
                                                            Expression::IntegerNode(
                                                                Token::new( TokenType::Integer(2), 10, 50),
                                                            ),
                                                        ],
                                                        end_token: Token::new( TokenType::ParenthesisRight, 10, 51),
                                                    },
                                                ],
                                            }),
                                        },
                                        Expression::StringNode(
                                            Token::new( TokenType::String(" - ".to_string()), 11, 13),
                                        ),
                                    ],
                                }),
                            },
                            Expression::Tag {
                                start_token: Token::new( TokenType::AngleBracketLeft, 13, 9),
                                token: Token::new( TokenType::ID("album".to_string()), 13, 10),
                            },
                        ],
                    }),
                    operator: Token::new( TokenType::DoubleAmpersand, 14, 7),
                    right: Box::new(Expression::StringNode(
                        Token::new( TokenType::String("/".to_string()), 14, 10),
                    )),
                },
                Expression::TernaryOp {
                    condition: Box::new(Expression::Tag {
                        start_token: Token::new( TokenType::AngleBracketLeft, 16, 5),
                        token: Token::new( TokenType::ID("discnumber".to_string()), 16, 6),
                    }),
                    true_expr: Box::new(Expression::Function {
                        start_token: Token::new( TokenType::ID("num".to_string()), 16, 21),
                        arguments: vec![
                            Expression::Tag {
                                start_token: Token::new( TokenType::AngleBracketLeft, 16, 25),
                                token: Token::new( TokenType::ID("discnumber".to_string()), 16, 26),
                            },
                            Expression::IntegerNode(
                                Token::new( TokenType::Integer(1), 16, 39),
                            ),
                        ],
                        end_token: Token::new( TokenType::ParenthesisRight, 16, 40),
                    }),
                    false_expr: Box::new(Expression::StringNode(
                        Token::new( TokenType::String("".to_string()), 16, 44),
                    )),
                },
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token::new( TokenType::AngleBracketLeft, 17, 5),
                        token: Token::new( TokenType::ID("tracknumber".to_string()), 17, 6),
                    }),
                    operator: Token::new( TokenType::Ampersand, 17, 19),
                    right: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::Function {
                                start_token: Token::new( TokenType::ID("num".to_string()), 17, 23),
                                arguments: vec![
                                    Expression::Tag {
                                        start_token: Token::new( TokenType::AngleBracketLeft, 17, 27),
                                        token: Token::new( TokenType::ID("tracknumber".to_string()), 17, 28),
                                    },
                                    Expression::IntegerNode(
                                        Token::new( TokenType::Integer(2), 17, 42),
                                    ),
                                ],
                                end_token: Token::new( TokenType::ParenthesisRight, 17, 43),
                            },
                            Expression::StringNode(
                                Token::new( TokenType::String(" - ".to_string()), 17, 44),
                            ),
                        ],
                    }),
                },
                Expression::Function {
                    start_token: Token::new( TokenType::ID("if".to_string()), 18, 6),
                    arguments: vec![
                        Expression::Tag {
                            start_token: Token::new( TokenType::AngleBracketLeft, 18, 9),
                            token: Token::new( TokenType::ID("albumartist".to_string()), 18, 10),
                        },
                        Expression::Group {
                            expressions: vec![
                                Expression::Tag {
                                    start_token: Token::new( TokenType::AngleBracketLeft, 18, 25),
                                    token: Token::new( TokenType::ID("artist".to_string()), 18, 26),
                                },
                                Expression::StringNode(
                                    Token::new( TokenType::String(" - ".to_string()), 18, 33),
                                ),
                            ],
                        },
                        Expression::StringNode(
                            Token::new( TokenType::String("".to_string()), 18, 41),
                        ),
                    ],
                    end_token: Token::new( TokenType::ParenthesisRight, 18, 43),
                },
                Expression::Tag {
                    start_token: Token::new( TokenType::AngleBracketLeft, 19, 5),
                    token: Token::new( TokenType::ID("title".to_string()), 19, 6),
                },
            ],
        },
    };
    file_test("typical_input.tfmt", Some(reference))
}
