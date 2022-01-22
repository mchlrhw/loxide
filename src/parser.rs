use crate::token::{Literal, Token, TokenType};
use crate::{ast::Expr, report};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    ParseError,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Self {
            tokens: tokens.to_owned(),
            current: 0,
        }
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ() == &TokenType::Eof
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().typ() == &typ
        }
    }

    fn is_match(&mut self, types: &[TokenType]) -> bool {
        for typ in types {
            if self.check(*typ) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn error(&mut self, token: Token, message: &str) {
        if token.typ() == &TokenType::Eof {
            report(token.line(), " at end", message);
        } else {
            let lexeme = token.lexeme();
            report(token.line(), &format!(" at '{lexeme}'"), message);
        };
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(typ) {
            return Ok(self.advance());
        }

        self.error(self.peek(), message);

        Err(Error::ParseError)
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        if self.is_match(&[TokenType::False]) {
            Ok(Expr::Literal(Literal::Boolean(false)))
        } else if self.is_match(&[TokenType::True]) {
            Ok(Expr::Literal(Literal::Boolean(true)))
        } else if self.is_match(&[TokenType::Nil]) {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.is_match(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::Literal(
                self.previous()
                    .literal()
                    .clone()
                    .expect("must have a literal"),
            ))
        } else if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;

            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            self.error(self.peek(), "Expect expression.");

            Err(Error::ParseError)
        }
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.is_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.expression()
    }
}
