use crate::{
    ast::Stmt,
    callable::Callable,
    interpreter::{Environment, Error, Interpreter},
    token::Token,
    value::Value,
};
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone, Debug)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
        }
    }

    pub fn value(self) -> Value {
        Value::Callable(Box::new(self))
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
            Ok(_) => Ok(Value::Nil),
            Err(Error::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
    }

    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }
}
