//! Bridge between the global settings and the pure GTK theme engine.
//!
//! Nothing is parsed or loaded while the engine is `off`. A theme change builds
//! one `EngineTheme`; per-frame node resolution only runs the M2 cascade.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
use rf_theme_engine::model::{Color, LengthUnit, Paint, StateFlags, ThemeNode};
use rf_theme_engine::style::{ComputedStyle, LegacyPalette};
use rf_theme_engine::theme::{EngineTheme, ThemePref, ThemeVersion, choose_version};

use crate::theme::Palette;

const MAX_FILE: u64 = 4 * 1024 * 1024;

static CURRENT: RwLock<Option<Arc<ThemeHandle>>> = RwLock::new(None);
static KEY: RwLock<String> = RwLock::new(String::new());
static PARSES: AtomicU64 = AtomicU64::new(0);

/// A parsed theme plus the style every node falls back to. Comparing against
/// the baseline keeps the bridge from overwriting styles the app already set.
struct ThemeHandle {
    theme: Arc<EngineTheme>,
    baseline: ComputedStyle,
}

/// GPUI-free style differences. `None` means "the theme did not change this".
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct StyleDelta {
    pub bg: Option<Color>,
    pub text: Option<Color>,
    pub radius: Option<f32>,
    pub padding: Option<[f32; 4]>,
    pub font_size: Option<f32>,
    pub opacity: Option<f32>,
}

impl StyleDelta {
    pub(crate) fn is_empty(&self) -> bool {
        self.bg.is_none()
            && self.text.is_none()
            && self.radius.is_none()
            && self.padding.is_none()
            && self.font_size.is_none()
            && self.opacity.is_none()
    }
}

impl ThemeHandle {
    fn new(theme: EngineTheme) -> Self {
        let baseline = ComputedStyle::from_palette(theme.fallback.clone());
        Self { theme: Arc::new(theme), baseline }
    }

    fn delta(&self, node: &ThemeNode) -> StyleDelta {
        let style = rf_theme_engine::resolve(node, &self.theme);
        let base = &self.baseline;
        let mut out = StyleDelta {
            bg: if style.background == base.background { None } else { first_paint_color(&style.background) },
            text: (style.color != base.color).then_some(style.color),
            radius: (style.radius != base.radius).then(|| px_len(style.radius.top_left, 14.)),
            padding: (style.padding != base.padding).then(|| {
                [px_len(style.padding.top, 14.), px_len(style.padding.right, 14.), px_len(style.padding.bottom, 14.), px_len(style.padding.left, 14.)]
            }),
            font_size: style.font.size.filter(|v| Some(*v) != base.font.size).map(|v| px_len(v, 14.)),
            opacity: (style.opacity - base.opacity).abs().gt(&f32::EPSILON).then_some(style.opacity),
        };
        apply_legacy_overrides(&self.theme.overrides, node, &mut out);
        out
    }
}

/// Reload after a settings or appearance change. Identical inputs keep the
/// parsed theme: state changes must only rerun the cascade, not parse CSS.
pub(crate) fn reload(folder: &str) {
    let key = format!("{folder}\0{}", crate::settings::effective_dark());
    if *KEY.read() == key {
        return;
    }
    *KEY.write() = key;
    let theme = load(folder);
    *CURRENT.write() = theme.map(Arc::new);
}

/// GTK language found in a theme folder, for the settings dropdown label.
pub(crate) fn detected_version(folder: &str) -> Option<ThemeVersion> {
    let root = theme_dir(folder)?;
    discover(&root).map(|c| c.version)
}

/// Resolve one adapter node. Returns an empty delta when no engine is active.
pub(crate) fn delta(node: &ThemeNode) -> StyleDelta {
    CURRENT.read().as_ref().map(|h| h.delta(node)).unwrap_or_default()
}

#[cfg(test)]
pub(crate) fn parse_count() -> u64 {
    PARSES.load(Ordering::Relaxed)
}

fn load(folder: &str) -> Option<ThemeHandle> {
    let root = theme_dir(folder)?;
    let chosen = discover(&root)?;
    let mut theme = EngineTheme {
        fallback: fallback_palette(),
        ..EngineTheme::default()
    };
    if let Ok(text) = read_limited(&root.join("theme.toml"))
        && let Ok((dark, light)) = crate::themes::parse_colors(&text)
    {
        for (key, value) in if crate::settings::effective_dark() { dark } else { light } {
            theme.overrides.insert(key, color(value));
        }
    }
    let css = read_limited(&chosen.file).ok()?;
    theme.add_css(&css, &chosen.display);
    if let Some(dark) = chosen.dark {
        if let Ok(text) = read_limited(&dark) {
            theme.add_css(&text, dark.to_string_lossy().as_ref());
        }
    }
    PARSES.fetch_add(1, Ordering::Relaxed);
    let issues: usize = theme.stylesheets.iter().map(|s| s.issues.len()).sum();
    if issues != 0 {
        log::debug!("theme engine: {} parse issues in {folder}", issues);
    }
    Some(ThemeHandle::new(theme))
}

