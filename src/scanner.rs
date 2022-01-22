use crate::token::{Literal, Token, TokenType};
use itertools::{Itertools, MultiPeek};
use once_cell::sync::Lazy;
use std::{collections::HashMap, str::Chars};

static KEYWORDS: Lazy<HashMap<&str, TokenType>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert("and", TokenType::And);
    m.insert("class", TokenType::Class);
    m.insert("else", TokenType::Else);
    m.insert("false", TokenType::False);
    m.insert("for", TokenType::For);
    m.insert("fun", TokenType::Fun);
    m.insert("if", TokenType::If);
    m.insert("nil", TokenType::Nil);
    m.insert("or", TokenType::Or);
    m.insert("print", TokenType::Print);
    m.insert("return", TokenType::Return);
    m.insert("super", TokenType::Super);
    m.insert("this", TokenType::This);
    m.insert("true", TokenType::True);
    m.insert("var", TokenType::Var);
    m.insert("while", TokenType::While);

    m
});

pub struct Scanner<'a> {
    source: &'a str,
    chars: MultiPeek<Chars<'a>>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars = source.chars().multipeek();

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

    fn add_token(&mut self, typ: TokenType, literal: Option<Literal>) {
        let lexeme = &self.source[self.start..self.current];
        let token = Token::new(typ, lexeme, literal, self.line);
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
            crate::error(self.line, "Unterminated string.");
            return;
        }

        self.advance(); // The closing ".

        // Trim the surrounding quotes.
        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String, Some(Literal::String(value.to_string())));
    }

    fn number(&mut self) {
        while let Some(c) = self.chars.peek() {
            if !c.is_digit(10) {
                self.chars.reset_peek();
                break;
            }
            self.advance();
        }

        if let Some('.') = self.chars.peek() {
            match self.chars.peek() {
                Some(c) if c.is_digit(10) => {
                    self.advance();

                    while let Some(c) = self.chars.peek() {
                        if !c.is_digit(10) {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => {}
            }
        }

        let lexeme = &self.source[self.start..self.current];
        let value = lexeme.parse().expect("must have a valid double");

        self.add_token(TokenType::Number, Some(Literal::Number(value)));
    }

    fn identifier(&mut self) {
        while let Some(c) = self.chars.peek() {
            if *c == '_' || c.is_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.source[self.start..self.current];
        let typ = KEYWORDS.get(lexeme).unwrap_or(&TokenType::Identifier);

        self.add_token(*typ, None);
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let typ = if self.is_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(typ, None);
            }
            '=' => {
                let typ = if self.is_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(typ, None);
            }
            '<' => {
                let typ = if self.is_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(typ, None);
            }
            '>' => {
                let typ = if self.is_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(typ, None);
            }
            '/' => {
                if self.is_match('/') {
                    while let Some(c) = self.chars.peek() {
                        if *c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            ' ' | '\r' | '\t' => {} // Ignore whitespace.
            '\n' => self.line += 1,
            '"' => self.string(),
            c if c.is_digit(10) => self.number(),
            c if c == '_' || c.is_alphabetic() => self.identifier(),
            _ => crate::error(self.line, "Unexpected character."),
        }
    }

    pub fn scan(&'a mut self) -> &'a [Token] {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "", None, self.line));

        &self.tokens
    }
}
