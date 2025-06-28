use crate::{
    expr::{Binary, BinaryOp, Expr, Grouping, Literal, Unary, UnaryOp},
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
    pub fn new(
        tokens: Vec<Token>,
        lox: &'a mut Lox,
        report: &'a dyn Fn(&mut Lox, u32, u32, &str, &str),
    ) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
            report,
        }
    }

    pub fn parse(&mut self) -> Expr {
        match self.expression() {
            Some(expr) => expr,
            None => todo!(), // TODO: for later chapter
        }
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
        } else {
            (self.report)(self.lox, self.tokens[self.current as usize].line, 0, &self.tokens[self.current as usize].lexeme, "Unexpected character encountered");
            None
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

    fn try_consume(&mut self, token_type: TokenType, arg: &str) -> Option<&Token> {
        let peek = self.peek();
        let line = peek.line;
        let chars_in_line = peek.lexeme.clone(); // don't remember if this is actually the chars in line lol AND I DONT EFFIN CARE!!!!!!!!!
        if self.check(&token_type) {
            return Some(self.advance());
        } else {
            (self.report)(self.lox, line, 0, &chars_in_line, arg);
            None
        }
    }
}
