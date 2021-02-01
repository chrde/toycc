use crate::tokenizer::TokenKind;
use crate::{tokenizer::Token, Error, NodeArena};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    code: &'a str,
    arena: &'a mut NodeArena,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, tokens: Vec<Token>, arena: &'a mut NodeArena) -> Self {
        Self {
            code,
            tokens,
            arena,
        }
    }

    fn eof(&self) -> Token {
        Token {
            start: self.code.len(),
            end: self.code.len() + 1,
            kind: TokenKind::Eof,
        }
    }

    fn peek(&self) -> Token {
        self.tokens.last().cloned().unwrap_or_else(|| self.eof())
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or_else(|| self.eof())
    }

    fn consume(&mut self, kind: TokenKind) -> Token {
        let next = self.next();
        assert_eq!(kind, next.kind);
        next
    }

    fn consume_binary(&mut self) -> Token {
        let next = self.next();
        assert!(next.kind.binary());
        next
    }

    fn binary(&mut self, lhs: NodeId, min_bp: u8) -> Result<NodeId, Error> {
        let kind = match self.consume_binary().kind {
            TokenKind::Lower => NodeKind::LowerCmp,
            TokenKind::LowerEqual => NodeKind::LowerEqCmp,
            TokenKind::Greater => NodeKind::GreaterCmp,
            TokenKind::GreaterEqual => NodeKind::GreaterEqCmp,
            TokenKind::NotEqual => NodeKind::NeqCmp,
            TokenKind::EqualEqual => NodeKind::EqCmp,
            TokenKind::Plus => NodeKind::Add,
            TokenKind::Minus => NodeKind::Sub,
            TokenKind::Slash => NodeKind::Div,
            TokenKind::Star => NodeKind::Mul,
            k => unimplemented!("{:?}", k),
        };
        let rhs = self.expression(min_bp)?;
        Ok(self.arena.push(Node::binary(kind, lhs, rhs)))
    }

    // parens
    fn grouping(&mut self) -> Result<NodeId, Error> {
        self.consume(TokenKind::LeftParen);
        let expr = self.expression(0)?;
        self.consume(TokenKind::RightParen);
        Ok(expr)
    }

    fn primary(&mut self) -> Result<NodeId, Error> {
        let t = self.next();
        match t.kind {
            TokenKind::Num(val) => Ok(self.arena.push(Node::num(val))),
            TokenKind::Ident(val) => {
                let id = self.arena.push_local(val);
                Ok(self.arena.push(Node::ident(id)))
            }
            k => unimplemented!("{:?}", k),
        }
    }

    // unary
    fn unary(&mut self) -> Result<NodeId, Error> {
        let t = self.next();
        let ((), r_bp) = prefix_binding_power(&t.kind);
        let lhs = self.expression(r_bp)?;
        let id = match &t.kind {
            TokenKind::Plus => lhs,
            TokenKind::Minus => self.arena.push(Node::unary(NodeKind::Neg, lhs)),
            k => unimplemented!("{:?}", k),
        };
        Ok(id)
    }

    pub fn expression(&mut self, min_bp: u8) -> Result<NodeId, Error> {
        use crate::tokenizer::TokenKind::*;
        let mut lhs = match self.peek().kind {
            Num(_) | Ident(_) => self.primary()?,
            LeftParen => self.grouping()?,
            Plus | Minus => self.unary()?,
            k => unimplemented!("{:?}", k),
        };

        loop {
            let next = self.peek().kind;
            match next {
                Eof | RightParen | Semicolon | Equal => break,
                Star | Slash | Plus | Minus | EqualEqual | NotEqual | Lower | Greater
                | LowerEqual | GreaterEqual => {}
                k => unimplemented!("{:?}", k),
            };
            if let Some((l_bp, r_bp)) = infix_binding_power(&next) {
                if l_bp < min_bp {
                    break;
                }
                lhs = self.binary(lhs, r_bp)?;
            }
        }

        Ok(lhs)
    }

    fn assignment(&mut self) -> Result<NodeId, Error> {
        let mut lhs = self.expression(0)?;
        while self.peek().kind == TokenKind::Equal {
            self.consume(TokenKind::Equal);
            let node = Node::binary(NodeKind::Assignment, lhs, self.assignment()?);
            lhs = self.arena.push(node);
        }
        Ok(lhs)
    }

    // TODO(chrde): rename this
    fn expr(&mut self) -> Result<NodeId, Error> {
        let id = self.assignment()?;
        Ok(id)
    }

    fn statement(&mut self) -> Result<NodeId, Error> {
        match self.peek().kind {
            TokenKind::Return => {
                self.consume(TokenKind::Return);
                let lhs = self.expr()?;
                self.consume(TokenKind::Semicolon);
                let node = Node::unary(NodeKind::Return, lhs);
                Ok(self.arena.push(node))
            }
            _ => {
                let lhs = self.expr()?;
                self.consume(TokenKind::Semicolon);
                let node = Node::unary(NodeKind::Statement, lhs);
                Ok(self.arena.push(node))
            }
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while self.peek().kind != TokenKind::Eof {
            let _ = self.statement()?;
        }

        Ok(())
    }
}

