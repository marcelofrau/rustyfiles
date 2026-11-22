//! GTK CSS ruleset parser with error recovery. One bad rule never drops later rules.

use std::collections::HashSet;

use super::lexer::{LexedToken, Token, lex, text as tokens_text};
use super::selector::{self, ComplexSelector};
use super::value::{self, Definitions, Value};
use super::{ParseIssue, warn_issue};
use crate::style;

#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: Value,
    pub important: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<ComplexSelector>,
    pub declarations: Vec<Declaration>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct KeyframeStep {
    pub name: String,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Keyframes {
    pub name: String,
    pub steps: Vec<KeyframeStep>,
    pub line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
    pub imports: Vec<Import>,
    pub keyframes: Vec<Keyframes>,
    pub issues: Vec<ParseIssue>,
}

/// Parse one CSS file. `file` is only used in issue messages.
pub fn parse(input: &str, file: &str) -> Stylesheet {
    let tokens = lex(input);
    let mut issues = Vec::new();
    let mut defs = Definitions::default();
    collect_definitions(&tokens, &mut defs, &mut issues, file);

    let mut out = Stylesheet::default();
    let mut reported_unknown = HashSet::new();
    let mut i = 0;
    while i < tokens.len() {
        i = skip_ws(&tokens, i);
        if i >= tokens.len() {
            break;
        }
        match &tokens[i].token {
            Token::At(name) if name == "import" => i = parse_import(&tokens, i, &mut out, &mut issues, file),
            Token::At(name) if name == "define-color" => {
                let end = skip_statement(&tokens, i);
                i = if end < tokens.len() && tokens[end].token == Token::Semicolon { end + 1 } else { end.max(i + 1) };
            }
            Token::At(name) if name == "keyframes" => {
                let (frames, next) = parse_keyframes(&tokens, i, &defs, &mut issues, &mut reported_unknown, file);
                if let Some(frames) = frames {
                    out.keyframes.push(frames);
                }
                i = next;
            }
            Token::At(name) => {
                warn_issue(&mut issues, file, tokens[i].line, format!("unsupported at-rule `@{name}`"));
                i = skip_at_rule(&tokens, i);
            }
            Token::Colon if matches!(tokens.get(i + 1).map(|t| &t.token), Some(Token::Ident(name)) if name == "root") => {
                i = skip_rule(&tokens, i);
            }
            _ => {
                let (rule, next) = parse_rule(&tokens, i, &defs, &mut issues, &mut reported_unknown, file);
                if let Some(rule) = rule {
                    out.rules.push(rule);
                }
                i = next.max(i + 1);
            }
        }
    }
    out.issues = issues;
    out
}

fn collect_definitions(tokens: &[LexedToken], defs: &mut Definitions, issues: &mut Vec<ParseIssue>, file: &str) {
    let mut i = 0;
    while i < tokens.len() {
        i = skip_ws(tokens, i);
        if let Some(Token::At(name)) = tokens.get(i).map(|t| &t.token)
            && name == "define-color"
        {
            let line = tokens[i].line;
            i += 1;
            i = skip_ws(tokens, i);
            let name = match tokens.get(i).map(|t| &t.token) {
                Some(Token::Ident(name)) => name.clone(),
                _ => {
                    warn_issue(issues, file, line, "`@define-color` without a name");
                    i = skip_statement(tokens, i).max(i + 1);
                    continue;
                }
            };
            i += 1;
            let start = i;
            while i < tokens.len() && !matches!(tokens[i].token, Token::Semicolon | Token::RBrace) {
                i += 1;
            }
            let raw = tokens_text(&tokens[start..i]);
            if raw.is_empty() {
                warn_issue(issues, file, line, format!("`@define-color {name}` has no value"));
            } else {
                defs.colors.insert(name, raw);
            }
            if i < tokens.len() && tokens[i].token == Token::Semicolon {
                i += 1;
            }
            continue;
        }
        if matches!(tokens.get(i).map(|t| &t.token), Some(Token::Colon))
            && matches!(tokens.get(i + 1).map(|t| &t.token), Some(Token::Ident(name)) if name == "root")
        {
            i += 2;
            i = skip_ws(tokens, i);
            if i < tokens.len() && tokens[i].token == Token::LBrace {
                let end = matching_brace(tokens, i).unwrap_or(tokens.len());
                collect_custom_properties(&tokens[i + 1..end], defs);
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
}

fn collect_custom_properties(body: &[LexedToken], defs: &mut Definitions) {
    let mut i = 0;
    while i < body.len() {
        i = skip_ws(body, i);
        let Some(Token::Ident(name)) = body.get(i).map(|t| &t.token) else {
            i += 1;
            continue;
        };
        if !name.starts_with("--") {
            i += 1;
            continue;
        }
        let name = name.trim_start_matches('-').to_owned();
        i += 1;
        i = skip_ws(body, i);
        if !matches!(body.get(i).map(|t| &t.token), Some(Token::Colon)) {
            continue;
        }
        i += 1;
        let start = i;
        while i < body.len() && body[i].token != Token::Semicolon {
            i += 1;
        }
        defs.custom.insert(name, tokens_text(&body[start..i]));
        i += 1;
    }
}

fn parse_import(tokens: &[LexedToken], at: usize, out: &mut Stylesheet, issues: &mut Vec<ParseIssue>, file: &str) -> usize {
    let line = tokens[at].line;
    let end = skip_statement(tokens, at);
    let slice = &tokens[at + 1..end];
    let mut path = None;
    let mut k = 0;
    while k < slice.len() {
        match &slice[k].token {
            Token::String(s) => {
                path = Some(s.clone());
                break;
            }
            Token::Ident(name) if name.as_str() == "url" => {
                if let Some(Token::String(s)) = slice.get(k + 1).map(|t| &t.token) {
                    path = Some(s.clone());
                    break;
                }
                k += 1;
            }
            _ => k += 1,
        }
    }
    match path {
        Some(path) => out.imports.push(Import { path, line }),
        None => warn_issue(issues, file, line, "`@import` without a path"),
    }
    end.saturating_add(1)
}

fn parse_rule(
    tokens: &[LexedToken],
    start: usize,
    defs: &Definitions,
    issues: &mut Vec<ParseIssue>,
    reported_unknown: &mut HashSet<String>,
    file: &str,
) -> (Option<Rule>, usize) {
    let line = tokens[start].line;
    let mut brace = start;
    while brace < tokens.len() {
        match tokens[brace].token {
            Token::LBrace => break,
            Token::Semicolon | Token::RBrace => {
                warn_issue(issues, file, tokens[brace].line, "declaration outside a rule");
                return (None, brace + 1);
            }
            _ => brace += 1,
        }
    }
    if brace >= tokens.len() {
        warn_issue(issues, file, line, "rule without a body");
        return (None, tokens.len());
    }
    let selectors = selector::parse_list(&tokens[start..brace], issues, file);
    if selectors.is_empty() {
        warn_issue(issues, file, line, "rule without a selector");
    }
    let end = matching_brace(tokens, brace).unwrap_or(tokens.len());
    let declarations = parse_declarations(&tokens[brace + 1..end], defs, issues, reported_unknown, file);
    (Some(Rule { selectors, declarations, line }), end + 1)
}

fn parse_declarations(
    body: &[LexedToken],
    defs: &Definitions,
    issues: &mut Vec<ParseIssue>,
    reported_unknown: &mut HashSet<String>,
    file: &str,
) -> Vec<Declaration> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < body.len() {
        i = skip_ws(body, i);
        while i < body.len() && body[i].token == Token::Semicolon {
            i += 1;
        }
        if i >= body.len() || body[i].token == Token::RBrace {
            break;
        }
        let line = body[i].line;
        let property = match &body[i].token {
            Token::Ident(name) => name.clone(),
            _ => {
                warn_issue(issues, file, line, format!("invalid declaration start: {:?}", body[i].token));
                i = skip_statement(body, i).max(i + 1);
                continue;
            }
        };
        i += 1;
        i = skip_ws(body, i);
        if !matches!(body.get(i).map(|t| &t.token), Some(Token::Colon)) {
            warn_issue(issues, file, line, format!("missing `:` after `{property}`"));
            i = skip_statement(body, i).max(i + 1);
            continue;
        }
        i += 1;
        let start = i;
        let mut paren = 0_i32;
        while i < body.len() {
            match body[i].token {
                Token::LParen => paren += 1,
                Token::RParen => paren -= 1,
                Token::Semicolon if paren == 0 => break,
                Token::RBrace if paren == 0 => break,
                _ => {}
            }
            i += 1;
        }
        let (value_tokens, important) = strip_important(&body[start..i]);
        if value_tokens.iter().all(|t| t.token == Token::Whitespace) {
            warn_issue(issues, file, line, format!("empty value for `{property}`"));
        }
        if !style::is_known_property(&property) && reported_unknown.insert(property.clone()) {
            warn_issue(issues, file, line, format!("unknown property `{property}`"));
        }
        out.push(Declaration {
            property: property.clone(),
            value: value::parse_for_property(value_tokens, &property, defs),
            important,
            line,
        });
        if i < body.len() && body[i].token == Token::Semicolon {
            i += 1;
        }
    }
    out
}

fn parse_keyframes(
    tokens: &[LexedToken],
    at: usize,
    defs: &Definitions,
    issues: &mut Vec<ParseIssue>,
    reported_unknown: &mut HashSet<String>,
    file: &str,
) -> (Option<Keyframes>, usize) {
    let line = tokens[at].line;
    let mut i = skip_ws(tokens, at + 1);
    let name = match tokens.get(i).map(|t| &t.token) {
        Some(Token::Ident(name)) | Some(Token::String(name)) => name.clone(),
        _ => {
            warn_issue(issues, file, line, "`@keyframes` without a name");
            return (None, skip_at_rule(tokens, at));
        }
    };
    i = skip_ws(tokens, i + 1);
    if !matches!(tokens.get(i).map(|t| &t.token), Some(Token::LBrace)) {
        warn_issue(issues, file, line, "`@keyframes` without a body");
        return (None, skip_at_rule(tokens, at));
    }
    let end = matching_brace(tokens, i).unwrap_or(tokens.len());
    let mut steps = Vec::new();
    let mut j = i + 1;
    while j < end {
        j = skip_ws(tokens, j);
        let step_start = j;
        while j < end && tokens[j].token != Token::LBrace {
            j += 1;
        }
        if j >= end {
            break;
        }
        let step_name = tokens_text(&tokens[step_start..j]);
        let body_end = matching_brace(tokens, j).unwrap_or(end);
        let declarations = parse_declarations(&tokens[j + 1..body_end], defs, issues, reported_unknown, file);
        steps.push(KeyframeStep { name: step_name, declarations });
        j = body_end + 1;
    }
    (Some(Keyframes { name, steps, line }), end + 1)
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

fn skip_ws(tokens: &[LexedToken], mut i: usize) -> usize {
    while i < tokens.len() && tokens[i].token == Token::Whitespace {
        i += 1;
    }
    i
}

fn skip_statement(tokens: &[LexedToken], from: usize) -> usize {
    let mut i = from;
    while i < tokens.len() && tokens[i].token != Token::Semicolon && tokens[i].token != Token::RBrace {
        i += 1;
    }
    i
}

fn skip_at_rule(tokens: &[LexedToken], from: usize) -> usize {
    let mut i = skip_ws(tokens, from + 1);
    while i < tokens.len() {
        match tokens[i].token {
            Token::Semicolon => return i + 1,
            Token::LBrace => return matching_brace(tokens, i).map_or(tokens.len(), |end| end + 1),
            _ => i += 1,
        }
    }
    i
}

fn skip_rule(tokens: &[LexedToken], from: usize) -> usize {
    let mut i = from;
    while i < tokens.len() && tokens[i].token != Token::LBrace {
        i += 1;
    }
    if i < tokens.len() {
        matching_brace(tokens, i).map_or(tokens.len(), |end| end + 1)
    } else {
        tokens.len()
    }
}

fn matching_brace(tokens: &[LexedToken], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, item) in tokens.iter().enumerate().skip(open) {
        match item.token {
            Token::LBrace => depth += 1,
            Token::RBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::value::Value;

    const GTK3: &str = include_str!("../../tests/fixtures/base-gtk3/gtk.css");
    const GTK4: &str = include_str!("../../tests/fixtures/effects-gtk4/gtk.css");

    #[test]
    fn parses_gtk3_fixture() {
        let sheet = parse(GTK3, "base-gtk3/gtk.css");
        assert!(sheet.issues.is_empty(), "{:?}", sheet.issues);
        assert!(sheet.rules.len() >= 4);
        let rule = sheet.rules.iter().find(|r| r.selectors.iter().any(|s| s.compounds.iter().any(|c| c.pseudos.contains(&super::super::selector::PseudoClass::Selected)))).unwrap();
        assert!(rule.declarations.iter().any(|d| d.property == "background-color"));
    }

    #[test]
    fn parses_gtk4_fixture_with_imports_and_gradients() {
        let sheet = parse(GTK4, "effects-gtk4/gtk.css");
        assert!(sheet.issues.is_empty(), "{:?}", sheet.issues);
        assert_eq!(sheet.imports.len(), 1);
        assert_eq!(sheet.imports[0].path, "base-gtk3/gtk.css");
        let background = sheet.rules[0].declarations.iter().find(|d| d.property == "background-image").unwrap();
        assert!(matches!(background.value, Value::Gradient(_)));
    }

    #[test]
    fn parses_define_color_keyframes_and_important() {
        let css = "@define-color bg #123456;\n:root { --accent: red; }\n@keyframes pulse { 0% { opacity: 0.2; } to { opacity: 1; } }\na { color: @bg !important; }";
        let sheet = parse(css, "inline.css");
        assert!(sheet.issues.is_empty(), "{:?}", sheet.issues);
        assert_eq!(sheet.keyframes.len(), 1);
        assert_eq!(sheet.keyframes[0].steps.len(), 2);
        let decl = &sheet.rules[0].declarations[0];
        assert!(decl.important);
        assert_eq!(decl.value, Value::Color(crate::model::Color::new(18, 52, 86, 255)));
    }

    #[test]
    fn recovers_after_a_bad_rule_and_reports_unknown_once() {
        let css = "a { color: red } c { color ; } b { color: blue; } x { -rf-nope: 1; } y { -rf-nope: 2; } z { color: green; } d { color: ; }";
        let sheet = parse(css, "broken.css");
        assert_eq!(sheet.rules.len(), 7, "later rules survive the bad declaration");
        assert!(sheet.rules.iter().any(|r| r.line == 1));
        assert_eq!(sheet.issues.iter().filter(|i| i.message.contains("unknown property")).count(), 1);
        assert!(sheet.issues.iter().any(|i| i.message.contains("missing `:`")));
        assert!(sheet.issues.iter().any(|i| i.message.contains("empty value")));
    }
}
