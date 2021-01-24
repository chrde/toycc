// NOTE(chrde): it does not support UTF-8
pub struct Tokenizer<'a> {
    code: &'a str,
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            pos: 0,
            tokens: vec![],
        }
    }

    pub fn run(mut self) -> Vec<Token> {
        while self.pos < self.code.len() {
            self.whitespace();
            self.digit();
            self.punctuator();
        }
        self.tokens.push(Token {
            start: self.pos,
            end: self.pos + 1,
            kind: TokenKind::Eof,
        });
        self.tokens
    }

    fn punctuator(&mut self) {
        match self.peek() {
            Some('+') => {
                self.advance();
                self.tokens.push(Token {
                    start: self.pos,
                    end: self.pos + 1,
                    kind: TokenKind::Punctuator(Punctuator::Plus),
                });
            }
            Some('-') => {
                self.advance();
                self.tokens.push(Token {
                    start: self.pos,
                    end: self.pos + 1,
                    kind: TokenKind::Punctuator(Punctuator::Minus),
                });
            }
            _ => {}
        }
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn rest(&self) -> &str {
        &self.code[self.pos..]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn token_from(&self, pos: usize) -> &str {
        &self.code[pos..self.pos]
    }

    fn whitespace(&mut self) {
        if self
            .peek()
            .map(|c| c.is_ascii_whitespace())
            .unwrap_or(false)
        {
            self.advance();
        }
    }

    fn digit(&mut self) {
        let start = self.pos;
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }
        if start != self.pos {
            let val: isize = self.token_from(start).parse().unwrap();
            let token = Token {
                start,
                end: self.pos,
                kind: TokenKind::Num(val),
            };
            self.tokens.push(token)
        }
    }
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    Reserved,
    Num(isize),
    Punctuator(Punctuator),
    Eof,
}

#[derive(Clone, Debug)]
pub enum Punctuator {
    Plus,
    Minus,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}
