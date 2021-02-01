use codegen::Assembly;
use parser::{Function, Local, Node, NodeId};
use parser::{NodeKind, Parser};
use std::{fmt, mem};
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

#[derive(Clone, Debug, Default)]
pub struct NodeArena {
    nodes: Vec<Node>,
    statements: Vec<Node>,
    locals: Vec<Local>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_local(&mut self, name: String) -> NodeId {
        let id = match self.locals.iter().position(|x| x.name() == &name) {
            Some(x) => x,
            None => {
                self.locals.push(Local::new(name));
                self.locals.len() - 1
            }
        };
        NodeId(id)
    }

    pub fn into_function(&mut self) -> Function {
        Function::new(
            mem::take(&mut self.statements),
            mem::take(&mut self.nodes),
            mem::take(&mut self.locals),
        )
    }

    pub fn push(&mut self, node: Node) -> NodeId {
        match node.kind() {
            NodeKind::Statement | NodeKind::Return => {
                self.statements.push(node);
                NodeId(self.statements.len() - 1)
            }
            _ => {
                self.nodes.push(node);
                NodeId(self.nodes.len() - 1)
            }
        }
    }

    pub fn get(&self, node: NodeId) -> &Node {
        &self.nodes[node.0]
    }

    pub fn statements(&self) -> &[Node] {
        &self.statements
    }
}

pub fn run<'a>(code: &'a str) -> Result<String, Error> {
    let t = Tokenizer::new(code);
    let tokens = t.run();
    let mut arena = NodeArena::new();
    let mut parser = Parser::new(code, tokens, &mut arena);

    parser.run()?;
    //assert parser is at EOF

    // dbg!(&arena);
    let func = arena.into_function();
    let mut assembly = Assembly::new(&func);
    assembly.gen();

    Ok(assembly.finish())
}
