//! Property vocabulary. Parsers live in M1/M2; this module only records intent.

/// What the engine does with a property once a parser has read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyStatus {
    /// Parsed and painted in M4.
    Paint,
    /// Kept in the AST, with no visual effect until a later milestone.
    Parse,
    /// Deliberately not applied.
    Ignore,
}

/// One property contract.
#[derive(Debug, Clone, Copy)]
pub struct Property {
    pub name: &'static str,
    pub status: PropertyStatus,
    pub reason: &'static str,
}

/// Properties seen in the M0 corpus, plus common GTK extensions. Used only to
/// report genuinely unknown declarations; parsing never drops them.
const KNOWN_PROPERTIES: &[&str] = &[
    "background", "background-color", "background-image", "background-size", "background-position",
    "background-repeat", "background-clip", "background-origin", "background-gradient-direction",
    "background-gradient-start", "background-gradient-end", "border", "border-color", "border-style",
    "border-width", "border-top", "border-top-color", "border-top-style", "border-top-width",
    "border-right", "border-right-color", "border-right-style", "border-right-width", "border-bottom",
    "border-bottom-color", "border-bottom-style", "border-bottom-width", "border-left", "border-left-color",
    "border-left-style", "border-left-width", "border-radius", "border-top-left-radius",
    "border-top-right-radius", "border-bottom-left-radius", "border-bottom-right-radius",
    "border-radius-topleft", "border-radius-topright", "border-image", "border-image-source",
    "border-image-slice", "border-image-repeat", "border-spacing", "box-shadow", "text-shadow", "color",
    "caret-color", "selection-background-color", "selected-color", "font", "font-family", "font-size",
    "font-style", "font-weight", "font-feature-settings", "text-align", "text-decoration-line",
    "text-decoration-style", "text-decoration-color", "padding", "padding-left", "padding-right",
    "padding-top", "padding-bottom", "margin", "margin-left", "margin-right", "margin-top",
    "margin-bottom", "width", "height", "min-width", "min-height", "max-width", "max-height", "opacity",
    "outline", "outline-color", "outline-style", "outline-width", "outline-offset", "spacing",
    "spacing-rows", "spacing-columns", "icon-size", "display", "visibility", "position", "filter",
    "transform", "transform-origin", "animation", "transition", "transition-property",
    "transition-duration", "transition-delay", "transition-timing-function", "stroke", "fill",
    "stroke-width", "-gtk-icon-source", "-gtk-icon-size", "-gtk-icon-transform", "-gtk-icon-shadow",
    "-gtk-icon-filter", "-gtk-icon-effect", "-gtk-icon-style", "-gtk-outline-radius",
    "-GtkWidget-window-dragging", "-st-icon-style", "-st-vfade-offset", "-st-hfade-offset",
    "-boxpointer-gap", "-arrow-rise", "-arrow-border-radius", "-arrow-background-color",
    "-arrow-border-width", "-arrow-border-color", "-arrow-base", "-slider-height",
    "-slider-background-color", "-slider-border-color", "-gradient-height", "-gradient-start",
    "-gradient-end", "-margin-horizontal", "-PnlDockBin-handle-size", "-natural-hpadding",
    "-minimum-hpadding", "-GtkArrow-arrow-scaling",
];

/// Whether a declaration name is known. Unknown names are reported, not dropped.
pub fn is_known_property(name: &str) -> bool {
    name.starts_with("--")
        || name.starts_with("-gtk-")
        || name.starts_with("-Gtk")
        || name.starts_with("-st-")
        || name.starts_with("-Pnl")
        || name.starts_with("-arrow-")
        || name.starts_with("-slider-")
        || KNOWN_PROPERTIES.contains(&name)
}

/// Core subset of the M0 corpus registry. The full 312-property classification
/// lives in `docs/design/gtk-theme-engine/property-registry.md`.
pub const CORE_PROPERTIES: &[Property] = &[
    Property { name: "background-color", status: PropertyStatus::Paint, reason: "surface fill" },
    Property { name: "color", status: PropertyStatus::Paint, reason: "foreground text" },
    Property { name: "background", status: PropertyStatus::Paint, reason: "shorthand fill" },
    Property { name: "background-image", status: PropertyStatus::Paint, reason: "gradient or themed image" },
    Property { name: "box-shadow", status: PropertyStatus::Paint, reason: "elevation" },
    Property { name: "border", status: PropertyStatus::Paint, reason: "edge shorthand" },
    Property { name: "border-radius", status: PropertyStatus::Paint, reason: "corner rounding" },
    Property { name: "padding", status: PropertyStatus::Paint, reason: "inner spacing" },
    Property { name: "margin", status: PropertyStatus::Paint, reason: "outer spacing" },
    Property { name: "font-size", status: PropertyStatus::Paint, reason: "text size" },
    Property { name: "font-weight", status: PropertyStatus::Paint, reason: "text weight" },
    Property { name: "opacity", status: PropertyStatus::Paint, reason: "surface alpha" },
    Property { name: "transition", status: PropertyStatus::Parse, reason: "animation deferred to M7" },
    Property { name: "-gtk-icon-source", status: PropertyStatus::Parse, reason: "icon slots stay in icon-theme work" },
    Property { name: "-GtkWidget-window-dragging", status: PropertyStatus::Ignore, reason: "no mapped RustyFiles node" },
];

/// Look up the contract for a property name, if it is in the core subset.
pub fn status_of(name: &str) -> Option<PropertyStatus> {
    CORE_PROPERTIES.iter().find(|p| p.name == name).map(|p| p.status)
}

use std::collections::BTreeMap;

