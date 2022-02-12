use crate::{
    callable::Callable,
    interpreter::{Error, Interpreter},
    value::Value,
};
use std::{
    any::Any,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug)]
pub struct Clock;

impl Clock {
    pub fn value() -> Value {
        Value::Callable(Box::new(Self))
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _: &mut Interpreter, _: Vec<Value>) -> Result<Value, Error> {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("we mustn't travel back in time")
            .as_secs_f64();

        Ok(Value::Number(secs))
    }

    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
