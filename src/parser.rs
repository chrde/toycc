use crate::ast::*;
use crate::tokenizer::TokenKind;
use crate::tokenizer::TokenKind::*;
use crate::{tokenizer::Token, Error};

pub struct Parser<'a> {
    code: &'a str,
    pub locals: Vec<Local>,
    tokens: Vec<Token>,
}

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

    fn skip(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            let _ = self.next();
            true
        } else {
            false
        }
    }

    fn consume_binary(&mut self) -> Token {
        let next = self.next();
        assert!(next.kind.binary());
        next
    }

    fn binary(&mut self, lhs: Expression, min_bp: u8) -> Result<Expression, Error> {
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
        Ok(Expression::Binary(BinaryExpr {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    }

    // parens
    fn grouping(&mut self) -> Result<Expression, Error> {
        self.consume(TokenKind::LeftParen);
        let expr = self.expression(0)?;
        self.consume(TokenKind::RightParen);
        Ok(expr)
    }

    // fn primary(&mut self) -> Result<PrimaryNode, Error> {
    //     let t = self.next();
    //     match t.kind {
    //         TokenKind::Num(val) => Ok(PrimaryNode::Num(val)),
    //         TokenKind::Ident(name) => Ok(PrimaryNode::Ident(self.push_local(name))),
    //         k => unimplemented!("{:?}", k),
    //     }
    // }

    // unary
    fn unary(&mut self) -> Result<Expression, Error> {
        let t = self.next();
        let ((), r_bp) = prefix_binding_power(&t.kind);
        let lhs = self.expression(r_bp)?;
        let e = match &t.kind {
            TokenKind::Plus => Expression::Unary(UnaryExpr {
                op: UnaryOp::NoOp,
                lhs: Box::new(lhs),
            }),
            TokenKind::Minus => Expression::Unary(UnaryExpr {
                op: UnaryOp::Neg,
                lhs: Box::new(lhs),
            }),
            TokenKind::Amp => Expression::Pointer(PointerExpr {
                op: PointerOp::Ref,
                arg: Box::new(lhs),
            }),
            TokenKind::Star => Expression::Pointer(PointerExpr {
                op: PointerOp::Deref,
                arg: Box::new(lhs),
            }),
            k => unimplemented!("{:?}", k),
        };
        Ok(e)
    }

    fn assignment(&mut self, lhs: Expression, min_bp: u8) -> Result<Expression, Error> {
        // let mut lhs = Expression::Unary(self.expression(0)?);
        let op = match self.next().kind {
            TokenKind::Equal => AssignmentOp::Eq,
            k => unimplemented!("{:?}", k),
        };

        let rhs = self.expression(min_bp)?;
        Ok(Expression::Assignment(AssignmentExpr {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op
        }))

        // while self.peek().kind == TokenKind::Equal {
        //     let lvalue = if let Expression::Unary(ExprStmt::Primary(PrimaryNode::Ident(local))) = lhs {
        //         LValue::Ident(local)
        //     } else {
        //         panic!("wrong LValue for an assignment: {:?}", lhs);
        //     };
        //     self.consume(TokenKind::Equal);
        //     lhs = Expression::Assignment(AssignmentNode {
        //         lhs: lvalue,
        //         rhs: Box::new(self.assignment()?),
        //     })
        // }
        // Ok(lhs)
    }

    pub fn expression(&mut self, min_bp: u8) -> Result<Expression, Error> {
        let mut lhs = match self.peek().kind {
            Num(v) => {
                self.next();
                Expression::NumberLiteral(v)
            }
            Ident(i) => {
                self.next();
                Expression::Identifier(self.push_local(i))
            }
    //         Num(_) | Ident(_) => ExprStmt::Primary(self.primary()?),
            LeftParen => self.grouping()?,
            Plus | Minus | Star | Amp => self.unary()?,
            k => unimplemented!("{:?}", k),
        };

        loop {
            let next = self.peek().kind;
    //         match next {
    //             Eof | RightParen | Semicolon | Equal => break,
    //             Star | Slash | Plus | Minus | EqualEqual | NotEqual | Lower | Greater
    //             | LowerEqual | GreaterEqual => {}
    //             k => unimplemented!("{:?}", k),
    //         };
            if let Some((l_bp, r_bp)) = infix_binding_power(&next) {
                if l_bp < min_bp {
                    break;
                }
                lhs = match next {
                    Equal => self.assignment(lhs, r_bp)?,
                    _ => self.binary(lhs, r_bp)?,
                };
                // if next.binary() {
                //     lhs = self.binary(lhs, r_bp)?;
                // }
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    // fn expr(&mut self) -> Result<Expression, Error> {
    //     self.assignment()
    // }

    // fn expr_stmt(&mut self) -> Result<Expression, Error> {
    //     let id = self.expr()?;
    //     self.consume(TokenKind::Semicolon);
    //     Ok(id)
    // }

    fn statement(&mut self) -> Result<Statement, Error> {
        match self.peek().kind {
            TokenKind::While => {
                self.consume(TokenKind::While);
                self.consume(TokenKind::LeftParen);
                let condition = Some(self.expression(0)?);
                self.consume(TokenKind::RightParen);
                let body = Box::new(self.statement()?);
                Ok(Statement::While(WhileStatement {
                    condition,
                    body,
                }))
            }
            TokenKind::For => {
                self.consume(TokenKind::For);
                self.consume(TokenKind::LeftParen);
                let init = if self.skip(TokenKind::Semicolon) {
                    None
                } else {
                    let init = Some(self.expression(0)?);
                    self.consume(TokenKind::Semicolon);
                    init
                };
                let condition = if self.skip(TokenKind::Semicolon) {
                    None
                } else {
                    let condition = Some(self.expression(0)?);
                    self.consume(TokenKind::Semicolon);
                    condition
                };
                let update = if self.skip(TokenKind::RightParen) {
                    None
                } else {
                    let update = Some(self.expression(0)?);
                    self.consume(TokenKind::RightParen);
                    update
                };
                let body = Box::new(self.statement()?);
                Ok(Statement::For(ForStatement {
                    init,
                    condition,
                    update,
                    body,
                }))
            }
            TokenKind::If => {
                self.consume(TokenKind::If);
                self.consume(TokenKind::LeftParen);
                let condition = self.expression(0)?;
                self.consume(TokenKind::RightParen);
                let then_branch = Box::new(self.statement()?);
                let else_branch = if self.skip(TokenKind::Else) {
                    let res = Some(Box::new(self.statement()?));
                    res
                } else {
                    None
                };
                Ok(Statement::If(IfStatement {
                    condition,
                    then_branch,
                    else_branch,
                }))
            }
            TokenKind::Return => {
                self.consume(TokenKind::Return);
                let lhs = self.expression(0)?;
                self.consume(TokenKind::Semicolon);
                Ok(Statement::Return(lhs))
            }
            TokenKind::LeftCurly => {
                let mut stmts = vec![];
                self.consume(TokenKind::LeftCurly);
                while self.peek().kind != TokenKind::RightCurly {
                    stmts.push(self.statement()?);
                }
                self.consume(TokenKind::RightCurly);
                Ok(Statement::Compound(CompoundStatement { stmts }))
            }
            TokenKind::Semicolon => {
                loop {
                    if !self.skip(TokenKind::Semicolon) {
                        break;
                    }
                }
                Ok(Statement::Empty)
            }
            _ => {
                    let lhs = self.expression(0)?;
                    self.consume(TokenKind::Semicolon);
                    Ok(Statement::Expr(lhs))
            }
        }
    }

    pub fn run(&mut self) -> Result<Program, Error> {
        let mut program = Program::default();
        self.consume(TokenKind::LeftCurly);
        while self.peek().kind != TokenKind::RightCurly {
            let statement = self.statement()?;
            program.stmt.stmts.push(statement);
        }
        self.consume(TokenKind::RightCurly);
        self.consume(TokenKind::Eof);

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
        TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Amp => ((), PREC_UNARY),
        t => unimplemented!("{:?}", t),
    }
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

#[derive(Clone, Debug)]
pub struct Function {
    stmt: CompoundStatement,
    pub locals: Vec<Local>,
    stack_size: usize,
}

impl Function {
    pub fn new(program: Program, locals: Vec<Local>) -> Self {
        let mut result = Self {
            stmt: program.stmt,
            locals,
            stack_size: 0,
        };
        result.assign_locals_offsets();
        result
    }

    pub fn stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn body(&self) -> &CompoundStatement {
        &self.stmt
    }

    pub fn local(&self, id: LocalId) -> &Local {
        &self.locals[id.0]
    }

    fn assign_locals_offsets(&mut self) {
        let mut total = 0;
        for l in self.locals.iter_mut().rev() {
            total += 8;
            l.offset = total;
        }
        self.stack_size = Self::align_stack_size(total, 16)
    }

    fn align_stack_size(size: usize, base_align: usize) -> usize {
        base_align * ((size + base_align - 1) / base_align)
    }
}
