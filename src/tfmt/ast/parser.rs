use super::node::{self, Expression};
use crate::tfmt::error::ParserError;
use crate::tfmt::lexer::{Lexer, LexerResult};
use crate::tfmt::token::{Token, TokenType};
use log::trace;

type Result<T> = std::result::Result<T, ParserError>;

/// Reads a stream of [Token]s and build an Abstract Syntax Tree.
pub struct Parser<I>
where
    I: Iterator<Item = LexerResult>,
{
    iterator: I,
    depth: u64,
    // Put tokens in Rc instead of cloning
    current_token: Option<Token>,
    previous_token: Option<Token>,
}

impl<'a> Parser<Lexer<'a>> {
    /// Create a [Parser<Lexer<'a>>] from a string.
    pub fn from_string<S: AsRef<str>>(input_text: &'a S) -> Result<Self> {
        Ok(Self::new(Lexer::new(input_text)?))
    }
}

impl<I> Parser<I>
where
    I: Iterator<Item = LexerResult>,
{
    /// Create a [Parser] from an [Iterator] returning [LexerResult].
    pub fn new(iterator: I) -> Self {
        Self {
            iterator,
            depth: 0,
            current_token: None,
            previous_token: None,
        }
    }

    /// Run [Parser] to create an Abstract Syntax Tree.
    pub fn parse(&mut self) -> Result<node::Program> {
        // Prime parser
        self._advance(true)?;
        self.program()
    }

    // TODO? Why do we need previous_token? Just for error handling?
    fn _advance(&mut self, ignore: bool) -> Result<()> {
        // Allows replacing without deinit, even without Clone/Copy
        let new = match self.iterator.next() {
            Some(result) => result?,
            None => {
                if let Some(token) = self.current_token.take() {
                    self.previous_token = Some(token);
                    return Ok(());
                } else {
                    return Err(ParserError::ExhaustedTokens);
                }
            }
        };

        let prev = self.current_token.replace(new);

        if !ignore {
            self.previous_token = prev
        }

        if self.current_type().is_ignored() {
            self._advance(true)?;
        }

        Ok(())
    }

    fn current_type(&self) -> &TokenType {
        // current_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.current_token.is_some());
        &self.current_token.as_ref().unwrap().token_type
    }

    fn advance(&mut self) -> Result<()> {
        self._advance(false)
    }

    fn consume(&mut self, expected: &TokenType) -> Result<Token> {
        let token_type = self.current_type();

        if token_type != expected {
            return Err(ParserError::UnexpectedTokenType {
                expected: expected.clone(),
                found: token_type.clone(),
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

        if !matches!(token_type, TokenType::ID(..)) {
            return Err(ParserError::UnexpectedTokenType {
                expected: TokenType::ID(String::new()),
                found: token_type.clone(),
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

        if !matches!(token_type, TokenType::String(..)) {
            return Err(ParserError::UnexpectedTokenType {
                expected: TokenType::String(String::new()),
                found: token_type.clone(),
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

        if !matches!(token_type, TokenType::Integer(..)) {
            return Err(ParserError::UnexpectedTokenType {
                expected: TokenType::Integer(0),
                found: token_type.clone(),
            });
        }

        self.advance()?;

        // previous_token is guaranteed to be Some() by Parser::advance(), so
        // unwrap should be safe.
        debug_assert!(self.previous_token.is_some());

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.as_ref().unwrap().clone())
    }

    /// Depth Prefix
    fn dp(&mut self) -> String {
        (0..self.depth).map(|i| (i % 10).to_string()).collect()
    }

    // Grammar functions
    fn program(&mut self) -> Result<node::Program> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        self.depth += 1;

        let name = self.consume_id()?;

        trace!(
            r#"{} Program: "{}""#,
            self.dp(),
            name.get_string_unchecked()
        );

        self.consume(&TokenType::ParenthesisLeft)?;

        let parameters = self.parameters()?;

        self.consume(&TokenType::ParenthesisRight)?;

        // Optional, so ok to return Err.
        let description = self.consume_string().ok();

        self.consume(&TokenType::CurlyBraceLeft)?;
        let block = self.block()?;
        self.consume(&TokenType::CurlyBraceRight)?;

        self.depth -= 1;

        Ok(node::Program {
            name,
            parameters,
            description,
            block,
        })
    }

    fn parameters(&mut self) -> Result<node::Parameters> {
        // ( Parameter ( "," Parameter )* )?
        self.depth += 1;

        trace!("{} Parameters", self.dp());

        let mut parameters = Vec::new();

        loop {
            match self.parameter() {
                Ok(parameter) => parameters.push(parameter),
                Err(_) => break,
            }

            if self.consume(&TokenType::Comma).is_err() {
                break;
            }
        }
        self.depth -= 1;
        Ok(node::Parameters { parameters })
    }

    fn parameter(&mut self) -> Result<node::Parameter> {
        // ID ( "=" ( Integer | String ) )?
        self.depth += 1;
        let identifier = self.consume_id()?;
        trace!(
            r#"{} Parameter: "{}""#,
            self.dp(),
            identifier.get_string_unchecked()
        );

        let default = match self.consume(&TokenType::Equals) {
            Ok(_) => {
                if let Ok(token) = self.consume_int() {
                    Some(token)
                } else if let Ok(token) = self.consume_string() {
                    Some(token)
                } else {
                    // TODO? Create a separate error?
                    return Err(ParserError::Generic(
                        "Parameter has invalid default!".to_string(),
                    ));
                }
            }
            Err(_) => None,
        };

        self.depth -= 1;

        Ok(node::Parameter {
            token: identifier,
            default,
        })
    }

    fn block(&mut self) -> Result<node::Block> {
        // ( DriveLetter )? Expression*
        self.depth += 1;
        trace!("{} Block", self.dp());

        let expressions: Vec<Expression> =
            self.expressions(&[TokenType::CurlyBraceRight])?;

        self.depth -= 1;
        Ok(node::Block { expressions })
    }

    fn expressions(
        &mut self,
        terminators: &[TokenType],
    ) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = Vec::new();

        while !terminators.contains(self.current_type()) {
            trace!(
                "{} Gathering expressions until {:?}",
                self.dp(),
                terminators
                    .iter()
                    .map(|tt| format!("{:?}", tt))
                    .collect::<Vec<String>>()
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

        while *self.current_type() == TokenType::QuestionMark {
            self.consume(&TokenType::QuestionMark)?;
            let true_expr = self.ternary()?;
            self.consume(&TokenType::Colon)?;
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
            let operator = match *self.current_type() {
                TokenType::DoubleVerticalBar => {
                    self.consume(&TokenType::DoubleVerticalBar)?
                }
                TokenType::VerticalBar => {
                    self.consume(&TokenType::VerticalBar)?
                }
                _ => break,
            };

            ternary = Expression::BinaryOp {
                left: Box::new(ternary),
                operator,
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
            let operator = match *self.current_type() {
                TokenType::DoubleAmpersand => {
                    self.consume(&TokenType::DoubleAmpersand)?
                }
                TokenType::Ampersand => self.consume(&TokenType::Ampersand)?,
                _ => break,
            };

            disjunct = Expression::BinaryOp {
                left: Box::new(disjunct),
                operator,
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
            let operator = match *self.current_type() {
                TokenType::Plus => self.consume(&TokenType::Plus)?,
                TokenType::Hyphen => self.consume(&TokenType::Hyphen)?,
                _ => break,
            };

            conjunct = Expression::BinaryOp {
                left: Box::new(conjunct),
                operator,
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
            let operator = match *self.current_type() {
                TokenType::Asterisk => self.consume(&TokenType::Asterisk)?,
                TokenType::SlashForward => {
                    self.consume(&TokenType::SlashForward)?
                }
                TokenType::Percent => self.consume(&TokenType::Percent)?,
                _ => break,
            };

            term = Expression::BinaryOp {
                left: Box::new(term),
                operator,
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
            let operator = match *self.current_type() {
                TokenType::DoubleAsterisk => {
                    self.consume(&TokenType::DoubleAsterisk)?
                }
                TokenType::Caret => self.consume(&TokenType::Caret)?,
                _ => break,
            };

            factor = Expression::BinaryOp {
                left: Box::new(factor),
                operator,
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

        let exponent = match *self.current_type() {
            TokenType::Plus => Expression::UnaryOp {
                operator: self.consume(&TokenType::Plus)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::Hyphen => Expression::UnaryOp {
                operator: self.consume(&TokenType::Hyphen)?,
                operand: Box::new(self.exponent()?),
            },
            TokenType::ParenthesisLeft => {
                self.consume(&TokenType::ParenthesisLeft)?;

                let expressions: Vec<Expression> =
                    self.expressions(&[TokenType::ParenthesisRight])?;

                self.consume(&TokenType::ParenthesisRight)?;

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

        let ttype = self.current_type();

        let statement = match ttype {
            TokenType::Dollar => {
                self.consume(&TokenType::Dollar)?;

                if self.current_type() == &TokenType::ParenthesisLeft {
                    self.consume(&TokenType::ParenthesisLeft)?;
                    let substitution = Expression::Symbol(self.consume_id()?);
                    self.consume(&TokenType::ParenthesisRight)?;
                    substitution
                } else {
                    self.function()?
                }
            }
            TokenType::AngleBracketLeft => self.tag()?,
            TokenType::Integer(..) => {
                Expression::IntegerNode(self.consume_int()?)
            }
            TokenType::String(..) => {
                Expression::StringNode(self.consume_string()?)
            }
            _ => return Err(ParserError::UnrecognizedToken(ttype.clone())),
        };

        self.depth -= 1;
        Ok(statement)
    }

    fn function(&mut self) -> Result<Expression> {
        self.depth += 1;
        trace!("{} Function", self.dp());

        let identifier = self.consume_id()?;

        self.consume(&TokenType::ParenthesisLeft)?;

        let mut arguments: Vec<Expression> = Vec::new();

        // while self.current_token.ttype() != TokenType::ParenthesisRight {
        loop {
            arguments.push(self.expression()?);
            if self.consume(&TokenType::Comma).is_err() {
                break;
            }
        }

        let function = Expression::Function {
            start_token: identifier,
            arguments,
            end_token: self.consume(&TokenType::ParenthesisRight)?,
        };

        self.depth -= 1;
        Ok(function)
    }

    fn tag(&mut self) -> Result<Expression> {
        self.depth += 1;
        trace!("{} Tag", self.dp());

        let start_token = self.consume(&TokenType::AngleBracketLeft)?;

        let identifier = self.consume_id()?;

        self.consume(&TokenType::AngleBracketRight)?;

        let tag = Expression::Tag {
            start_token,
            token: identifier,
        };

        self.depth -= 1;
        Ok(tag)
    }
}
