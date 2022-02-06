use crate::{
    ast::{Expr, ExprKind, Stmt},
    interpreter::Interpreter,
    token::Token,
};
use std::collections::HashMap;

pub struct Resolver<'r> {
    interpreter: &'r mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl<'r> Resolver<'r> {
    pub fn new(interpreter: &'r mut Interpreter) -> Self {
        let scopes = vec![];

        Self {
            interpreter,
            scopes,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.lexeme()) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_expr(&mut self, expr: Expr) {
        let expr_clone = expr.clone();
        match expr.kind {
            ExprKind::Assign { name, value } => {
                self.resolve_expr(*value);
                self.resolve_local(expr_clone, &name);
            }
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            ExprKind::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(*callee);
                for expr in arguments {
                    self.resolve_expr(expr);
                }
            }
            ExprKind::Grouping(expr) => {
                self.resolve_expr(*expr);
            }
            ExprKind::Literal(_) => {}
            ExprKind::Logical { left, right, .. } => {
                self.resolve_expr(*left);
                self.resolve_expr(*right);
            }
            ExprKind::Unary { right, .. } => {
                self.resolve_expr(*right);
            }
            ExprKind::Variable(name) => {
                if let Some(scope) = self.scopes.last() {
                    if matches!(scope.get(name.lexeme()), Some(false)) {
                        crate::error(
                            name.line(),
                            "Can't read local variable in its own initializer.",
                        );
                    }
                }

                self.resolve_local(expr_clone, &name);
            }
        }
    }

    fn resolve_function(&mut self, params: Vec<Token>, body: Vec<Stmt>) {
        self.begin_scope();
        for param in params {
            self.declare(param.lexeme());
            self.define(param.lexeme());
        }
        self.resolve_statements(body);
        self.end_scope();
    }

    fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_statements(statements);
                self.end_scope();
            }
            Stmt::Expression(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Function { name, params, body } => {
                self.declare(name.lexeme());
                self.define(name.lexeme());
                self.resolve_function(params, body);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(*else_branch);
                }
            }
            Stmt::Print(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Return { value, .. } => {
                self.resolve_expr(value);
            }
            Stmt::Var { name, initializer } => {
                self.declare(&name);
                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer);
                }
                self.define(&name);
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*body);
            }
        }
    }

    pub fn resolve_statements(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }
}
