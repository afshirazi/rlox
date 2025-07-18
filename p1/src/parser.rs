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
    environment: Rc<RefCell<Environment>>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            current: 0,
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn reset_tokens(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens;
        self.current = 0;
    }

    pub fn parse(&mut self) -> Vec<Result<Stmt, LoxError>> {
        let mut stmts = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => stmts.push(Ok(stmt)),
                Err(e) => {
                    stmts.push(Err(e));
                    self.synchronize();
                }
            }
        }
        stmts
    }

    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.adv_if_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self
            .try_consume(TokenType::Identifier, "Expect variable name")?
            .clone();

        let initializer = self
            .adv_if_match(&[TokenType::Equal])
            .then(|| self.expression());

        self.try_consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        match initializer {
            Some(val) => Ok(Stmt::Var(
                Var::with_init(name, val?),
                self.environment.clone(),
            )),
            None => Ok(Stmt::Var(Var::new(name), self.environment.clone())),
        }
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.adv_if_match(&[TokenType::Print]) {
            self.print_statement()
        } else if self.adv_if_match(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let expr = self.expression()?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression")?;
        Ok(Stmt::Print(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut stmts = vec![];
        let enclosing_env_ref = self.environment.clone();
        self.environment = Rc::new(RefCell::new(Environment::with_enclosing(enclosing_env_ref.clone())));

        while !(self.check(&TokenType::RightBrace) || self.is_at_end()) {
            stmts.push(self.declaration()?);
        }

        self.environment = enclosing_env_ref;
        self.try_consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(stmts)
    }

    fn expr_statement(&mut self) -> Result<Stmt, LoxError> {
        let expr = self.expression()?;
        self.try_consume(TokenType::Semicolon, "Expected semicolon after expression")?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;

        if self.adv_if_match(&[TokenType::Equal]) {
            let value = self.assignment()?;

            expr = match expr {
                Expr::Variable(name, _) => {
                    Expr::Assign(Assign::new(name, Box::new(value)), self.environment.clone())
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
    fn equality(&mut self) -> Result<Expr, LoxError> {
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

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
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

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
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

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
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

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        match self.adv_if_match(&[TokenType::Minus, TokenType::Bang]) {
            true => {
                let op = match self.previous().token_type {
                    TokenType::Minus => UnaryOp::Minus,
                    TokenType::Bang => UnaryOp::Bang,
                    _ => unreachable!(),
                };
                let expr = self.unary()?;
                Ok(Expr::Unary(Unary::new(op, Box::new(expr))))
            }
            false => self.primary(),
        }
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
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
            let expr = self.expression()?;
            self.try_consume(TokenType::RightParen, "')' Expected after expression")?;
            Ok(Expr::Grouping(Grouping::new(Box::new(expr))))
        } else if self.adv_if_match(&[TokenType::Identifier]) {
            Ok(Expr::Variable(
                self.previous().clone(),
                self.environment.clone(),
            )) //TODO: replace call to previous().clone() with reference maybe?
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
