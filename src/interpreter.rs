use crate::{
    ast::Expr,
    token::{Literal, TokenType},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("runtime error")]
    Runtime,
}

fn is_truthy(literal: Literal) -> bool {
    match literal {
        Literal::Nil => false,
        Literal::Boolean(value) => value,
        _ => true,
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    fn evaluate(&self, expr: Expr) -> Result<Literal, Error> {
        match expr {
            Expr::Literal(literal) => Ok(literal),
            Expr::Grouping(group) => self.evaluate(*group),
            Expr::Unary(op, right) => {
                let literal = self.evaluate(*right)?;

                match op.typ() {
                    TokenType::Minus => {
                        if let Literal::Number(value) = literal {
                            Ok(Literal::Number(-value))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::Bang => Ok(Literal::Boolean(!is_truthy(literal))),
                    _ => Err(Error::Runtime),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match op.typ() {
                    TokenType::Greater => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left > right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::GreaterEqual => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left >= right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::Less => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left < right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::LessEqual => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left <= right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
                    TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
                    TokenType::Minus => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left - right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::Plus => {
                        if let (Literal::Number(left), Literal::Number(right)) =
                            (left.clone(), right.clone())
                        {
                            Ok(Literal::Number(left - right))
                        } else {
                            if let (Literal::String(left), Literal::String(right)) = (left, right) {
                                Ok(Literal::String(format!("{left}{right}")))
                            } else {
                                Err(Error::Runtime)
                            }
                        }
                    }
                    TokenType::Slash => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left / right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    TokenType::Star => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left * right))
                        } else {
                            Err(Error::Runtime)
                        }
                    }
                    _ => Err(Error::Runtime),
                }
            }
        }
    }

    pub fn interpret(&self, expr: Expr) -> Result<(), Error> {
        let value = self.evaluate(expr)?;
        println!("{}", value);

        Ok(())
    }
}
