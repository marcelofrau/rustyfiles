//! Small CSS tokenizer for GTK stylesheets. Whitespace is kept so selectors can
//! distinguish descendant combinators from adjacent compounds.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    At(String),
    Hash(String),
    String(String),
    Number(f32),
    Dimension(f32, String),
    Colon,
    Dot,
    Semicolon,
    Comma,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Greater,
    Plus,
    Tilde,
    Star,
    Bang,
    Whitespace,
    Delim(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexedToken {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

/// Tokenize GTK CSS. Comments become whitespace; malformed strings end at EOF
/// instead of aborting the whole theme.
pub fn lex(input: &str) -> Vec<LexedToken> {
    let b = input.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    while i < b.len() {
        let c = b[i];

        if c == b'\n' {
            i += 1;
            line += 1;
            col = 1;
            out.push(LexedToken { token: Token::Whitespace, line, col });
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            col += 1;
            out.push(LexedToken { token: Token::Whitespace, line, col });
            continue;
        }
        if c == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            col += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                if b[i] == b'\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                i += 1;
            }
            if i + 1 < b.len() {
                i += 2;
                col += 2;
            }
            out.push(LexedToken { token: Token::Whitespace, line, col });
            continue;
        }

        let start_line = line;
        let start_col = col;

        if c == b'"' || c == b'\'' {
            let quote = c;
            i += 1;
            col += 1;
            let mut s = Vec::new();
            while i < b.len() {
                let ch = b[i];
                if ch == quote {
                    i += 1;
                    col += 1;
                    break;
                }
                if ch == b'\\' && i + 1 < b.len() {
                    let escaped = b[i + 1];
                    s.push(escaped);
                    i += 2;
                    col += 2;
                    if escaped == b'\n' {
                        line += 1;
                        col = 1;
                    }
                    continue;
                }
                if ch == b'\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                s.push(ch);
                i += 1;
            }
            out.push(LexedToken {
                token: Token::String(String::from_utf8_lossy(&s).into_owned()),
                line: start_line,
                col: start_col,
            });
            continue;
        }

        if c == b'@' {
            i += 1;
            col += 1;
            let name = take_ident(b, &mut i, &mut col);
            out.push(LexedToken { token: Token::At(name), line: start_line, col: start_col });
            continue;
        }

        if c == b'#' {
            i += 1;
            col += 1;
            let name = take_ident(b, &mut i, &mut col);
            out.push(LexedToken { token: Token::Hash(name), line: start_line, col: start_col });
            continue;
        }

        if starts_number(b, i) {
            let (value, unit) = take_number(b, &mut i, &mut col);
            let token = if unit.is_empty() { Token::Number(value) } else { Token::Dimension(value, unit) };
            out.push(LexedToken { token, line: start_line, col: start_col });
            continue;
        }

        if is_ident_start(c) {
            let name = take_ident(b, &mut i, &mut col);
            out.push(LexedToken { token: Token::Ident(name), line: start_line, col: start_col });
            continue;
        }

        let token = match c {
            b':' => Token::Colon,
            b'.' => Token::Dot,
            b';' => Token::Semicolon,
            b',' => Token::Comma,
            b'{' => Token::LBrace,
            b'}' => Token::RBrace,
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'[' => Token::LBracket,
            b']' => Token::RBracket,
            b'>' => Token::Greater,
            b'+' => Token::Plus,
            b'~' => Token::Tilde,
            b'*' => Token::Star,
            b'!' => Token::Bang,
            _ => Token::Delim(c as char),
        };
        i += 1;
        col += 1;
        out.push(LexedToken { token, line: start_line, col: start_col });
    }
    out
}

fn starts_number(b: &[u8], i: usize) -> bool {
    let c = b[i];
    if c.is_ascii_digit() {
        return true;
    }
    if c == b'.' {
        return i + 1 < b.len() && b[i + 1].is_ascii_digit();
    }
    if c == b'-' || c == b'+' {
        return i + 1 < b.len()
            && (b[i + 1].is_ascii_digit()
                || (b[i + 1] == b'.' && i + 2 < b.len() && b[i + 2].is_ascii_digit()));
    }
    false
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'-' || c >= 0x80
}

fn is_ident_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c >= 0x80
}

fn take_ident(b: &[u8], i: &mut usize, col: &mut usize) -> String {
    let start = *i;
    while *i < b.len() && is_ident_char(b[*i]) {
        *i += 1;
        *col += 1;
    }
    String::from_utf8_lossy(&b[start..*i]).into_owned()
}

