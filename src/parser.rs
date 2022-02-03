use crate::token::{Literal, Token, TokenType};
use crate::{
    ast::{Expr, Stmt},
    report,
};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    ParseError,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Self {
            tokens: tokens.to_owned(),
            current: 0,
            errors: vec![],
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

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().typ() == &TokenType::Semicolon
                || [
                    TokenType::Class,
                    TokenType::For,
                    TokenType::Fun,
                    TokenType::If,
                    TokenType::Print,
                    TokenType::Return,
                    TokenType::Var,
                    TokenType::While,
                ]
                .contains(self.peek().typ())
            {
                break;
            }

            self.advance();
        }
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
        } else if self.is_match(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous()))
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

    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.is_match(&[TokenType::And]) {
            let operator = self.previous();
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;

        while self.is_match(&[TokenType::Or]) {
            let operator = self.previous();
            let right = Box::new(self.and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;

        if self.is_match(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            self.error(equals, "Invalid assignment target.");

            return Err(Error::ParseError);
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;
        if self.is_match(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Stmt::Print(value))
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn block(&mut self) -> Result<Vec<Box<Stmt>>, Error> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(Box::new(stmt));
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;

        Ok(Stmt::Expression(expr))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        let stmt = if self.is_match(&[TokenType::If]) {
            self.if_statement()?
        } else if self.is_match(&[TokenType::Print]) {
            self.print_statement()?
        } else if self.is_match(&[TokenType::While]) {
            self.while_statement()?
        } else if self.is_match(&[TokenType::LeftBrace]) {
            match self.block() {
                Ok(statements) => Stmt::Block(statements),
                Err(error) => return Err(error),
            }
        } else {
            self.expression_statement()?
        };

        Ok(stmt)
    }

    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .lexeme()
            .to_string();

        let mut initializer = None;
        if self.is_match(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.is_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Err(error) => {
                self.errors.push(error);
                self.synchronize();

                None
            }
            Ok(stmt) => Some(stmt),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = vec![];
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors[0].clone())
        }
    }
}
