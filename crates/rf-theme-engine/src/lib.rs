//! Pure-Rust GTK theme contracts and parsers. No GPUI, no Windows APIs, no file I/O.
//!
//! M0 defined the vocabulary. M1 reads GTK3/GTK4 CSS into an inert AST. M2 will
//! add the cascade and computed styles; M3/M4 will bridge those styles to GPUI.

pub mod css;
pub mod model;
pub mod style;
