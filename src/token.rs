use crate::value::Value;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    typ: TokenType,
    lexeme: String,
    value: Option<Value>,
    line: usize,
}

impl Token {
    pub fn new(typ: TokenType, lexeme: &str, value: Option<Value>, line: usize) -> Self {
        Self {
            typ,
            lexeme: lexeme.to_string(),
            value,
            line,
        }
    }

    pub fn typ(&self) -> &TokenType {
        &self.typ
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn value(&self) -> &Option<Value> {
        &self.value
    }

    pub fn line(&self) -> usize {
        self.line
    }
}
