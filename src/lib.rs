use codegen::Assembly;
use parser::Parser;
use parser::{Node, NodeId};
use std::fmt;
use thiserror::Error;
use tokenizer::Tokenizer;

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

#[derive(Clone, Debug)]
pub struct NodeArena {
    nodes: Vec<Node>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn push(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        NodeId(id)
    }

    pub fn get(&self, node: NodeId) -> Node {
        self.nodes[node.0]
    }
}

pub fn run<'a>(code: &'a str) -> Result<String, Error> {
    let t = Tokenizer::new(code);
    let tokens = t.run();
    let mut arena = NodeArena::new();
    let mut parser = Parser::new(code, tokens, &mut arena);

    let expr = parser.expression(0)?;
    //assert parser is at EOF

    // dbg!(&arena);
    let mut assembly = Assembly::new();
    assembly.writeln("  .globl main");
    assembly.writeln("main:");
    assembly.gen_expr(expr, &arena);

    assembly.writeln("  ret");

    Ok(assembly.finish())
}
