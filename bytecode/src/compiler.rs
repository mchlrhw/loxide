use crate::scanner::{Scanner, Token, TokenType};

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);

    let mut prev_line = 0;
    loop {
        let Token { typ, lexeme, line } = scanner.scan_token();
        if line != prev_line {
            print!("{line:4}");
            prev_line = line;
        } else {
            print!("   | ");
        }
        println!("{typ:10} '{lexeme}'");

        if matches!(typ, TokenType::Eof) {
            break;
        }
    }
}
