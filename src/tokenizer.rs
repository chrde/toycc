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
            self.ident();
        }
        self.tokens.reverse();
        self.tokens
    }

    pub fn push_token(&mut self, kind: TokenKind) {
        let t = Token {
            start: self.pos,
            end: kind.len(),
            kind,
        };
        self.tokens.push(t);
    }

    fn punctuator(&mut self) {
        match self.peek() {
            Some(';') => {
                self.advance();
                self.push_token(TokenKind::Semicolon)
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.push_token(TokenKind::GreaterEqual);
                } else {
                    self.push_token(TokenKind::Greater);
                }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.push_token(TokenKind::LowerEqual);
                } else {
                    self.push_token(TokenKind::Lower);
                }
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.push_token(TokenKind::NotEqual);
                } else {
                    self.push_token(TokenKind::Not);
                }
            }
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.push_token(TokenKind::EqualEqual);
                } else {
                    self.push_token(TokenKind::Equal);
                }
            }
            Some('+') => {
                self.advance();
                self.push_token(TokenKind::Plus);
            }
            Some('-') => {
                self.advance();
                self.push_token(TokenKind::Minus)
            }
            Some('*') => {
                self.advance();
                self.push_token(TokenKind::Star);
            }
            Some('/') => {
                self.advance();
                self.push_token(TokenKind::Slash);
            }
            Some('}') => {
                self.advance();
                self.push_token(TokenKind::RightCurly);
            }
            Some('{') => {
                self.advance();
                self.push_token(TokenKind::LeftCurly);
            }
            Some(')') => {
                self.advance();
                self.push_token(TokenKind::RightParen);
            }
            Some('(') => {
                self.advance();
                self.push_token(TokenKind::LeftParen);
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

    fn ident(&mut self) {
        let start = self.pos;
        if self
            .peek()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        {
            self.advance();
            while self
                .peek()
                .map(|c| c.is_ascii_alphanumeric() || c == '_')
                .unwrap_or(false)
            {
                self.advance();
            }
        }
        if start != self.pos {
            let token = Token {
                start,
                end: self.pos,
                kind: identifier(self.token_from(start)),
            };
            self.tokens.push(token)
        }
    }

    fn digit(&mut self) {
        let start = self.pos;
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }
        if start != self.pos {
            let val: usize = self.token_from(start).parse().unwrap();
            let token = Token {
                start,
                end: self.pos,
                kind: TokenKind::Num(val),
            };
            self.tokens.push(token)
        }
    }
}

fn identifier(ident: &str) -> TokenKind {
    use TokenKind::*;
    match ident {
        "return" => Return,
        i => Ident(i.to_string()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Num(usize),
    Ident(String),
    LeftCurly,
    LeftParen,
    Star,
    Slash,
    Plus,
    Minus,
    RightParen,
    RightCurly,
    Equal,
    EqualEqual,
    Not,
    NotEqual,
    Lower,
    Greater,
    LowerEqual,
    GreaterEqual,
    Semicolon,
    Eof,

    // Reserved words,
    Return,
}

impl TokenKind {
    fn len(&self) -> usize {
        use TokenKind::*;
        match self {
            EqualEqual | LowerEqual | GreaterEqual => 2,
            Num(_) => unreachable!(),
            Eof => 0,
            _ => 1,
        }
    }
}

impl TokenKind {
    pub fn binary(&self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Star | Slash
                | Plus
                | Minus
                | EqualEqual
                | NotEqual
                | Lower
                | Greater
                | LowerEqual
                | GreaterEqual
        )
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}
