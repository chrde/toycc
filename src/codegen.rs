use crate::{ast::*, parser::Function};
use std::fmt::{self, Write};

pub struct Assembly<'a> {
    counter: usize,
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
            counter: 0,
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

    fn count_inc(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    pub fn gen(&mut self) {
        self.writeln("  .globl main");
        self.writeln("main:");

        self.writeln("  push %rbp");
        self.writeln("  mov %rsp, %rbp");
        writeln!(self.content, "  sub ${}, %rsp", self.func.stack_size()).unwrap();
        for stmt in self.func.body() {
            self.gen_stmt(stmt);
        }
        self.writeln(".L.return:");
        self.writeln("  mov %rbp, %rsp");
        self.writeln("  pop %rbp");

        self.writeln("  ret");
    }

    pub fn gen_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expr(e) => self.gen_expr(&e),
            Statement::Return(r) => {
                self.gen_expr(&r.lhs);
                self.writeln("  jmp .L.return");
            }
            Statement::Block(b) => {
                for stmt in &b.stmts {
                    self.gen_stmt(&stmt);
                }
            }
            Statement::If(i) => {
                self.gen_expr(&i.cond);
                self.writeln("  cmp $0, %rax");
                let count = self.count_inc();
                writeln!(self.content, "  je .L.else.{}", count).unwrap();
                self.gen_stmt(&i.then_branch);
                writeln!(self.content, "  jmp .L.end.{}", count).unwrap();
                writeln!(self.content, ".L.else.{}:", count).unwrap();
                if let Some(e) = &i.else_branch {
                    self.gen_stmt(e);
                }
                writeln!(self.content, ".L.end.{}:", count).unwrap();
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
