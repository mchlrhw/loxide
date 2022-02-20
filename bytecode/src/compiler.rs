use crate::{
    chunk::{Chunk, OpCode},
    scanner::{Scanner, Token, TokenType},
    value::Value,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::rc::Rc;

#[derive(TryFromPrimitive, IntoPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl std::ops::Add<u8> for Precedence {
    type Output = Self;

    fn add(self, other: u8) -> Self::Output {
        let byte: u8 = self.into();
        let res = Self::try_from(byte + other);

        res.expect("must be valid precedence")
    }
}

struct Parser<'p> {
    scanner: Scanner<'p>,
    previous: Option<Rc<Token>>,
    current: Option<Rc<Token>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'p> Parser<'p> {
    fn new(scanner: Scanner<'p>) -> Self {
        Parser {
            scanner,
            previous: None,
            current: None,
            had_error: false,
            panic_mode: false,
        }
    }

    fn previous(&self) -> Rc<Token> {
        self.previous.clone().expect("must have previous token")
    }

    fn current(&self) -> Rc<Token> {
        self.current.clone().expect("must have current token")
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        };
        self.panic_mode = true;

        let line = token.line;
        eprint!("[line {line}] Error");

        if matches!(token.typ, TokenType::Eof) {
            eprint!(" at end");
        } else if matches!(token.typ, TokenType::Error) {
            // Nothing.
        } else {
            let lexeme = &token.lexeme;
            eprint!(" at '{lexeme}'");
        }

        eprintln!(": {message}");
        self.had_error = true;
    }

    fn error(&mut self, message: &str) {
        let previous = self.previous();
        self.error_at(&previous, message);
    }

    fn error_at_current(&mut self, message: &str) {
        let current = self.current();
        self.error_at(&current, message)
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = Some(Rc::new(self.scanner.scan_token()));
            if !matches!(self.current().typ, TokenType::Error) {
                break;
            }

            self.error_at_current(&self.current().lexeme);
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if self.current().typ == typ {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn emit_byte<B: Into<u8>>(&self, chunk: &mut Chunk, byte: B) {
        chunk.write(byte, self.previous().line);
    }

    fn emit_bytes<B1: Into<u8>, B2: Into<u8>>(&self, chunk: &mut Chunk, byte1: B1, byte2: B2) {
        self.emit_byte(chunk, byte1);
        self.emit_byte(chunk, byte2);
    }

    fn emit_return(&self, chunk: &mut Chunk) {
        self.emit_byte(chunk, OpCode::Return)
    }

    fn end_compilation(&self, chunk: &mut Chunk) {
        self.emit_return(chunk);

        #[cfg(feature = "print_code")]
        if !self.had_error {
            chunk.disassemble("code").expect("opcodes must be valid");
        }
    }

    // XXX: It was seemingly impossible to move fn(&mut Self, &mut Chunk)
    //      to a type definition due to anonymous lifetimes, hence the
    //      allow below.
    #[allow(clippy::type_complexity)]
    fn get_rule(
        &self,
        operator_type: &TokenType,
    ) -> (
        Option<fn(&mut Self, &mut Chunk)>,
        Option<fn(&mut Self, &mut Chunk)>,
        Precedence,
    ) {
        match operator_type {
            TokenType::LeftParen => (Some(Self::grouping), None, Precedence::None),
            TokenType::RightParen => (None, None, Precedence::None),
            TokenType::LeftBrace => (None, None, Precedence::None),
            TokenType::RightBrace => (None, None, Precedence::None),
            TokenType::Comma => (None, None, Precedence::None),
            TokenType::Dot => (None, None, Precedence::None),
            TokenType::Minus => (Some(Self::unary), Some(Self::binary), Precedence::Term),
            TokenType::Plus => (None, Some(Self::binary), Precedence::Term),
            TokenType::Semicolon => (None, None, Precedence::None),
            TokenType::Slash => (None, Some(Self::binary), Precedence::Factor),
            TokenType::Star => (None, Some(Self::binary), Precedence::Factor),
            TokenType::Bang => (None, None, Precedence::None),
            TokenType::BangEqual => (None, None, Precedence::None),
            TokenType::Equal => (None, None, Precedence::None),
            TokenType::EqualEqual => (None, None, Precedence::None),
            TokenType::Greater => (None, None, Precedence::None),
            TokenType::GreaterEqual => (None, None, Precedence::None),
            TokenType::Less => (None, None, Precedence::None),
            TokenType::LessEqual => (None, None, Precedence::None),
            TokenType::Identifier => (None, None, Precedence::None),
            TokenType::String => (None, None, Precedence::None),
            TokenType::Number => (Some(Self::number), None, Precedence::None),
            TokenType::And => (None, None, Precedence::None),
            TokenType::Class => (None, None, Precedence::None),
            TokenType::Else => (None, None, Precedence::None),
            TokenType::False => (Some(Self::literal), None, Precedence::None),
            TokenType::For => (None, None, Precedence::None),
            TokenType::Fun => (None, None, Precedence::None),
            TokenType::If => (None, None, Precedence::None),
            TokenType::Nil => (Some(Self::literal), None, Precedence::None),
            TokenType::Or => (None, None, Precedence::None),
            TokenType::Print => (None, None, Precedence::None),
            TokenType::Return => (None, None, Precedence::None),
            TokenType::Super => (None, None, Precedence::None),
            TokenType::This => (None, None, Precedence::None),
            TokenType::True => (Some(Self::literal), None, Precedence::None),
            TokenType::Var => (None, None, Precedence::None),
            TokenType::While => (None, None, Precedence::None),
            TokenType::Error => (None, None, Precedence::None),
            TokenType::Eof => (None, None, Precedence::None),
        }
    }

    fn binary(&mut self, chunk: &mut Chunk) {
        let operator_type = &self.previous().typ;

        let rule = self.get_rule(operator_type);
        self.parse_precedence(chunk, rule.2 + 1);

        match operator_type {
            TokenType::Plus => self.emit_byte(chunk, OpCode::Add),
            TokenType::Minus => self.emit_byte(chunk, OpCode::Subtract),
            TokenType::Star => self.emit_byte(chunk, OpCode::Multiply),
            TokenType::Slash => self.emit_byte(chunk, OpCode::Divide),
            _ => {}
        };
    }

    fn literal(&mut self, chunk: &mut Chunk) {
        match self.previous().typ {
            TokenType::False => self.emit_byte(chunk, OpCode::False),
            TokenType::Nil => self.emit_byte(chunk, OpCode::Nil),
            TokenType::True => self.emit_byte(chunk, OpCode::True),
            _ => {}
        }
    }

    fn number(&mut self, chunk: &mut Chunk) {
        let value: f64 = self.previous().lexeme.parse().expect("must be a number");
        let constant = chunk.add_constant(Value::Number(value));
        self.emit_bytes(chunk, OpCode::Constant, constant);
    }

    fn unary(&mut self, chunk: &mut Chunk) {
        let operator_type = &self.previous().typ;

        // Compile the operand.
        self.parse_precedence(chunk, Precedence::Unary);

        if operator_type == &TokenType::Minus {
            self.emit_byte(chunk, OpCode::Negate);
        }
    }

    fn parse_precedence(&mut self, chunk: &mut Chunk, precedence: Precedence) {
        self.advance();
        if let Some(prefix_rule) = self.get_rule(&self.previous().typ).0 {
            prefix_rule(self, chunk);
        } else {
            self.error("Expect expression.");
            return;
        };

        while precedence <= self.get_rule(&self.current().typ).2 {
            self.advance();
            if let Some(infix_rule) = self.get_rule(&self.previous().typ).1 {
                infix_rule(self, chunk);
            }
        }
    }

    fn grouping(&mut self, chunk: &mut Chunk) {
        self.expression(chunk);
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn expression(&mut self, chunk: &mut Chunk) {
        self.parse_precedence(chunk, Precedence::Assignment);
    }
}

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner);

    parser.advance();
    parser.expression(chunk);
    parser.consume(TokenType::Eof, "Expect end of expression.");
    parser.end_compilation(chunk);

    !parser.had_error
}
