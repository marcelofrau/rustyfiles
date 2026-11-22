//! M2: cascade, inheritance, state matching and legacy overrides.

use rf_theme_engine::model::{BorderStyle, Color, Paint, StateFlags, ThemeNode};
use rf_theme_engine::theme::EngineTheme;
use rf_theme_engine::{resolve, resolve_with_parent};

const GTK3: &str = include_str!("fixtures/base-gtk3/gtk.css");
const GTK4: &str = include_str!("fixtures/effects-gtk4/gtk.css");

fn color(hex: &str) -> Color {
    Color::parse_hex(hex).expect("valid fixture color")
}

#[test]
fn important_beats_specificity_and_source_order() {
    let theme = EngineTheme::with_css(
        "window { background-color: #111111; }\n\
         window.flat { background-color: #222222; }\n\
         window.flat { background-color: #333333 !important; }\n",
        "important.css",
    );
    let node = ThemeNode::new("window").with_class("flat");
    assert_eq!(resolve(&node, &theme).background, vec![Paint::Color(color("#333333"))]);

    let theme = EngineTheme::with_css(
        "window { background-color: #111111; }\n\
         window.flat { background-color: #222222; }\n\
         window.flat { background-color: #444444; }\n",
        "order.css",
    );
    assert_eq!(resolve(&node, &theme).background, vec![Paint::Color(color("#444444"))]);
}

#[test]
fn specificity_beats_earlier_source_order() {
    let theme = EngineTheme::with_css(
        "window.flat { background-color: #222222; }\n\
         window { background-color: #111111; }\n",
        "specificity.css",
    );
    let node = ThemeNode::new("window").with_class("flat");
    assert_eq!(resolve(&node, &theme).background, vec![Paint::Color(color("#222222"))]);
}

#[test]
fn typography_inherits_but_surfaces_do_not() {
    let theme = EngineTheme::with_css(
        "window { color: #111111; font-size: 12px; background-color: #eeeeee; }\n",
        "inherit.css",
    );
    let parent = resolve(&ThemeNode::new("window"), &theme);
    let child = resolve_with_parent(&ThemeNode::new("button"), Some(&parent), &theme);
    assert_eq!(child.color, color("#111111"));
    assert_eq!(child.font.size.unwrap().value, 12.0);
    assert!(child.background.is_empty(), "background-color must not inherit");

    let forced = EngineTheme::with_css("button { background-color: inherit; }\n", "forced.css");
    let child = resolve_with_parent(&ThemeNode::new("button"), Some(&parent), &forced);
    assert_eq!(child.background, vec![Paint::Color(color("#eeeeee"))]);
}

#[test]
fn states_and_negative_selectors_match() {
    let theme = EngineTheme::with_css(
        "button:hover { color: #ff0000; }\n\
         button:not(.flat) { border-color: #00ff00; }\n",
        "states.css",
    );
    let flat = ThemeNode::new("button").with_class("flat").with_state(StateFlags::HOVER);
    let style = resolve(&flat, &theme);
    assert_eq!(style.color, color("#ff0000"));
    assert_ne!(style.border.top.color, Some(color("#00ff00")));

    let plain = ThemeNode::new("button").with_state(StateFlags::HOVER);
    let style = resolve(&plain, &theme);
    assert_eq!(style.border.top.color, Some(color("#00ff00")));
}

#[test]
fn combinators_use_the_node_context() {
    let theme = EngineTheme::with_css(
        "window row:selected { color: #0000ff; }\n\
         window > row { padding: 2px; }\n\
         button + label { opacity: 0.5; }\n",
        "combinators.css",
    );
    let mut row = ThemeNode::new("row").with_state(StateFlags::SELECTED);
    row.ancestors.push(ThemeNode::new("window"));
    let style = resolve(&row, &theme);
    assert_eq!(style.color, color("#0000ff"));
    assert_eq!(style.padding.top.value, 2.0);

    let mut label = ThemeNode::new("label");
    label.previous.push(ThemeNode::new("button"));
    let style = resolve(&label, &theme);
    assert_eq!(style.opacity, 0.5);
}

