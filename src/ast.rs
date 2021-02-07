#[derive(Clone, Debug, Default)]
pub struct Program {
    pub stmts: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Expr(Expression),
    Return(ReturnStmt),
    Block(BlockNode),
    If(IfStmt),
    For(ForStmt),
}

#[derive(Clone, Debug)]
pub struct ForStmt {
    pub init: Option<Expression>,
    pub cond: Option<Expression>,
    pub inc: Option<Expression>,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub cond: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
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
    pub name: String,
    pub offset: usize,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalId(pub usize);
