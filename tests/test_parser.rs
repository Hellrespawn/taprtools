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
        name: Token {
            line_no: 1,
            col_no: 1,
            ttype: TokenType::ID,
            value: Some("simple_input".to_string()),
        },
        parameters: ast::Parameters {
            parameters: Vec::new(),
        },
        description: None,
        block: ast::Block {
            drive: None,
            expressions: vec![
                Expression::Tag {
                    start_token: Token {
                        line_no: 2,
                        col_no: 5,
                        ttype: TokenType::AngleBracketLeft,
                        value: None,
                    },
                    token: Token {
                        line_no: 2,
                        col_no: 6,
                        ttype: TokenType::ID,
                        value: Some("artist".to_string()),
                    },
                },
                Expression::StringNode(Token {
                    line_no: 2,
                    col_no: 14,
                    ttype: TokenType::String,
                    value: Some("/".to_string()),
                }),
                Expression::Tag {
                    start_token: Token {
                        line_no: 2,
                        col_no: 18,
                        ttype: TokenType::AngleBracketLeft,
                        value: None,
                    },
                    token: Token {
                        line_no: 2,
                        col_no: 19,
                        ttype: TokenType::ID,
                        value: Some("title".to_string()),
                    },
                },
            ],
        },
    };
    file_test("simple_input.tfmt", Some(reference))
}

