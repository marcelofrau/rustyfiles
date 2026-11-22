//! Selector AST and parser. Matching and inheritance belong to M2.

use super::lexer::{LexedToken, Token, text as tokens_text};
use super::warn_issue;
use crate::css::ParseIssue;
use crate::model::{StateFlags, ThemeNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
    Adjacent,
    Sibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PseudoClass {
    Hover,
    Active,
    Focus,
    FocusWithin,
    FocusVisible,
    Selected,
    Checked,
    Disabled,
    Backdrop,
    Root,
    FirstChild,
    LastChild,
    Not(Vec<ComplexSelector>),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Compound {
    pub element: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub pseudos: Vec<PseudoClass>,
}

impl Compound {
    fn is_empty(&self) -> bool {
        self.element.is_none() && self.id.is_none() && self.classes.is_empty() && self.pseudos.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexSelector {
    pub compounds: Vec<Compound>,
    pub combinators: Vec<Combinator>,
}

impl ComplexSelector {
    /// CSS-style `(ids, classes/pseudo-classes, elements)`.
    pub fn specificity(&self) -> (u32, u32, u32) {
        let mut ids = 0;
        let mut classes = 0;
        let mut elements = 0;
        for compound in &self.compounds {
            if compound.id.is_some() {
                ids += 1;
            }
            classes += compound.classes.len() as u32;
            for pseudo in &compound.pseudos {
                if let PseudoClass::Not(inner) = pseudo {
                    // Selectors 4 uses the most specific argument of :not().
                    let (inner_ids, inner_classes, inner_elements) = inner
                        .iter()
                        .map(ComplexSelector::specificity)
                        .fold((0_u32, 0_u32, 0_u32), |max, item| {
                            (max.0.max(item.0), max.1.max(item.1), max.2.max(item.2))
                        });
                    ids += inner_ids;
                    classes += inner_classes;
                    elements += inner_elements;
                } else {
                    classes += 1;
                }
            }
            if compound.element.is_some() {
                elements += 1;
            }
        }
        (ids, classes, elements)
    }
}

/// Match a node against one complex selector, including ancestors and siblings.
pub fn matches(node: &ThemeNode, selector: &ComplexSelector) -> bool {
    matches_at(node, &selector.compounds, &selector.combinators)
}

fn matches_at(node: &ThemeNode, compounds: &[Compound], combinators: &[Combinator]) -> bool {
    let Some((last, head)) = compounds.split_last() else {
        return true;
    };
    if !matches_compound(node, last) {
        return false;
    }
    if head.is_empty() {
        return true;
    }
    let combinator = combinators[head.len() - 1];
    let rest = &combinators[..head.len() - 1];
    match combinator {
        Combinator::Child => node.ancestors.last().is_some_and(|parent| matches_at(parent, head, rest)),
        Combinator::Descendant => node.ancestors.iter().any(|parent| matches_at(parent, head, rest)),
        Combinator::Adjacent => node.previous.last().is_some_and(|sibling| matches_at(sibling, head, rest)),
        Combinator::Sibling => node.previous.iter().any(|sibling| matches_at(sibling, head, rest)),
    }
}

fn matches_compound(node: &ThemeNode, compound: &Compound) -> bool {
    if let Some(element) = &compound.element
        && !element_matches(element, &node.name)
    {
        return false;
    }
    if let Some(id) = &compound.id
        && node.id.as_deref() != Some(id.as_str())
    {
        return false;
    }
    if !compound.classes.iter().all(|class| node.classes.iter().any(|c| c.eq_ignore_ascii_case(class))) {
        return false;
    }
    compound.pseudos.iter().all(|pseudo| matches_pseudo(node, pseudo))
}

fn element_matches(selector: &str, node: &str) -> bool {
    let selector = selector.to_ascii_lowercase();
    let selector = selector.strip_prefix("gtk").unwrap_or(&selector);
    selector == node.to_ascii_lowercase()
}

fn matches_pseudo(node: &ThemeNode, pseudo: &PseudoClass) -> bool {
    match pseudo {
        PseudoClass::Hover => node.states.contains(StateFlags::HOVER),
        PseudoClass::Active => node.states.contains(StateFlags::ACTIVE),
        PseudoClass::Focus => node.states.contains(StateFlags::FOCUS),
        PseudoClass::FocusWithin => node.states.contains(StateFlags::FOCUS_WITHIN),
        PseudoClass::FocusVisible => node.states.contains(StateFlags::FOCUS_VISIBLE),
        PseudoClass::Selected => node.states.contains(StateFlags::SELECTED),
        PseudoClass::Checked => node.states.contains(StateFlags::CHECKED),
        PseudoClass::Disabled => node.states.contains(StateFlags::DISABLED),
        PseudoClass::Backdrop => node.states.contains(StateFlags::BACKDROP),
        PseudoClass::Root => node.ancestors.is_empty(),
        PseudoClass::FirstChild => node.index == 0,
        PseudoClass::LastChild => node.index + 1 == node.sibling_count,
        PseudoClass::Not(inner) => !inner.iter().any(|selector| matches(node, selector)),
        PseudoClass::Other(name) if name == "prelight" => node.states.contains(StateFlags::HOVER),
        PseudoClass::Other(name) if name == "insensitive" => node.states.contains(StateFlags::DISABLED),
        PseudoClass::Other(_) => false,
    }
}

/// Parse a comma-separated selector list. Whitespace means descendant.
pub fn parse_list(tokens: &[LexedToken], issues: &mut Vec<ParseIssue>, file: &str) -> Vec<ComplexSelector> {
    let mut list = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        i = skip_ws(tokens, i);
        if i >= tokens.len() || matches!(tokens[i].token, Token::Comma | Token::LBrace | Token::RBrace) {
            break;
        }
        let (selector, next) = parse_complex(tokens, i, issues, file);
        i = next;
        if !selector.compounds.is_empty() {
            list.push(selector);
        }
        i = skip_ws(tokens, i);
        if i < tokens.len() && tokens[i].token == Token::Comma {
            i += 1;
            continue;
        }
        break;
    }
    list
}

fn parse_complex(
    tokens: &[LexedToken],
    mut i: usize,
    issues: &mut Vec<ParseIssue>,
    file: &str,
) -> (ComplexSelector, usize) {
    let mut compounds = Vec::new();
    let mut combinators = Vec::new();
    let (first, next) = parse_compound(tokens, i, issues, file);
    compounds.push(first);
    i = next;

    loop {
        let before = i;
        i = skip_ws(tokens, i);
        let had_ws = i > before;
        if i >= tokens.len() || matches!(tokens[i].token, Token::Comma | Token::LBrace | Token::RBrace) {
            break;
        }
        let combinator = match tokens[i].token {
            Token::Greater => Some(Combinator::Child),
            Token::Plus => Some(Combinator::Adjacent),
            Token::Tilde => Some(Combinator::Sibling),
            _ if had_ws => Some(Combinator::Descendant),
            _ => None,
        };
        if let Some(kind) = combinator {
            combinators.push(kind);
            i += 1;
            i = skip_ws(tokens, i);
        }
        let (compound, next) = parse_compound(tokens, i, issues, file);
        if compound.is_empty() {
            if let Some(item) = tokens.get(i) {
                warn_issue(issues, file, item.line, format!("unexpected token in selector: {:?}", item.token));
            }
            i = next;
            break;
        }
        compounds.push(compound);
        i = next;
    }
    (ComplexSelector { compounds, combinators }, i)
}

fn parse_compound(
    tokens: &[LexedToken],
    mut i: usize,
    issues: &mut Vec<ParseIssue>,
    file: &str,
) -> (Compound, usize) {
    let mut out = Compound::default();
    while i < tokens.len() {
        match &tokens[i].token {
            Token::Whitespace | Token::Comma | Token::LBrace | Token::RBrace | Token::Greater | Token::Plus | Token::Tilde => break,
            Token::Star => {
                i += 1;
            }
            Token::Ident(name) => {
                let name = name.clone();
                if out.element.is_none() {
                    out.element = Some(name);
                } else {
                    warn_issue(issues, file, tokens[i].line, format!("second element name `{name}` in one compound"));
                }
                i += 1;
            }
            Token::Hash(id) => {
                out.id = Some(id.clone());
                i += 1;
            }
            Token::Dot => {
                i += 1;
                if let Some(Token::Ident(name)) = tokens.get(i).map(|t| &t.token) {
                    out.classes.push(name.clone());
                    i += 1;
                } else {
                    let line = tokens.get(i).map_or(0, |t| t.line);
                    warn_issue(issues, file, line, "class selector without a name");
                    break;
                }
            }
            Token::Colon => {
                let line = tokens[i].line;
                let (pseudo, next) = parse_pseudo(tokens, i + 1, issues, file, line);
                if let Some(pseudo) = pseudo {
                    out.pseudos.push(pseudo);
                }
                i = next;
            }
            Token::LBracket => {
                let line = tokens[i].line;
                i = skip_balanced(tokens, i, Token::LBracket, Token::RBracket);
                warn_issue(issues, file, line, "attribute selectors are parsed but not matched");
            }
            _ => break,
        }
    }
    (out, i)
}

fn parse_pseudo(
    tokens: &[LexedToken],
    mut i: usize,
    issues: &mut Vec<ParseIssue>,
    file: &str,
    line: usize,
) -> (Option<PseudoClass>, usize) {
    let name = match tokens.get(i).map(|t| &t.token) {
        Some(Token::Ident(name)) => name.clone(),
        _ => {
            warn_issue(issues, file, line, "pseudo-class without a name");
            return (None, i.saturating_add(1));
        }
    };
    i += 1;

    let mut argument: Option<Vec<LexedToken>> = None;
    if i < tokens.len() && tokens[i].token == Token::LParen {
        let start = i + 1;
        i = skip_balanced(tokens, i, Token::LParen, Token::RParen);
        argument = Some(tokens[start..i.saturating_sub(1)].to_vec());
    }

    let pseudo = match name.as_str() {
        "hover" => PseudoClass::Hover,
        "active" => PseudoClass::Active,
        "focus" => PseudoClass::Focus,
        "focus-within" => PseudoClass::FocusWithin,
        "focus-visible" => PseudoClass::FocusVisible,
        "selected" => PseudoClass::Selected,
        "checked" => PseudoClass::Checked,
        "disabled" => PseudoClass::Disabled,
        "backdrop" => PseudoClass::Backdrop,
        "root" => PseudoClass::Root,
        "first-child" => PseudoClass::FirstChild,
        "last-child" => PseudoClass::LastChild,
        "not" => {
            let inner = argument.as_deref().unwrap_or(&[]);
            let mut list = parse_list(inner, issues, file);
            if list.is_empty() {
                list.push(ComplexSelector {
                    compounds: vec![Compound::default()],
                    combinators: vec![],
                });
            }
            PseudoClass::Not(list)
        }
        "nth-child" | "nth-last-child" | "nth-of-type" => {
            let arg = argument.as_deref().map(tokens_text).unwrap_or_default();
            PseudoClass::Other(format!("{name}({arg})"))
        }
        _ => {
            warn_issue(issues, file, line, format!("unknown pseudo-class `:{name}`"));
            PseudoClass::Other(name)
        }
    };
    (Some(pseudo), i)
}

fn skip_ws(tokens: &[LexedToken], mut i: usize) -> usize {
    while i < tokens.len() && tokens[i].token == Token::Whitespace {
        i += 1;
    }
    i
}

fn skip_balanced(tokens: &[LexedToken], i: usize, open: Token, close: Token) -> usize {
    let mut depth = 0;
    let mut j = i;
    while j < tokens.len() {
        if tokens[j].token == open {
            depth += 1;
        } else if tokens[j].token == close {
            depth -= 1;
            if depth == 0 {
                return j + 1;
            }
        }
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::lexer::lex;

    fn parse(s: &str) -> (Vec<ComplexSelector>, Vec<ParseIssue>) {
        let tokens = lex(s);
        let mut issues = Vec::new();
        let list = parse_list(&tokens, &mut issues, "test.css");
        (list, issues)
    }

    #[test]
    fn parses_compounds_and_combinators() {
        let (list, issues) = parse("window.background.rf-window > row:selected, treeview.view row:hover");
        assert!(issues.is_empty());
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].compounds[0].element.as_deref(), Some("window"));
        assert_eq!(list[0].compounds[0].classes, vec!["background", "rf-window"]);
        assert_eq!(list[0].combinators, vec![Combinator::Child]);
        assert_eq!(list[0].compounds[1].pseudos, vec![PseudoClass::Selected]);
        assert_eq!(list[1].combinators, vec![Combinator::Descendant]);
    }

    #[test]
    fn parses_not_and_computes_specificity() {
        let (list, issues) = parse("button:not(.flat)");
        assert!(issues.is_empty());
        let inner = match &list[0].compounds[0].pseudos[0] {
            PseudoClass::Not(inner) => inner,
            other => panic!("expected :not, got {other:?}"),
        };
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0].compounds[0].classes, vec!["flat"]);
        assert_eq!(list[0].specificity(), (0, 1, 1));
    }

    #[test]
    fn not_keeps_every_argument() {
        let (list, issues) = parse("button:not(.flat, #special)");
        assert!(issues.is_empty());
        let inner = match &list[0].compounds[0].pseudos[0] {
            PseudoClass::Not(inner) => inner,
            other => panic!("expected :not, got {other:?}"),
        };
        assert_eq!(inner.len(), 2);
        // The ID argument is the most specific one.
        assert_eq!(list[0].specificity(), (1, 1, 1));
    }
}
