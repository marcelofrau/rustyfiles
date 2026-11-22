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