fn take_number(b: &[u8], i: &mut usize, col: &mut usize) -> (f32, String) {
    let start = *i;
    if b[*i] == b'-' || b[*i] == b'+' {
        *i += 1;
        *col += 1;
    }
    while *i < b.len() && b[*i].is_ascii_digit() {
        *i += 1;
        *col += 1;
    }
    if *i < b.len() && b[*i] == b'.' {
        *i += 1;
        *col += 1;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
            *col += 1;
        }
    }
    if *i < b.len() && (b[*i] == b'e' || b[*i] == b'E') {
        let mut j = *i + 1;
        if j < b.len() && (b[j] == b'-' || b[j] == b'+') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            *col += j - *i;
            *i = j;
        }
    }
    let text = String::from_utf8_lossy(&b[start..*i]);
    let number = text.parse::<f32>().unwrap_or(0.0);

    let unit_start = *i;
    while *i < b.len() && (is_ident_char(b[*i]) || b[*i] == b'%') {
        *i += 1;
        *col += 1;
    }
    let unit = String::from_utf8_lossy(&b[unit_start..*i]).into_owned();
    (number, unit)
}

/// Rebuild source-like text from tokens, used for definitions and raw values.
pub fn text(tokens: &[LexedToken]) -> String {
    let mut out = String::new();
    for (index, item) in tokens.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        match &item.token {
            Token::Ident(s) | Token::At(s) | Token::String(s) => out.push_str(s),
            Token::Hash(s) => {
                out.push('#');
                out.push_str(s);
            }
            Token::Number(n) => out.push_str(&n.to_string()),
            Token::Dimension(n, unit) => {
                out.push_str(&n.to_string());
                out.push_str(unit);
            }
            Token::Colon => out.push(':'),
            Token::Dot => out.push('.'),
            Token::Semicolon => out.push(';'),
            Token::Comma => out.push(','),
            Token::LBrace => out.push('{'),
            Token::RBrace => out.push('}'),
            Token::LParen => out.push('('),
            Token::RParen => out.push(')'),
            Token::LBracket => out.push('['),
            Token::RBracket => out.push(']'),
            Token::Greater => out.push('>'),
            Token::Plus => out.push('+'),
            Token::Tilde => out.push('~'),
            Token::Star => out.push('*'),
            Token::Bang => out.push('!'),
            Token::Whitespace => out.push(' '),
            Token::Delim(c) => out.push(*c),
        }
    }
    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_comments_strings_and_escapes() {
        let tokens = lex("/* note */ button::after { content: \"a\\\"b\"; }");
        let values: Vec<_> = tokens.iter().map(|t| t.token.clone()).collect();
        assert!(values.contains(&Token::Ident("button".into())));
        assert!(values.contains(&Token::String("a\"b".into())));
        assert!(values.iter().filter(|t| **t == Token::Whitespace).count() >= 2);
    }

    #[test]
    fn handles_at_rules_hex_and_dimensions() {
        let tokens = lex("@define-color rf_fg #11223380;\nmargin: 0.5em 4px !important;");
        let values: Vec<_> = tokens.iter().map(|t| t.token.clone()).collect();
        assert!(values.contains(&Token::At("define-color".into())));
        assert!(values.contains(&Token::Hash("11223380".into())));
        assert!(values.contains(&Token::Dimension(0.5, "em".into())));
        assert!(values.contains(&Token::Dimension(4.0, "px".into())));
        assert!(values.contains(&Token::Bang));
        assert!(tokens.iter().any(|t| t.line == 2));
    }

    #[test]
    fn handles_leading_dot_numbers() {
        let tokens = lex("rgba(0, 0, 0, .2) translate(-.5px)");
        let values: Vec<_> = tokens.iter().map(|t| t.token.clone()).collect();
        assert!(values.contains(&Token::Number(0.2)));
        assert!(values.contains(&Token::Dimension(-0.5, "px".into())));
    }

    #[test]
    fn handles_functions_and_pseudo_selectors() {
        let tokens = lex("button:not(.flat) { background: rgba(0, 0, 0, 0.5); }");
        let values: Vec<_> = tokens.iter().map(|t| t.token.clone()).collect();
        assert!(values.contains(&Token::Ident("not".into())));
        assert!(values.contains(&Token::Ident("rgba".into())));
        assert!(values.contains(&Token::LParen));
        assert!(values.contains(&Token::RParen));
    }
}
