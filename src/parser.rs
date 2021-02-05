use crate::tokenizer::TokenKind;
use crate::{tokenizer::Token, Error};

pub struct Parser<'a> {
    code: &'a str,
    pub locals: Vec<Local>,
    tokens: Vec<Token>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalId(pub usize);

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            code,
            locals: vec![],
            tokens,
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

    fn push_local(&mut self, name: String) -> LocalId {
        let id = match self.locals.iter().position(|x| x.name() == &name) {
            Some(x) => x,
            None => {
                self.locals.push(Local::new(name));
                self.locals.len() - 1
            }
        };
        LocalId(id)
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

    fn binary(&mut self, lhs: ExprStmt, min_bp: u8) -> Result<ExprStmt, Error> {
        let op = match self.consume_binary().kind {
            TokenKind::Lower => BinOp::LowerCmp,
            TokenKind::LowerEqual => BinOp::LowerEqCmp,
            TokenKind::Greater => BinOp::GreaterCmp,
            TokenKind::GreaterEqual => BinOp::GreaterEqCmp,
            TokenKind::NotEqual => BinOp::NeqCmp,
            TokenKind::EqualEqual => BinOp::EqCmp,
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Star => BinOp::Mul,
            k => unimplemented!("{:?}", k),
        };
        let rhs = self.expression(min_bp)?;
        Ok(ExprStmt::Binary(BinaryNode {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    }

    // parens
    fn grouping(&mut self) -> Result<ExprStmt, Error> {
        self.consume(TokenKind::LeftParen);
        let expr = self.expression(0)?;
        self.consume(TokenKind::RightParen);
        Ok(expr)
    }

    fn primary(&mut self) -> Result<ExprStmt, Error> {
        let t = self.next();
        match t.kind {
            TokenKind::Num(val) => Ok(ExprStmt::Primary(Unary::Num(val))),
            TokenKind::Ident(name) => Ok(ExprStmt::Primary(Unary::Ident(self.push_local(name)))),
            k => unimplemented!("{:?}", k),
        }
    }

    // unary
    fn unary(&mut self) -> Result<UnaryNode, Error> {
        let t = self.next();
        let ((), r_bp) = prefix_binding_power(&t.kind);
        let lhs = self.expression(r_bp)?;
        let id = match &t.kind {
            TokenKind::Plus => UnaryNode {
                op: UnaryOp::NoOp,
                lhs: Unary::Expr(Box::new(lhs)),
            },
            TokenKind::Minus => UnaryNode {
                op: UnaryOp::Neg,
                lhs: Unary::Expr(Box::new(lhs)),
            },
            k => unimplemented!("{:?}", k),
        };
        Ok(id)
    }

    pub fn expression(&mut self, min_bp: u8) -> Result<ExprStmt, Error> {
        use crate::tokenizer::TokenKind::*;
        let mut lhs = match self.peek().kind {
            Num(_) | Ident(_) => self.primary()?,
            LeftParen => self.grouping()?,
            Plus | Minus => ExprStmt::Unary(self.unary()?),
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

    fn assignment(&mut self) -> Result<Expression, Error> {
        // TODO(chrde): somehow, this must be a variable
        let mut lhs = Expression::Unary(self.expression(0)?);
        while self.peek().kind == TokenKind::Equal {
            let lvalue = if let Expression::Unary(ExprStmt::Primary(Unary::Ident(local))) = lhs {
                LValue::Ident(local)
            } else {
                panic!("wrong LValue for an assignment: {:?}", lhs);
            };
            self.consume(TokenKind::Equal);
            lhs = Expression::Assignment(AssignmentNode {
                lhs: lvalue,
                rhs: Box::new(self.assignment()?),
            })
        }
        Ok(lhs)
    }

    // TODO(chrde): rename this
    fn expr(&mut self) -> Result<Expression, Error> {
        let id = self.assignment()?;
        Ok(id)
    }

    fn statement(&mut self) -> Result<Statement, Error> {
        match self.peek().kind {
            TokenKind::Return => {
                self.consume(TokenKind::Return);
                let lhs = self.expr()?;
                self.consume(TokenKind::Semicolon);
                Ok(Statement::Return(ReturnStmt { lhs }))
            }
            TokenKind::LeftCurly => {
                let mut stmts = vec![];
                self.consume(TokenKind::LeftCurly);
                while self.peek().kind != TokenKind::RightCurly {
                    stmts.push(self.statement()?);
                }
                self.consume(TokenKind::RightCurly);
                Ok(Statement::Block(BlockNode { stmts }))
            }
            _ => {
                let expr = self.expr()?;
                self.consume(TokenKind::Semicolon);
                Ok(Statement::Expr(expr))
            }
        }
    }

    pub fn run(&mut self) -> Result<Program, Error> {
        let mut program = Program::default();
        while self.peek().kind != TokenKind::Eof {
            let statement = self.statement()?;
            program.stmts.push(statement);
        }

        Ok(program)
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

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub enum NodeKind {
//     Statement,
//     Block,

//     Return,

//     // expr
//     NoOp,
//     Add,
//     Sub,
//     Mul,
//     Div,
//     Neg,
//     EqCmp,
//     NeqCmp,
//     LowerCmp,
//     LowerEqCmp,
//     GreaterCmp,
//     GreaterEqCmp,

//     // identifiers
//     Num(usize),
//     Ident(NodeId),
// }

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub stmts: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expr(Expression),
    Return(ReturnStmt),
    Block(BlockNode),
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    pub lhs: Expression,
}

#[derive(Clone, Debug)]
pub enum ExprStmt {
    Unary(UnaryNode),
    Primary(Unary),
    Binary(BinaryNode),
}

#[derive(Clone, Debug)]
pub struct UnaryNode {
    pub op: UnaryOp,
    pub lhs: Unary,
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    Neg,
    // TODO(chrde): this is a hack...
    NoOp,
}

#[derive(Clone, Debug)]
pub enum Unary {
    Num(usize),
    Ident(LocalId),
    Expr(Box<ExprStmt>),
}

#[derive(Clone, Debug)]
pub enum LValue {
    Ident(LocalId),
}

#[derive(Clone, Debug)]
pub struct BinaryNode {
    pub op: BinOp,
    pub lhs: Box<ExprStmt>,
    pub rhs: Box<ExprStmt>,
}

#[derive(Clone, Debug)]
pub struct BlockNode {
    pub stmts: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum Expression {
    Assignment(AssignmentNode),
    Unary(ExprStmt),
}

#[derive(Clone, Debug)]
pub struct AssignmentNode {
    pub lhs: LValue,
    pub rhs: Box<Expression>,
}

#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    EqCmp,
    NeqCmp,
    LowerCmp,
    LowerEqCmp,
    GreaterCmp,
    GreaterEqCmp,
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
    stmts: Vec<Statement>,
    locals: Vec<Local>,
    stack_size: usize,
}

impl Function {
    pub fn new(stmts: Vec<Statement>, locals: Vec<Local>) -> Self {
        let mut result = Self {
            stmts,
            locals,
            stack_size: 0,
        };
        result.assign_locals_offsets();
        result
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn body(&self) -> &[Statement] {
        &self.stmts
    }

    pub fn local(&self, id: LocalId) -> &Local {
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
        base_align * ((size + base_align - 1) / base_align)
    }
}