struct Candidate {
    version: ThemeVersion,
    file: PathBuf,
    dark: Option<PathBuf>,
    display: String,
}

fn discover(root: &Path) -> Option<Candidate> {
    const DIRS: &[(&str, ThemeVersion)] = &[
        ("gtk-4.0", ThemeVersion::Gtk4),
        ("gtk-3.0", ThemeVersion::Gtk3),
        ("gtk-3.20", ThemeVersion::Gtk3),
        ("gtk-2.0", ThemeVersion::Gtk2),
    ];
    let mut found: Vec<Candidate> = Vec::new();
    for (dir, version) in DIRS {
        let Some(base) = safe_dir(root, dir) else { continue };
        let (main, dark_name) = version.file_names();
        let Some(file) = safe_file(&base, main) else { continue };
        let dark = safe_file(&base, dark_name);
        found.push(Candidate {
            version: *version,
            file,
            dark,
            display: format!("{dir}/{main}"),
        });
    }
    let available: Vec<ThemeVersion> = found.iter().map(|c| c.version).collect();
    let version = choose_version(ThemePref::Auto, &available)?;
    found.into_iter().find(|c| c.version == version)
}

fn theme_dir(folder: &str) -> Option<PathBuf> {
    if folder.is_empty() {
        return None;
    }
    let base = crate::themes::dir()?;
    safe_dir(&base, folder)
}

fn safe_dir(base: &Path, name: &str) -> Option<PathBuf> {
    let path = base.join(name).canonicalize().ok()?;
    path.starts_with(base.canonicalize().ok()?).then_some(path)
}

fn safe_file(base: &Path, name: &str) -> Option<PathBuf> {
    let path = base.join(name).canonicalize().ok()?;
    path.starts_with(base).then_some(path)
}

fn read_limited(path: &Path) -> Result<String, ()> {
    if std::fs::metadata(path).is_ok_and(|m| m.len() > MAX_FILE) {
        log::warn!("theme engine: {} exceeds {MAX_FILE} bytes", path.display());
        return Err(());
    }
    std::fs::read_to_string(path).map_err(|e| {
        log::warn!("theme engine: {}: {e}", path.display());
    })
}

fn fallback_palette() -> LegacyPalette {
    let dark = crate::settings::effective_dark();
    let mut p = if dark { Palette::dark() } else { Palette::light() };
    crate::themes::apply(&mut p);
    LegacyPalette {
        window_bg: color(p.window_bg),
        titlebar_bg: color(p.titlebar_bg),
        sidebar_bg: color(p.sidebar_bg),
        content_bg: color(p.content_bg),
        card_bg: color(p.card_bg),
        bar_bg: color(p.bar_bg),
        tile_bg: color(p.tile_bg),
        tile_hover: color(p.tile_hover),
        border: color(p.border),
        divider: color(p.divider),
        text: color(p.text),
        text_secondary: color(p.text_secondary),
        text_tertiary: color(p.text_tertiary),
        accent: color(p.accent),
        accent_text: color(p.accent_text),
        hover: color(p.hover),
        pressed: color(p.pressed),
        selection: color(p.selection),
        selection_border: color(p.selection_border),
        tab_active: color(p.tab_active),
        danger: color(p.danger),
    }
}

fn color(c: gpui_kit::Hsla) -> Color {
    let c = c.to_rgb();
    Color::new(byte(c.r), byte(c.g), byte(c.b), byte(c.a))
}

fn byte(v: f32) -> u8 {
    (v.clamp(0., 1.) * 255.).round() as u8
}

/// `theme.toml` is legacy compatibility: its colours win over GTK rules.
fn apply_legacy_overrides(overrides: &BTreeMap<String, Color>, node: &ThemeNode, out: &mut StyleDelta) {
    let class = |name: &str| node.classes.iter().any(|c| c == name);
    let surface = if class("rf-titlebar") {
        "titlebar_bg"
    } else if class("rf-tabstrip") {
        "tab_active"
    } else if class("sidebar-row") {
        "sidebar_bg"
    } else if class("file-row") {
        "content_bg"
    } else {
        ""
    };
    if let Some(c) = overrides.get(surface) {
        out.bg = Some(*c);
    }
    if node.states.contains(StateFlags::SELECTED)
        && let Some(c) = overrides.get("selection")
    {
        out.bg = Some(*c);
    }
    if let Some(c) = overrides.get("text") {
        out.text = Some(*c);
    }
}

