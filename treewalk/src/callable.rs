use crate::{
    interpreter::{Error, Interpreter},
    value::Value,
};
use std::{
    any::Any,
    fmt::{Debug, Display},
};

pub trait Callable: Debug + Display {
    fn arity(&self) -> usize;

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, Error>;

    fn box_clone(&self) -> Box<dyn Callable>;

    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
