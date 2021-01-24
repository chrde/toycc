use std::fmt::{self};
use thiserror::Error;
use tokenizer::*;

mod tokenizer;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("error: `{0}`")]
    Generic(String),
}

pub struct Error<'a> {
    kind: ErrorKind,
    span: Option<Span<'a>>,
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(span) = &self.span {
            write!(f, "\n{}\n", span.code)?;
            write!(f, "{:>width$}", "^", width = span.pos)?;
        }
        Ok(())
    }
}

impl From<fmt::Error> for ErrorKind {
    fn from(e: fmt::Error) -> Self {
        Self::Generic(e.to_string())
    }
}

pub struct Assembly {
    content: String,
}

pub struct Span<'a> {
    pos: usize,
    code: &'a str,
}

impl<'a> Span<'a> {
    pub fn new(code: &'a str, pos: usize) -> Self {
        Self { pos, code }
    }
}

impl fmt::Display for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl Assembly {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn write(&mut self, s: impl ToString) {
        self.content.push_str(&s.to_string())
    }
}

pub fn run<'a>(code: &'a str) -> Result<Assembly, Error<'a>> {
    let t = Tokenizer::new(code);
    let tokens = t.run();
    let mut parser = Parser::new(code, tokens);

    let mut assembly = Assembly::new();
    assembly.write("  .globl main\n");
    assembly.write("main:\n");

    assembly.write(format!("  mov ${}, %rax\n", parser.number()?));
    while let Some(t) = parser.next() {
        match t.kind {
            TokenKind::Eof => break,
            TokenKind::Punctuator(Punctuator::Plus) => {
                let number = parser.number()?;
                assembly.write(format!("  add ${}, %rax\n", number));
            }
            TokenKind::Punctuator(Punctuator::Minus) => {
                let number = parser.number()?;
                assembly.write(format!("  sub ${}, %rax\n", number));
            }
            _ => {
                return Err(Error {
                    kind: ErrorKind::Generic(format!("{:?}", t)),
                    span: None,
                })
            }
        }
    }
    assembly.write("  ret");

    Ok(assembly)
}

fn error_at(span: Option<Span>, msg: impl ToString) -> Error {
    Error {
        kind: ErrorKind::Generic(msg.to_string()),
        span,
    }
}

struct Parser<'a> {
    tokens: Vec<Token>,
    code: &'a str,
    cur: usize,
}

impl<'a> Parser<'a> {
    fn new(code: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            code,
            tokens,
            cur: 0,
        }
    }

    fn next(&mut self) -> Option<&Token> {
        let result = self.tokens.get(self.cur);
        self.cur += 1;
        result
    }

    fn span(&self) -> Option<Span<'a>> {
        if let Some(t) = self.tokens.get(self.cur) {
            Some(Span::new(self.code, t.end))
        } else {
            None
        }
    }

    fn number(&mut self) -> Result<isize, Error<'a>> {
        let token = self.tokens.get(self.cur).expect("No more tokens");
        match token.kind {
            TokenKind::Num(val) => {
                self.cur += 1;
                Ok(val)
            }
            _ => Err(error_at(self.span(), "expected number")),
        }
    }
}