fn first_paint_color(paints: &[Paint]) -> Option<Color> {
    paints.iter().find_map(|p| match p {
        Paint::Color(c) => Some(*c),
        Paint::Gradient(g) => g.stops.iter().find_map(|s| s.color),
        Paint::Image(_) => None,
    })
}

fn px_len(l: rf_theme_engine::model::Length, base: f32) -> f32 {
    let v = l.value;
    match l.unit {
        LengthUnit::Px | LengthUnit::Vh | LengthUnit::Vw | LengthUnit::None => v,
        LengthUnit::Pt => v * 96. / 72.,
        LengthUnit::In => v * 96.,
        LengthUnit::Mm => v * 96. / 25.4,
        LengthUnit::Cm => v * 96. / 2.54,
        LengthUnit::Em | LengthUnit::Rem => v * base,
        LengthUnit::Percent => v / 100. * base,
        LengthUnit::Deg | LengthUnit::Ms | LengthUnit::S => v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme_nodes::{file_row_node, sidebar_row_node, tabstrip_node, titlebar_node};
    use rf_theme_engine::model::StateFlags;

    const CSS: &str = "headerbar { background-color: #123456; color: #abcdef; border-radius: 7px; font-size: 13px; opacity: 0.75; } headerbar:selected { background-color: #654321; }";
    const SLICE_CSS: &str = include_str!("../../../docs/design/gtk-theme-engine/fixtures/slice/gtk-3.0/gtk.css");

    fn handle() -> ThemeHandle {
        let mut theme = EngineTheme::default();
        theme.add_css(CSS, "slice.css");
        ThemeHandle::new(theme)
    }

    #[test]
    fn resolves_a_slice_style() {
        let d = handle().delta(&ThemeNode::new("headerbar").with_class("rf-titlebar"));
        assert_eq!(d.bg, Color::parse_hex("#123456"));
        assert_eq!(d.text, Color::parse_hex("#abcdef"));
        assert_eq!(d.radius, Some(7.));
        assert_eq!(d.font_size, Some(13.));
        assert_eq!(d.opacity, Some(0.75));
    }

    #[test]
    fn theme_toml_colours_win_over_css() {
        let mut theme = EngineTheme::with_css(CSS, "slice.css");
        theme.overrides.insert("titlebar_bg".into(), Color::parse_hex("#00ff00").unwrap());
        theme.overrides.insert("text".into(), Color::parse_hex("#ffff00").unwrap());
        let h = ThemeHandle::new(theme);
        let d = h.delta(&ThemeNode::new("headerbar").with_class("rf-titlebar"));
        assert_eq!(d.bg, Color::parse_hex("#00ff00"));
        assert_eq!(d.text, Color::parse_hex("#ffff00"));
    }

    #[test]
    fn fixture_styles_all_four_slice_nodes() {
        let mut theme = EngineTheme::with_css(SLICE_CSS, "slice/gtk.css");
        theme.overrides.insert("titlebar_bg".into(), Color::parse_hex("#00695c").unwrap());
        let h = ThemeHandle::new(theme);

        assert_eq!(h.delta(&titlebar_node(true)).bg, Color::parse_hex("#00695c"));
        assert_eq!(h.delta(&tabstrip_node(true)).bg, Color::parse_hex("#1565c0"));
        assert_eq!(h.delta(&sidebar_row_node(true, false, false, true)).bg, Color::parse_hex("#ff8f00"));
        assert_eq!(h.delta(&file_row_node(false, true, false, false, true)).bg, Color::parse_hex("#ef6c00"));
    }

    #[test]
    fn state_changes_only_rerun_the_cascade() {
        let h = handle();
        let before = parse_count();
        let plain = ThemeNode::new("headerbar");
        let mut selected = plain.clone();
        selected.states = StateFlags::SELECTED;
        assert_ne!(h.delta(&plain), h.delta(&selected));
        assert_eq!(parse_count(), before);
    }

    #[test]
    fn empty_theme_folder_builds_no_handle() {
        assert!(load("").is_none());
    }

    #[test]
    fn detects_gtk_language_from_folder_contents() {
        let base = std::env::temp_dir().join(format!("rf-theme-detect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("gtk-3.0")).unwrap();
        std::fs::write(base.join("gtk-3.0/gtk.css"), "window { color: #fff; }").unwrap();
        let found = discover(&base).map(|c| c.version);
        let _ = std::fs::remove_dir_all(&base);
        assert_eq!(found, Some(ThemeVersion::Gtk3));
    }
}
