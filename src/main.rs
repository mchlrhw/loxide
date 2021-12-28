use lox::token::{Token, TokenType};
use std::{env, io::Write, iter::Peekable, process, str::Chars};

fn report(line: usize, where_: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, where_, message);
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        let chars = source.chars().peekable();

        Self {
            source,
            chars,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    fn advance(&mut self) -> char {
        let c = self.chars.next().expect("source must not be at end");
        self.current += c.len_utf8();

        c
    }

    fn is_match(&mut self, expected: char) -> bool {
        if let Some(c) = self.chars.peek() {
            if *c == expected {
                self.advance();

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn add_token(&mut self, typ: TokenType) {
        let lexeme = &self.source[self.start..self.current];
        let token = Token::new(typ, lexeme, self.line);
        self.tokens.push(token);
    }

    fn string(&mut self) {
        while let Some(c) = self.chars.peek() {
            if *c == '"' {
                break;
            } else if *c == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string.");
            return;
        }

        self.advance(); // The closing ".

        // Trim the surrounding quotes.
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String(value.to_string()));
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let typ = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(typ);
            }
            '=' => {
                let typ = if self.is_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(typ);
            }
            '<' => {
                let typ = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(typ);
            }
            '>' => {
                let typ = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(typ);
            }
            '/' => {
                if self.is_match('/') {
                    while let Some(c) = self.chars.peek() {
                        if *c == '\n' {
                            break;
                        } else {
                            self.advance();
                        }
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {} // Ignore whitespace.
            '\n' => self.line += 1,
            '"' => self.string(),
            _ => error(self.line, "Unexpected character."),
        }
    }

    fn scan(&'a mut self) -> &'a [Token<'a>] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::Eof, "", self.line));

        &self.tokens
    }
}

fn run(source: &str) -> anyhow::Result<()> {
    let mut scanner = Scanner::new(source);

    for token in scanner.scan() {
        println!("{:?}", token);
    }

    Ok(())
}

fn run_prompt() -> anyhow::Result<()> {
    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.is_empty() {
            break;
        }

        run(&line)?;
    }

    Ok(())
}

fn run_file(_path: &str) -> anyhow::Result<()> {
    todo!()
}

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]),
        _ => {
            println!("Usage: lox [script]");
            process::exit(1);
        }
    }
}
