use crate::error::ParserError;
use crate::tfmt::ast::{self, Expression};
use crate::tfmt::{Lexer, LexerResult, Token, TokenType};
use log::trace;
use std::path::Path;
use std::str::FromStr;

type Result<T> = std::result::Result<T, ParserError>;

/// Reads a stream of [Token]s and build an Abstract Syntax Tree.
pub struct Parser<I>
where
    I: Iterator<Item = LexerResult>,
{
    iterator: I,
    depth: u64,
    current_token: Token,
    previous_token: Token,
}

impl FromStr for Parser<Lexer> {
    type Err = ParserError;

    fn from_str(string: &str) -> Result<Parser<Lexer>> {
        Ok(Parser::from_iterator(Lexer::from_str(string)?))
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = LexerResult>,
{
    /// Create [Parser] from Iterator<Item = [LexerResult]>.
    pub fn from_iterator(iterator: I) -> Parser<I> {
        Parser {
            iterator,
            depth: 0,
            // TokenType::Uninitialized doesn't take a value, so should never fail.
            current_token: Token::new(0, 0, TokenType::Uninitialized, None)
                .unwrap(),
            previous_token: Token::new(0, 0, TokenType::Uninitialized, None)
                .unwrap(),
        }
    }

    /// Create [Parser] from path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Parser<Lexer>> {
        Ok(Parser::from_iterator(Lexer::from_path(path)?))
    }

    /// Wrapper function for starting [Parser].
    pub fn parse(&mut self) -> Result<ast::Program> {
        // Prime parser
        self._advance(true)?;
        self.program()
    }

    // TODO? Why do we need previous_token? Just for error handling?
    fn _advance(&mut self, ignore: bool) -> Result<()> {
        // Allows replacing without deinit, even without Clone/Copy
        let prev = std::mem::replace(
            &mut self.current_token,
            match self.iterator.next() {
                Some(a) => a?,
                None => return Err(ParserError::ExpectedEOF),
            },
        );

        if !ignore {
            self.previous_token = prev
        }

        if TokenType::IGNORED.contains(&self.current_token.ttype) {
            self._advance(true)?;
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<()> {
        self._advance(false)
    }

    fn consume(&mut self, expected_ttype: TokenType) -> Result<Token> {
        let current_ttype = self.current_token.ttype;

        if current_ttype == TokenType::EOF {
            return Err(ParserError::ExhaustedTokens(expected_ttype));
        }

        if current_ttype != expected_ttype {
            return Err(ParserError::UnexpectedToken(
                expected_ttype,
                current_ttype,
            ));
        }

        self.advance()?;

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.clone())
    }

    /// Depth Prefix
    fn dp(&mut self) -> String {
        (0..self.depth).map(|i| (i % 10).to_string()).collect()
    }

    // Grammar functions
    fn program(&mut self) -> Result<ast::Program> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        self.depth += 1;

        let name = self.consume(TokenType::ID)?;

        trace!(r#"{} Program: "{}""#, self.dp(), name.get_value_unchecked());

        self.consume(TokenType::ParenthesisLeft)?;

        let parameters = self.parameters()?;

        self.consume(TokenType::ParenthesisRight)?;

        // Optional, so ok to return Err.
        let description = self.consume(TokenType::String).ok();

        self.consume(TokenType::CurlyBraceLeft)?;
        let block = self.block()?;
        self.consume(TokenType::CurlyBraceRight)?;

        self.depth -= 1;

        Ok(ast::Program {
            name,
            parameters,
            description,
            block,
        })
    }

    fn parameters(&mut self) -> Result<ast::Parameters> {
        // ( Parameter ( "," Parameter )* )?
        self.depth += 1;

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
        self.depth -= 1;
        Ok(ast::Parameters { parameters })
    }

    fn parameter(&mut self) -> Result<ast::Parameter> {
        // ID ( "=" ( Integer | String ) )?
        self.depth += 1;
        let identifier = self.consume(TokenType::ID)?;
        trace!(
            r#"{} Parameter: "{}""#,
            self.dp(),
            identifier.get_value_unchecked()
        );

        let default = match self.consume(TokenType::Equals) {
            Ok(_) => {
                if let Ok(token) = self.consume(TokenType::Integer) {
                    Some(token)
                } else if let Ok(token) = self.consume(TokenType::String) {
                    Some(token)
                } else {
                    return Err(ParserError::Generic(
                        "Paramater has invalid default!".to_string(),
                    ));
                }
            }
            Err(_) => None,
        };

        self.depth -= 1;

        Ok(ast::Parameter {
            token: identifier,
            default,
        })
    }

    fn block(&mut self) -> Result<ast::Block> {
        // ( DriveLetter )? Expression*
        self.depth += 1;
        trace!("{} Block", self.dp());

        let expressions: Vec<Expression> =
            self.expressions(&[TokenType::CurlyBraceRight])?;

        self.depth -= 1;
        Ok(ast::Block { expressions })
    }

    fn expressions(
        &mut self,
        terminators: &[TokenType],
    ) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = Vec::new();

        while !terminators.contains(&self.current_token.ttype) {
            trace!(
                "{} Gathering expressions until {:?}",
                self.dp(),
                terminators
                    .iter()
                    .map(|tt| tt.as_str())
                    .collect::<Vec<&str>>()
            );

            if self.depth > 48 {
                return Err(ParserError::MaxIteration(48));
            }
            expressions.push(self.expression()?);
        }

        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Expression> {
        // Ternary ( "?" Ternary ":" Ternary )*
        self.depth += 1;
        trace!("{} Expression", self.dp());
        let mut expression = self.ternary()?;

        while self.current_token.ttype == TokenType::QuestionMark {
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

        self.depth -= 1;
        Ok(expression)
    }

    fn ternary(&mut self) -> Result<Expression> {
        // Disjunct ( ( "||" | "|" ) Disjunct )*
        self.depth += 1;
        trace!("{} Ternary", self.dp());

        let mut ternary = self.disjunct()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleVerticalBar => self.consume(ttype)?,
                TokenType::VerticalBar => self.consume(ttype)?,
                _ => break,
            };

            ternary = Expression::BinaryOp {
                left: Box::new(ternary),
                token: operator,
                right: Box::new(self.ternary()?),
            };
        }

        self.depth -= 1;
        Ok(ternary)
    }

    fn disjunct(&mut self) -> Result<Expression> {
        // Conjunct ( ( "&&" | "&" ) Conjunct )*
        self.depth += 1;
        trace!("{} Disjunct", self.dp());

        let mut disjunct = self.conjunct()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleAmpersand => self.consume(ttype)?,
                TokenType::Ampersand => self.consume(ttype)?,
                _ => break,
            };

            disjunct = Expression::BinaryOp {
                left: Box::new(disjunct),
                token: operator,
                right: Box::new(self.disjunct()?),
            };
        }

