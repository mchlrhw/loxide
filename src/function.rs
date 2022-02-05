use crate::{
    ast::Stmt,
    callable::Callable,
    interpreter::{Environment, Error, Interpreter},
    token::Token,
    value::Value,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl LoxFunction {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
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
        // FIXME: This should be interpreter.globals() instead.
        let mut environment = Environment::new(interpreter.environment().to_owned());
        for (idx, param) in self.params.iter().enumerate() {
            environment.define(param.lexeme(), &arguments[idx]);
        }

        interpreter.execute_block(self.body.clone(), environment)?;

        Ok(Value::Nil)
    }

    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }
}
