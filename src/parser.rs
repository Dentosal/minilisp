pub fn split_tokens(s: String) -> Result<Vec<String>, ()> {
    let mut parts: Vec<String> = Vec::new();
    for c in s.chars() {
        if c == ' ' {
            parts.push(String::new());
        } else if c == '(' || c == ')' {
            parts.push(c.to_string());
        } else if parts
            .last()
            .map(|p| p == "" || p == "(" || p == ")")
            .unwrap_or(true)
        {
            parts.push(c.to_string());
        } else {
            let mut newt = parts.pop().unwrap();
            newt.push(c);
            parts.push(newt);
        }
    }
    Ok(parts.iter().cloned().filter(|t| !t.is_empty()).collect())
}

pub fn take_expr(tokens: Vec<String>) -> Result<(Vec<String>, Vec<String>), String> {
    if tokens.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    match tokens.first().unwrap().as_str() {
        "(" => {
            let mut depth: usize = 1;
            let mut index: usize = 0;
            for (i, t) in tokens.iter().enumerate().skip(1) {
                if t == "(" {
                    depth += 1;
                } else if t == ")" {
                    depth -= 1;
                    if depth == 0 {
                        index = i + 1;
                        break;
                    }
                }
            }
            if depth > 0 {
                Err("Unbalanced (end)".to_owned())
            } else {
                debug_assert!(index > 0);
                Ok((tokens[0..index].to_vec(), tokens[index..].to_vec()))
            }
        },
        ")" => Err("Unbalanced (start)".to_owned()),
        ident => Ok((vec![ident.to_owned()], tokens[1..].to_vec())),
    }
}
