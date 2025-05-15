use crate::{
    expr::{Binary, BinaryOp, Expr, Literal, Unary, UnaryOp},
    tokens::{self, Token, TokenType},
    Lox,
};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: u32,
    lox: &'a mut Lox,
    report: &'a dyn Fn(&mut Lox, u32, u32, &str, &str), // this still feels stupid
}

impl<'a> Parser<'a> {
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    // comparison ( (== | !=) comparison ) *
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.adv_if_match(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let op = match self.previous().token_type {
                TokenType::BangEqual => BinaryOp::BangEqual,
                TokenType::EqualEqual => BinaryOp::EqualEqual,
                _ => unreachable!(), // unreachable guaranteed by check in adv_if_match
            };
            let right = self.comparison();
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

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
            let right = self.term();
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.adv_if_match(&[TokenType::Minus, TokenType::Plus]) {
            let op = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Minus,
                TokenType::Plus => BinaryOp::Plus,
                _ => unreachable!(),
            };
            let right = self.factor();
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.adv_if_match(&[TokenType::Slash, TokenType::Star]) {
            let op = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Slash,
                TokenType::Star => BinaryOp::Star,
                _ => unreachable!(),
            };
            let right = self.unary();
            expr = Expr::Binary(Binary::new(Box::new(expr), op, Box::new(right)));
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        match self.adv_if_match(&[TokenType::Minus, TokenType::Bang]) {
            true => {
                let expr = self.unary();
                let op = match self.previous().token_type {
                    TokenType::Minus => UnaryOp::Minus,
                    TokenType::Bang => UnaryOp::Bang,
                    _ => unreachable!(),
                };
                Expr::Unary(Unary::new(op, Box::new(expr)))
            }
            false => self.primary(),
        }
    }

    fn primary(&mut self) -> Expr {
        if self.adv_if_match(&[TokenType::False]) {
            Expr::Literal(Literal::False)
        } else if self.adv_if_match(&[TokenType::True]) {
            Expr::Literal(Literal::True)
        } else if self.adv_if_match(&[TokenType::Nil]) {
            Expr::Literal(Literal::Nil)
        } else if self.adv_if_match(&[TokenType::Number, TokenType::String]) {
            let lit = match self.previous().literal.as_ref().unwrap() {
                tokens::Literal::Number(n) => Literal::Number(*n),
                tokens::Literal::String(s) => Literal::String(s.clone()),
                _ => unreachable!(),
            };
            Expr::Literal(lit)
        } else if self.adv_if_match(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume_or_panic(TokenType::RightParen, "')' Expected after expression.");
            expr
        } else {
            panic!("wtf!!!");
        }
    }

    fn adv_if_match(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if (self.check(t)) {
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

    fn consume_or_panic(&mut self, token_type: TokenType, arg: &str) -> &Token {
        let peek = self.peek();
        let line = peek.line;
        let chars_in_line = peek.lexeme.clone(); // don't remember if this is actually the chars in line lol
        if self.check(&token_type) {
            return self.advance();
        } else {
            (self.report) (self.lox,
                line,
                0,
                &chars_in_line,
                arg
            );
            panic!()
        }
    }
}
