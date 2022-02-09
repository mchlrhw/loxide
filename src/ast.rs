use crate::{token::Token, value::Value};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum ExprKind {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This(Token),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
}

#[derive(Clone, Debug)]
pub struct Expr {
    id: Uuid,
    pub kind: ExprKind,
}

impl PartialEq for Expr {
    fn eq(&self, other: &Expr) -> bool {
        self.id == other.id
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        let id = Uuid::new_v4();

        Self { id, kind }
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        methods: Vec<Stmt>,
    },
    Expression(Expr),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
