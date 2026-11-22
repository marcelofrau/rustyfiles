//! Theme selection and the pure rule database. No filesystem access here: the
//! caller reads files and passes their text plus their source name.

use std::collections::BTreeMap;

use crate::css::parser::parse;
use crate::model::Color;
use crate::style::LegacyPalette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVersion {
    Gtk4,
    Gtk3,
    Gtk2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePref {
    Auto,
    Gtk4,
    Gtk3,
    Gtk2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

impl ThemeVersion {
    /// Main and dark file names for this GTK version.
    pub fn file_names(self) -> (&'static str, &'static str) {
        match self {
            Self::Gtk4 | Self::Gtk3 => ("gtk.css", "gtk-dark.css"),
            Self::Gtk2 => ("gtkrc", "gtkrc-dark"),
        }
    }
}

/// Auto prefers the newest version present in the theme folder.
pub fn choose_version(pref: ThemePref, available: &[ThemeVersion]) -> Option<ThemeVersion> {
    match pref {
        ThemePref::Auto => [ThemeVersion::Gtk4, ThemeVersion::Gtk3, ThemeVersion::Gtk2]
            .into_iter()
            .find(|v| available.contains(v)),
        ThemePref::Gtk4 => available.contains(&ThemeVersion::Gtk4).then_some(ThemeVersion::Gtk4),
        ThemePref::Gtk3 => available.contains(&ThemeVersion::Gtk3).then_some(ThemeVersion::Gtk3),
        ThemePref::Gtk2 => available.contains(&ThemeVersion::Gtk2).then_some(ThemeVersion::Gtk2),
    }
}

/// Pick the dark variant only when the theme actually has one.
pub fn variant_file(version: ThemeVersion, appearance: Appearance, has_dark: bool) -> &'static str {
    let (light, dark) = version.file_names();
    if appearance == Appearance::Dark && has_dark {
        dark
    } else {
        light
    }
}

/// A parsed, in-memory theme. `fallback` stands in for `Palette::for_window`;
/// `overrides` stands in for the legacy `theme.toml` colour table.
#[derive(Debug, Clone, Default)]
pub struct EngineTheme {
    pub stylesheets: Vec<crate::css::parser::Stylesheet>,
    pub fallback: LegacyPalette,
    pub overrides: BTreeMap<String, Color>,
}

impl EngineTheme {
    pub fn add_css(&mut self, css: &str, file: &str) {
        self.stylesheets.push(parse(css, file));
    }

    pub fn with_css(css: &str, file: &str) -> Self {
        let mut theme = Self::default();
        theme.add_css(css, file);
        theme
    }

    pub fn set_override(&mut self, key: &str, color: Color) {
        self.overrides.insert(key.to_owned(), color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_picks_newest_available_version() {
        assert_eq!(choose_version(ThemePref::Auto, &[ThemeVersion::Gtk2, ThemeVersion::Gtk3]), Some(ThemeVersion::Gtk3));
        assert_eq!(choose_version(ThemePref::Auto, &[]), None);
        assert_eq!(choose_version(ThemePref::Gtk4, &[ThemeVersion::Gtk3]), None);
        assert_eq!(choose_version(ThemePref::Gtk3, &[ThemeVersion::Gtk3]), Some(ThemeVersion::Gtk3));
        assert_eq!(choose_version(ThemePref::Gtk2, &[ThemeVersion::Gtk2]), Some(ThemeVersion::Gtk2));
    }

    #[test]
    fn dark_variant_needs_a_dark_file() {
        assert_eq!(variant_file(ThemeVersion::Gtk4, Appearance::Dark, true), "gtk-dark.css");
        assert_eq!(variant_file(ThemeVersion::Gtk4, Appearance::Dark, false), "gtk.css");
        assert_eq!(variant_file(ThemeVersion::Gtk4, Appearance::Light, true), "gtk.css");
        assert_eq!(variant_file(ThemeVersion::Gtk2, Appearance::Dark, true), "gtkrc-dark");
        assert_eq!(variant_file(ThemeVersion::Gtk2, Appearance::Light, true), "gtkrc");
    }
}
