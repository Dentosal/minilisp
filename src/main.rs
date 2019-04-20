#![feature(bind_by_move_pattern_guards)]
#![feature(box_syntax)]
#![feature(box_patterns)]

use std::env;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod corelib;
mod interpreter;
mod parser;
mod value;

pub use self::interpreter::Interpreter;
pub use self::value::Value;

fn main() {
    let mut intp = Interpreter::new().init();

    if let Some(filename) = env::args().skip(1).nth(0) {
        if let Err(e) = intp.execute_file(filename.as_str()) {
            println!("Error: {}", e);
        }
    } else {
        let mut rl = Editor::<()>::new();
        loop {
            match rl.readline("> ") {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());

                    let mut tokens = parser::split_tokens(line).unwrap();

                    while !tokens.is_empty() {
                        let (exprt, newt) = parser::take_expr(tokens).expect("Invalid Expression");
                        tokens = newt;

                        match Value::parse(exprt) {
                            Ok(expr) => match intp.execute(expr) {
                                Ok(v) => println!("{}", v.format()),
                                Err(e) => println!("Error: {:?}", e),
                            },
                            Err(e) => {
                                println!("Error: {:?}", e);
                            },
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                },
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                },
            }
        }
    }
}
