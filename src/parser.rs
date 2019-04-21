/// Language token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// Any non-paren word
    Symbol(String),
}

/// State of the split state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitMode {
    /// Nothing currently in-progress
    Start,
    /// Append to previous token (current symbol) if possible
    Symbol,
    /// Comment, remove all until EOL
    Comment,
}

/// Split source code to tokens, drops comments
pub fn split_tokens(s: String) -> Result<Vec<Token>, ()> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut mode = SplitMode::Start;
    for c in s.chars() {
        let (new_mode, new_token) = match mode {
            SplitMode::Start | SplitMode::Symbol => match c {
                ' ' => (SplitMode::Start, None),
                '\n' => (SplitMode::Start, None),
                '(' => (SplitMode::Start, Some(Token::OpenParen)),
                ')' => (SplitMode::Start, Some(Token::CloseParen)),
                '#' => (SplitMode::Comment, None),
                _ => {
                    if mode == SplitMode::Start {
                        (SplitMode::Symbol, Some(Token::Symbol(c.to_string())))
                    } else {
                        // Append to previous symbol
                        if let Token::Symbol(ref mut s) = tokens.last_mut().expect("Invalid state") {
                            s.push(c);
                        } else {
                            panic!("Invalid state");
                        }
                        (SplitMode::Symbol, None)
                    }
                },
            },
            SplitMode::Comment => match c {
                '\n' => (SplitMode::Start, None),
                _ => (SplitMode::Comment, None),
            },
        };

        mode = new_mode;
        if let Some(token) = new_token {
            tokens.push(token);
        }
    }

    Ok(tokens)
}

pub fn take_expr(tokens: Vec<Token>) -> Result<(Vec<Token>, Vec<Token>), String> {
    if tokens.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    match tokens.first().unwrap() {
        Token::OpenParen => {
            let mut depth: usize = 1;
            let mut index: usize = 0;
            for (i, t) in tokens.iter().enumerate().skip(1) {
                if t == &Token::OpenParen {
                    depth += 1;
                } else if t == &Token::CloseParen {
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
        Token::CloseParen => Err("Unbalanced (start)".to_owned()),
        other => Ok((vec![other.clone()], tokens[1..].to_vec())),
    }
}
