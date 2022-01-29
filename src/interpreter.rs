use crate::{
    ast::{Expr, Stmt},
    token::{Literal, Token, TokenType},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}\n[line {line}]")]
    Runtime { message: String, line: usize },
}

fn is_truthy(literal: Literal) -> bool {
    match literal {
        Literal::Nil => false,
        Literal::Boolean(value) => value,
        _ => true,
    }
}

fn check_number_operand(operator: Token, operand: Literal) -> Result<f64, Error> {
    if let Literal::Number(value) = operand {
        Ok(value)
    } else {
        Err(Error::Runtime {
            message: format!("Operand must be a number."),
            line: operator.line(),
        })
    }
}

fn check_number_operands(
    operator: Token,
    left: Literal,
    right: Literal,
) -> Result<(f64, f64), Error> {
    if let (Literal::Number(left_val), Literal::Number(right_val)) = (left, right) {
        Ok((left_val, right_val))
    } else {
        Err(Error::Runtime {
            message: format!("Operands must be a numbers."),
            line: operator.line(),
        })
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
                        let value = check_number_operand(op, literal)?;

                        Ok(Literal::Number(-value))
                    }
                    TokenType::Bang => Ok(Literal::Boolean(!is_truthy(literal))),
                    typ => panic!("{typ:?} is not a valid unary operator"),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match op.typ() {
                    TokenType::Greater => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Boolean(left > right))
                    }
                    TokenType::GreaterEqual => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Boolean(left >= right))
                    }
                    TokenType::Less => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Boolean(left < right))
                    }
                    TokenType::LessEqual => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Boolean(left <= right))
                    }
                    TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
                    TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
                    TokenType::Minus => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Number(left - right))
                    }
                    TokenType::Plus => {
                        if let (Literal::Number(left), Literal::Number(right)) =
                            (left.clone(), right.clone())
                        {
                            Ok(Literal::Number(left + right))
                        } else {
                            if let (Literal::String(left), Literal::String(right)) = (left, right) {
                                Ok(Literal::String(format!("{left}{right}")))
                            } else {
                                Err(Error::Runtime {
                                    message: format!(
                                        "Operands must be two numbers or two strings."
                                    ),
                                    line: op.line(),
                                })
                            }
                        }
                    }
                    TokenType::Slash => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Number(left / right))
                    }
                    TokenType::Star => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Literal::Number(left * right))
                    }
                    typ => panic!("{typ:?} is not a valid binary operator."),
                }
            }
        }
    }

    fn execute(&self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Expression(expression) => {
                self.evaluate(expression)?;
            }
            Stmt::Print(expression) => {
                let value = self.evaluate(expression)?;
                println!("{value}");
            }
        }

        Ok(())
    }

    pub fn interpret(&self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                println!("{error}");
            }
        }
    }
}
