pub mod ast;
pub mod parser;
pub mod scanner;
pub mod token;

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);
}

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}
