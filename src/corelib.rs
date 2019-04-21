use super::{Interpreter, Value};

pub const BUILTINS: [&str; 21] = [
    // Core language
    "quote",
    "unquote",
    "discard",
    "assert",
    "lambda",
    "block",
    "branch",
    // Equality check
    "eqtree?",
    // Namespace operators
    "set",
    // Operations on quoted expressions
    "q:append",
    "q:prepend",
    "q:first",
    "q:head",
    "q:tail",
    "q:last",
    "q:init",
    "q:rot",
    "q:rotf",
    "q:expr?",
    // I/O
    "println",
    // Integer operations
    "dec",
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
pub fn call(intp: &mut Interpreter, name: String, args: Vec<Value>) -> Option<Result<Value, String>> {
    match name.as_str() {
        // decrement
        "dec" => {
            if args.len() != 1 {
                return Some(Err("Arg count".to_owned()));
            }
            if let Value::Quot(box Value::Idfr(e)) = args[0].clone() {
                if let Ok(mut v) = e.parse::<u32>() {
                    v -= 1;
                    Some(Ok(Value::Quot(box Value::Idfr(v.to_string()))))
                } else {
                    Some(Ok(Value::Unit))
                }
            } else {
                Some(Ok(Value::Unit))
            }
        },
        // evaluate quoted expression
        "unquote" => {
            if args.len() != 1 {
                return Some(Err("Arg count".to_owned()));
            }
            if let Value::Quot(e) = args[0].clone() {
                Some(Ok(*e))
            } else {
                Some(Err(format!(
                    "Only quote can be unquoted, {:?} is invalid",
                    args[0]
                )))
            }
        },
        // discard an expression
        "discard" => Some(Ok(Value::Unit)),
        // throws error if the paramter is Unit. returns original value
        "assert" => {
            if args.len() != 1 {
                return Some(Err("Arg count".to_owned()));
            }

            if args[0] == Value::Unit {
                Some(Err("Assertion failed".to_owned()))
            } else {
                Some(Ok(args[0].clone()))
            }
        },
        // anonymous function, i.e. parameter substitution
        "lambda" => {
            let params_r: Result<Vec<String>, String> = args[..args.len() - 1]
                .iter()
                .cloned()
                .map(|a| match a {
                    Value::Quot(box Value::Idfr(n)) => Ok(n),
                    _ => Err("Quoted identifier required as lambda parameters".to_owned()),
                })
                .collect();

            let params = match params_r {
                Ok(v) => v,
                Err(e) => {
                    return Some(Err(e));
                },
            };

            let body = args[args.len() - 1].clone();

            Some(Ok(Value::Lmbd(params, box body)))
        },
        // unquote a list of statements sequentially, returning the last result
        "block" => {
            let mut res: Value = Value::Unit;
            for a in args {
                res = match intp.execute(Value::Expr(vec![Value::Bltn("unquote".to_owned()), a.clone()])) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                }
            }
            Some(Ok(res))
        },
        // if-else condition select, unquotes the relevant side
        "branch" => {
            if args.len() != 3 {
                Some(Err("Arg count".to_owned()))
            } else {
                if args[0] != Value::Unit {
                    // true branch
                    Some(Ok(Value::Expr(vec![
                        Value::Bltn("unquote".to_owned()),
                        args[1].clone(),
                    ])))
                } else {
                    // false branch
                    Some(Ok(Value::Expr(vec![
                        Value::Bltn("unquote".to_owned()),
                        args[2].clone(),
                    ])))
                }
            }
        },
        // test exact structure equality
        "eqtree?" => {
            if args.len() != 2 {
                Some(Err("Arg count".to_owned()))
            } else {
                Some(Ok(boolvalue!(args[0] == args[1])))
            }
        },
        // test if the top-level item in quotes is an expression
        "q:expr?" => {
            if args.len() != 1 {
                Some(Err("Arg count".to_owned()))
            } else {
                Some(Ok(match args[0] {
                    Value::Unit | Value::Expr(_) => boolvalue!(true),
                    _ => boolvalue!(false),
                }))
            }
        },
        // bind a value to a name, and return the value: (bind (quote another_true) true)
        "set" => {
            if args.len() != 2 {
                Some(Err("Arg count".to_owned()))
            } else {
                if let Value::Quot(q) = args[0].clone() {
                    if let Value::Idfr(n) = (*q).clone() {
                        intp.bind(n, args[1].clone());
                        Some(Ok(args[1].clone()))
                    } else {
                        Some(Err("Must bind to a quoted identifier".to_owned()))
                    }
                } else {
                    Some(Err("Must bind to a quoted identifier".to_owned()))
                }
            }
        },
        // print the arguments and a line break, returning Unit
        "println" => {
            println!("{}", args.iter().map(Value::format).collect::<Vec<_>>().join(" "));
            Some(Ok(Value::Unit))
        },
        _ => None,
    }
}
