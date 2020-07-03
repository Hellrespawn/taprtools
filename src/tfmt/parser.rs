use log::trace;

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
        // Prime parser
        self._advance(true)?;
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

    fn trace(&mut self, log_string: &str) {
        let mut string: String = String::new();
        for i in 1..=self.depth {
            string.push_str(&(i % 10).to_string());
        }
        string.push_str(" ");
        string.push_str(log_string);

        trace!("{}", string)
    }

    // Grammar functions
    fn program(&mut self) -> Result<ast::Program, TFMTError> {
        // ID "(" Parameters ")" ( String )? "{" Block "}"
        self.depth += 1;
        self.trace("Program");

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

        self.depth -= 1;

        Ok(ast::Program {
            name,
            parameters,
            description,
            block,
        })
    }

    fn parameters(&mut self) -> Result<ast::Parameters, TFMTError> {
        // ( Parameter ( "," Parameter )* )?
        self.depth += 1;
        self.trace("Parameters");

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
        self.depth -= 1;
        Ok(ast::Parameters { parameters })
    }

    fn parameter(&mut self) -> Result<ast::Parameter, TFMTError> {
        // ID ( "=" ( Integer | String ) )?
        self.depth += 1;
        self.trace("Parameter");
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

        self.depth -= 1;

        Ok(ast::Parameter {
            token: identifier,
            default,
        })
    }

    fn block(&mut self) -> Result<ast::Block, TFMTError> {
        // ( DriveLetter )? Expression*
        self.depth += 1;
        self.trace("Block");

        let drive = match self.consume(TokenType::DRIVE) {
            Ok(token) => Some(ast::DriveLetter { token }),
            Err(_) => None,
        };

        let expressions: Vec<Box<dyn ast::Node>> =
            self.expressions(vec![TokenType::CURLY_BRACE_RIGHT])?;

        self.depth -= 1;
        Ok(ast::Block { drive, expressions })
    }

    fn expressions(
        &mut self,
        terminators: Vec<TokenType>,
    ) -> Result<Vec<Box<dyn ast::Node>>, TFMTError> {
        let mut expressions: Vec<Box<dyn ast::Node>> = Vec::new();

        while !terminators.contains(&self.current_token().ttype()) {
            self.trace(&format!(
                "Gathering expressions until {:?}",
                terminators
            ));

            if self.depth > 24 {
                return Err(TFMTError::Parser(
                    "Iteration depth > 24!".to_owned(),
                ));
            }
            expressions.push(self.expression()?);
        }

        Ok(expressions)
    }

    fn expression(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Ternary ( "?" Ternary ":" Ternary )*
        self.depth += 1;
        self.trace("Expression");
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

        self.depth -= 1;
        Ok(expression)
    }

    fn ternary(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Disjunct ( ( "||" | "|" ) Disjunct )*
        self.depth += 1;
        self.trace("Ternary");

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

        self.depth -= 1;
        Ok(ternary)
    }

    fn disjunct(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Conjunct ( ( "&&" | "&" ) Conjunct )*
        self.depth += 1;
        self.trace("Disjunct");

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

        self.depth -= 1;
        Ok(disjunct)
    }

    fn conjunct(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Term ( ( "+" | "-" ) Term )*
        self.depth += 1;
        self.trace("Conjunct");
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

        self.depth -= 1;
        Ok(conjunct)
    }

    fn term(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Factor ( ( "*" | "/" | "%" ) Factor )*
        self.depth += 1;
        self.trace("Term");

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

        self.depth -= 1;
        Ok(term)
    }

    fn factor(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Exponent ( ( "**" | "^" ) Exponent )*
        self.depth += 1;
        self.trace("Factor");

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

        self.depth -= 1;
        Ok(factor)
    }

    fn exponent(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // "+" Exponent | "-" Exponent | "(" Expression+ ")" | Statement
        self.depth += 1;
        self.trace("Exponent");

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

        self.depth -= 1;
        Ok(exponent)
    }

    fn statement(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        // Comment | Function | Integer | String | Substitution | Tag
        self.depth += 1;
        self.trace("Statement");

        let ttype = self.current_token().ttype();

        let statement: Box<dyn ast::Node> = match ttype {
            TokenType::DOLLAR => {
                self.consume(ttype)?;

                if self.current_token().ttype() == TokenType::PARENTHESIS_LEFT {
                    self.consume(TokenType::PARENTHESIS_LEFT)?;
                    let substitution = ast::Substitution {
                        token: self.consume(TokenType::ID)?,
                    };
                    self.consume(TokenType::PARENTHESIS_LEFT)?;
                    Box::new(substitution)
                } else {
                    self.function()?
                }
            }
            TokenType::ANGLE_BRACKET_LEFT => self.tag()?,
            TokenType::INTEGER => Box::new(ast::Integer {
                token: self.consume(ttype)?,
            }),
            TokenType::STRING => Box::new(ast::StringNode {
                token: self.consume(ttype)?,
            }),
            _ => return Err(TFMTError::UnrecognizedToken(ttype)),
        };

        self.depth -= 1;
        Ok(statement)
    }

    fn function(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        self.depth += 1;
        self.trace("Function (unimpl)");
        self.depth -= 1;
        self.advance()?;
        Ok(Box::new(ast::StringNode {
            token: Token::new(
                0,
                0,
                TokenType::STRING,
                Some("function".to_owned()),
            ),
        }))
    }

    fn tag(&mut self) -> Result<Box<dyn ast::Node>, TFMTError> {
        self.depth += 1;
        self.trace("Tag");

        let start_token = self.consume(TokenType::ANGLE_BRACKET_LEFT)?;

        let identifier = self.consume(TokenType::ID)?;

        self.consume(TokenType::ANGLE_BRACKET_RIGHT)?;

        let tag = ast::Tag {
            start_token,
            token: identifier
        };

        self.depth -= 1;
        Ok(Box::new(tag))
    }
}
