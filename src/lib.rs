pub mod ast;
pub mod callable;
pub mod clock;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod token;
pub mod value;

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {line}] Error{where_}: {message}");
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}
