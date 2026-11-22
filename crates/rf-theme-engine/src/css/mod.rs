//! GTK3/GTK4 CSS reader. The AST is inert; no cascade or painting happens here.

pub mod lexer;
pub mod parser;
pub mod selector;
pub mod value;

use std::fmt;

/// One non-fatal parse problem, including the source file and line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

/// Record a problem once per file/line/message and emit it through `log`.
pub(crate) fn warn_issue(issues: &mut Vec<ParseIssue>, file: &str, line: usize, message: impl Into<String>) {
    let issue = ParseIssue { file: file.to_owned(), line, message: message.into() };
    log::warn!("{issue}");
    if !issues.contains(&issue) {
        issues.push(issue);
    }
}
