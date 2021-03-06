use crate::{callable::Callable, class::LoxInstance};
use std::{
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Callable(Box<dyn Callable>),
    Instance(Rc<RefCell<LoxInstance>>),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Callable(c) => write!(f, "{c}"),
            Self::Instance(i) => write!(f, "{}", i.borrow()),
            Self::Nil => write!(f, "nil"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(s), Value::Boolean(o)) => s == o,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(s), Value::Number(o)) => s == o,
            (Value::String(s), Value::String(o)) => s == o,
            _ => false,
        }
    }
}
