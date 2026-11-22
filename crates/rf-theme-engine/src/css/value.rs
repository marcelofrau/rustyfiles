//! CSS value parser. Values become structured data where useful and keep the
//! original text otherwise, so M2 can implement inheritance without losing data.

use std::collections::HashMap;

use super::lexer::{LexedToken, Token, lex, text as tokens_text};
use crate::model::{Color, ColorStop, Edges, Gradient, GradientKind, Length, LengthUnit, Shadow};

/// Colors from `@define-color` and custom properties from `:root`.
#[derive(Debug, Clone, Default)]
pub struct Definitions {
    pub colors: HashMap<String, String>,
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Color(Color),
    Length(Length),
    Number(f32),
    Ident(String),
    String(String),
    Url(String),
    Gradient(Gradient),
    Shadow(Shadow),
    Edges(Edges),
    Function(Function),
    List(Vec<Value>),
    Inherit,
    Initial,
    Unset,
    Raw(String),
}

/// Parse a declaration value, using the property name for shorthands.
pub fn parse_for_property(tokens: &[LexedToken], property: &str, defs: &Definitions) -> Value {
    let expanded = expand(tokens, defs, 0);
    match property {
        "padding" | "margin" | "border-spacing" => parse_edges(&expanded).map_or_else(|| generic(&expanded), Value::Edges),
        "box-shadow" | "text-shadow" | "-gtk-icon-shadow" => parse_shadows(&expanded),
        _ => generic(&expanded),
    }
}

/// Replace `@color`, `var(--name)` and their fallbacks with definition text.
pub fn expand(tokens: &[LexedToken], defs: &Definitions, depth: u8) -> Vec<LexedToken> {
    if depth > 4 || defs.colors.is_empty() && defs.custom.is_empty() {
        return tokens.to_vec();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Token::At(name) = &tokens[i].token
            && let Some(raw) = defs.colors.get(name)
        {
            let inner = lex(raw);
            out.extend(expand(&inner, defs, depth + 1));
            i += 1;
            continue;
        }
        if matches!(&tokens[i].token, Token::Ident(name) if name == "var") && i + 1 < tokens.len() && tokens[i + 1].token == Token::LParen {
            let end = skip_balanced(tokens, i + 1);
            let args = &tokens[i + 2..end.saturating_sub(1)];
            let parts = split_top_level(args);
            let name = parts.first().and_then(|p| p.first()).and_then(|t| match &t.token {
                Token::String(s) | Token::Ident(s) => Some(s.trim_start_matches('-').to_owned()),
                _ => None,
            });
            if let Some(name) = name {
                if let Some(raw) = defs.custom.get(&name) {
                    out.extend(expand(&lex(raw), defs, depth + 1));
                } else if parts.len() > 1 {
                    out.extend(expand(parts[1], defs, depth + 1));
                }
            }
            i = end;
            continue;
        }
        out.push(tokens[i].clone());
        i += 1;
    }
    out
}

fn generic(tokens: &[LexedToken]) -> Value {
    let tokens = strip_ws(tokens);
    let (tokens, important) = strip_important(tokens);
    if tokens.is_empty() {
        return Value::Raw(String::new());
    }
    let _ = important; // `!important` affects the cascade, not the value itself.
    let parts = split_top_level(tokens);
    if parts.len() > 1 {
        return Value::List(parts.iter().map(|p| generic(p)).collect());
    }
    if is_function(tokens) {
        let name = match &tokens[0].token {
            Token::Ident(name) => name.clone(),
            _ => unreachable!(),
        };
        let end = skip_balanced(tokens, 1);
        return parse_function(&name, &tokens[2..end.saturating_sub(1)]);
    }
    let parts = split_whitespace_top_level(tokens);
    if parts.len() > 1 {
        return Value::List(parts.iter().map(|p| generic(p)).collect());
    }
    parse_simple(tokens)
}

fn is_function(tokens: &[LexedToken]) -> bool {
    matches!(&tokens[0].token, Token::Ident(_))
        && tokens.get(1).is_some_and(|t| t.token == Token::LParen)
        && skip_balanced(tokens, 1) == tokens.len()
}

fn split_whitespace_top_level(tokens: &[LexedToken]) -> Vec<&[LexedToken]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    for (index, item) in tokens.iter().enumerate() {
        match item.token {
            Token::LParen => paren += 1,
            Token::RParen => paren -= 1,
            Token::LBracket => bracket += 1,
            Token::RBracket => bracket -= 1,
            Token::Whitespace if paren == 0 && bracket == 0 => {
                let part = strip_ws(&tokens[start..index]);
                if !part.is_empty() {
                    parts.push(part);
                }
                start = index + 1;
            }
            _ => {}
        }
    }
    let last = strip_ws(&tokens[start..]);
    if !last.is_empty() {
        parts.push(last);
    }
    parts
}

