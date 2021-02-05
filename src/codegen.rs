use crate::parser::{
    BinOp, BinaryNode, ExprStmt, Expression, Function, LValue, LocalId, Statement, Unary, UnaryOp,
};
use std::fmt::{self, Write};

pub struct Assembly<'a> {
    depth: usize,
    content: String,
    func: &'a Function,
}

impl fmt::Display for Assembly<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl<'a> Assembly<'a> {
    pub fn new(func: &'a Function) -> Self {
        Self {
            depth: 0,
            content: String::new(),
            func,
        }
    }

    pub fn finish(self) -> String {
        assert_eq!(0, self.depth);
        self.content
    }

    pub fn push(&mut self) {
        self.writeln("  push %rax");
        self.depth += 1;
    }

    pub fn pop(&mut self, s: &str) {
        writeln!(self.content, "  pop {}", s).unwrap();
        self.depth -= 1;
    }

    pub fn gen(&mut self) {
        self.writeln("  .globl main");
        self.writeln("main:");

        self.writeln("  push %rbp");
        self.writeln("  mov %rsp, %rbp");
        writeln!(self.content, "  sub ${}, %rsp", self.func.stack_size()).unwrap();
        self.gen_stmts(self.func.body());
        self.writeln(".L.return:");
        self.writeln("  mov %rbp, %rsp");
        self.writeln("  pop %rbp");

        self.writeln("  ret");
    }

    pub fn gen_stmts(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            match stmt {
                Statement::Expr(e) => self.gen_expr(&e),
                Statement::Return(r) => {
                    self.gen_expr(&r.lhs);
                    self.writeln("  jmp .L.return");
                }
                Statement::Block(b) => self.gen_stmts(&b.stmts),
            }
        }
    }

    pub fn gen_addr(&mut self, local: LocalId) {
        let l = self.func.local(local);
        writeln!(self.content, "  lea -{}(%rbp), %rax", l.offset()).unwrap();
    }

    pub fn recurse_binary(&mut self, bin: &BinaryNode) {
        self.gen_expr_stmt(&bin.rhs);
        self.push();
        self.gen_expr_stmt(&bin.lhs);
        self.pop("%rdi");
    }

    pub fn gen_unary(&mut self, u: &Unary) {
        match u {
            Unary::Num(n) => writeln!(self.content, "  mov ${}, %rax", n).unwrap(),
            Unary::Ident(local) => {
                self.gen_addr(*local);
                self.writeln("  mov (%rax), %rax");
            }
            Unary::Expr(stmt) => self.gen_expr_stmt(stmt),
        }
    }

    pub fn gen_binary(&mut self, node: &BinaryNode) {
        match node.op {
            BinOp::Add => {
                self.recurse_binary(node);
                self.writeln("  add %rdi, %rax")
            }
            BinOp::Sub => {
                self.recurse_binary(node);
                self.writeln("  sub %rdi, %rax")
            }
            BinOp::Mul => {
                self.recurse_binary(node);
                self.writeln("  imul %rdi, %rax")
            }
            BinOp::Div => {
                self.recurse_binary(node);
                self.writeln("  cqo");
                self.writeln("  idiv %rdi");
            }
            BinOp::GreaterEqCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setge %al");
                self.writeln("  movzb %al, %rax");
            }
            BinOp::LowerEqCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setle %al");
                self.writeln("  movzb %al, %rax");
            }
            BinOp::GreaterCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setg %al");
                self.writeln("  movzb %al, %rax");
            }
            BinOp::LowerCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setl %al");
                self.writeln("  movzb %al, %rax");
            }
            BinOp::EqCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  sete %al");
                self.writeln("  movzb %al, %rax");
            }
            BinOp::NeqCmp => {
                self.recurse_binary(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setne %al");
                self.writeln("  movzb %al, %rax");
            }
        }
    }

    pub fn gen_expr_stmt(&mut self, e: &ExprStmt) {
        match e {
            ExprStmt::Unary(u) => match u.op {
                UnaryOp::Neg => {
                    self.gen_unary(&u.lhs);
                    self.writeln("  neg %rax");
                }
                UnaryOp::NoOp => self.gen_unary(&u.lhs),
            },
            ExprStmt::Primary(u) => self.gen_unary(u),
            ExprStmt::Binary(b) => self.gen_binary(b),
        }
    }

    pub fn gen_expr(&mut self, stmt: &Expression) {
        match stmt {
            Expression::Unary(e) => self.gen_expr_stmt(e),
            Expression::Assignment(a) => {
                match a.lhs {
                    LValue::Ident(local) => self.gen_addr(local),
                }
                self.push();
                self.gen_expr(&a.rhs);
                self.pop("%rdi");
                self.writeln("  mov %rax, (%rdi)");
            }
        }
    }

    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.content.push('\n')
    }

    pub fn write(&mut self, s: &str) {
        self.content.push_str(s)
    }
}
