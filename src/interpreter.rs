use crate::{
    ast::{Expr, Stmt},
    token::{Literal, Token, TokenType},
};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}\n[line {line}]")]
    Runtime { message: String, line: usize },
}

#[derive(Clone, Default)]
struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    fn new(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: &str, value: &Literal) {
        self.values.insert(name.to_string(), value.clone());
    }

    fn assign(&mut self, name: &Token, value: &Literal) -> Result<(), Error> {
        let lexeme = name.lexeme();

        if self.values.contains_key(lexeme) {
            self.values.insert(lexeme.to_string(), value.clone());

            return Ok(());
        }

        if let Some(ref mut enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(Error::Runtime {
                message: format!("Undefined variable '{lexeme}'."),
                line: name.line(),
            })
        }
    }

    fn get(&self, name: &Token) -> Result<Literal, Error> {
        let lexeme = name.lexeme();

        if self.values.contains_key(lexeme) {
            return Ok(self.values.get(lexeme).unwrap().clone());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(Error::Runtime {
                message: format!("Undefined variable '{lexeme}'."),
                line: name.line(),
            })
        }
    }
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

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Environment::default();

        Self { environment }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Literal, Error> {
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
            Expr::Variable(name) => self.environment.get(&name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.environment.assign(&name, &value)?;

                Ok(value)
            }
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Expression(expression) => {
                self.evaluate(expression)?;
            }
            Stmt::Print(expression) => {
                let value = self.evaluate(expression)?;
                println!("{value}");
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(initializer) = initializer {
                    self.evaluate(initializer)?
                } else {
                    Literal::Nil
                };

                self.environment.define(&name, &value);
            }
            Stmt::Block(statements) => {
                self.environment = Environment::new(self.environment.clone());

                for stmt in statements {
                    self.execute(*stmt)?;
                }

                if let Some(environment) = self.environment.enclosing.clone() {
                    self.environment = *environment;
                }
            }
        }

        Ok(())
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                println!("{error}");
            }
        }
    }
}
