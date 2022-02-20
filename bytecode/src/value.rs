use std::{fmt, ops};

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (a, b) => panic!("can't add values of type {a:?} and {b:?}"),
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            (a, b) => panic!("can't sub values of type {a:?} and {b:?}"),
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            (a, b) => panic!("can't mul values of type {a:?} and {b:?}"),
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Self::Number(a / b),
            (a, b) => panic!("can't div values of type {a:?} and {b:?}"),
        }
    }
}

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(-n),
            a => panic!("can't neg values of type {a:?}"),
        }
    }
}