use crate::model::{BorderEdges, Color, Corners, Edges, FontSpec, Paint, Shadow};

/// The 21 colour slots a `theme.toml` may override. Mirrors rf-app's
/// `themes::COLOR_KEYS` without depending on GPUI.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPalette {
    pub window_bg: Color,
    pub titlebar_bg: Color,
    pub sidebar_bg: Color,
    pub content_bg: Color,
    pub card_bg: Color,
    pub bar_bg: Color,
    pub tile_bg: Color,
    pub tile_hover: Color,
    pub border: Color,
    pub divider: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub hover: Color,
    pub pressed: Color,
    pub selection: Color,
    pub selection_border: Color,
    pub tab_active: Color,
    pub danger: Color,
}

pub const LEGACY_COLOR_KEYS: &[&str] = &[
    "window_bg", "titlebar_bg", "sidebar_bg", "content_bg", "card_bg", "bar_bg", "tile_bg", "tile_hover", "border", "divider",
    "text", "text_secondary", "text_tertiary", "accent", "accent_text", "hover", "pressed", "selection", "selection_border",
    "tab_active", "danger",
];

impl Default for LegacyPalette {
    fn default() -> Self {
        let c = Color::new;
        Self {
            window_bg: c(248, 248, 248, 255),
            titlebar_bg: c(243, 243, 243, 255),
            sidebar_bg: c(240, 240, 240, 255),
            content_bg: c(255, 255, 255, 255),
            card_bg: c(255, 255, 255, 255),
            bar_bg: c(250, 250, 250, 255),
            tile_bg: c(250, 250, 250, 255),
            tile_hover: c(235, 235, 235, 255),
            border: c(220, 220, 220, 255),
            divider: c(230, 230, 230, 255),
            text: c(30, 30, 30, 255),
            text_secondary: c(90, 90, 90, 255),
            text_tertiary: c(140, 140, 140, 255),
            accent: c(0, 95, 184, 255),
            accent_text: c(255, 255, 255, 255),
            hover: c(232, 232, 232, 255),
            pressed: c(218, 218, 218, 255),
            selection: c(200, 225, 250, 255),
            selection_border: c(0, 95, 184, 255),
            tab_active: c(255, 255, 255, 255),
            danger: c(200, 45, 45, 255),
        }
    }
}

impl LegacyPalette {
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Color> {
        Some(match key {
            "window_bg" => &mut self.window_bg,
            "titlebar_bg" => &mut self.titlebar_bg,
            "sidebar_bg" => &mut self.sidebar_bg,
            "content_bg" => &mut self.content_bg,
            "card_bg" => &mut self.card_bg,
            "bar_bg" => &mut self.bar_bg,
            "tile_bg" => &mut self.tile_bg,
            "tile_hover" => &mut self.tile_hover,
            "border" => &mut self.border,
            "divider" => &mut self.divider,
            "text" => &mut self.text,
            "text_secondary" => &mut self.text_secondary,
            "text_tertiary" => &mut self.text_tertiary,
            "accent" => &mut self.accent,
            "accent_text" => &mut self.accent_text,
            "hover" => &mut self.hover,
            "pressed" => &mut self.pressed,
            "selection" => &mut self.selection,
            "selection_border" => &mut self.selection_border,
            "tab_active" => &mut self.tab_active,
            "danger" => &mut self.danger,
            _ => return None,
        })
    }

    /// `theme.toml` wins over every GTK rule. Unknown keys are ignored.
    pub fn apply_overrides(&mut self, overrides: &BTreeMap<String, Color>) {
        for (key, color) in overrides {
            if let Some(slot) = self.get_mut(key) {
                *slot = *color;
            }
        }
    }
}

/// One resolved node style. No GPUI types; the bridge maps this later.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub color: Color,
    pub background: Vec<Paint>,
    pub border: BorderEdges,
    pub radius: Corners,
    pub shadow: Vec<Shadow>,
    pub text_shadow: Vec<Shadow>,
    pub font: FontSpec,
    pub padding: Edges,
    pub margin: Edges,
    pub opacity: f32,
    pub icon_style: Option<String>,
    pub palette: LegacyPalette,
}

impl ComputedStyle {
    pub fn from_palette(palette: LegacyPalette) -> Self {
        Self {
            color: palette.text,
            background: Vec::new(),
            border: BorderEdges::none(),
            radius: Corners::zero(),
            shadow: Vec::new(),
            text_shadow: Vec::new(),
            font: FontSpec::default(),
            padding: Edges::zero(),
            margin: Edges::zero(),
            opacity: 1.0,
            icon_style: None,
            palette,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GTK3: &str = include_str!("../tests/fixtures/base-gtk3/gtk.css");
    const GTK4: &str = include_str!("../tests/fixtures/effects-gtk4/gtk.css");
    const GTK2: &str = include_str!("../tests/fixtures/classic-gtk2/gtkrc");

    #[test]
    fn fixtures_exist_and_are_gtk_flavours() {
        assert!(GTK3.contains("row:selected"));
        assert!(GTK4.contains("linear-gradient"));
        assert!(GTK2.contains("style \"rf-base\""));
        assert!(GTK2.contains("widget_class"));
    }

    #[test]
    fn core_registry_covers_fixture_properties() {
        assert_eq!(status_of("background-color"), Some(PropertyStatus::Paint));
        assert_eq!(status_of("box-shadow"), Some(PropertyStatus::Paint));
        assert_eq!(status_of("transition"), Some(PropertyStatus::Parse));
        assert_eq!(status_of("-GtkWidget-window-dragging"), Some(PropertyStatus::Ignore));
        assert_eq!(status_of("not-a-property"), None);
    }
}
