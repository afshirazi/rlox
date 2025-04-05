use crate::tokens::{Literal, Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,

    // lexeme-specific, start and current are offsets in the string
    // start: start of word being scanned,
    // current: current character of word being scanned
    // line: current line in file
    start: i32,
    current: i32,
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

    pub fn scanTokens(&self) -> () {
        while (!self.isAtEnd()) {
            self.start = self.current;

            self.scanToken();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_owned(),
            Literal::Identifier,
            self.line,
        ));
    }

    fn isAtEnd(&self) -> bool {
        self.current as usize >= self.source.len()
    }
}
