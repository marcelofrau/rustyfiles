//! M1.4: parse extracted real GTK themes when the local corpus is available.
//! The corpus ships as tar.gz archives on this machine, so this test skips
//! instead of failing when nothing has been extracted. Set GTK_THEME_CORPUS to
//! a folder containing theme directories to run it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use rf_theme_engine::css::parser::parse;
use rf_theme_engine::model::{StateFlags, ThemeNode};
use rf_theme_engine::theme::EngineTheme;
use rf_theme_engine::resolve;

fn corpus_root() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("GTK_THEME_CORPUS") {
        let path = PathBuf::from(value);
        if path.is_dir() {
            return Some(path);
        }
    }
    let path = PathBuf::from(r"H:\Downloads\themes\gtk");
    path.is_dir().then_some(path)
}

fn collect_css(dir: &Path, depth: u8, out: &mut Vec<PathBuf>) {
    if depth > 4 || out.len() >= 16 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut names: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    names.sort();
    for path in names {
        if path.is_dir() {
            collect_css(&path, depth + 1, out);
        } else if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("css")) {
            out.push(path);
        }
    }
}

#[test]
fn parses_extracted_real_themes_without_panic() {
    let Some(root) = corpus_root() else {
        eprintln!("skip: no GTK theme corpus at H:\\Downloads\\themes\\gtk or GTK_THEME_CORPUS");
        return;
    };
    let mut files = Vec::new();
    collect_css(&root, 0, &mut files);
    if files.is_empty() {
        eprintln!("skip: no gtk.css files under {}", root.display());
        return;
    }

    let mut rules = 0;
    let mut definitions = 0;
    let mut issues = 0;
    let mut unknown = BTreeSet::new();
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_default();
        let name = file.to_string_lossy();
        let sheet = parse(&source, &name);
        rules += sheet.rules.len();
        definitions += sheet.keyframes.len() + sheet.imports.len();
        issues += sheet.issues.len();
        for issue in &sheet.issues {
            if let Some(rest) = issue.message.strip_prefix("unknown property `")
                && let Some(prop) = rest.strip_suffix('`')
            {
                unknown.insert(prop.to_owned());
            }
        }
    }
    eprintln!(
        "parsed {} files: {rules} rules, {definitions} imports/keyframes, {issues} issues, {} unique unknown properties",
        files.len(),
        unknown.len()
    );
    eprintln!("unknown properties: {}", unknown.into_iter().collect::<Vec<_>>().join(", "));
}

#[test]
fn resolves_synthetic_nodes_against_real_themes_without_panic() {
    let Some(root) = corpus_root() else {
        eprintln!("skip: no GTK theme corpus at H:\\Downloads\\themes\\gtk or GTK_THEME_CORPUS");
        return;
    };
    let mut files = Vec::new();
    collect_css(&root, 0, &mut files);
    if files.is_empty() {
        eprintln!("skip: no gtk.css files under {}", root.display());
        return;
    }

    let mut theme = EngineTheme::default();
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_default();
        let name = file.to_string_lossy();
        theme.stylesheets.push(parse(&source, &name));
    }

    let mut row = ThemeNode::new("row").with_state(StateFlags::SELECTED);
    row.ancestors.push(ThemeNode::new("treeview").with_class("view"));
    let mut label = ThemeNode::new("label");
    label.previous.push(ThemeNode::new("button"));

    let nodes = [
        ThemeNode::new("window").with_class("background"),
        ThemeNode::new("button").with_class("flat").with_state(StateFlags::HOVER),
        ThemeNode::new("entry").with_state(StateFlags::FOCUS),
        row,
        label,
    ];
    for node in &nodes {
        let first = resolve(node, &theme);
        assert_eq!(resolve(node, &theme), first, "cascade must be deterministic for {}", node.name);
    }
}
