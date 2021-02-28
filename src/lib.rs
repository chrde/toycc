use codegen::Assembly;
use parser::{Function, Parser};
use std::{fmt, mem};
use thiserror::Error;
use tokenizer::Tokenizer;

mod ast;
mod codegen;
mod parser;
mod tokenizer;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("error: `{0}`")]
    Generic(String),
}

pub struct Error {
    kind: ErrorKind,
    span: Option<Span>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(span) = &self.span {
            // write!(f, "\n{}\n", span.code)?;
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

pub struct Span {
    pos: usize,
}

impl Span {
    pub fn new(pos: usize) -> Self {
        Self { pos }
    }
}

pub fn run<'a>(code: &'a str) -> Result<String, Error> {
    let t = Tokenizer::new(code);
    let tokens = t.run();
    let mut parser = Parser::new(code, tokens);

    let program = parser.run()?;
    eprintln!("{}\n{:#?}", code, program);
    let fun = Function::new(program, mem::take(&mut parser.locals));
    //assert parser is at EOF

    dbg!(&fun.locals);
    let mut assembly = Assembly::new(&fun);
    // Ok("".to_string())
    assembly.gen();

    Ok(assembly.finish())
}
