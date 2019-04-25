//! MiniLisp interpreter

use rustyline::error::ReadlineError;
use rustyline::Editor;

use clap;

use minilisp::parser;
use minilisp::Interpreter;
use minilisp::Value;

fn main() {
    // Parse arguments
    let matches = clap::App::new("minilisp")
        .version("0.1")
        .about("The minilisp interpreter")
        .arg(
            clap::Arg::with_name("SOURCE")
                .help("Source code file")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Verbosity level"),
        )
        .get_matches();

    // Interpreter initalization
    let mut intp = Interpreter::new().init();

    if matches.occurrences_of("v") > 0 {
        intp.set_debug_print(true);
    }

    if let Some(filename) = matches.value_of("SOURCE") {
        if let Err(e) = intp.execute_file(filename) {
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
