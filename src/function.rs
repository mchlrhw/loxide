use crate::{
    ast::Stmt,
    callable::Callable,
    class::LoxInstance,
    interpreter::{Environment, Error, Interpreter},
    token::{Token, TokenType},
    value::Value,
};
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn value(self) -> Value {
        Value::Callable(Box::new(self))
    }

    pub fn bind(self, instance: Rc<RefCell<LoxInstance>>) -> Self {
        let environment = Environment::wrap(self.closure);
        environment
            .borrow_mut()
            .define("this", &Value::Instance(instance));

        LoxFunction::new(
            self.name,
            self.params,
            self.body,
            environment,
            self.is_initializer,
        )
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.lexeme();

        write!(f, "<fn {name}>")
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, Error> {
        let environment = Environment::wrap(self.closure.clone());
        for (idx, param) in self.params.iter().enumerate() {
            environment
                .borrow_mut()
                .define(param.lexeme(), &arguments[idx]);
        }

        match interpreter.execute_block(self.body.clone(), environment) {
            Ok(_) => {
                if !self.is_initializer {
                    Ok(Value::Nil)
                } else {
                    self.closure
                        .borrow()
                        .get(&Token::new(TokenType::This, "this", None, 42))
                }
            }
            Err(Error::Return { value }) => {
                if self.is_initializer {
                    self.closure
                        .borrow()
                        .get(&Token::new(TokenType::This, "this", None, 42))
                } else {
                    Ok(value)
                }
            }
            Err(error) => Err(error),
        }
    }

    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
