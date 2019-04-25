use std::collections::HashMap;
use std::fs;

use super::corelib;
use super::parser;
use super::value::Value;

/// Language interpreter
#[derive(Debug, Clone)]
pub struct Interpreter {
    namespace: HashMap<String, Value>,
    exec_depth: usize,
    debug_print: bool,
}
impl Interpreter {
    /// Create new, empty interpreter
    pub fn new() -> Self {
        Self {
            namespace: HashMap::new(),
            exec_depth: 0,
            debug_print: false,
        }
    }

    /// Bind corelib and stdlib functions
    pub fn init(mut self) -> Self {
        // Corelib builtins
        for &name in corelib::BUILTINS.iter() {
            let n: String = name.into();
            self.bind(n.clone(), Value::Bltn(n));
        }

        // Stdlib / prelude imports
        // self.debug_print = true;
        self.execute_file("src/stdlib/logic.mls").expect("STDLIB ERROR");
        self.execute_file("src/stdlib/peano.mls").expect("STDLIB ERROR");

        self
    }

    /// Set debug printing on or off
    pub fn set_debug_print(&mut self, v: bool) {
        self.debug_print = v;
    }

    /// Namespace bind
    pub fn bind(&mut self, name: String, value: Value) {
        self.namespace.insert(name, value);
    }

    /// Namespace delete
    pub fn delete(&mut self, name: String) {
        self.namespace.remove(&name);
    }

    /// Symbol name resolution
    #[must_use]
    pub fn resolve(&self, name: String) -> Result<Value, String> {
        self.namespace
            .get(&name)
            .cloned()
            .ok_or(format!("Resolution failed '{:?}'", name))
    }

    /// Read file and execute contents
    #[must_use]
    pub fn execute_file(&mut self, filename: &str) -> Result<(), String> {
        if self.debug_print {
            println!("{}EXECUTING FILE: {}", " ".repeat(self.exec_depth * 2), filename);
        }
        let source = fs::read_to_string(filename).expect("Could not read file");

        self.execute_source(source)
    }

    /// Execute source code text
    #[must_use]
    pub fn execute_source(&mut self, source: String) -> Result<(), String> {
        let mut tokens = parser::split_tokens(source).unwrap();
        while !tokens.is_empty() {
            let (exprt, newt) = parser::take_expr(tokens).expect("Invalid Expression");
            tokens = newt;
            self.execute(Value::parse(exprt)?)?;
        }
        Ok(())
    }

    /// Execute a value
    #[must_use]
    pub fn execute(&mut self, mut value: Value) -> Result<Value, String> {
        if self.debug_print {
            println!("{}EXEC: {}", " ".repeat(self.exec_depth * 2), value);
        }

        self.exec_depth += 1;
        loop {
            let oldv = value.clone();
            value = self.execute_step(value)?;
            if value == oldv {
                break;
            }
        }
        self.exec_depth -= 1;
        if self.debug_print {
            println!("{}DONE: {}", " ".repeat(self.exec_depth * 2), value);
        }
        Ok(value)
    }

    /// Do one reduction step on a value
    #[must_use]
    pub fn execute_step(&mut self, value: Value) -> Result<Value, String> {
        if self.debug_print {
            println!("{}EXEC s: {}", " ".repeat(self.exec_depth * 2), value);
        }
        match value {
            Value::Idfr(name) => self.resolve(name),
            Value::Lmbd(params, box body) if params.is_empty() => {
                if let Value::Quot(box q) = body {
                    Ok(q)
                } else {
                    Err("Lambda body must be quoted".to_owned())
                }
            },
            Value::Expr(args) => {
                if args.len() == 1 {
                    Ok(args[0].clone())
                } else {
                    match args[0].clone() {
                        Value::Expr(a) => {
                            let na = self.execute(Value::Expr(a))?;
                            let mut newargs = vec![na];
                            newargs.extend_from_slice(&args[1..]);
                            Ok(Value::Expr(newargs))
                        },
                        Value::Idfr(idfr) => {
                            let na = self.resolve(idfr)?;
                            let mut newargs = vec![na];
                            newargs.extend_from_slice(&args[1..]);
                            Ok(Value::Expr(newargs))
                        },
                        Value::Bltn(name) => {
                            if name == "error" {
                                Err(format!(
                                    "Runtime Error: {}",
                                    args[1..]
                                        .iter()
                                        .map(|a| format!("{}", a))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                ))
                            } else if name == "quote" {
                                if args.len() == 2 {
                                    Ok(Value::Quot(box args[1].clone()))
                                } else {
                                    Err("Arg count".to_owned())
                                }
                            } else {
                                let args_e: Vec<Value> = args[1..]
                                    .iter()
                                    .cloned()
                                    .map(|a| self.execute(a))
                                    .collect::<Result<_, _>>()?;

                                if self.debug_print {
                                    println!(
                                        "{}EXEC b: ({} {})",
                                        " ".repeat(self.exec_depth * 2),
                                        name,
                                        args_e
                                            .iter()
                                            .map(|a| format!("{}", a))
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                    );
                                }

                                corelib::call(self, name, args_e).ok_or("Function not found")?
                            }
                        },
                        Value::Lmbd(params, body) => {
                            if params.is_empty() {
                                if args.len() > 1 {
                                    Err("Trying to apply too much arguments to a lambda".to_owned())
                                } else {
                                    Ok(*body)
                                }
                            } else {
                                let sym = params[0].clone();
                                let val = self.execute(args[1].clone())?;

                                Ok(Value::Expr(
                                    vec![Value::Lmbd(params[1..].to_vec(), box (*body).replace(&sym, val))]
                                        .iter()
                                        .chain(args[2..args.len()].iter())
                                        .cloned()
                                        .collect(),
                                ))
                            }
                        },
                        Value::Quot(q) => Err(format!("Quote cannot be executed: {}", q)),
                        _ => Ok(Value::Expr(args)),
                    }
                }
            },
            v => Ok(v),
        }
    }
}
