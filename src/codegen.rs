use crate::{NodeArena, parser::{Node, NodeId}};
use std::fmt::{self, Write};

pub struct Assembly {
    depth: usize,
    content: String,
}

impl fmt::Display for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl Assembly {
    pub fn new() -> Self {
        Self {
            depth: 0,
            content: String::new(),
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

    pub fn recurse(&mut self, node: Node, arena: &NodeArena) {
        self.gen_expr(node.rhs(), arena);
        self.push();
        self.gen_expr(node.lhs(), arena);
        self.pop("%rdi");
    }

    pub fn gen_expr(&mut self, id: NodeId, arena: &NodeArena) {
        use crate::parser::NodeKind::*;
        let node = arena.get(id);
        match node.kind() {
            Add => {
                self.recurse(node, arena);
                self.writeln("  add %rdi, %rax")
            }
            Sub => {
                self.recurse(node, arena);
                self.writeln("  sub %rdi, %rax")
            }
            Mul => {
                self.recurse(node, arena);
                self.writeln("  imul %rdi, %rax")
            }
            Div => {
                self.recurse(node, arena);
                self.writeln("  cqo");
                self.writeln("  idiv %rdi");
            }
            GreaterEqCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setge %al");
                self.writeln("  movzb %al, %rax");
            }
            LowerEqCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setle %al");
                self.writeln("  movzb %al, %rax");
            }
            GreaterCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setg %al");
                self.writeln("  movzb %al, %rax");
            }
            LowerCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setl %al");
                self.writeln("  movzb %al, %rax");
            }
            EqCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  sete %al");
                self.writeln("  movzb %al, %rax");
            }
            NeqCmp => {
                self.recurse(node, arena);
                self.writeln("  cmp %rdi, %rax");
                self.writeln("  setne %al");
                self.writeln("  movzb %al, %rax");
            }
            Neg => {
                self.gen_expr(node.lhs(), arena);
                self.writeln("  neg %rax");
            }
            Num(val) => writeln!(self.content, "  mov ${}, %rax", val).unwrap(),
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

