use crate::{
    ast::{Expr, ExprKind, Stmt},
    class::{LoxClass, LoxInstance},
    clock::Clock,
    function::LoxFunction,
    token::{Token, TokenType},
    value::Value,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}\n[line {line}]")]
    Runtime { message: String, line: usize },

    #[error("Returning {value:?}")]
    Return { value: Value },
}

#[derive(Clone, Default, Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn wrap(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        let environment = Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        };

        Rc::new(RefCell::new(environment))
    }

    pub fn define(&mut self, name: &str, value: &Value) {
        self.values.insert(name.to_string(), value.clone());
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self.enclosing.clone().expect("must have an ancestor");
        for _ in 1..distance {
            let new_env = environment
                .borrow()
                .enclosing
                .clone()
                .expect("must have an ancestor");
            environment = new_env;
        }

        environment
    }

    fn assign(&mut self, name: &Token, value: &Value) -> Result<(), Error> {
        let lexeme = name.lexeme();

        if self.values.contains_key(lexeme) {
            self.values.insert(lexeme.to_string(), value.clone());

            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(Error::Runtime {
                message: format!("Undefined variable '{lexeme}'."),
                line: name.line(),
            })
        }
    }

    fn assign_at(&mut self, distance: usize, name: &Token, value: &Value) -> Result<(), Error> {
        if distance == 0 {
            self.assign(name, value)
        } else {
            self.ancestor(distance).borrow_mut().assign(name, value)
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        let lexeme = name.lexeme();

        if let Some(value) = self.values.get(lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(Error::Runtime {
                message: format!("Undefined variable '{lexeme}'."),
                line: name.line(),
            })
        }
    }

    fn get_at(&self, distance: usize, name: &Token) -> Result<Value, Error> {
        if distance == 0 {
            self.get(name)
        } else {
            self.ancestor(distance).borrow().get(name)
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

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let globals = Rc::new(RefCell::new(Environment::default()));
        globals.borrow_mut().define("clock", &Clock::value());

        let environment = globals.clone();
        let locals = HashMap::new();

        Self {
            globals,
            environment,
            locals,
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn globals(&self) -> Rc<RefCell<Environment>> {
        self.globals.clone()
    }

    fn lookup_variable(&self, name: &Token, expr: &Expr) -> Result<Value, Error> {
        let distance = self.locals.get(expr);
        if let Some(distance) = distance {
            self.environment.borrow().get_at(*distance, name)
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Value, Error> {
        match expr.kind {
            ExprKind::Literal(value) => Ok(value),
            ExprKind::Grouping(group) => self.evaluate(*group),
            ExprKind::Unary { operator, right } => {
                let value = self.evaluate(*right)?;

                match operator.typ() {
                    TokenType::Minus => {
                        let n = check_number_operand(operator, value)?;

                        Ok(Value::Number(-n))
                    }
                    TokenType::Bang => Ok(Value::Boolean(!is_truthy(&value))),
                    typ => panic!("{typ:?} is not a valid unary operator"),
                }
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;

                match operator.typ() {
                    TokenType::Greater => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Boolean(left > right))
                    }
                    TokenType::GreaterEqual => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Boolean(left >= right))
                    }
                    TokenType::Less => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Boolean(left < right))
                    }
                    TokenType::LessEqual => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Boolean(left <= right))
                    }
                    TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
                    TokenType::BangEqual => Ok(Value::Boolean(left != right)),
                    TokenType::Minus => {
                        let (left, right) = check_number_operands(operator, left, right)?;

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
                                line: operator.line(),
                            })
                        }
                    }
                    TokenType::Slash => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Number(left / right))
                    }
                    TokenType::Star => {
                        let (left, right) = check_number_operands(operator, left, right)?;

                        Ok(Value::Number(left * right))
                    }
                    typ => panic!("{typ:?} is not a valid binary operator."),
                }
            }
            ExprKind::Variable(ref name) => self.lookup_variable(name, &expr),
            ExprKind::Assign {
                ref name,
                ref value,
            } => {
                let value = self.evaluate(*value.clone())?;

                if let Some(distance) = self.locals.get(&expr) {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name, &value)?;
                } else {
                    self.globals.borrow_mut().assign(name, &value)?;
                }

                Ok(value)
            }
            ExprKind::Logical {
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
            ExprKind::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(*callee)?;

                let mut evaluated_args = vec![];
                for expr in arguments {
                    evaluated_args.push(self.evaluate(expr)?);
                }

                if let Value::Callable(function) = callee {
                    let arity = function.arity();
                    let arg_cnt = evaluated_args.len();
                    if arg_cnt != arity {
                        Err(Error::Runtime {
                            message: format!("Expected {arity} arguments but got {arg_cnt}."),
                            line: paren.line(),
                        })
                    } else {
                        function.call(self, evaluated_args)
                    }
                } else {
                    Err(Error::Runtime {
                        message: "Can only call functions and classes.".to_string(),
                        line: paren.line(),
                    })
                }
            }
            ExprKind::Get { object, name } => {
                if let Value::Instance(instance) = self.evaluate(*object)? {
                    LoxInstance::get(instance, &name)
                } else {
                    Err(Error::Runtime {
                        message: "Only instances have properties.".to_string(),
                        line: name.line(),
                    })
                }
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => {
                if let Value::Instance(instance) = self.evaluate(*object)? {
                    let value = self.evaluate(*value)?;
                    instance.borrow_mut().set(&name, value.clone());

                    Ok(value)
                } else {
                    Err(Error::Runtime {
                        message: "Only instances have fields.".to_string(),
                        line: name.line(),
                    })
                }
            }
            ExprKind::This(ref keyword) => self.lookup_variable(keyword, &expr),
            ExprKind::Super { ref method, .. } => {
                let distance = self.locals.get(&expr).expect("must have super in locals");

                let superclass = {
                    self.environment
                        .borrow()
                        .get_at(*distance, &Token::new(TokenType::Super, "super", None, 42))?
                };

                let object = {
                    self.environment.borrow().get_at(
                        distance - 1,
                        &Token::new(TokenType::Super, "this", None, 42),
                    )?
                };

                if let Value::Callable(callable) = superclass {
                    if let Some(class) = callable.as_any().downcast_ref::<LoxClass>() {
                        let name = method.lexeme();
                        let method = class.find_method(name).ok_or(Error::Runtime {
                            message: format!("Undefined property '{name}'."),
                            line: method.line(),
                        })?;

                        if let Value::Instance(object) = object {
                            return Ok(method.bind(object).value());
                        }

                        panic!("object must be an instance");
                    }

                    panic!("superclass must be a class");
                }

                panic!("superclass must be a callable");
            }
        }
    }

    pub fn execute_block_inner(&mut self, statements: Vec<Stmt>) -> Result<(), Error> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        let previous = self.environment.clone();
        self.environment = environment;

        let res = self.execute_block_inner(statements);

        self.environment = previous;

        res
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

                self.environment.borrow_mut().define(name.lexeme(), &value);
            }
            Stmt::Block(statements) => {
                self.execute_block(statements, Environment::wrap(self.environment.clone()))?;
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
            Stmt::Function { name, params, body } => {
                let function =
                    LoxFunction::new(name.clone(), params, body, self.environment.clone(), false)
                        .value();
                self.environment
                    .borrow_mut()
                    .define(name.lexeme(), &function);
            }
            Stmt::Return { value, .. } => {
                let value = if let Some(value) = value {
                    self.evaluate(value)?
                } else {
                    Value::Nil
                };

                return Err(Error::Return { value });
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let mut sc = None;

                if let Some(superclass) = superclass {
                    let value = self.evaluate(superclass)?;
                    if let Value::Callable(ref callable) = value {
                        if let Some(class) = callable.as_any().downcast_ref::<LoxClass>().cloned() {
                            sc = Some(Box::new(class));
                        } else {
                            return Err(Error::Runtime {
                                message: "Superclass must be a class".to_string(),
                                line: name.line(),
                            });
                        }
                    } else {
                        return Err(Error::Runtime {
                            message: "Superclass must be a class".to_string(),
                            line: name.line(),
                        });
                    }
                }

                {
                    self.environment
                        .borrow_mut()
                        .define(name.lexeme(), &Value::Nil);
                }

                if let Some(ref superclass) = &sc {
                    self.environment = Environment::wrap(self.environment.clone());
                    self.environment
                        .borrow_mut()
                        .define("super", &superclass.clone().value());
                }

                let mut functions = HashMap::new();
                for method in methods {
                    if let Stmt::Function { name, params, body } = method {
                        let function = LoxFunction::new(
                            name.clone(),
                            params,
                            body,
                            self.environment.clone(),
                            name.lexeme() == "init",
                        );
                        functions.insert(name.lexeme().to_string(), function);
                    }
                }

                if sc.is_some() {
                    let enclosing = { self.environment.borrow().ancestor(0) };
                    self.environment = enclosing;
                }

                let class = LoxClass::new(name.lexeme(), sc, functions).value();

                self.environment.borrow_mut().assign(&name, &class)?;
            }
        }

        Ok(())
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
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
