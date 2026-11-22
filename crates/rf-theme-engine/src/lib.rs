//! Pure-Rust GTK theme contracts, parsers and cascade. No GPUI, no Windows APIs,
//! no file I/O.
//!
//! M0 defined the vocabulary, M1 reads GTK3/GTK4 CSS, and M2 resolves a node to
//! a `ComputedStyle`. M3/M4 will bridge those styles to GPUI.

pub mod css;
pub mod model;
pub mod style;
pub mod theme;

pub use css::cascade::{resolve, resolve_with_parent};

