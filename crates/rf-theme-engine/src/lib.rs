//! Pure-Rust GTK theme contracts. No GPUI, no Windows APIs, no file I/O.
//!
//! M0 defines the vocabulary shared by the CSS parser (M1), the `gtkrc` parser
//! (M2), the GPUI bridge (M3) and the painter (M4).

pub mod style;
