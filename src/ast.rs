#[derive(Clone, Debug, Default)]
pub struct Program {
    pub stmt: CompoundStatement,
}

// pub struct BlockItemList {
//     items: Vec<BlockItem>
// }

// pub enum BlockItem {
//     Statement(Statement),
//     Declaration,
// }

#[derive(Clone, Debug, Default)]
pub struct CompoundStatement {
    pub stmts: Vec<Statement>,
}

// non_case_statement
#[derive(Clone, Debug)]
pub enum Statement {
    Compound(CompoundStatement),
    Expr(Expression),
    Return(Expression),
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    Empty,
    // Block(BlockNode),
    // If(IfStmt),
    // For(ForStmt),
}

#[derive(Clone, Debug)]
pub struct WhileStatement {
    pub condition: Option<Expression>,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug)]
pub struct ForStatement {
    pub init: Option<Expression>, // TODO(chrde): this should be a Declaration
    pub condition: Option<Expression>,
    pub update: Option<Expression>,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub lhs: Box<Expression>,
    pub op: UnaryOp,
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub op: BinOp,
}

#[derive(Clone, Debug)]
pub struct AssignmentExpr {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub op: AssignmentOp,
}

#[derive(Clone, Debug)]
pub struct PointerExpr {
    pub arg: Box<Expression>,
    pub op: PointerOp,
}

#[derive(Copy, Clone, Debug)]
pub enum PointerOp {
    Ref,
    Deref,
}

#[derive(Clone, Debug)]
pub enum Expression {
    NumberLiteral(usize),
    Identifier(LocalId),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Assignment(AssignmentExpr),
    Pointer(PointerExpr),
}

// old

// #[derive(Clone, Debug)]
// pub struct ForStmt {
//     pub init: Option<Expression>,
//     pub cond: Option<Expression>,
//     pub inc: Option<Expression>,
//     pub body: Box<Statement>,
// }

// #[derive(Clone, Debug)]
// pub struct IfStmt {
//     pub cond: Expression,
//     pub then_branch: Box<Statement>,
//     pub else_branch: Option<Box<Statement>>,
// }

// #[derive(Clone, Debug)]
// pub struct ReturnStmt {
//     pub lhs: Expression,
// }

// #[derive(Clone, Debug)]
// pub enum ExprStmt {
//     Unary(UnaryNode),
//     Primary(PrimaryNode),
//     Binary(BinaryNode),
// }

// #[derive(Clone, Debug)]
// pub struct UnaryNode {
//     pub op: UnaryOp,
//     pub lhs: Box<ExprStmt>,
// }

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    Neg,
    Addr,
    Deref,
    // TODO(chrde): this is a hack...
    NoOp,
}

// #[derive(Clone, Debug)]
// pub enum PrimaryNode {
//     Num(usize),
//     Ident(LocalId),
// }

// #[derive(Clone, Debug)]
// pub enum LValue {
//     Ident(LocalId),
// }

// #[derive(Clone, Debug)]
// pub struct BinaryNode {
//     pub op: BinOp,
//     pub lhs: Box<ExprStmt>,
//     pub rhs: Box<ExprStmt>,
// }

// #[derive(Clone, Debug)]
// pub struct BlockNode {
//     pub stmts: Vec<Statement>,
// }

// #[derive(Clone, Debug)]
// pub enum Expression {
//     Assignment(AssignmentNode),
//     Unary(ExprStmt),
//     NumberLiteral(usize),
// }

// #[derive(Clone, Debug)]
// pub struct AssignmentNode {
//     pub lhs: LValue,
//     pub rhs: Box<Expression>,
// }

#[derive(Clone, Debug)]
pub enum AssignmentOp {
    Eq,
}

#[derive(Copy, Clone, Debug)]
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
