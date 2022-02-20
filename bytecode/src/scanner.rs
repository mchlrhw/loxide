use itertools::{peek_nth, PeekNth};
use std::{fmt, str::Chars};

#[derive(Debug, PartialEq, Eq)]
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

    Error,
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

fn identifier_type(lexeme: &str) -> TokenType {
    match lexeme {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "else" => TokenType::Else,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => TokenType::Identifier,
    }
}

#[derive(Debug)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    fn new(typ: TokenType, lexeme: String, line: usize) -> Self {
        Self { typ, lexeme, line }
    }

    fn error(message: &str, line: usize) -> Self {
        Self {
            typ: TokenType::Error,
            lexeme: message.to_string(),
            line,
        }
    }
}

pub struct Scanner<'s> {
    source: PeekNth<Chars<'s>>,
    line: usize,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source: peek_nth(source.chars()),
            line: 1,
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.source.peek().is_none()
    }

    fn advance(&mut self) -> char {
        self.source.next().expect("we shouldn't be at the end")
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.source.peek() {
                match c {
                    ' ' | '\r' | '\t' => {
                        self.advance();
                        continue;
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                        continue;
                    }
                    '/' => {
                        if matches!(self.source.peek_nth(2), Some('/')) {
                            while !matches!(self.source.peek(), Some('\n')) && !self.is_at_end() {
                                self.advance();
                            }
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            break;
        }
    }

    fn next_is_match(&mut self, expected: char) -> bool {
        if let Some(c) = self.source.peek() {
            *c == expected
        } else {
            false
        }
    }

    fn scan_two_char_token(
        &mut self,
        mut lexeme: String,
        c: char,
        then: TokenType,
        els: TokenType,
    ) -> Token {
        if self.next_is_match(c) {
            lexeme.push(self.advance());

            Token::new(then, lexeme, self.line)
        } else {
            Token::new(els, lexeme, self.line)
        }
    }

    fn scan_string(&mut self) -> Token {
        let mut lexeme = String::new();
        while !self.next_is_match('"') && !self.is_at_end() {
            let c = self.advance();

            if c == '\n' {
                self.line += 1;
            }

            lexeme.push(c);
        }

        if self.is_at_end() {
            Token::error("Unterminated string.", self.line)
        } else {
            // The closing quote.
            self.advance();

            Token::new(TokenType::String, lexeme, self.line)
        }
    }

    fn scan_number(&mut self, mut lexeme: String) -> Token {
        while matches!(self.source.peek(), Some(c) if c.is_digit(10)) {
            lexeme.push(self.advance());
        }

        if self.next_is_match('.') && matches!(self.source.peek_nth(2), Some(c) if c.is_digit(10)) {
            lexeme.push(self.advance());
            while matches!(self.source.peek(), Some(c) if c.is_digit(10)) {
                lexeme.push(self.advance());
            }
        }

        Token::new(TokenType::Number, lexeme, self.line)
    }

    fn scan_identifier(&mut self, mut lexeme: String) -> Token {
        while matches!(self.source.peek(), Some(c) if c.is_alphabetic() || c.is_digit(10) || *c == '_')
        {
            lexeme.push(self.advance());
        }

        Token::new(identifier_type(&lexeme), lexeme, self.line)
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.is_at_end() {
            return Token::new(TokenType::Eof, String::default(), self.line);
        }

        let c = self.advance();

        if c.is_alphabetic() || c == '_' {
            return self.scan_identifier(c.to_string());
        }
        if c.is_digit(10) {
            return self.scan_number(c.to_string());
        }

        match c {
            '(' => Token::new(TokenType::LeftParen, c.to_string(), self.line),
            ')' => Token::new(TokenType::RightParen, c.to_string(), self.line),
            '{' => Token::new(TokenType::LeftBrace, c.to_string(), self.line),
            '}' => Token::new(TokenType::RightBrace, c.to_string(), self.line),
            ';' => Token::new(TokenType::Semicolon, c.to_string(), self.line),
            ',' => Token::new(TokenType::Comma, c.to_string(), self.line),
            '.' => Token::new(TokenType::Dot, c.to_string(), self.line),
            '-' => Token::new(TokenType::Minus, c.to_string(), self.line),
            '+' => Token::new(TokenType::Plus, c.to_string(), self.line),
            '/' => Token::new(TokenType::Slash, c.to_string(), self.line),
            '*' => Token::new(TokenType::Star, c.to_string(), self.line),
            '!' => {
                let lexeme = c.to_string();
                self.scan_two_char_token(lexeme, '=', TokenType::BangEqual, TokenType::Bang)
            }
            '=' => {
                let lexeme = c.to_string();
                self.scan_two_char_token(lexeme, '=', TokenType::EqualEqual, TokenType::Equal)
            }
            '<' => {
                let lexeme = c.to_string();
                self.scan_two_char_token(lexeme, '=', TokenType::LessEqual, TokenType::Less)
            }
            '>' => {
                let lexeme = c.to_string();
                self.scan_two_char_token(lexeme, '=', TokenType::GreaterEqual, TokenType::Greater)
            }
            '"' => self.scan_string(),
            _ => Token::error("Unexpected character.", self.line),
        }
    }
}
