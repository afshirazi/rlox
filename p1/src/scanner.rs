use crate::{
    tokens::{Literal, Token, TokenType},
    Lox,
};

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,

    // lexeme-specific, start and current are offsets in the string
    // start: start of word being scanned,
    // current: current character of word being scanned
    // line: current line in file
    start: u32,
    current: u32,
    line: u32,
    lox: &'a mut Lox,
    report: &'a dyn Fn(&mut Lox, u32, u32, &str, &str), // this feels stupid
}

impl<'a> Scanner<'a> {
    pub fn new(
        source: String,
        lox: &'a mut Lox,
        report_fn: &'a dyn Fn(&mut Lox, u32, u32, &str, &str),
    ) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
            lox,
            report: report_fn,
        }
    }

    //TODO: can be abstracted into a separate type-state, e.g. ScannedTokens?
    // consumes, scanner presumably won't be used after getting the tokens
    pub fn tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;

            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".to_owned(), None, self.line));
    }

    // note: not important but possibly somewhere to use #[inline]
    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => {
                let token_type = if self.match_token('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type, None)
            }
            '=' => {
                let token_type = if self.match_token('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type, None)
            }
            '>' => {
                let token_type = if self.match_token('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type, None)
            }
            '<' => {
                let token_type = if self.match_token('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type, None)
            }
            '/' => {
                if self.match_token('/') {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
            }
            '\n' => self.line += 1,
            '\t' => (),
            ' ' => (),
            '\r' => (),
            '"' => self.string(),
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_ascii_alphabetic() {
                    self.identifier();
                } else {
                    (self.report)(
                        self.lox,
                        self.line,
                        self.start,
                        &self.source[self.start as usize..self.current as usize],
                        "Unexpected character",
                    );
                }
            }
        };
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current as usize];
        self.current += 1;
        c as char
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        // note: will panic if start..current doesn't encompass a valid character sequence
        let text = &self.source[self.start as usize..self.current as usize]; // no off by one, advance() increments current by 1
        let token = Token::new(token_type, text.to_owned(), literal, self.line);
        self.tokens.push(token);
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current as usize] as char
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            (self.report)(
                self.lox,
                self.line,
                self.start,
                &self.source[self.start as usize..self.current as usize],
                "Unterminated string",
            );
        }

        self.advance(); // consume closing "

        let string = self.source[(self.start + 1) as usize..(self.current - 1) as usize].to_owned();

        self.add_token(TokenType::String, Some(Literal::String(string)));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let num = match self.source[self.start as usize..self.current as usize].parse::<f64>() {
            Ok(num) => num,
            Err(_) => {
                (self.report)(
                    self.lox,
                    self.line,
                    self.start,
                    &self.source[self.start as usize..self.current as usize],
                    "Expected number but failed to parse",
                );
                return;
            }
        };
        self.add_token(TokenType::Number, Some(Literal::Number(num)));
    }

    fn peek_next(&self) -> char {
        if (self.current + 1) as usize >= self.source.len() {
            return '\0';
        }
        self.source.as_bytes()[(self.current + 1) as usize] as char
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() {
            self.advance();
        }

        match &self.source[self.start as usize..self.current as usize] {
            "and" => self.add_token(TokenType::And, None),
            "class" => self.add_token(TokenType::Class, None),
            "else" => self.add_token(TokenType::Else, None),
            "false" => self.add_token(TokenType::False, None),
            "for" => self.add_token(TokenType::For, None),
            "fun" => self.add_token(TokenType::Fun, None),
            "if" => self.add_token(TokenType::If, None),
            "nil" => self.add_token(TokenType::Nil, None),
            "or" => self.add_token(TokenType::Or, None),
            "print" => self.add_token(TokenType::Print, None),
            "return" => self.add_token(TokenType::Return, None),
            "super" => self.add_token(TokenType::Super, None),
            "this" => self.add_token(TokenType::This, None),
            "true" => self.add_token(TokenType::True, None),
            "var" => self.add_token(TokenType::Var, None),
            "while" => self.add_token(TokenType::While, None),
            user_literal => self.add_token(
                TokenType::Identifier,
                Some(Literal::Identifier(user_literal.to_owned())),
            ),
        }
    }
}
