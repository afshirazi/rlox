use crate::{tokens::{Literal, Token, TokenType}, Lox};

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
    report: &'a dyn Fn(u32, u32, &str, &str) -> () // this feels stupid
}

impl <'a> Scanner <'a> {
    pub fn new(source: String, report_fn: &'a dyn Fn(u32, u32, &str, &str) -> ()) -> Self {
        Self {
            source: source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
            report: report_fn
        }
    }

    //TODO: can be abstracted into a separate type-state, e.g. ScannedTokens?
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn scan_tokens(&mut self) -> () {
        while (!self.is_at_end()) {
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

    fn scan_token(&mut self) -> () {
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
            '!' => self.add_token(
                if self.match_token('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                },
                None,
            ),
            '=' => self.add_token(
                if self.match_token('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                },
                None,
            ),
            '>' => self.add_token(
                if self.match_token('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                },
                None,
            ),
            '<' => self.add_token(
                if self.match_token('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                },
                None,
            ),
            '/' => {
                if self.match_token('/') {
                    while !self.is_at_end() && !(self.peek() == '\n') {
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
            _ => (self.report)(
                self.line,
                self.start,
                &self.source[self.start as usize..self.current as usize],
                "Unexpected character.",
            ),
        };
    }

    fn match_token(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // TODO: horrible, figure out how to use an iterator instead
    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current as usize];
        self.current += 1;
        c as char
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) -> () {
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

    fn string(&mut self) -> () {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            (self.report)(self.line, self.start, &self.source[self.start as usize..self.current as usize], "");
        }
    }
}
