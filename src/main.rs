use std::env;
use toycc::run;
// use tree_sitter::{Language, Parser, TreeCursor};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(2, args.len());
    match run(&args[1]) {
        Ok(asm) => println!("{}", asm),
        Err(e) => eprint!("{}", e),
    }
}

// extern "C" { fn tree_sitter_c() -> Language; }

// fn main() {
//     let lang = unsafe { tree_sitter_c() };
//     let mut parser = Parser::new();
//     parser.set_language(lang).unwrap();

//     let text = "{ x=3; y=5; *(&y-8)=7; return x; }";
//     let tree = parser.parse(text, None).unwrap();
//     dbg!(&tree);
//     let mut cursor = tree.walk();
//     visit_node(&cursor);
//     while cursor.goto_first_child() {
//         visit_node(&cursor);
//     }
//     while cursor.goto_next_sibling() {
//         visit_node(&cursor);
//     }
// }

// fn visit_node(cursor: &TreeCursor) {
//     let name = cursor.node().kind();
//     println!("{}", name);
// }

// struct DfsIterator<'a, 'b: 'a> {
//     cursor: &'a mut TreeCursor<'b>
// }

// impl<'a, 'b: 'a> DfsIterator<'a, 'b> {
//     fn advance(&mut self) -> {
//         loop {
//             if self.cursor.goto_first_child() {
//                 if self.cursor.
//             }
//         }
//     }
// }


// impl<'a, 'b: 'a> Iterator for DfsIterator<'a, 'b> {
//     type Item = &'static str;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if let Some(n) = self.cursor.field_name() {
//                 if self.cursor.goto_first_child() {
//                     Some(n)
//                 } else if self.cursor.goto_next_sibling() {
//                     Some(n)
//                 } else if self.cursor.goto_parent() {
//                     Some(n
//                 }
//             }
//         }
//     }
// }
