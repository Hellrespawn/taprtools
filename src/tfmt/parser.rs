use log::trace;

use anyhow::Result;

use super::ast;
use super::lexer::Lexer;
use super::token::{self, Token, TokenType};
use crate::error::TFMTError;
// use std::error::Error;

pub struct Parser {
    lexer: Lexer,
    depth: u64,
    current_token: Token,
    previous_token: Token,
}

impl Parser {
    // Constructors
    pub fn from_lexer(lexer: Lexer) -> Parser {
        Parser {
            lexer,
            depth: 0,
            //Dummy tokens
            current_token: Token::new(0, 0, TokenType::Uninitialized, None),
            previous_token: Token::new(0, 0, TokenType::Uninitialized, None),
        }
    }

    pub fn from_string(string: &str) -> Parser {
        Parser::from_lexer(Lexer::new(string))
    }

    pub fn parse(&mut self) -> Result<ast::Program> {
        // Prime parser
        self._advance(true)?;
        self.program()
    }

    // FIXME Why do we need previous_token? Just for error handling?
    fn _advance(&mut self, ignore: bool) -> Result<()> {
        // Allows replacing without deinit, even without Clone/Copy
        let prev = std::mem::replace(
            &mut self.current_token,
            self.lexer.next_token()?.expect("FIXME this is temporary"),
        );

        if !ignore {
            self.previous_token = prev
        }

        if token::IGNORED.contains(&self.current_token.ttype) {
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
            return Err(TFMTError::ExhaustedTokens(current_ttype).into());
        }

        if current_ttype != expected_ttype {
            return Err(TFMTError::UnexpectedToken(
                expected_ttype,
                current_ttype,
            )
            .into());
        }

        self.advance()?;

        // Explicitly clone here, we need the original in previous, but also to return it.
        Ok(self.previous_token.clone())
    }

    fn trace(&mut self, log_string: &str) {
        let mut string: String = String::new();
        for i in 1..=self.depth {
            string.push_str(&(i % 10).to_string());
        }
        string.push(' ');
        string.push_str(log_string);

        trace!("{}", string)
    }

    // Grammar functions
    fn program(&mut self) -> Result<ast::Program> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        self.depth += 1;
        self.trace("Program");

        let name = self.consume(TokenType::ID)?;

        self.consume(TokenType::ParenthesisLeft)?;

        let parameters = self.parameters()?;

        self.consume(TokenType::ParenthesisRight)?;

        let description = match self.consume(TokenType::String) {
            Ok(token) => Some(token),
            Err(_) => None,
        };

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
        self.trace("Parameters");

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
        self.trace("Parameter");
        let identifier = self.consume(TokenType::ID)?;

        let default = match self.consume(TokenType::Equals) {
            Ok(_) => {
                if let Ok(token) = self.consume(TokenType::Integer) {
                    Some(token)
                } else if let Ok(token) = self.consume(TokenType::String) {
                    Some(token)
                } else {
                    return Err(TFMTError::Parser(
                        "Paramater has invalid default!".to_owned(),
                    )
                    .into());
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
        self.trace("Block");

        let drive = match self.consume(TokenType::Drive) {
            Ok(token) => Some(ast::DriveLetter { token }),
            Err(_) => None,
        };

        let expressions: Vec<Box<dyn ast::Node>> =
            self.expressions(vec![TokenType::CurlyBraceRight])?;

        self.depth -= 1;
        Ok(ast::Block { drive, expressions })
    }

    fn expressions(
        &mut self,
        terminators: Vec<TokenType>,
    ) -> Result<Vec<Box<dyn ast::Node>>> {
        let mut expressions: Vec<Box<dyn ast::Node>> = Vec::new();

        while !terminators.contains(&self.current_token.ttype) {
            self.trace(&format!(
                "Gathering expressions until {:?}",
                terminators
            ));

            if self.depth > 48 {
                return Err(TFMTError::Parser(
                    "Iteration depth > 48!".to_owned(),
                )
                .into());
            }
            expressions.push(self.expression()?);
        }

        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Box<dyn ast::Node>> {
        // FIXME add Expression(s) node, so we don't need to use trait?
        // FIXME Tighten up grammar?
        // Ternary ( "?" Ternary ":" Ternary )*
        self.depth += 1;
        self.trace("Expression");
        let mut expression: Box<dyn ast::Node> = self.ternary()?;

        while self.current_token.ttype == TokenType::QuestionMark {
            self.consume(TokenType::QuestionMark)?;
            let true_expr = self.ternary()?;
            self.consume(TokenType::Colon)?;
            let false_expr = self.ternary()?;

            expression = Box::new(ast::TernaryOp {
                condition: expression,
                true_expr,
                false_expr,
            });
        }

        self.depth -= 1;
        Ok(expression)
    }

    fn ternary(&mut self) -> Result<Box<dyn ast::Node>> {
        // Disjunct ( ( "||" | "|" ) Disjunct )*
        self.depth += 1;
        self.trace("Ternary");

        let mut ternary = self.disjunct()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleVerticalBar => self.consume(ttype)?,
                TokenType::VerticalBar => self.consume(ttype)?,
                _ => break,
            };

            ternary = Box::new(ast::BinaryOp {
                left: ternary,
                token: operator,
                right: self.ternary()?,
            });
        }

        self.depth -= 1;
        Ok(ternary)
    }

    fn disjunct(&mut self) -> Result<Box<dyn ast::Node>> {
        // Conjunct ( ( "&&" | "&" ) Conjunct )*
        self.depth += 1;
        self.trace("Disjunct");

        let mut disjunct = self.conjunct()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleAmpersand => self.consume(ttype)?,
                TokenType::Ampersand => self.consume(ttype)?,
                _ => break,
            };

            disjunct = Box::new(ast::BinaryOp {
                left: disjunct,
                token: operator,
                right: self.disjunct()?,
            });
        }

