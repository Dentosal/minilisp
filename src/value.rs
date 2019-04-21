use super::parser;
use std::fmt;

/// A concrete run-time value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Unit type (empty tuple)
    Unit,
    /// Identifier
    Idfr(String),
    /// Expression
    /// Empty expressions resolve to the unit type, and is not allowed here
    Expr(Vec<Value>),
    /// Builtin function (a black box)
    Bltn(String),
    /// No-evaluate marker
    Quot(Box<Value>),
    /// Lambda (parameter substitution)
    Lmbd(Vec<String>, Box<Value>),
}
impl Value {
    /// Expects a single value (expression) already checked syntactically valid
    pub fn parse(mut tokens: Vec<parser::Token>) -> Result<Self, String> {
        assert!(!tokens.is_empty());
        if tokens[0] == parser::Token::OpenParen {
            let mut args = Vec::new();
            tokens = tokens[1..tokens.len() - 1].to_vec();
            while !tokens.is_empty() {
                let (t, ts) = parser::take_expr(tokens).unwrap();
                tokens = ts;
                args.push(Self::parse(t)?);
            }
            if args.is_empty() {
                Ok(Value::Unit)
            } else {
                Ok(Value::Expr(args))
            }
        } else {
            assert_eq!(tokens.len(), 1);
            if let parser::Token::Symbol(sym) = tokens[0].clone() {
                Ok(Value::Idfr(sym))
            } else {
                panic!("Invalid data passed to value");
            }
        }
    }

    /// Substitute symbol (identifier) with a value
    pub fn replace(self, sym: &str, val: Self) -> Self {
        match self {
            Value::Unit => Value::Unit,
            Value::Idfr(n) if n == sym => val.clone(),
            Value::Quot(q) => q.replace(sym, val),
            Value::Expr(e) => Value::Expr(
                e.into_iter()
                    .map(|q| q.replace(sym, val.clone()))
                    .collect::<Vec<_>>(),
            ),
            Value::Lmbd(a, b) => {
                // skip shadowed parameters
                if a.contains(&sym.to_owned()) {
                    Value::Lmbd(a, b)
                } else {
                    Value::Lmbd(a, box b.replace(sym, val))
                }
            },
            a => a,
        }
    }

    /// Human-readable form
    pub fn format(&self) -> String {
        match self {
            Value::Unit => "Unit".to_owned(),
            Value::Idfr(n) => format!(":{}", n.clone()),
            Value::Bltn(n) => format!("#{}", n.clone()),
            Value::Quot(q) => format!("'{}", q.format()),
            Value::Expr(e) => format!("({})", e.iter().map(Value::format).collect::<Vec<_>>().join(" ")),
            Value::Lmbd(a, b) => format!("(\\ {} -> {})", a.join(" "), b.format()),
        }
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}
