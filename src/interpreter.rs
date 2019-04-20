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
}
impl Interpreter {
    pub fn new() -> Self {
        Self {
            namespace: HashMap::new(),
            exec_depth: 0,
        }
    }

    pub fn init(mut self) -> Self {
        // Corelib builtins
        for &name in corelib::BUILTINS.iter() {
            let n: String = name.into();
            self.bind(n.clone(), Value::Bltn(n));
        }

        // Stdlib / prelude imports
        self.execute_file("src/stdlib/logic.mls").expect("STDLIB ERROR");

        self
    }

    pub fn bind(&mut self, name: String, value: Value) {
        self.namespace.insert(name, value);
    }

    #[must_use]
    pub fn resolve(&self, name: String) -> Result<Value, String> {
        self.namespace
            .get(&name)
            .cloned()
            .ok_or(format!("Resolution failed '{:?}'", name))
    }

    #[must_use]
    pub fn execute_file(&mut self, filename: &str) -> Result<(), String> {
        let source = fs::read_to_string(filename)
            .expect("Could not read file")
            .replace("\n", " ");

        self.execute_source(source)
    }

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

    #[must_use]
    pub fn execute(&mut self, mut value: Value) -> Result<Value, String> {
        // println!("{}EXEC: {}", " ".repeat(self.exec_depth * 2), value);
        self.exec_depth += 1;
        loop {
            let oldv = value.clone();
            value = self.execute_step(value)?;
            if value == oldv {
                break;
            }
        }
        self.exec_depth -= 1;
        // println!("{}DONE: {}", " ".repeat(self.exec_depth * 2), value);
        Ok(value)
    }

    #[must_use]
    pub fn execute_step(&mut self, value: Value) -> Result<Value, String> {
        // println!("{}EXEC s: {}", " ".repeat(self.exec_depth * 2), value);
        match value {
            Value::Idfr(name) => self.resolve(name),
            Value::Lmbd(params, box body) if params.is_empty() => Ok(body),
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
                            if name == "quote" {
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
                                let val = args[1].clone();
                                Ok(Value::Expr(
                                    vec![Value::Lmbd(params[1..].to_vec(), box (*body).replace(&sym, val))]
                                        .iter()
                                        .chain(args[2..args.len()].iter())
                                        .cloned()
                                        .collect(),
                                ))
                            }
                        },
                        Value::Quot(_) => Err("Quote cannot be executed".to_owned()),
                        _ => Ok(Value::Expr(args)),
                    }
                }
            },
            v => Ok(v),
        }
    }
}
