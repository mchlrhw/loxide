use crate::{
    ast::{Expr, Stmt},
    token::{Token, TokenType, Value},
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
    values: HashMap<String, Value>,
}

impl Environment {
    fn new(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: &str, value: &Value) {
        self.values.insert(name.to_string(), value.clone());
    }

    fn assign(&mut self, name: &Token, value: &Value) -> Result<(), Error> {
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

    fn get(&self, name: &Token) -> Result<Value, Error> {
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

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Boolean(b) => *b,
        _ => true,
    }
}

fn check_number_operand(operator: Token, operand: Value) -> Result<f64, Error> {
    if let Value::Number(n) = operand {
        Ok(n)
    } else {
        Err(Error::Runtime {
            message: "Operand must be a number.".to_string(),
            line: operator.line(),
        })
    }
}

fn check_number_operands(operator: Token, left: Value, right: Value) -> Result<(f64, f64), Error> {
    if let (Value::Number(left_n), Value::Number(right_n)) = (left, right) {
        Ok((left_n, right_n))
    } else {
        Err(Error::Runtime {
            message: "Operands must be a numbers.".to_string(),
            line: operator.line(),
        })
    }
}

#[derive(Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(value) => Ok(value),
            Expr::Grouping(group) => self.evaluate(*group),
            Expr::Unary(op, right) => {
                let value = self.evaluate(*right)?;

                match op.typ() {
                    TokenType::Minus => {
                        let n = check_number_operand(op, value)?;

                        Ok(Value::Number(-n))
                    }
                    TokenType::Bang => Ok(Value::Boolean(!is_truthy(&value))),
                    typ => panic!("{typ:?} is not a valid unary operator"),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match op.typ() {
                    TokenType::Greater => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Boolean(left > right))
                    }
                    TokenType::GreaterEqual => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Boolean(left >= right))
                    }
                    TokenType::Less => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Boolean(left < right))
                    }
                    TokenType::LessEqual => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Boolean(left <= right))
                    }
                    TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    TokenType::Minus => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Number(left - right))
                    }
                    TokenType::Plus => {
                        if let (Value::Number(left), Value::Number(right)) =
                            (left.clone(), right.clone())
                        {
                            Ok(Value::Number(left + right))
                        } else if let (Value::String(left), Value::String(right)) = (left, right) {
                            Ok(Value::String(format!("{left}{right}")))
                        } else {
                            Err(Error::Runtime {
                                message: "Operands must be two numbers or two strings.".to_string(),
                                line: op.line(),
                            })
                        }
                    }
                    TokenType::Slash => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Number(left / right))
                    }
                    TokenType::Star => {
                        let (left, right) = check_number_operands(op, left, right)?;

                        Ok(Value::Number(left * right))
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
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;

                if matches!(operator.typ(), TokenType::Or) {
                    if is_truthy(&left) {
                        return Ok(left);
                    }
                } else if !is_truthy(&left) {
                    return Ok(left);
                }

                self.evaluate(*right)
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
                    Value::Nil
                };

                self.environment.define(&name, &value);
            }
            Stmt::Block(statements) => {
                self.environment = Environment::new(self.environment.clone());

                for stmt in statements {
                    self.execute(stmt)?;
                }

                if let Some(environment) = self.environment.enclosing.clone() {
                    self.environment = *environment;
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition.clone())?) {
                    self.execute(*body.clone())?;
                }
            }
        }

        Ok(())
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            if let Err(error) = self.execute(statement) {
                println!("{error}");
                return;
            }
        }
    }
}
