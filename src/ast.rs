use crate::token::{Literal, Token};

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Variable(Token),
    Assign { name: Token, value: Box<Expr> },
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: String,
        initializer: Option<Expr>,
    },
}
