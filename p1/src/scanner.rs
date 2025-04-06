use crate::{report, tokens::{Literal, Token, TokenType}};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    // lexeme-specific, start and current are offsets in the string
    // start: start of word being scanned,
    // current: current character of word being scanned
    // line: current line in file
    start: u32,
    current: u32,
    line: u32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
        }
    }

    //TODO: can be abstracted into a separate type-state, e.g. ScannedTokens?
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn scanTokens(&mut self) -> () {
        while (!self.isAtEnd()) {
            self.start = self.current;

            self.scanToken();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_owned(),
            None,
            self.line,
        ));
    }

    // note: not important but possibly somewhere to use #[inline]
    fn isAtEnd(&self) -> bool {
        self.current as usize >= self.source.len()
    }
    
    fn scanToken(&mut self) -> () {
        let c = self.advance();

        match c {
            '(' => self.tokens.push(self.addToken(TokenType::LeftParen, None)),
            ')' => self.tokens.push(self.addToken(TokenType::RightParen, None)),
            '{' => self.tokens.push(self.addToken(TokenType::LeftBrace, None)),
            '}' => self.tokens.push(self.addToken(TokenType::RightBrace, None)),
            ',' => self.tokens.push(self.addToken(TokenType::Comma, None)),
            '.' => self.tokens.push(self.addToken(TokenType::Dot, None)),
            '-' => self.tokens.push(self.addToken(TokenType::Minus, None)),
            '+' => self.tokens.push(self.addToken(TokenType::Plus, None)),
            ';' => self.tokens.push(self.addToken(TokenType::Semicolon, None)),
            '*' => self.tokens.push(self.addToken(TokenType::Star, None)),
            _ => report(self.line, self.start, &self.source[self.start as usize..self.current as usize], "Unexpected character."),
        };
    }
    
    // TODO: horrible, figure out how to use an iterator instead
    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current as usize];
        self.current += 1;
        c as char
    }
    
    fn addToken(&self, token_type: TokenType, literal: Option<Literal>) -> Token {
        // note: will panic if start..current doesn't encompass a valid character sequence
        let text = &self.source[self.start as usize..self.current as usize]; // no off by one, advance() increments current by 1
        Token::new(token_type, text.to_owned(), literal, self.line)
    }
}


