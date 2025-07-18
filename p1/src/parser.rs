use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    expr::{Assign, Binary, BinaryOp, Expr, Grouping, Literal, Unary, UnaryOp},
    stmt::{Stmt, Var},
    tokens::{self, Token, TokenType},
    LoxError,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: u32,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            current: 0,
        }
    }

    pub fn reset_tokens(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens;
        self.current = 0;
    }

    pub fn parse(&mut self) -> Vec<Result<Stmt, LoxError>> {
        let mut stmts = vec![];
        let top_level_env = Rc::new(RefCell::new(Environment::new()));
        while !self.is_at_end() {
            match self.declaration(top_level_env.clone()) {
                Ok(stmt) => stmts.push(Ok(stmt)),
                Err(e) => {
                    stmts.push(Err(e));
                    self.synchronize();
                }
            }
        }
        stmts
    }

    fn declaration(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Stmt, LoxError> {
        if self.adv_if_match(&[TokenType::Var]) {
            self.var_declaration(environment)
        } else {
            self.statement(environment)
        }
    }

    fn var_declaration(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Stmt, LoxError> {
        let name = self
            .try_consume(TokenType::Identifier, "Expect variable name")?
            .clone();

        let initializer = self
            .adv_if_match(&[TokenType::Equal])
            .then(|| self.expression(environment.clone()));

        self.try_consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        match initializer {
            Some(val) => Ok(Stmt::Var(Var::with_init(name, val?), environment.clone())),
            None => Ok(Stmt::Var(Var::new(name), environment.clone())),
        }
    }

    fn statement(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Stmt, LoxError> {
        if self.adv_if_match(&[TokenType::Print]) {
            self.print_statement(environment)
        } else if self.adv_if_match(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block(environment)?))
        } else {
            self.expr_statement(environment)
        }
    }

    fn print_statement(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Stmt, LoxError> {
        let expr = self.expression(environment)?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression")?;
        Ok(Stmt::Print(expr))
    }

    fn block(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Vec<Stmt>, LoxError> {
        let mut stmts = vec![];
        let new_env = Rc::new(RefCell::new(Environment::with_enclosing(
            environment.clone(),
        )));

        while !(self.check(&TokenType::RightBrace) || self.is_at_end()) {
            stmts.push(self.declaration(new_env.clone())?);
        }

        self.try_consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(stmts)
    }

    fn expr_statement(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Stmt, LoxError> {
        let expr = self.expression(environment)?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        self.assignment(environment)
    }

    fn assignment(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        let mut expr = self.equality(environment.clone())?;

        if self.adv_if_match(&[TokenType::Equal]) {
            let value = self.assignment(environment.clone())?;

            expr = match expr {
                Expr::Variable(name, _) => {
                    Expr::Assign(Assign::new(name, Box::new(value)), environment.clone())
                }
                _ => {
                    return Err(LoxError::new(
                        self.peek().line,
                        self.current,
                        "".to_owned(),
                        "Invalid assignment target".to_owned(),
                    ))
                }
            }
        }

        Ok(expr)
    }

    // comparison ( (== | !=) comparison ) *
    fn equality(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        let mut expr = self.comparison(environment.clone())?;
        while self.adv_if_match(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.previous().token_type {
                TokenType::BangEqual => BinaryOp::BangEqual,
                TokenType::EqualEqual => BinaryOp::EqualEqual,
                _ => unreachable!(), // unreachable guaranteed by check in adv_if_match
            };
            let right = self.comparison(environment.clone())?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Ok(expr)
    }

    fn comparison(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        let mut expr = self.term(environment.clone())?;

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
            let right = self.term(environment.clone())?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Ok(expr)
    }

    fn term(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        let mut expr = self.factor(environment.clone())?;

        while self.adv_if_match(&[TokenType::Minus, TokenType::Plus]) {
            let op = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Minus,
                TokenType::Plus => BinaryOp::Plus,
                _ => unreachable!(),
            };
            let right = self.factor(environment.clone())?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Ok(expr)
    }

    fn factor(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        let mut expr = self.unary(environment.clone())?;

        while self.adv_if_match(&[TokenType::Slash, TokenType::Star]) {
            let op = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Slash,
                TokenType::Star => BinaryOp::Star,
                _ => unreachable!(),
            };
            let right = self.unary(environment.clone())?;
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        Ok(expr)
    }

    fn unary(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        match self.adv_if_match(&[TokenType::Minus, TokenType::Bang]) {
            true => {
                let op = match self.previous().token_type {
                    TokenType::Minus => UnaryOp::Minus,
                    TokenType::Bang => UnaryOp::Bang,
                    _ => unreachable!(),
                };
                let expr = self.unary(environment)?;
                Ok(Expr::Unary(Unary::new(op, Box::new(expr))))
            }
            false => self.primary(environment),
        }
    }

    fn primary(&mut self, environment: Rc<RefCell<Environment>>) -> Result<Expr, LoxError> {
        if self.adv_if_match(&[TokenType::False]) {
            Ok(Expr::Literal(Literal::Boolean(false)))
        } else if self.adv_if_match(&[TokenType::True]) {
            Ok(Expr::Literal(Literal::Boolean(true)))
        } else if self.adv_if_match(&[TokenType::Nil]) {
            Ok(Expr::Literal(Literal::Nil))
        } else if self.adv_if_match(&[TokenType::Number, TokenType::String]) {
            let lit = match self.previous().literal.as_ref().unwrap() {
                tokens::Literal::Number(n) => Literal::Number(*n),
                tokens::Literal::String(s) => Literal::String(s.clone()),
                _ => unreachable!(),
            };
            Ok(Expr::Literal(lit))
        } else if self.adv_if_match(&[TokenType::LeftParen]) {
            let expr = self.expression(environment)?;
            self.try_consume(TokenType::RightParen, "')' Expected after expression")?;
            Ok(Expr::Grouping(Grouping::new(Box::new(expr))))
        } else if self.adv_if_match(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous().clone(), environment.clone())) //TODO: replace call to previous().clone() with reference maybe?
        } else {
            Err(LoxError::new(
                self.tokens[self.current as usize].line,
                0,
                self.tokens[self.current as usize].lexeme.clone(),
                "Unexpected character encountered".to_owned(),
            ))
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

    fn try_consume(&mut self, token_type: TokenType, err_msg: &str) -> Result<&Token, LoxError> {
        let peek = self.peek();
        let line = peek.line;
        let chars_in_line = peek.lexeme.clone(); // don't remember if this is actually the chars in line lol AND I DONT EFFIN CARE!!!!!!!!!
        if self.check(&token_type) {
            return Ok(self.advance());
        } else {
            Err(LoxError::new(line, 0, chars_in_line, err_msg.to_owned()))
        }
    }
}
