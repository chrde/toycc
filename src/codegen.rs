use crate::parser::{
    Function, Node, NodeId,
    NodeKind::{self, *},
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
        for stmt in self.func.body() {
            match stmt.kind() {
                NodeKind::Statement => self.gen_expr(stmt.lhs()),
                NodeKind::Return => {
                    self.gen_expr(stmt.lhs());
                    self.writeln("  jmp .L.return");
                }
                k => unreachable!("{:?}", k),
            }
        }

        self.writeln(".L.return:");
        self.writeln("  mov %rbp, %rsp");
        self.writeln("  pop %rbp");

        self.writeln("  ret");
    }

    pub fn gen_addr(&mut self, node: &Node) {
        match node.kind() {
            NodeKind::Ident(l) => {
                let l = self.func.local(*l);
                writeln!(self.content, "  lea -{}(%rbp), %rax", l.offset()).unwrap();
            }
            k => unreachable!("{:?}", k),
        }
    }

    pub fn recurse_expr(&mut self, node: &Node) {
        self.gen_expr(node.rhs());
        self.push();
        self.gen_expr(node.lhs());
        self.pop("%rdi");
    }

    pub fn gen_expr(&mut self, id: NodeId) {
        let node = self.func.node(id);
        match node.kind() {
            Add => {
                self.recurse_expr(node);
                self.writeln("  add %rdi, %rax")
            }
            Sub => {
                self.recurse_expr(node);
                self.writeln("  sub %rdi, %rax")
            }
            Mul => {
                self.recurse_expr(node);
                self.writeln("  imul %rdi, %rax")
            }
            Div => {
                self.recurse_expr(node);
                self.writeln("  cqo");
                self.writeln("  idiv %rdi");
            }
            GreaterEqCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setge %al");
                self.writeln("  movzb %al, %rax");
            }
            LowerEqCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setle %al");
                self.writeln("  movzb %al, %rax");
            }
            GreaterCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setg %al");
                self.writeln("  movzb %al, %rax");
            }
            LowerCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setl %al");
                self.writeln("  movzb %al, %rax");
            }
            EqCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  sete %al");
                self.writeln("  movzb %al, %rax");
            }
            NeqCmp => {
                self.recurse_expr(node);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setne %al");
                self.writeln("  movzb %al, %rax");
            }
            Neg => {
                self.gen_expr(node.lhs());
                self.writeln("  neg %rax");
            }
            Num(val) => writeln!(self.content, "  mov ${}, %rax", val).unwrap(),
            Ident(_) => {
                self.gen_addr(node);
                self.writeln("  mov (%rax), %rax");
            }
            Assignment => {
                self.gen_addr(self.func.node(node.lhs()));
                self.push();
                self.gen_expr(node.rhs());
                self.pop("%rdi");
                self.writeln("  mov %rax, (%rdi)");
            }
            k => unreachable!("{:?}", k),
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
