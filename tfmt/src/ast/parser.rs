use crate::ast::node::{self, Expression};
use crate::error::{ErrorContext, ParserError};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use log::trace;

type Result<T> = std::result::Result<T, ParserError>;

const MAX_PARSING_DEPTH: u64 = 64;

/// Reads a stream of [Token]s and build an Abstract Syntax Tree.
pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    depth: u64,
    // Put tokens in Rc instead of cloning
    current_token: Option<Token>,
    previous_token: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Create a [Parser<'a>] from a string.
    pub(crate) fn new(input_text: &'a str) -> Result<Self> {
        Ok(Parser {
            lexer: Lexer::new(input_text)?,
            depth: 0,
            current_token: None,
            previous_token: None,
        })
    }

    /// Run [Parser] to create an Abstract Syntax Tree.
    pub(crate) fn parse(&mut self) -> Result<node::Program> {
        // Prime parser
        self._advance(true)?;
        self.program()
    }

    fn current_context(&self) -> ErrorContext {
        // current_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.current_token.is_some());
        ErrorContext::from_token(
            self.lexer.input_text(),
            self.current_token.as_ref().unwrap(),
        )
    }

    fn current_type(&self) -> TokenType {
        // current_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.current_token.is_some());
        self.current_token.as_ref().unwrap().token_type()
    }

    fn _advance(&mut self, ignore: bool) -> Result<()> {
        // Allows replacing without deinit, even without Clone/Copy
        let new = if let Some(result) = self.lexer.next() {
            result?
        } else {
            if let Some(token) = self.current_token.take() {
                self.previous_token = Some(token);
                return Ok(());
            }

            return Err(ParserError::ExhaustedTokens);
        };

        let prev = self.current_token.replace(new);

        if !ignore {
            self.previous_token = prev;
        }

        if self.current_type().is_ignored() {
            self._advance(true)?;
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<()> {
        self._advance(false)
    }

    fn consume(&mut self, expected: TokenType) -> Result<Token> {
        let token_type = self.current_type();

        if token_type != expected {
            return Err(ParserError::UnexpectedTokenType {
                context: self.current_context(),
                expected,
                found: token_type,
            });
        }

        self.advance()?;

        // previous_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.previous_token.is_some());

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.as_ref().unwrap().clone())
    }

    fn consume_id(&mut self) -> Result<Token> {
        let token_type = self.current_type();

        if !matches!(token_type, TokenType::ID) {
            return Err(ParserError::UnexpectedTokenType {
                context: self.current_context(),
                expected: TokenType::ID,
                found: token_type,
            });
        }

        self.advance()?;

        // previous_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.previous_token.is_some());

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.as_ref().unwrap().clone())
    }

    fn consume_string(&mut self) -> Result<Token> {
        let token_type = self.current_type();

        if !matches!(token_type, TokenType::String) {
            return Err(ParserError::UnexpectedTokenType {
                context: self.current_context(),
                expected: TokenType::String,
                found: token_type,
            });
        }

        self.advance()?;

        // previous_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.previous_token.is_some());

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.as_ref().unwrap().clone())
    }

    fn consume_int(&mut self) -> Result<Token> {
        let token_type = self.current_type();

        if !matches!(token_type, TokenType::Integer) {
            return Err(ParserError::UnexpectedTokenType {
                context: self.current_context(),
                expected: TokenType::Integer,
                found: token_type,
            });
        }

        self.advance()?;

        // previous_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.previous_token.is_some());

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.as_ref().unwrap().clone())
    }

    fn inc_depth(&mut self) -> Result<()> {
        self.depth += 1;

        if self.depth > MAX_PARSING_DEPTH {
            return Err(ParserError::MaxDepth(MAX_PARSING_DEPTH));
        }

        Ok(())
    }

    fn dec_depth(&mut self) {
        self.depth -= 1;
    }

    /// Depth Prefix
    fn dp(&mut self) -> String {
        (0..self.depth).map(|i| (i % 10).to_string()).collect()
    }

    // Grammar functions
    fn program(&mut self) -> Result<node::Program> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        self.inc_depth()?;

        let name = self.consume_id()?;

        trace!(
            r#"{} Program: "{}""#,
            self.dp(),
            name.literal().expect("Unchecked literal!")
        );

        self.consume(TokenType::ParenthesisLeft)?;

        let parameters = self.parameters()?;

        self.consume(TokenType::ParenthesisRight)?;

        // Optional, so ok to return Err.
        let description = self.consume_string().ok();

        self.consume(TokenType::CurlyBraceLeft)?;
        let block = self.block()?;
        self.consume(TokenType::CurlyBraceRight)?;

        self.dec_depth();

        Ok(node::Program::new(name, parameters, description, block))
    }

    fn parameters(&mut self) -> Result<node::Parameters> {
        // ( Parameter ( "," Parameter )* )?
        self.inc_depth()?;

        trace!("{} Parameters", self.dp());

        let mut parameters = Vec::new();

        loop {
            match self.parameter() {
                Ok(parameter) => parameters.push(parameter),
                Err(_) => break,
            }

            if self.consume(TokenType::Comma).is_err() {
                break;
            }
        }
        self.dec_depth();
        Ok(node::Parameters::new(parameters))
    }

    fn parameter(&mut self) -> Result<node::Parameter> {
        // ID ( "=" ( Integer | String ) )?
        self.inc_depth()?;
        let name = self.consume_id()?;
        trace!(
            r#"{} Parameter: "{}""#,
            self.dp(),
            name.literal().expect("Unchecked literal!")
        );

        let default = match self.consume(TokenType::Equals) {
            Ok(_) => {
                if let Ok(token) = self.consume_int() {
                    Some(token)
                } else if let Ok(token) = self.consume_string() {
                    Some(token)
                } else {
                    return Err(ParserError::InvalidDefault(
                        self.current_context(),
                        self.current_type(),
                    ));
                }
            }
            Err(_) => None,
        };

        self.dec_depth();

        Ok(node::Parameter::new(name, default))
    }

    fn block(&mut self) -> Result<node::Block> {
        // ( DriveLetter )? Expression*
        self.inc_depth()?;
        trace!("{} Block", self.dp());

        let expressions: Vec<Expression> =
            self.expressions(&[TokenType::CurlyBraceRight])?;

        self.dec_depth();
        Ok(node::Block::new(expressions))
    }

    fn expressions(
        &mut self,
        terminators: &[TokenType],
    ) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = Vec::new();

        while !terminators.contains(&self.current_type()) {
            trace!(
                "{} Gathering expressions until {:?}",
                self.dp(),
                terminators
                    .iter()
                    .map(|tt| format!("{:?}", tt))
                    .collect::<Vec<String>>()
            );

            expressions.push(self.expression()?);
        }

        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Expression> {
        // Ternary ( "?" Ternary ":" Ternary )*
        self.inc_depth()?;
        trace!("{} Expression", self.dp());
        let mut expression = self.ternary()?;

        while self.current_type() == TokenType::QuestionMark {
            self.consume(TokenType::QuestionMark)?;
            let true_expr = self.ternary()?;
            self.consume(TokenType::Colon)?;
            let false_expr = self.ternary()?;

            expression = Expression::TernaryOp {
                condition: Box::new(expression),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }

        self.dec_depth();
        Ok(expression)
    }

    fn ternary(&mut self) -> Result<Expression> {
        // Disjunct ( ( "||" | "|" ) Disjunct )*
        self.inc_depth()?;
        trace!("{} Ternary", self.dp());

        let mut ternary = self.disjunct()?;

        loop {
            let operator = match self.current_type() {
                TokenType::DoubleVerticalBar => {
                    self.consume(TokenType::DoubleVerticalBar)?
                }
                TokenType::VerticalBar => {
                    self.consume(TokenType::VerticalBar)?
                }
                _ => break,
            };

            ternary = Expression::BinaryOp {
                left: Box::new(ternary),
                operator,
                right: Box::new(self.ternary()?),
            };
        }

        self.dec_depth();
        Ok(ternary)
    }

    fn disjunct(&mut self) -> Result<Expression> {
        // Conjunct ( ( "&&" | "&" ) Conjunct )*
        self.inc_depth()?;
        trace!("{} Disjunct", self.dp());

        let mut disjunct = self.conjunct()?;

        loop {
            let operator = match self.current_type() {
                TokenType::DoubleAmpersand => {
                    self.consume(TokenType::DoubleAmpersand)?
                }
                TokenType::Ampersand => self.consume(TokenType::Ampersand)?,
                _ => break,
            };

            disjunct = Expression::BinaryOp {
                left: Box::new(disjunct),
                operator,
                right: Box::new(self.disjunct()?),
            };
        }

        self.dec_depth();
        Ok(disjunct)
    }

    fn conjunct(&mut self) -> Result<Expression> {
        // Term ( ( "+" | "-" ) Term )*
        self.inc_depth()?;
        trace!("{} Conjunct", self.dp());
        let mut conjunct = self.term()?;

        loop {
            let operator = match self.current_type() {
                TokenType::Plus => self.consume(TokenType::Plus)?,
                TokenType::Hyphen => self.consume(TokenType::Hyphen)?,
                _ => break,
            };

            conjunct = Expression::BinaryOp {
                left: Box::new(conjunct),
                operator,
                right: Box::new(self.conjunct()?),
            };
        }

        self.dec_depth();
        Ok(conjunct)
    }

    fn term(&mut self) -> Result<Expression> {
        // Factor ( ( "*" | "/" | "%" ) Factor )*
        self.inc_depth()?;
        trace!("{} Term", self.dp());

        let mut term = self.factor()?;

        loop {
            let operator = match self.current_type() {
                TokenType::Asterisk => self.consume(TokenType::Asterisk)?,
                TokenType::SlashForward => {
                    self.consume(TokenType::SlashForward)?
                }
                TokenType::Percent => self.consume(TokenType::Percent)?,
                _ => break,
            };

            term = Expression::BinaryOp {
                left: Box::new(term),
                operator,
                right: Box::new(self.term()?),
            };
        }

        self.dec_depth();
        Ok(term)
    }

    fn factor(&mut self) -> Result<Expression> {
        // Exponent ( ( "**" | "^" ) Exponent )*
        self.inc_depth()?;
        trace!("{} Factor", self.dp());

        let mut factor = self.exponent()?;

        loop {
            let operator = match self.current_type() {
                TokenType::DoubleAsterisk => {
                    self.consume(TokenType::DoubleAsterisk)?
                }
                TokenType::Caret => self.consume(TokenType::Caret)?,
                _ => break,
            };

            factor = Expression::BinaryOp {
                left: Box::new(factor),
                operator,
                right: Box::new(self.factor()?),
            };
        }

        self.dec_depth();
        Ok(factor)
    }

    fn exponent(&mut self) -> Result<Expression> {
        // "+" Exponent | "-" Exponent | "(" Expression+ ")" | Statement
        self.inc_depth()?;
        trace!("{} Exponent", self.dp());

        let exponent = match self.current_type() {
            TokenType::Plus => Expression::UnaryOp {
                operator: self.consume(TokenType::Plus)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::Hyphen => Expression::UnaryOp {
                operator: self.consume(TokenType::Hyphen)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::ParenthesisLeft => {
                self.consume(TokenType::ParenthesisLeft)?;

                let expressions: Vec<Expression> =
                    self.expressions(&[TokenType::ParenthesisRight])?;

                self.consume(TokenType::ParenthesisRight)?;

                if expressions.is_empty() {
                    return Err(ParserError::EmptyGroup(
                        self.current_context(),
                    ));
                }

                Expression::Group { expressions }
            }
            _ => self.statement()?,
        };

        self.dec_depth();
        Ok(exponent)
    }

    fn statement(&mut self) -> Result<Expression> {
        // Comment | Function | Integer | String | Substitution | Tag
        self.inc_depth()?;
        trace!("{} Statement", self.dp());

        let ttype = self.current_type();

        let statement = match ttype {
            TokenType::Dollar => {
                self.consume(TokenType::Dollar)?;

                if self.current_type() == TokenType::ParenthesisLeft {
                    self.consume(TokenType::ParenthesisLeft)?;
                    let substitution = Expression::Symbol(self.consume_id()?);
                    self.consume(TokenType::ParenthesisRight)?;
                    substitution
                } else {
                    self.function()?
                }
            }
            TokenType::AngleBracketLeft => self.tag()?,
            TokenType::Integer => Expression::IntegerNode(self.consume_int()?),
            TokenType::String => Expression::StringNode(self.consume_string()?),
            _ => {
                return Err(ParserError::UnrecognizedToken(
                    self.current_context(),
                    ttype,
                ))
            }
        };

        self.dec_depth();
        Ok(statement)
    }

    fn function(&mut self) -> Result<Expression> {
        self.inc_depth()?;
        trace!("{} Function", self.dp());

        let identifier = self.consume_id()?;

        self.consume(TokenType::ParenthesisLeft)?;

        let mut arguments: Vec<Expression> = Vec::new();

        // while self.current_token.ttype() != TokenType::ParenthesisRight {
        loop {
            arguments.push(self.expression()?);
            if self.consume(TokenType::Comma).is_err() {
                break;
            }
        }

        let function = Expression::Function {
            start_token: identifier,
            arguments,
            end_token: self.consume(TokenType::ParenthesisRight)?,
        };

        self.dec_depth();
        Ok(function)
    }

    fn tag(&mut self) -> Result<Expression> {
        self.inc_depth()?;
        trace!("{} Tag", self.dp());

        let start_token = self.consume(TokenType::AngleBracketLeft)?;

        let identifier = self.consume_id()?;

        self.consume(TokenType::AngleBracketRight)?;

        let tag = Expression::Tag {
            start_token,
            token: identifier,
        };

        self.dec_depth();
        Ok(tag)
    }
}

#[cfg(test)]
mod test {
    use crate::ast::node::{self, Expression};
    use crate::ast::Parser;
    use crate::token::{Token, TokenType};
    use anyhow::Result;

    #[cfg(unix)]
    const SIMPLE_INPUT: &str = include_str!("../../testdata/simple_input.tfmt");

    #[cfg(windows)]
    const SIMPLE_INPUT: &str =
        include_str!("..\\..\\testdata\\simple_input.tfmt");

    #[cfg(unix)]
    const TYPICAL_INPUT: &str =
        include_str!("../../testdata/typical_input.tfmt");

    #[cfg(windows)]
    const TYPICAL_INPUT: &str =
        include_str!("..\\..\\testdata\\typical_input.tfmt");

    fn file_test(input: &str, reference: Option<node::Program>) -> Result<()> {
        let normalized_input = crate::normalize_eol(input);
        let mut parser = Parser::new(&normalized_input)?;

        let program = parser.parse()?;

        if let Some(reference) = reference {
            assert_eq!(program, reference);
        }

        Ok(())
    }

    #[test]
    fn parser_simple_input_test() -> Result<()> {
        let reference = node::Program::new(
            Token::with_literal(
                TokenType::ID,
                1,
                1,
                "simple_input".to_string(),
            ),
            node::Parameters::new(Vec::new()),
            None,
            node::Block::new(vec![
                Expression::Tag {
                    start_token: Token::new(TokenType::AngleBracketLeft, 2, 5),
                    token: Token::with_literal(
                        TokenType::ID,
                        2,
                        6,
                        "artist".to_string(),
                    ),
                },
                Expression::StringNode(Token::with_literal(
                    TokenType::String,
                    2,
                    14,
                    "/".to_string(),
                )),
                Expression::Tag {
                    start_token: Token::new(TokenType::AngleBracketLeft, 2, 18),
                    token: Token::with_literal(
                        TokenType::ID,
                        2,
                        19,
                        "title".to_string(),
                    ),
                },
            ]),
        );
        file_test(SIMPLE_INPUT, Some(reference))
    }

    // Too many lines is unavoidable here, given the size of the reference.
    #[test]
    #[allow(clippy::too_many_lines)]
    fn parser_typical_input_test() -> Result<()> {
        let reference = node::Program::new(
            Token::with_literal( TokenType::ID, 1, 1, "typical_input".to_string()),
            node::Parameters::new(
                vec![
                    node::Parameter::new(
                        Token::with_literal( TokenType::ID, 1, 15, "folder".to_string()),
                        Some(
                            Token::with_literal( TokenType::String, 1, 22, "destination".to_string()),
                        ),
                    ), //
                ],
                ),
            Some(
                Token::with_literal( TokenType::String, 1, 37, "This file is used to test tfmttools.".to_string()),
            ),
            node::Block::new(
                vec![
                    Expression::Symbol(
                        Token::with_literal( TokenType::ID, 3, 7, "folder".to_string()),
                    ),
                    Expression::StringNode(
                        Token::with_literal( TokenType::String, 3, 15, "/".to_string()),
                    ),
                    Expression::BinaryOp {
                        left: Box::new(Expression::Tag {
                            start_token: Token::new( TokenType::AngleBracketLeft, 4, 5),
                            token: Token::with_literal( TokenType::ID, 4, 6, "albumartist".to_string()),
                        }),
                        operator: Token::new( TokenType::VerticalBar, 4, 19),
                        right: Box::new(Expression::Tag {
                            start_token: Token::new( TokenType::AngleBracketLeft, 4, 21),
                            token: Token::with_literal( TokenType::ID, 4, 22, "artist".to_string()),
                        }),
                    },
                    Expression::StringNode(
                        Token::with_literal( TokenType::String, 5, 5, "/".to_string()),
                    ),
                    Expression::BinaryOp {
                        left: Box::new(Expression::Group {
                            expressions: vec![
                                Expression::BinaryOp {
                                    left: Box::new(Expression::Tag {
                                        start_token: Token::new( TokenType::AngleBracketLeft, 8, 9),
                                        token: Token::with_literal( TokenType::ID, 8, 10, "date".to_string()),
                                    }),
                                    operator: Token::new( TokenType::Ampersand, 8, 16),
                                    right: Box::new(Expression::Group {
                                        expressions: vec![
                                            Expression::Function {
                                                start_token: Token::with_literal( TokenType::ID, 9, 14, "year_from_date".to_string()),
                                                arguments: vec![
                                                    Expression::Tag {
                                                        start_token: Token::new( TokenType::AngleBracketLeft, 9, 29),
                                                        token: Token::with_literal( TokenType::ID, 9, 30, "date".to_string()),
                                                    },
                                                ],
                                                end_token: Token::new( TokenType::ParenthesisRight, 9, 35),
                                            },
                                            Expression::BinaryOp {
                                                left: Box::new(Expression::Tag {
                                                    start_token: Token::new( TokenType::AngleBracketLeft, 10, 13),
                                                    token: Token::with_literal( TokenType::ID, 10, 14, "albumsort".to_string()),
                                                }),
                                                operator: Token::new( TokenType::Ampersand, 10, 25),
                                                right: Box::new(Expression::Group {
                                                    expressions: vec![
                                                        Expression::StringNode(
                                                            Token::with_literal( TokenType::String, 10, 28, ".".to_string()),
                                                        ),
                                                        Expression::Function {
                                                            start_token: Token::with_literal( TokenType::ID, 10, 33, "num".to_string()),
                                                            arguments: vec![
                                                                Expression::Tag {
                                                                    start_token: Token::new( TokenType::AngleBracketLeft, 10, 37),
                                                                    token: Token::with_literal( TokenType::ID, 10, 38, "albumsort".to_string()),
                                                                },
                                                                Expression::IntegerNode(
                                                                    Token::with_literal( TokenType::Integer, 10, 50, "2".to_string()),
                                                                ),
                                                            ],
                                                            end_token: Token::new( TokenType::ParenthesisRight, 10, 51),
                                                        },
                                                    ],
                                                }),
                                            },
                                            Expression::StringNode(
                                                Token::with_literal( TokenType::String, 11, 13, " - ".to_string()),
                                            ),
                                        ],
                                    }),
                                },
                                Expression::Tag {
                                    start_token: Token::new( TokenType::AngleBracketLeft, 13, 9),
                                    token: Token::with_literal( TokenType::ID, 13, 10, "album".to_string()),
                                },
                            ],
                        }),
                        operator: Token::new( TokenType::DoubleAmpersand, 14, 7),
                        right: Box::new(Expression::StringNode(
                            Token::with_literal( TokenType::String, 14, 10, "/".to_string()),
                        )),
                    },
                    Expression::TernaryOp {
                        condition: Box::new(Expression::Tag {
                            start_token: Token::new( TokenType::AngleBracketLeft, 16, 5),
                            token: Token::with_literal( TokenType::ID, 16, 6, "discnumber".to_string()),
                        }),
                        true_expr: Box::new(Expression::Function {
                            start_token: Token::with_literal( TokenType::ID, 16, 21, "num".to_string()),
                            arguments: vec![
                                Expression::Tag {
                                    start_token: Token::new( TokenType::AngleBracketLeft, 16, 25),
                                    token: Token::with_literal( TokenType::ID, 16, 26, "discnumber".to_string()),
                                },
                                Expression::IntegerNode(
                                    Token::with_literal( TokenType::Integer, 16, 39, "1".to_string()),
                                ),
                            ],
                            end_token: Token::new( TokenType::ParenthesisRight, 16, 40),
                        }),
                        false_expr: Box::new(Expression::StringNode(
                            Token::with_literal( TokenType::String, 16, 44, "".to_string()),
                        )),
                    },
                    Expression::BinaryOp {
                        left: Box::new(Expression::Tag {
                            start_token: Token::new( TokenType::AngleBracketLeft, 17, 5),
                            token: Token::with_literal( TokenType::ID, 17, 6, "tracknumber".to_string()),
                        }),
                        operator: Token::new( TokenType::Ampersand, 17, 19),
                        right: Box::new(Expression::Group {
                            expressions: vec![
                                Expression::Function {
                                    start_token: Token::with_literal( TokenType::ID, 17, 23, "num".to_string()),
                                    arguments: vec![
                                        Expression::Tag {
                                            start_token: Token::new( TokenType::AngleBracketLeft, 17, 27),
                                            token: Token::with_literal( TokenType::ID, 17, 28, "tracknumber".to_string()),
                                        },
                                        Expression::IntegerNode(
                                            Token::with_literal( TokenType::Integer, 17, 42, "2".to_string()),
                                        ),
                                    ],
                                    end_token: Token::new( TokenType::ParenthesisRight, 17, 43),
                                },
                                Expression::StringNode(
                                    Token::with_literal( TokenType::String, 17, 44, " - ".to_string()),
                                ),
                            ],
                        }),
                    },
                    Expression::Function {
                        start_token: Token::with_literal( TokenType::ID, 18, 6, "if".to_string()),
                        arguments: vec![
                            Expression::Tag {
                                start_token: Token::new( TokenType::AngleBracketLeft, 18, 9),
                                token: Token::with_literal( TokenType::ID, 18, 10, "albumartist".to_string()),
                            },
                            Expression::Group {
                                expressions: vec![
                                    Expression::Tag {
                                        start_token: Token::new( TokenType::AngleBracketLeft, 18, 25),
                                        token: Token::with_literal( TokenType::ID, 18, 26, "artist".to_string()),
                                    },
                                    Expression::StringNode(
                                        Token::with_literal( TokenType::String, 18, 33, " - ".to_string()),
                                    ),
                                ],
                            },
                            Expression::StringNode(
                                Token::with_literal( TokenType::String, 18, 41, "".to_string()),
                            ),
                        ],
                        end_token: Token::new( TokenType::ParenthesisRight, 18, 43),
                    },
                    Expression::Tag {
                        start_token: Token::new( TokenType::AngleBracketLeft, 19, 5),
                        token: Token::with_literal( TokenType::ID, 19, 6, "title".to_string()),
                    },
                ],
                ),
        );
        file_test(TYPICAL_INPUT, Some(reference))
    }
}