#[test]
fn typical_input() -> Result<()> {
    let reference = ast::Program {
        name: Token {
            line_no: 1,
            col_no: 1,
            ttype: TokenType::ID,
            value: Some(
                "typical_input".to_string(),
            ),
        },
        parameters: ast::Parameters {
            parameters: vec![
                ast::Parameter {
                    token: Token {
                        line_no: 1,
                        col_no: 15,
                        ttype: TokenType::ID,
                        value: Some(
                            "folder".to_string(),
                        ),
                    },
                    default: Some(
                        Token {
                            line_no: 1,
                            col_no: 22,
                            ttype: TokenType::String,
                            value: Some(
                                "destination".to_string(),
                            ),
                        },
                    ),
                },
            ],
        },
        description: Some(
            Token {
                line_no: 1,
                col_no: 37,
                ttype: TokenType::String,
                value: Some(
                    "This file is used to test tfmttools.".to_string(),
                ),
            },
        ),
        block: ast::Block {
            drive: None,
            expressions: vec![
                Expression::Substitution(
                    Token {
                        line_no: 3,
                        col_no: 7,
                        ttype: TokenType::ID,
                        value: Some(
                            "folder".to_string(),
                        ),
                    },
                ),
                Expression::StringNode(
                    Token {
                        line_no: 3,
                        col_no: 15,
                        ttype: TokenType::String,
                        value: Some(
                            "/".to_string(),
                        ),
                    },
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token {
                            line_no: 4,
                            col_no: 5,
                            ttype: TokenType::AngleBracketLeft,
                            value: None,
                        },
                        token: Token {
                            line_no: 4,
                            col_no: 6,
                            ttype: TokenType::ID,
                            value: Some(
                                "albumartist".to_string(),
                            ),
                        },
                    }),
                    token: Token {
                        line_no: 4,
                        col_no: 19,
                        ttype: TokenType::VerticalBar,
                        value: None,
                    },
                    right: Box::new(Expression::Tag {
                        start_token: Token {
                            line_no: 4,
                            col_no: 21,
                            ttype: TokenType::AngleBracketLeft,
                            value: None,
                        },
                        token: Token {
                            line_no: 4,
                            col_no: 22,
                            ttype: TokenType::ID,
                            value: Some(
                                "artist".to_string(),
                            ),
                        },
                    }),
                },
                Expression::StringNode(
                    Token {
                        line_no: 5,
                        col_no: 5,
                        ttype: TokenType::String,
                        value: Some(
                            "/".to_string(),
                        ),
                    },
                ),
                Expression::BinaryOp {
                    left: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::BinaryOp {
                                left: Box::new(Expression::Tag {
                                    start_token: Token {
                                        line_no: 8,
                                        col_no: 9,
                                        ttype: TokenType::AngleBracketLeft,
                                        value: None,
                                    },
                                    token: Token {
                                        line_no: 8,
                                        col_no: 10,
                                        ttype: TokenType::ID,
                                        value: Some(
                                            "date".to_string(),
                                        ),
                                    },
                                }),
                                token: Token {
                                    line_no: 8,
                                    col_no: 16,
                                    ttype: TokenType::Ampersand,
                                    value: None,
                                },
                                right: Box::new(Expression::Group {
                                    expressions: vec![
                                        Expression::Function {
                                            start_token: Token {
                                                line_no: 9,
                                                col_no: 14,
                                                ttype: TokenType::ID,
                                                value: Some(
                                                    "year_from_date".to_string(),
                                                ),
                                            },
                                            arguments: vec![
                                                Expression::Tag {
                                                    start_token: Token {
                                                        line_no: 9,
                                                        col_no: 29,
                                                        ttype: TokenType::AngleBracketLeft,
                                                        value: None,
                                                    },
                                                    token: Token {
                                                        line_no: 9,
                                                        col_no: 30,
                                                        ttype: TokenType::ID,
                                                        value: Some(
                                                            "date".to_string(),
                                                        ),
                                                    },
                                                },
                                            ],
                                            end_token: Token {
                                                line_no: 9,
                                                col_no: 35,
                                                ttype: TokenType::ParenthesisRight,
                                                value: None,
                                            },
                                        },
                                        Expression::BinaryOp {
                                            left: Box::new(Expression::Tag {
                                                start_token: Token {
                                                    line_no: 10,
                                                    col_no: 13,
                                                    ttype: TokenType::AngleBracketLeft,
                                                    value: None,
                                                },
                                                token: Token {
                                                    line_no: 10,
                                                    col_no: 14,
                                                    ttype: TokenType::ID,
                                                    value: Some(
                                                        "albumsort".to_string(),
                                                    ),
                                                },
                                            }),
                                            token: Token {
                                                line_no: 10,
                                                col_no: 25,
                                                ttype: TokenType::Ampersand,
                                                value: None,
                                            },
                                            right: Box::new(Expression::Group {
                                                expressions: vec![
                                                    Expression::StringNode(
                                                        Token {
                                                            line_no: 10,
                                                            col_no: 28,
                                                            ttype: TokenType::String,
                                                            value: Some(
                                                                ".".to_string(),
                                                            ),
                                                        },
                                                    ),
                                                    Expression::Function {
                                                        start_token: Token {
                                                            line_no: 10,
                                                            col_no: 33,
                                                            ttype: TokenType::ID,
                                                            value: Some(
                                                                "num".to_string(),
                                                            ),
                                                        },
                                                        arguments: vec![
                                                            Expression::Tag {
                                                                start_token: Token {
                                                                    line_no: 10,
                                                                    col_no: 37,
                                                                    ttype: TokenType::AngleBracketLeft,
                                                                    value: None,
                                                                },
                                                                token: Token {
                                                                    line_no: 10,
                                                                    col_no: 38,
                                                                    ttype: TokenType::ID,
                                                                    value: Some(
                                                                        "albumsort".to_string(),
                                                                    ),
                                                                },
                                                            },
                                                            Expression::IntegerNode(
                                                                Token {
                                                                    line_no: 10,
                                                                    col_no: 50,
                                                                    ttype: TokenType::Integer,
                                                                    value: Some(
                                                                        "2".to_string(),
                                                                    ),
                                                                },
                                                            ),
                                                        ],
                                                        end_token: Token {
                                                            line_no: 10,
                                                            col_no: 51,
                                                            ttype: TokenType::ParenthesisRight,
                                                            value: None,
                                                        },
                                                    },
                                                ],
                                            }),
                                        },
                                        Expression::StringNode(
                                            Token {
                                                line_no: 11,
                                                col_no: 13,
                                                ttype: TokenType::String,
                                                value: Some(
                                                    " - ".to_string(),
                                                ),
                                            },
                                        ),
                                    ],
                                }),
                            },
                            Expression::Tag {
                                start_token: Token {
                                    line_no: 13,
                                    col_no: 9,
                                    ttype: TokenType::AngleBracketLeft,
                                    value: None,
                                },
                                token: Token {
                                    line_no: 13,
                                    col_no: 10,
                                    ttype: TokenType::ID,
                                    value: Some(
                                        "album".to_string(),
                                    ),
                                },
                            },
                        ],
                    }),
                    token: Token {
                        line_no: 14,
                        col_no: 7,
                        ttype: TokenType::DoubleAmpersand,
                        value: None,
                    },
                    right: Box::new(Expression::StringNode(
                        Token {
                            line_no: 14,
                            col_no: 10,
                            ttype: TokenType::String,
                            value: Some(
                                "/".to_string(),
                            ),
                        },
                    )),
                },
                Expression::TernaryOp {
                    condition: Box::new(Expression::Tag {
                        start_token: Token {
                            line_no: 16,
                            col_no: 5,
                            ttype: TokenType::AngleBracketLeft,
                            value: None,
                        },
                        token: Token {
                            line_no: 16,
                            col_no: 6,
                            ttype: TokenType::ID,
                            value: Some(
                                "discnumber".to_string(),
                            ),
                        },
                    }),
                    true_expr: Box::new(Expression::Function {
                        start_token: Token {
                            line_no: 16,
                            col_no: 21,
                            ttype: TokenType::ID,
                            value: Some(
                                "num".to_string(),
                            ),
                        },
                        arguments: vec![
                            Expression::Tag {
                                start_token: Token {
                                    line_no: 16,
                                    col_no: 25,
                                    ttype: TokenType::AngleBracketLeft,
                                    value: None,
                                },
                                token: Token {
                                    line_no: 16,
                                    col_no: 26,
                                    ttype: TokenType::ID,
                                    value: Some(
                                        "discnumber".to_string(),
                                    ),
                                },
                            },
                            Expression::IntegerNode(
                                Token {
                                    line_no: 16,
                                    col_no: 39,
                                    ttype: TokenType::Integer,
                                    value: Some(
                                        "1".to_string(),
                                    ),
                                },
                            ),
                        ],
                        end_token: Token {
                            line_no: 16,
                            col_no: 40,
                            ttype: TokenType::ParenthesisRight,
                            value: None,
                        },
                    }),
                    false_expr: Box::new(Expression::StringNode(
                        Token {
                            line_no: 16,
                            col_no: 44,
                            ttype: TokenType::String,
                            value: Some(
                                "".to_string(),
                            ),
                        },
                    )),
                },
                Expression::BinaryOp {
                    left: Box::new(Expression::Tag {
                        start_token: Token {
                            line_no: 17,
                            col_no: 5,
                            ttype: TokenType::AngleBracketLeft,
                            value: None,
                        },
                        token: Token {
                            line_no: 17,
                            col_no: 6,
                            ttype: TokenType::ID,
                            value: Some(
                                "tracknumber".to_string(),
                            ),
                        },
                    }),
                    token: Token {
                        line_no: 17,
                        col_no: 19,
                        ttype: TokenType::Ampersand,
                        value: None,
                    },
                    right: Box::new(Expression::Group {
                        expressions: vec![
                            Expression::Function {
                                start_token: Token {
                                    line_no: 17,
                                    col_no: 23,
                                    ttype: TokenType::ID,
                                    value: Some(
                                        "num".to_string(),
                                    ),
                                },
                                arguments: vec![
                                    Expression::Tag {
                                        start_token: Token {
                                            line_no: 17,
                                            col_no: 27,
                                            ttype: TokenType::AngleBracketLeft,
                                            value: None,
                                        },
                                        token: Token {
                                            line_no: 17,
                                            col_no: 28,
                                            ttype: TokenType::ID,
                                            value: Some(
                                                "tracknumber".to_string(),
                                            ),
                                        },
                                    },
                                    Expression::IntegerNode(
                                        Token {
                                            line_no: 17,
                                            col_no: 42,
                                            ttype: TokenType::Integer,
                                            value: Some(
                                                "2".to_string(),
                                            ),
                                        },
                                    ),
                                ],
                                end_token: Token {
                                    line_no: 17,
                                    col_no: 43,
                                    ttype: TokenType::ParenthesisRight,
                                    value: None,
                                },
                            },
                            Expression::StringNode(
                                Token {
                                    line_no: 17,
                                    col_no: 44,
                                    ttype: TokenType::String,
                                    value: Some(
                                        " - ".to_string(),
                                    ),
                                },
                            ),
                        ],
                    }),
                },
                Expression::Function {
                    start_token: Token {
                        line_no: 18,
                        col_no: 6,
                        ttype: TokenType::ID,
                        value: Some(
                            "if".to_string(),
                        ),
                    },
                    arguments: vec![
                        Expression::Tag {
                            start_token: Token {
                                line_no: 18,
                                col_no: 9,
                                ttype: TokenType::AngleBracketLeft,
                                value: None,
                            },
                            token: Token {
                                line_no: 18,
                                col_no: 10,
                                ttype: TokenType::ID,
                                value: Some(
                                    "albumartist".to_string(),
                                ),
                            },
                        },
                        Expression::Group {
                            expressions: vec![
                                Expression::Tag {
                                    start_token: Token {
                                        line_no: 18,
                                        col_no: 25,
                                        ttype: TokenType::AngleBracketLeft,
                                        value: None,
                                    },
                                    token: Token {
                                        line_no: 18,
                                        col_no: 26,
                                        ttype: TokenType::ID,
                                        value: Some(
                                            "artist".to_string(),
                                        ),
                                    },
                                },
                                Expression::StringNode(
                                    Token {
                                        line_no: 18,
                                        col_no: 33,
                                        ttype: TokenType::String,
                                        value: Some(
                                            " - ".to_string(),
                                        ),
                                    },
                                ),
                            ],
                        },
                        Expression::StringNode(
                            Token {
                                line_no: 18,
                                col_no: 41,
                                ttype: TokenType::String,
                                value: Some(
                                    "".to_string(),
                                ),
                            },
                        ),
                    ],
                    end_token: Token {
                        line_no: 18,
                        col_no: 43,
                        ttype: TokenType::ParenthesisRight,
                        value: None,
                    },
                },
                Expression::Tag {
                    start_token: Token {
                        line_no: 19,
                        col_no: 5,
                        ttype: TokenType::AngleBracketLeft,
                        value: None,
                    },
                    token: Token {
                        line_no: 19,
                        col_no: 6,
                        ttype: TokenType::ID,
                        value: Some(
                            "title".to_string(),
                        ),
                    },
                },
            ],
        },
    };
    file_test("typical_input.tfmt", Some(reference))
}
