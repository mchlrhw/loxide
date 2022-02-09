use crate::{
    ast::{Expr, ExprKind::*, Stmt},
    report,
    token::{Token, TokenType},
    value::Value,
};
use std::fmt;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    ParseError,
}

enum FunKind {
    Function,
    Method,
}

impl fmt::Display for FunKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Method => write!(f, "method"),
        }
    }
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
            Ok(Expr::new(Literal(Value::Boolean(false))))
        } else if self.is_match(&[TokenType::True]) {
            Ok(Expr::new(Literal(Value::Boolean(true))))
        } else if self.is_match(&[TokenType::Nil]) {
            Ok(Expr::new(Literal(Value::Nil)))
        } else if self.is_match(&[TokenType::Number, TokenType::String]) {
            Ok(Expr::new(Literal(
                self.previous()
                    .value()
                    .clone()
                    .expect("must have a literal"),
            )))
        } else if self.is_match(&[TokenType::Identifier]) {
            Ok(Expr::new(Variable(self.previous())))
        } else if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression")?;

            Ok(Expr::new(Grouping(Box::new(expr))))
        } else {
            self.error(self.peek(), "Expect expression.");

            Err(Error::ParseError)
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !self.is_match(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::new(Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }))
    }

    fn call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            if self.is_match(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.is_match(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expr = Expr::new(Get {
                    object: Box::new(expr),
                    name,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        let expr = if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = Box::new(self.unary()?);

            Expr::new(Unary { operator, right })
        } else {
            self.call()?
        };

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = Box::new(self.unary()?);

            expr = Expr::new(Binary {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.is_match(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = Box::new(self.factor()?);

            expr = Expr::new(Binary {
                left: Box::new(expr),
                operator,
                right,
            });
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
            let right = Box::new(self.term()?);

            expr = Expr::new(Binary {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = Box::new(self.comparison()?);

            expr = Expr::new(Binary {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.is_match(&[TokenType::And]) {
            let operator = self.previous();
            let right = Box::new(self.equality()?);

            expr = Expr::new(Logical {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;

        while self.is_match(&[TokenType::Or]) {
            let operator = self.previous();
            let right = Box::new(self.and()?);

            expr = Expr::new(Logical {
                left: Box::new(expr),
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;

        if self.is_match(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Variable(name) = expr.kind {
                return Ok(Expr::new(Assign {
                    name,
                    value: Box::new(value),
                }));
            }

            self.error(equals, "Invalid assignment target.");

            return Err(Error::ParseError);
        }

        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.is_match(&[TokenType::Semicolon]) {
            None
        } else if self.is_match(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let mut increment = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        let condition = match condition {
            None => Expr::new(Literal(Value::Boolean(true))),
            Some(expr) => expr,
        };

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
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

    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let keyword = self.previous();

        let mut value = Expr::new(Literal(Value::Nil));
        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
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
        let stmt = if self.is_match(&[TokenType::For]) {
            self.for_statement()?
        } else if self.is_match(&[TokenType::If]) {
            self.if_statement()?
        } else if self.is_match(&[TokenType::Print]) {
            self.print_statement()?
        } else if self.is_match(&[TokenType::Return]) {
            self.return_statement()?
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
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

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

    fn class_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Except class name.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function(FunKind::Method)?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class { name, methods })
    }

    fn function(&mut self, kind: FunKind) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {kind} name"))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {kind} name."),
        )?;

        let mut params = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters");
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
                if !self.is_match(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {kind} body."),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function { name, params, body })
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let res = if self.is_match(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.is_match(&[TokenType::Fun]) {
            self.function(FunKind::Function)
        } else if self.is_match(&[TokenType::Var]) {
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
