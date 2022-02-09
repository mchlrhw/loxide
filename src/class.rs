use crate::{
    callable::Callable,
    function::LoxFunction,
    interpreter::{Error, Interpreter},
    token::Token,
    value::Value,
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

#[derive(Clone, Debug)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, LoxFunction>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    pub fn value(self) -> Value {
        Value::Callable(Box::new(self))
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods.get(name).cloned()
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

impl Callable for LoxClass {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Value>) -> Result<Value, Error> {
        let value = LoxInstance::new(self).value();

        Ok(value)
    }

    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(class: &LoxClass) -> Self {
        Self {
            class: class.clone(),
            fields: HashMap::new(),
        }
    }

    pub fn value(self) -> Value {
        Value::Instance(Rc::new(RefCell::new(self)))
    }

    pub fn get(&self, name: &Token) -> Result<Value, Error> {
        if let Some(value) = self.fields.get(name.lexeme()) {
            Ok(value.clone())
        } else if let Some(method) = self.class.find_method(name.lexeme()) {
            Ok(method.value())
        } else {
            Err(Error::Runtime {
                message: format!("Undefined property '{}'.", name.lexeme()),
                line: name.line(),
            })
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme().to_string(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<inst {}>", self.class.name)
    }
}