fn parse_simple(tokens: &[LexedToken]) -> Value {
    if tokens.len() != 1 {
        return Value::Raw(tokens_text(tokens));
    }
    match &tokens[0].token {
        Token::Hash(s) => Color::parse_hex(s).map_or_else(|| Value::Raw(s.clone()), Value::Color),
        Token::String(s) => Value::String(s.clone()),
        Token::Ident(s) => match s.as_str() {
            "inherit" => Value::Inherit,
            "initial" => Value::Initial,
            "unset" => Value::Unset,
            _ => Color::parse_named(s).map_or_else(|| Value::Ident(s.clone()), Value::Color),
        },
        Token::Number(n) => Value::Number(*n),
        Token::Dimension(n, unit) => LengthUnit::parse(unit)
            .map_or_else(|| Value::Raw(tokens_text(tokens)), |unit| Value::Length(Length::new(*n, unit))),
        _ => Value::Raw(tokens_text(tokens)),
    }
}

fn parse_function(name: &str, args: &[LexedToken]) -> Value {
    if name == "url"
        && let Some(Token::String(path)) = args.first().map(|t| &t.token)
    {
        return Value::Url(path.clone());
    }
    if name.ends_with("-gradient") {
        return parse_gradient(name, args);
    }
    if (name == "rgb" || name == "rgba")
        && let Some(color) = parse_rgba(args)
    {
        return Value::Color(color);
    }
    Value::Function(Function { name: name.to_owned(), args: split_top_level(args).iter().map(|p| generic(p)).collect() })
}

fn parse_gradient(name: &str, args: &[LexedToken]) -> Value {
    let kind = if name.starts_with("linear") { GradientKind::Linear } else { GradientKind::Radial };
    let parts = split_top_level(args);
    if parts.is_empty() {
        return Value::Raw(format!("{name}()"));
    }
    let angle = parts[0].iter().find_map(|t| match &t.token {
        Token::Dimension(v, unit) if unit == "deg" => Some(Length::new(*v, LengthUnit::Deg)),
        _ => None,
    });
    let stop_parts = if parts[0].iter().any(|t| matches!(&t.token, Token::Ident(s) if s == "to") || angle.is_some()) {
        &parts[1..]
    } else {
        &parts[..]
    };
    let stops = stop_parts
        .iter()
        .map(|part| ColorStop { color: color_from_tokens(part), position: length_from_tokens(part) })
        .collect();
    Value::Gradient(Gradient { kind, angle, stops })
}

fn parse_rgba(args: &[LexedToken]) -> Option<Color> {
    let parts = split_top_level(args);
    if !(3..=4).contains(&parts.len()) {
        return None;
    }
    let numbers: Vec<f32> = parts
        .iter()
        .map(|part| match &part.first()?.token {
            Token::Number(n) => Some(*n),
            Token::Dimension(n, unit) if unit == "%" => Some(n / 100.0),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let alpha = numbers.get(3).copied().unwrap_or(1.0);
    Some(Color::from_rgba(numbers[0], numbers[1], numbers[2], alpha))
}

fn parse_edges(tokens: &[LexedToken]) -> Option<Edges> {
    let mut lengths = Vec::new();
    for item in tokens.iter().filter(|t| t.token != Token::Whitespace) {
        let length = match &item.token {
            Token::Dimension(v, unit) => Length::new(*v, LengthUnit::parse(unit)?),
            Token::Number(v) => Length::new(*v, LengthUnit::Px),
            _ => return None,
        };
        lengths.push(length);
    }
    Edges::from_lengths(&lengths)
}

fn parse_shadows(tokens: &[LexedToken]) -> Value {
    let parts = split_top_level(tokens);
    Value::List(parts.iter().map(|part| parse_shadow(part)).collect())
}

fn parse_shadow(tokens: &[LexedToken]) -> Value {
    let mut lengths = Vec::new();
    let mut color = None;
    let mut inset = false;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i].token {
            Token::Whitespace | Token::Bang => i += 1,
            Token::Ident(s) if s == "inset" => {
                inset = true;
                i += 1;
            }
            Token::Ident(s) if s == "important" => i += 1,
            Token::Hash(_) | Token::Ident(_) => {
                if let Some(found) = color_from_tokens(&tokens[i..]) {
                    color = Some(found);
                }
                i += 1;
            }
            Token::Number(v) => {
                lengths.push(Length::new(*v, LengthUnit::Px));
                i += 1;
            }
            Token::Dimension(v, unit) => {
                if let Some(unit) = LengthUnit::parse(unit) {
                    lengths.push(Length::new(*v, unit));
                }
                i += 1;
            }
            Token::LParen => {
                if let Some(found) = color_from_tokens(&tokens[i..]) {
                    color = Some(found);
                }
                i = skip_balanced(tokens, i);
            }
            _ => i += 1,
        }
    }
    let mut lengths = lengths.into_iter();
    Value::Shadow(Shadow {
        offset_x: lengths.next().unwrap_or_else(Length::zero),
        offset_y: lengths.next().unwrap_or_else(Length::zero),
        blur: lengths.next(),
        spread: lengths.next(),
        color,
        inset,
    })
}

