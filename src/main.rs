use toycc::run;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(2, args.len());
    match run(&args[1]) {
        Ok(asm) => println!("{}", asm),
        Err(e) => eprint!("{}", e)
    }
}