        self.depth -= 1;
        Ok(disjunct)
    }

    fn conjunct(&mut self) -> Result<Expression> {
        // Term ( ( "+" | "-" ) Term )*
        self.depth += 1;
        trace!("{} Conjunct", self.dp());
        let mut conjunct = self.term()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::Plus => self.consume(ttype)?,
                TokenType::Hyphen => self.consume(ttype)?,
                _ => break,
            };

            conjunct = Expression::BinaryOp {
                left: Box::new(conjunct),
                token: operator,
                right: Box::new(self.conjunct()?),
            };
        }

        self.depth -= 1;
        Ok(conjunct)
    }

    fn term(&mut self) -> Result<Expression> {
        // Factor ( ( "*" | "/" | "%" ) Factor )*
        self.depth += 1;
        trace!("{} Term", self.dp());

        let mut term = self.factor()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::Asterisk => self.consume(ttype)?,
                TokenType::SlashForward => self.consume(ttype)?,
                TokenType::Percent => self.consume(ttype)?,
                _ => break,
            };

            term = Expression::BinaryOp {
                left: Box::new(term),
                token: operator,
                right: Box::new(self.term()?),
            };
        }

        self.depth -= 1;
        Ok(term)
    }

    fn factor(&mut self) -> Result<Expression> {
        // Exponent ( ( "**" | "^" ) Exponent )*
        self.depth += 1;
        trace!("{} Factor", self.dp());

        let mut factor = self.exponent()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleAsterisk => self.consume(ttype)?,
                TokenType::Caret => self.consume(ttype)?,
                _ => break,
            };

            factor = Expression::BinaryOp {
                left: Box::new(factor),
                token: operator,
                right: Box::new(self.factor()?),
            };
        }

        self.depth -= 1;
        Ok(factor)
    }

    fn exponent(&mut self) -> Result<Expression> {
        // "+" Exponent | "-" Exponent | "(" Expression+ ")" | Statement
        self.depth += 1;
        trace!("{} Exponent", self.dp());

        let ttype = self.current_token.ttype;

        let exponent = match ttype {
            TokenType::Plus => Expression::UnaryOp {
                token: self.consume(ttype)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::Hyphen => Expression::UnaryOp {
                token: self.consume(ttype)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::ParenthesisLeft => {
                self.consume(ttype)?;

                let expressions: Vec<Expression> =
                    self.expressions(&[TokenType::ParenthesisRight])?;

                self.consume(TokenType::ParenthesisRight)?;

                if expressions.is_empty() {
                    return Err(ParserError::EmptyGroup);
                }

                Expression::Group { expressions }
            }
            _ => self.statement()?,
        };

        self.depth -= 1;
        Ok(exponent)
    }

    fn statement(&mut self) -> Result<Expression> {
        // Comment | Function | Integer | String | Substitution | Tag
        self.depth += 1;
        trace!("{} Statement", self.dp());

        let ttype = self.current_token.ttype;

        let statement = match ttype {
            TokenType::Dollar => {
                self.consume(ttype)?;

                if self.current_token.ttype == TokenType::ParenthesisLeft {
                    self.consume(TokenType::ParenthesisLeft)?;
                    let substitution =
                        Expression::Symbol(self.consume(TokenType::ID)?);
                    self.consume(TokenType::ParenthesisRight)?;
                    substitution
                } else {
                    self.function()?
                }
            }
            TokenType::AngleBracketLeft => self.tag()?,
            TokenType::Integer => Expression::IntegerNode(self.consume(ttype)?),
            TokenType::String => Expression::StringNode(self.consume(ttype)?),
            _ => return Err(ParserError::UnrecognizedToken(ttype)),
        };

        self.depth -= 1;
        Ok(statement)
    }

    fn function(&mut self) -> Result<Expression> {
        self.depth += 1;
        trace!("{} Function", self.dp());

        let identifier = self.consume(TokenType::ID)?;

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

        self.depth -= 1;
        Ok(function)
    }

    fn tag(&mut self) -> Result<Expression> {
        self.depth += 1;
        trace!("{} Tag", self.dp());

        let start_token = self.consume(TokenType::AngleBracketLeft)?;

        let identifier = self.consume(TokenType::ID)?;

        self.consume(TokenType::AngleBracketRight)?;

        let tag = Expression::Tag {
            start_token,
            token: identifier,
        };

        self.depth -= 1;
        Ok(tag)
    }
}
