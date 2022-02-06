use crate::{
    interpreter::{Error, Interpreter},
    value::Value,
};
use std::fmt::{Debug, Display};

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, Error>;

    fn box_clone(&self) -> Box<dyn Callable>;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
