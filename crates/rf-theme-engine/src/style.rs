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
