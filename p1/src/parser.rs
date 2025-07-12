use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    environment::Environment,
    expr::{Binary, BinaryOp, Expr, Grouping, Literal, Unary, UnaryOp},
    stmt::{Stmt, Var},
    tokens::{self, Token, TokenType},
    Lox,
};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: u32,
    lox: &'a mut Lox,
    environment: Rc<RefCell<Environment>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                stmts.push(stmt);
            } else {
                self.synchronize();
            }
        }
        stmts
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.adv_if_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Option<Stmt> {
        let name = self
            .try_consume(TokenType::Identifier, "Expect variable name")?
            .clone();

        let initializer = self
            .adv_if_match(&[TokenType::Equal])
            .then_some(self.expression())
            .flatten()?;

        self.try_consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Some(Stmt::Var(
            Var::with_init(name, initializer),
            self.environment.clone(),
        ))
    }

    fn statement(&mut self) -> Option<Stmt> {
        if self.adv_if_match(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression");
        Some(Stmt::Print(expr))
    }

    fn expr_statement(&mut self) -> Option<Stmt> {
        let expr = self.expression()?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression");
        Some(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Option<Expr> {
        Some(self.equality()?)
    }

    // comparison ( (== | !=) comparison ) *
    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison()?;
        while self.adv_if_match(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.previous().token_type {
                TokenType::BangEqual => BinaryOp::BangEqual,
                TokenType::EqualEqual => BinaryOp::EqualEqual,
                _ => unreachable!(), // unreachable guaranteed by check in adv_if_match
            };
            let right = self.comparison()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term()?;

        while self.adv_if_match(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let op = match self.previous().token_type {
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Some(expr)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor()?;

        while self.adv_if_match(&[TokenType::Minus, TokenType::Plus]) {
            let op = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Minus,
                TokenType::Plus => BinaryOp::Plus,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Some(expr)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary()?;

        while self.adv_if_match(&[TokenType::Slash, TokenType::Star]) {
            let op = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Slash,
                TokenType::Star => BinaryOp::Star,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Some(expr)
    }

    fn unary(&mut self) -> Option<Expr> {
        match self.adv_if_match(&[TokenType::Minus, TokenType::Bang]) {
            true => {
                let op = match self.previous().token_type {
                    TokenType::Minus => UnaryOp::Minus,
                    TokenType::Bang => UnaryOp::Bang,
                    _ => unreachable!(),
                };
                let expr = self.unary()?;
                Some(Expr::Unary(Unary::new(op, Box::new(expr))))
            }
            false => Some(self.primary()?),
        }
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.adv_if_match(&[TokenType::False]) {
            Some(Expr::Literal(Literal::Boolean(false)))
        } else if self.adv_if_match(&[TokenType::True]) {
            Some(Expr::Literal(Literal::Boolean(true)))
        } else if self.adv_if_match(&[TokenType::Nil]) {
            Some(Expr::Literal(Literal::Nil))
        } else if self.adv_if_match(&[TokenType::Number, TokenType::String]) {
            let lit = match self.previous().literal.as_ref().unwrap() {
                tokens::Literal::Number(n) => Literal::Number(*n),
                tokens::Literal::String(s) => Literal::String(s.clone()),
                _ => unreachable!(),
            };
            Some(Expr::Literal(lit))
        } else if self.adv_if_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.try_consume(TokenType::RightParen, "')' Expected after expression")?;
            Some(Expr::Grouping(Grouping::new(Box::new(expr))))
        } else if self.adv_if_match(&[TokenType::Identifier]) {
            Some(Expr::Identifier(
                self.previous().clone(),
                self.environment.clone(),
            )) //TODO: replace call to previous().clone() with reference maybe?
        } else {
            self.lox.report(
                self.tokens[self.current as usize].line,
                0,
                &self.tokens[self.current as usize].lexeme,
                "Unexpected character encountered",
            );
            None
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::For
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn adv_if_match(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn previous(&self) -> &Token {
        &self.tokens[(self.current - 1) as usize]
    }

    fn check(&self, t: &TokenType) -> bool {
        match self.is_at_end() {
            true => false,
            false => self.peek().token_type == *t,
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current as usize]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn try_consume(&mut self, token_type: TokenType, err_msg: &str) -> Option<&Token> {
        let peek = self.peek();
        let line = peek.line;
        let chars_in_line = peek.lexeme.clone(); // don't remember if this is actually the chars in line lol AND I DONT EFFIN CARE!!!!!!!!!
        if self.check(&token_type) {
            return Some(self.advance());
        } else {
            self.lox.report(line, 0, &chars_in_line, err_msg);
            None
        }
    }
}
