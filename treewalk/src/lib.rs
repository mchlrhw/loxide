pub mod ast;
pub mod callable;
pub mod class;
pub mod clock;
pub mod function;
pub mod interpreter;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod token;
pub mod value;

use token::{Token, TokenType};

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {line}] Error{where_}: {message}");
}

pub fn error_line(line: usize, message: &str) {
    report(line, "", message);
}

pub fn error_token(token: &Token, message: &str) {
    let line = token.line();
    if matches!(token.typ(), TokenType::Eof) {
        report(line, " at end", message);
    } else {
        let lexeme = token.lexeme();
        report(line, &format!(" at '{lexeme}'"), message);
    };
}
