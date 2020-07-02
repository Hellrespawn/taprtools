use super::ast;
use super::lexer::Lexer;
use super::token::{self, Token, TokenType};
use crate::error::TFMTError;
// use std::error::Error;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    depth: u32,
    current_token: Option<Token>,
    previous_token: Option<Token>,
}

impl<'a> Parser<'a> {
    // Constructors
    pub fn from_lexer(lexer: Lexer) -> Parser {
        Parser {
            lexer,
            depth: 0,
            current_token: None,
            previous_token: None,
        }
    }

    pub fn from_string(string: &str) -> Parser {
        Parser {
            lexer: Lexer::new(string),
            depth: 0,
            current_token: None,
            previous_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<ast::Program, TFMTError> {
        self.program()
    }

    // Fields
    fn current_token(&mut self) -> Token {
        self.current_token
            .clone()
            .expect("Should pretty much always be safe!")
    }

    fn previous_token(&mut self) -> Token {
        self.previous_token
            .clone()
            .expect("Should pretty much always be safe!")
    }

    fn _advance(&mut self, ignore: bool) -> Result<(), TFMTError> {
        if !ignore {
            self.previous_token = self.current_token.take()
        }

        self.current_token = self.lexer.next_token()?;

        if let Some(token) = self.current_token.as_ref() {
            if token::IGNORED.contains(&token.ttype()) {
                self._advance(true)?;
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), TFMTError> {
        self._advance(false)
    }

    fn consume(
        &mut self,
        expected_ttype: TokenType,
    ) -> Result<Token, TFMTError> {
        let current_ttype = self.current_token.as_ref().unwrap().ttype();

        if current_ttype == TokenType::EOF {
            return Err(TFMTError::ExhaustedTokens(current_ttype));
        }

        if current_ttype != expected_ttype {
            return Err(TFMTError::UnexpectedToken(
                expected_ttype,
                current_ttype,
            ));
        }

        self.advance()?;

        Ok(self.previous_token())
    }

    // Grammar functions
    fn program(&mut self) -> Result<ast::Program, TFMTError> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        let name = self.consume(TokenType::ID)?;

        self.consume(TokenType::PARENTHESIS_LEFT)?;

        let parameters = self.parameters()?;

        self.consume(TokenType::PARENTHESIS_RIGHT)?;

        let description = match self.consume(TokenType::STRING) {
            Ok(_) => Some(self.previous_token()),
            Err(_) => None,
        };

        self.consume(TokenType::CURLY_BRACE_LEFT)?;
        let block = self.block()?;
        self.consume(TokenType::CURLY_BRACE_RIGHT)?;

        Ok(ast::Program {
            name,
            parameters,
            description,
            block,
        })
    }

    fn parameters(&mut self) -> Result<ast::Parameters, TFMTError> {
        // ( Parameter ( "," Parameter )* )?
        let mut parameters = Vec::new();

        loop {
            match self.parameter() {
                Ok(parameter) => parameters.push(parameter),
                Err(_) => break,
            }

            if self.consume(TokenType::COMMA).is_err() {
                break;
            }
        }

        Ok(ast::Parameters { parameters })
    }

    fn parameter(&mut self) -> Result<ast::Parameter, TFMTError> {
        // ID ( "=" ( Integer | String ) )?
        let identifier = self.consume(TokenType::ID)?;

        let default = match self.consume(TokenType::EQUALS) {
            Ok(_) => {
                if let Ok(token) = self.consume(TokenType::INTEGER) {
                    Some(token)
                } else if let Ok(token) = self.consume(TokenType::STRING) {
                    Some(token)
                } else {
                    return Err(TFMTError::Parser(
                        "Paramater has invalid default!".to_owned(),
                    ));
                }
            }
            Err(_) => None,
        };

        Ok(ast::Parameter {
            token: identifier,
            default,
        })
    }

    fn block(&mut self) -> Result<ast::Block, TFMTError> {
        // ( DriveLetter )? Expression*
        let drive = match self.consume(TokenType::DRIVE) {
            Ok(token) => Some(ast::DriveLetter { token }),
            Err(_) => None,
        };

        let expressions: Vec<Box<dyn ast::Node>> =
            self.expressions(vec![TokenType::CURLY_BRACE_RIGHT])?;

        Ok(ast::Block { drive, expressions })
    }

    fn expressions(
        &mut self,
        terminators: Vec<TokenType>,
    ) -> Result<Vec<Box<dyn ast::Node>>, TFMTError> {
        let mut expressions: Vec<Box<dyn ast::Node>> = Vec::new();

        while !terminators.contains(&self.current_token().ttype()) {
            expressions.push(self.expression()?);
        }

        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Ternary ( "?" Ternary ":" Ternary )*
        let mut expression: Box<dyn ast::Node> = self.ternary()?;

        while self.current_token().ttype() == TokenType::QUESTION_MARK {
            self.consume(TokenType::QUESTION_MARK)?;
            let true_expr = self.ternary()?;
            self.consume(TokenType::COLON)?;
            let false_expr = self.ternary()?;

            expression = Box::new(ast::TernaryOp {
                condition: expression,
                true_expr,
                false_expr,
            });
        }

        Ok(expression)
    }

    fn ternary(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Disjunct ( ( "||" | "|" ) Disjunct )*
        let mut ternary = self.disjunct()?;

        loop {
            let ttype = self.current_token().ttype();
            let operator = match ttype {
                TokenType::DOUBLE_VERTICAL_BAR => self.consume(ttype)?,
                TokenType::VERTICAL_BAR => self.consume(ttype)?,
                _ => break,
            };

            ternary = Box::new(ast::BinOp {
                left: ternary,
                token: operator,
                right: self.ternary()?,
            });
        }

        Ok(ternary)
    }

    fn disjunct(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Conjunct ( ( "&&" | "&" ) Conjunct )*
        let mut disjunct = self.conjunct()?;

        loop {
            let ttype = self.current_token().ttype();
            let operator = match ttype {
                TokenType::DOUBLE_AMPERSAND => self.consume(ttype)?,
                TokenType::AMPERSAND => self.consume(ttype)?,
                _ => break,
            };

            disjunct = Box::new(ast::BinOp {
                left: disjunct,
                token: operator,
                right: self.disjunct()?,
            });
        }

        Ok(disjunct)
    }

    fn conjunct(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Term ( ( "+" | "-" ) Term )*
        let mut conjunct = self.term()?;

        loop {
            let ttype = self.current_token().ttype();
            let operator = match ttype {
                TokenType::PLUS => self.consume(ttype)?,
                TokenType::HYPHEN => self.consume(ttype)?,
                _ => break,
            };

            conjunct = Box::new(ast::BinOp {
                left: conjunct,
                token: operator,
                right: self.conjunct()?,
            });
        }

        Ok(conjunct)
    }

    fn term(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Factor ( ( "*" | "/" | "%" ) Factor )*
        let mut term = self.factor()?;

        loop {
            let ttype = self.current_token().ttype();
            let operator = match ttype {
                TokenType::ASTERISK => self.consume(ttype)?,
                TokenType::SLASH_FORWARD => self.consume(ttype)?,
                TokenType::PERCENT => self.consume(ttype)?,
                _ => break,
            };

            term = Box::new(ast::BinOp {
                left: term,
                token: operator,
                right: self.term()?,
            });
        }

        Ok(term)
    }

    fn factor(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Exponent ( ( "**" | "^" ) Exponent )*
        let mut factor = self.exponent()?;

        loop {
            let ttype = self.current_token().ttype();
            let operator = match ttype {
                TokenType::DOUBLE_ASTERISK => self.consume(ttype)?,
                TokenType::CARET => self.consume(ttype)?,
                _ => break,
            };

            factor = Box::new(ast::BinOp {
                left: factor,
                token: operator,
                right: self.factor()?,
            });
        }

        Ok(factor)
    }

    fn exponent(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // "+" Exponent | "-" Exponent | "(" Expression+ ")" | Statement
        let ttype = self.current_token().ttype();

        let exponent: Box<dyn ast::Node> = match ttype {
            TokenType::PLUS => Box::new(ast::UnaryOp {
                token: self.consume(ttype)?,
                operand: self.exponent()?,
            }),
            TokenType::HYPHEN => Box::new(ast::UnaryOp {
                token: self.consume(ttype)?,
                operand: self.exponent()?,
            }),
            TokenType::PARENTHESIS_LEFT => {
                self.consume(ttype)?;

                let expressions: Vec<Box<dyn ast::Node>> =
                    self.expressions(vec![TokenType::PARENTHESIS_RIGHT])?;

                self.consume(TokenType::PARENTHESIS_RIGHT)?;

                if expressions.is_empty() {
                    return Err(TFMTError::EmptyGroup);
                }

                Box::new(ast::Group { expressions })
            }
            _ => self.statement()?,
        };

        Ok(exponent)
    }

    fn statement(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Comment | Function | Integer | String | Substitution | Tag
        Err(TFMTError::Parser("Temp".to_owned()))
    }
}