fn color_from_tokens(tokens: &[LexedToken]) -> Option<Color> {
    for (index, item) in tokens.iter().enumerate() {
        match &item.token {
            Token::Hash(s) => {
                if let Some(color) = Color::parse_hex(s) {
                    return Some(color);
                }
            }
            Token::Ident(s) => {
                if let Some(color) = Color::parse_named(s) {
                    return Some(color);
                }
                if (s == "rgb" || s == "rgba") && tokens.get(index + 1).is_some_and(|t| t.token == Token::LParen) {
                    let end = skip_balanced(tokens, index + 1);
                    if let Some(color) = parse_rgba(&tokens[index + 2..end.saturating_sub(1)]) {
                        return Some(color);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn length_from_tokens(tokens: &[LexedToken]) -> Option<Length> {
    tokens.iter().find_map(|item| match &item.token {
        Token::Dimension(v, unit) => LengthUnit::parse(unit).map(|unit| Length::new(*v, unit)),
        Token::Number(v) => Some(Length::new(*v, LengthUnit::Percent)),
        _ => None,
    })
}

fn strip_ws(tokens: &[LexedToken]) -> &[LexedToken] {
    let mut start = 0;
    let mut end = tokens.len();
    while start < end && tokens[start].token == Token::Whitespace {
        start += 1;
    }
    while end > start && tokens[end - 1].token == Token::Whitespace {
        end -= 1;
    }
    &tokens[start..end]
}

fn strip_important(tokens: &[LexedToken]) -> (&[LexedToken], bool) {
    let mut end = tokens.len();
    while end > 0 && tokens[end - 1].token == Token::Whitespace {
        end -= 1;
    }
    if end >= 2
        && tokens[end - 1].token == Token::Ident("important".into())
        && tokens[end - 2].token == Token::Bang
    {
        return (&tokens[..end - 2], true);
    }
    (tokens, false)
}

fn split_top_level(tokens: &[LexedToken]) -> Vec<&[LexedToken]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    for (i, item) in tokens.iter().enumerate() {
        match item.token {
            Token::LParen => paren += 1,
            Token::RParen => paren -= 1,
            Token::LBracket => bracket += 1,
            Token::RBracket => bracket -= 1,
            Token::Comma if paren == 0 && bracket == 0 => {
                parts.push(strip_ws(&tokens[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(strip_ws(&tokens[start..]));
    parts
}

fn skip_balanced(tokens: &[LexedToken], start: usize) -> usize {
    let mut depth = 0;
    let mut i = start;
    while i < tokens.len() {
        match tokens[i].token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(s: &str, property: &str, defs: &Definitions) -> Value {
        let tokens = lex(s);
        parse_for_property(&tokens, property, defs)
    }

    #[test]
    fn parses_colors_lengths_and_edges() {
        assert_eq!(value("#abc", "color", &Definitions::default()), Value::Color(Color::new(170, 187, 204, 255)));
        assert_eq!(
            value("rgba(0, 0, 0, 50%)", "color", &Definitions::default()),
            Value::Color(Color::new(0, 0, 0, 128))
        );
        let edges = match value("4px 8px", "padding", &Definitions::default()) {
            Value::Edges(e) => e,
            other => panic!("expected edges, got {other:?}"),
        };
        assert_eq!(edges.top.value, 4.0);
        assert_eq!(edges.left.value, 8.0);
    }

    #[test]
    fn splits_whitespace_separated_shorthands() {
        match value("1px solid #fff", "border", &Definitions::default()) {
            Value::List(parts) => assert_eq!(parts.len(), 3),
            other => panic!("expected list, got {other:?}"),
        }
        match value("linear-gradient(#fff, #000) 4px", "background", &Definitions::default()) {
            Value::List(parts) => {
                assert_eq!(parts.len(), 2);
                assert!(matches!(parts[0], Value::Gradient(_)));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn parses_gradient_url_shadow_and_functions() {
        assert!(matches!(value("linear-gradient(to bottom, #fff 0%, #000 100%)", "background-image", &Definitions::default()), Value::Gradient(_)));
        assert_eq!(value("url(\"assets/border.png\")", "border-image-source", &Definitions::default()), Value::Url("assets/border.png".into()));
        assert!(matches!(value("0 2px 8px rgba(0,0,0,.2)", "box-shadow", &Definitions::default()), Value::List(_)));
        assert!(matches!(value("shade(@rf_bg, 0.9)", "color", &Definitions::default()), Value::Function(_)));
    }

    #[test]
    fn expands_define_color_and_var() {
        let defs = Definitions {
            colors: HashMap::from([("rf_bg".into(), "#123456".into())]),
            custom: HashMap::from([("accent".into(), "orange".into())]),
        };
        assert_eq!(value("@rf_bg", "color", &defs), Value::Color(Color::new(18, 52, 86, 255)));
        assert_eq!(value("var(--accent, blue)", "color", &defs), Value::Color(Color::new(255, 165, 0, 255)));
    }
}