#[test]
fn gtk_color_functions_and_effects_flow_into_the_style() {
    let theme = EngineTheme::with_css(
        "button:hover {\n\
             background-color: shade(#808080, 0.5);\n\
             background-image: linear-gradient(#ffffff, #000000);\n\
             box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);\n\
             border-radius: 4px;\n\
         }\n",
        "effects.css",
    );
    let node = ThemeNode::new("button").with_state(StateFlags::HOVER);
    let style = resolve(&node, &theme);
    assert_eq!(style.background.first(), Some(&Paint::Color(color("#404040"))));
    assert!(matches!(style.background.get(1), Some(Paint::Gradient(_))));
    assert_eq!(style.shadow.len(), 1);
    assert_eq!(style.shadow[0].blur.unwrap().value, 8.0);
    assert_eq!(style.radius.top_left.value, 4.0);
}

#[test]
fn legacy_overrides_win_and_resolution_is_deterministic() {
    let mut theme = EngineTheme::with_css(
        "window { color: #111111; background-color: #222222; }\n",
        "override.css",
    );
    theme.set_override("text", color("#abcdef"));
    let node = ThemeNode::new("window");
    let first = resolve(&node, &theme);
    assert_eq!(first.color, color("#abcdef"));
    assert_eq!(resolve(&node, &theme), first);
}

#[test]
fn resolves_hand_written_gtk3_fixture() {
    let theme = EngineTheme::with_css(GTK3, "base-gtk3/gtk.css");

    let window = ThemeNode::new("window").with_class("background").with_class("rf-window");
    let style = resolve(&window, &theme);
    assert_eq!(style.color, color("#2f2f2f"));
    assert_eq!(style.background, vec![Paint::Color(color("#f6f5f4"))]);
    assert_eq!(style.font.size.unwrap().value, 12.0);

    let button = ThemeNode::new("button").with_class("flat").with_state(StateFlags::HOVER);
    let style = resolve(&button, &theme);
    assert_eq!(style.radius.top_left.value, 4.0);
    assert_eq!(style.padding.left.value, 10.0);

    let mut row = ThemeNode::new("row").with_state(StateFlags::SELECTED);
    row.ancestors.push(ThemeNode::new("treeview").with_class("view"));
    let style = resolve(&row, &theme);
    assert_eq!(style.background, vec![Paint::Color(color("#4a90d9"))]);
    assert_eq!(style.color, color("#ffffff"));
}

#[test]
fn resolves_hand_written_gtk4_fixture() {
    let theme = EngineTheme::with_css(GTK4, "effects-gtk4/gtk.css");
    let window = ThemeNode::new("window").with_class("background").with_class("rf-window");
    let style = resolve(&window, &theme);
    assert!(matches!(style.background.first(), Some(Paint::Gradient(_))));
    assert_eq!(style.shadow.len(), 1);

    let button = ThemeNode::new("button").with_class("suggested-action");
    let style = resolve(&button, &theme);
    assert!(matches!(style.background.first(), Some(Paint::Gradient(_))));
}

#[test]
fn later_shorthand_wins_over_earlier_longhand() {
    let theme = EngineTheme::with_css("button { padding-left: 9px; padding: 2px; }", "test.css");
    let style = resolve(&ThemeNode::new("button"), &theme);
    assert_eq!(style.padding.left.value, 2.0);
}

#[test]
fn whitespace_shorthands_reach_computed_style() {
    let theme = EngineTheme::with_css(
        r#"
        button {
            border: 1px solid #123456;
            border-radius: 4px 8px;
            background: #fff url("tile.png");
        }
        "#,
        "test.css",
    );
    let style = resolve(&ThemeNode::new("button"), &theme);
    assert_eq!(style.border.top.width.value, 1.0);
    assert_eq!(style.border.top.style, BorderStyle::Solid);
    assert_eq!(style.border.top.color, Some(color("#123456")));
    assert_eq!(style.radius.top_left.value, 4.0);
    assert_eq!(style.radius.top_right.value, 8.0);
    assert_eq!(style.background.len(), 2);
    assert!(matches!(style.background[0], Paint::Color(_)));
    assert!(matches!(style.background[1], Paint::Image(_)));
}
