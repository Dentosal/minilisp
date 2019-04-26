use super::{Interpreter, Value};

pub const BUILTINS: [&str; 19] = [
    // Special items
    "error",
    "quote",
    // Core language
    "unquote",
    "discard",
    "assert",
    "lambda",
    "block",
    "branch",
    // Equality check
    "eq?",
    "eqtree?",
    // Namespace operators
    "set",
    "del",
    // Operations on quoted expressions as lists
    "q:reverse",
    "q:concat",
    "q:head",
    "q:tail",
    "q:empty?",
    "q:expr?",
    // I/O
    "println",
];

macro_rules! boolvalue {
    ($v:expr) => {{
        if $v {
            Value::Idfr("true".to_owned())
        } else {
            Value::Unit
        }
    }};
}

/// Outer options is None if the function is not found
/// Inner result marks success of exection
#[must_use]
pub fn call(intp: &mut Interpreter, name: String, args: Vec<Value>) -> Result<Value, String> {
    match name.as_str() {
        // evaluate quoted expression
        "unquote" => {
            if args.len() != 1 {
                return Err("Arg count".to_owned());
            }
            if let Value::Quot(e) = args[0].clone() {
                Ok(*e)
            } else {
                Err(format!("Only quote can be unquoted, {:?} is invalid", args[0]))
            }
        },
        // discard an expression
        "discard" => Ok(Value::Unit),
        // throws error if the paramter is Unit. returns original value
        "assert" => {
            if args.len() != 1 {
                return Err("Arg count".to_owned());
            }

            if args[0] == Value::Unit {
                Err("Assertion failed".to_owned())
            } else {
                Ok(args[0].clone())
            }
        },
        // anonymous function, i.e. parameter substitution
        "lambda" => {
            let params: Result<Vec<String>, String> = args[..args.len() - 1]
                .iter()
                .cloned()
                .map(|a| match a {
                    Value::Quot(box Value::Idfr(n)) => Ok(n),
                    _ => Err("Quoted identifier required as lambda parameters".to_owned()),
                })
                .collect();

            let body = args[args.len() - 1].clone();

            Ok(Value::Lmbd(params?, box body))
        },
        // unquote a list of statements sequentially, returning the last result
        "block" => {
            let mut res: Value = Value::Unit;
            for a in args {
                res = intp.execute(Value::Expr(vec![Value::Bltn("unquote".to_owned()), a.clone()]))?;
            }
            Ok(res)
        },
        // if-else condition select, unquotes the relevant side
        "branch" => {
            if args.len() != 3 {
                Err("Arg count".to_owned())
            } else {
                if args[0] != Value::Unit {
                    // strict true check
                    if args[0] != Value::Quot(box Value::Idfr("true".to_owned())) {
                        return Err("Strict true required".to_owned());
                    }

                    // true branch
                    Ok(Value::Expr(vec![
                        Value::Bltn("unquote".to_owned()),
                        args[1].clone(),
                    ]))
                } else {
                    // false branch
                    Ok(Value::Expr(vec![
                        Value::Bltn("unquote".to_owned()),
                        args[2].clone(),
                    ]))
                }
            }
        },
        // equality check, resolving names inside a quoted expression first
        // name resolution is done until a stop-idfr is reached
        "eq?" => {
            if args.len() != 2 {
                Err("Arg count".to_owned())
            } else {
                if args[0] == args[1] {
                    Ok(boolvalue!(true))
                } else {
                    Ok(boolvalue!(args[0].clone().resolve_all(intp)? == args[1].clone().resolve_all(intp)?))
                }
                // if let Value::Quot(box Value::Expr(e0)) = args[0].clone() {
                //     if let Value::Quot(box Value::Expr(e1)) = args[1].clone() {
                //         let v0: Result<Vec<_>, _> = e0.into_iter().map(|e| e.resolve_all(intp)).collect();
                //         let v1: Result<Vec<_>, _> = e1.into_iter().map(|e| e.resolve_all(intp)).collect();
                //         Ok(boolvalue!(v0? == v1?))
                //     } else {
                //         Ok(boolvalue!(false))
                //     }
                // } else {
                //     Ok(boolvalue!(args[0] == args[1]))
                // }
            }
        },
        // test exact structure equality
        "eqtree?" => {
            if args.len() != 2 {
                Err("Arg count".to_owned())
            } else {
                Ok(boolvalue!(args[0] == args[1]))
            }
        },
        // reverse quoted expression
        "q:reverse" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box Value::Expr(e)) = args[0].clone() {
                    Ok(Value::Quot(box Value::Expr(e.iter().rev().cloned().collect())))
                } else if let Value::Quot(box Value::Unit) = args[0] {
                    Ok(Value::Quot(box Value::Unit))
                } else {
                    Err("Quoted expression required".to_owned())
                }
            }
        },
        // concatenate two quoted expressions
        "q:concat" => {
            if args.len() != 2 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box Value::Expr(e0)) = args[0].clone() {
                    if let Value::Quot(box Value::Expr(e1)) = args[1].clone() {
                        Ok(Value::Quot(box Value::Expr(
                            e0.iter().chain(e1.iter()).cloned().collect(),
                        )))
                    } else if let Value::Quot(box Value::Unit) = args[1] {
                        Ok(args[0].clone())
                    } else {
                        Err("Quoted expression required".to_owned())
                    }
                } else if let Value::Quot(box Value::Unit) = args[0] {
                    Ok(args[1].clone())
                } else {
                    Err("Quoted expression required".to_owned())
                }
            }
        },
        // get first item from a quoted expression
        "q:head" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box Value::Expr(e0)) = args[0].clone() {
                    Ok(e0[0].clone())
                } else if let Value::Quot(box Value::Unit) = args[0] {
                    Err("Cannot get first element of an empty list".to_owned())
                } else {
                    Err("Quoted expression required".to_owned())
                }
            }
        },
        // remove first item from a quoted expression
        "q:tail" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box Value::Expr(e)) = args[0].clone() {
                    if e.len() == 1 {
                        Ok(Value::Quot(box Value::Unit))
                    } else {
                        Ok(Value::Quot(box Value::Expr(e[1..].to_vec())))
                    }
                } else if let Value::Quot(box Value::Unit) = args[0] {
                    Ok(Value::Quot(box Value::Unit))
                } else {
                    Err("Quoted expression required".to_owned())
                }
            }
        },
        // test if the top-level item in quotes is empty, i.e Unit
        "q:empty?" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box arg) = &args[0] {
                    Ok(boolvalue!(*arg == Value::Unit))
                } else {
                    Err("Quoted value required as argument".to_owned())
                }
            }
        },
        // test if the top-level item in quotes is an expression
        "q:expr?" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(box arg) = &args[0] {
                    Ok(match arg {
                        Value::Unit | Value::Expr(_) => boolvalue!(true),
                        _ => boolvalue!(false),
                    })
                } else {
                    Err("Quoted value required as argument".to_owned())
                }
            }
        },
        // bind a value to a name, and return the value: (bind (quote another_true) true)
        "set" => {
            if args.len() != 2 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(q) = args[0].clone() {
                    if let Value::Idfr(n) = (*q).clone() {
                        intp.bind(n, args[1].clone());
                        Ok(args[1].clone())
                    } else {
                        Err("Must bind to a quoted identifier".to_owned())
                    }
                } else {
                    Err("Must bind to a quoted identifier".to_owned())
                }
            }
        },
        // delete a symbol from namespace: (del (quote value_name))
        "del" => {
            if args.len() != 1 {
                Err("Arg count".to_owned())
            } else {
                if let Value::Quot(q) = args[0].clone() {
                    if let Value::Idfr(n) = (*q).clone() {
                        intp.delete(&n);
                        Ok(Value::Unit)
                    } else {
                        Err("Can only delete a quoted identifier".to_owned())
                    }
                } else {
                    Err("Can only delete a quoted identifier".to_owned())
                }
            }
        },
        // print the arguments and a line break, returning Unit
        "println" => {
            println!("{}", args.iter().map(Value::format).collect::<Vec<_>>().join(" "));
            Ok(Value::Unit)
        },
        n => Err(format!("Function {} is not yet defined", n)),
    }
}