        self.depth -= 1;
        Ok(disjunct)
    }

    fn conjunct(&mut self) -> Result<Box<dyn ast::Node>> {
        // Term ( ( "+" | "-" ) Term )*
        self.depth += 1;
        self.trace("Conjunct");
        let mut conjunct = self.term()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::Plus => self.consume(ttype)?,
                TokenType::Hyphen => self.consume(ttype)?,
                _ => break,
            };

            conjunct = Box::new(ast::BinaryOp {
                left: conjunct,
                token: operator,
                right: self.conjunct()?,
            });
        }

        self.depth -= 1;
        Ok(conjunct)
    }

    fn term(&mut self) -> Result<Box<dyn ast::Node>> {
        // Factor ( ( "*" | "/" | "%" ) Factor )*
        self.depth += 1;
        self.trace("Term");

        let mut term = self.factor()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::Asterisk => self.consume(ttype)?,
                TokenType::SlashForward => self.consume(ttype)?,
                TokenType::Percent => self.consume(ttype)?,
                _ => break,
            };

            term = Box::new(ast::BinaryOp {
                left: term,
                token: operator,
                right: self.term()?,
            });
        }

        self.depth -= 1;
        Ok(term)
    }

    fn factor(&mut self) -> Result<Box<dyn ast::Node>> {
        // Exponent ( ( "**" | "^" ) Exponent )*
        self.depth += 1;
        self.trace("Factor");

        let mut factor = self.exponent()?;

        loop {
            let ttype = self.current_token.ttype;
            let operator = match ttype {
                TokenType::DoubleAsterisk => self.consume(ttype)?,
                TokenType::Caret => self.consume(ttype)?,
                _ => break,
            };

            factor = Box::new(ast::BinaryOp {
                left: factor,
                token: operator,
                right: self.factor()?,
            });
        }

        self.depth -= 1;
        Ok(factor)
    }

    fn exponent(&mut self) -> Result<Box<dyn ast::Node>> {
        // "+" Exponent | "-" Exponent | "(" Expression+ ")" | Statement
        self.depth += 1;
        self.trace("Exponent");

        let ttype = self.current_token.ttype;

        let exponent: Box<dyn ast::Node> = match ttype {
            TokenType::Plus => Box::new(ast::UnaryOp {
                token: self.consume(ttype)?,
                operand: self.exponent()?,
            }),
            TokenType::Hyphen => Box::new(ast::UnaryOp {
                token: self.consume(ttype)?,
                operand: self.exponent()?,
            }),
            TokenType::ParenthesisLeft => {
                self.consume(ttype)?;

                let expressions: Vec<Box<dyn ast::Node>> =
                    self.expressions(vec![TokenType::ParenthesisRight])?;

                self.consume(TokenType::ParenthesisRight)?;

                if expressions.is_empty() {
                    return Err(TFMTError::EmptyGroup.into());
                }

                Box::new(ast::Group { expressions })
            }
            _ => self.statement()?,
        };

        self.depth -= 1;
        Ok(exponent)
    }

    fn statement(&mut self) -> Result<Box<dyn ast::Node>> {
        // Comment | Function | Integer | String | Substitution | Tag
        self.depth += 1;
        self.trace("Statement");

        let ttype = self.current_token.ttype;

        let statement: Box<dyn ast::Node> = match ttype {
            TokenType::Dollar => {
                self.consume(ttype)?;

                if self.current_token.ttype == TokenType::ParenthesisLeft {
                    self.consume(TokenType::ParenthesisLeft)?;
                    let substitution = ast::Substitution {
                        token: self.consume(TokenType::ID)?,
                    };
                    self.consume(TokenType::ParenthesisRight)?;
                    Box::new(substitution)
                } else {
                    self.function()?
                }
            }
            TokenType::AngleBracketLeft => self.tag()?,
            TokenType::Integer => Box::new(ast::IntegerNode {
                integer: self.consume(ttype)?,
            }),
            TokenType::String => Box::new(ast::StringNode {
                string: self.consume(ttype)?,
            }),
            _ => return Err(TFMTError::UnrecognizedToken(ttype).into()),
        };

        self.depth -= 1;
        Ok(statement)
    }

    fn function(&mut self) -> Result<Box<dyn ast::Node>> {
        self.depth += 1;
        self.trace("Function");

        let identifier = self.consume(TokenType::ID)?;

        self.consume(TokenType::ParenthesisLeft)?;

        let mut arguments: Vec<Box<dyn ast::Node>> = Vec::new();

        // while self.current_token.ttype() != TokenType::ParenthesisRight {
        loop {
            arguments.push(self.expression()?);
            if self.consume(TokenType::Comma).is_err() {
                break;
            }
        }

        let function = ast::Function {
            start_token: identifier,
            arguments,
            end_token: self.consume(TokenType::ParenthesisRight)?,
        };

        self.depth -= 1;
        Ok(Box::new(function))
    }

    fn tag(&mut self) -> Result<Box<dyn ast::Node>> {
        self.depth += 1;
        self.trace("Tag");

        let start_token = self.consume(TokenType::AngleBracketLeft)?;

        let identifier = self.consume(TokenType::ID)?;

        self.consume(TokenType::AngleBracketRight)?;

        let tag = ast::Tag {
            start_token,
            token: identifier,
        };

        self.depth -= 1;
        Ok(Box::new(tag))
    }
}