// https://en.cppreference.com/w/c/language/operator_precedence
const TOTAL: u8 = 15;
const PREC_UNARY: u8 = TOTAL - 2;
const PREC_FACTOR: u8 = TOTAL - 3;
const PREC_TERM: u8 = TOTAL - 4;
const PREC_RELATIONAL: u8 = TOTAL - 6;
const PREC_ASSIGNMENT: u8 = TOTAL - 14;

fn prefix_binding_power(t: &TokenKind) -> ((), u8) {
    match t {
        TokenKind::Plus | TokenKind::Minus => ((), PREC_UNARY),
        t => unimplemented!("{:?}", t),
    }
    // Some(res)
}

fn infix_binding_power(t: &TokenKind) -> Option<(u8, u8)> {
    use TokenKind::*;
    let res = match t {
        Equal => (PREC_ASSIGNMENT + 1, PREC_ASSIGNMENT),
        Plus | Minus => (PREC_TERM, PREC_TERM + 1),
        Star | Slash => (PREC_FACTOR, PREC_FACTOR + 1),
        EqualEqual | NotEqual | Lower | Greater | LowerEqual | GreaterEqual => {
            (PREC_RELATIONAL, PREC_RELATIONAL + 1)
        }
        _ => return None,
    };
    Some(res)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Statement,
    Assignment,

    Return,

    // expr
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    EqCmp,
    NeqCmp,
    LowerCmp,
    LowerEqCmp,
    GreaterCmp,
    GreaterEqCmp,

    // identifiers
    Num(usize),
    Ident(NodeId),
}

#[derive(Clone, Debug)]
pub struct Local {
    name: String,
    offset: usize,
}

impl Local {
    pub fn new(name: String) -> Self {
        Self { name, offset: 0 }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl Eq for Local {}

impl PartialEq for Local {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    body: Vec<Node>,
    nodes: Vec<Node>,
    locals: Vec<Local>,
    stack_size: usize,
}

impl Function {
    pub fn new(body: Vec<Node>, nodes: Vec<Node>, locals: Vec<Local>) -> Self {
        let mut result = Self {
            body,
            nodes,
            locals,
            stack_size: 0,
        };
        result.assign_locals_offsets();
        result
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn body(&self) -> &[Node] {
        &self.body
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn local(&self, id: NodeId) -> &Local {
        &self.locals[id.0]
    }

    fn assign_locals_offsets(&mut self) {
        let mut total = 0;
        for l in self.locals.iter_mut() {
            total += 8;
            l.offset = total;
        }
        self.stack_size = Self::align_stack_size(total, 16)
    }

    fn align_stack_size(size: usize, base_align: usize) -> usize {
        base_align * (size + base_align - 1) / base_align
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    kind: NodeKind,
    lhs: NodeId,
    rhs: NodeId,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NodeId(pub usize);

const NIL: NodeId = NodeId(0);

impl Node {
    pub fn new(kind: NodeKind, lhs: NodeId, rhs: NodeId) -> Self {
        Self { kind, lhs, rhs }
    }

    pub fn lhs(&self) -> NodeId {
        self.lhs
    }

    pub fn rhs(&self) -> NodeId {
        self.rhs
    }

    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub fn ident(ident_id: NodeId) -> Self {
        Self {
            kind: NodeKind::Ident(ident_id),
            lhs: NIL,
            rhs: NIL,
        }
    }

    pub fn num(val: usize) -> Self {
        Self {
            kind: NodeKind::Num(val),
            lhs: NIL,
            rhs: NIL,
        }
    }

    pub fn unary(kind: NodeKind, lhs: NodeId) -> Self {
        Self {
            kind,
            lhs,
            rhs: NIL,
        }
    }

    pub fn binary(kind: NodeKind, lhs: NodeId, rhs: NodeId) -> Self {
        Self { kind, lhs, rhs }
    }
}
