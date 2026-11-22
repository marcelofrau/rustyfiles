//! GPUI element adapter for the GTK theme engine.
//!
//! `themed()` wraps existing builders; with the engine off it returns the
//! element untouched, so the shipped look and its interaction code stay as-is.

use gpui_kit::{Styled, px};
use rf_theme_engine::model::{StateFlags, ThemeNode};

/// Attach a theme node and apply the style changes a theme actually declared.
pub(crate) fn themed<T: Styled>(element: T, node: ThemeNode) -> T {
    let d = crate::theme_engine::delta(&node);
    if d.is_empty() {
        return element;
    }
    let mut out = element;
    if let Some(c) = d.bg {
        out = out.bg(hsla(c));
    }
    if let Some(c) = d.text {
        out = out.text_color(hsla(c));
    }
    if let Some(r) = d.radius.filter(|r| *r > 0.) {
        out = out.rounded(px(r));
    }
    if let Some([top, right, bottom, left]) = d.padding {
        out = out.pt(px(top)).pr(px(right)).pb(px(bottom)).pl(px(left));
    }
    if let Some(size) = d.font_size.filter(|s| *s > 0.) {
        out = out.text_size(px(size));
    }
    if let Some(o) = d.opacity {
        out = out.opacity(o);
    }
    out
}

pub(crate) fn titlebar_node(active: bool) -> ThemeNode {
    let mut node = ThemeNode::new("headerbar").with_class("titlebar").with_class("rf-titlebar");
    if active {
        node = node.with_state(StateFlags::ACTIVE);
    } else {
        node = node.with_state(StateFlags::BACKDROP);
    }
    node.ancestors.push(window_node(active));
    node
}

pub(crate) fn tabstrip_node(active: bool) -> ThemeNode {
    let mut node = ThemeNode::new("header").with_class("tabs").with_class("rf-tabstrip");
    if active {
        node = node.with_state(StateFlags::ACTIVE);
    }
    let mut notebook = ThemeNode::new("notebook");
    notebook.ancestors.push(window_node(active));
    node.ancestors.push(notebook);
    node
}

pub(crate) fn sidebar_row_node(selected: bool, focused: bool, disabled: bool, window_active: bool) -> ThemeNode {
    let mut node = ThemeNode::new("row").with_class("sidebar-row").with_class("folder").with_class("rf-row");
    if selected {
        node = node.with_state(StateFlags::SELECTED);
    }
    if focused {
        node = node.with_state(StateFlags::FOCUS | StateFlags::FOCUS_VISIBLE);
    }
    if disabled {
        node = node.with_state(StateFlags::DISABLED);
    }
    if window_active {
        node = node.with_state(StateFlags::ACTIVE);
    } else {
        node = node.with_state(StateFlags::BACKDROP);
    }
    let mut sidebar = ThemeNode::new("placessidebar").with_class("sidebar").with_class("rf-sidebar");
    sidebar.ancestors.push(window_node(window_active));
    node.ancestors.push(sidebar);
    node
}

pub(crate) fn file_row_node(is_dir: bool, selected: bool, focused: bool, dim: bool, window_active: bool) -> ThemeNode {
    let mut node = ThemeNode::new("row").with_class("file-row").with_class(if is_dir { "folder" } else { "file" });
    if selected {
        node = node.with_state(StateFlags::SELECTED);
    }
    if focused {
        node = node.with_state(StateFlags::FOCUS | StateFlags::FOCUS_VISIBLE);
    }
    if dim {
        node = node.with_state(StateFlags::DISABLED);
    }
    if window_active {
        node = node.with_state(StateFlags::ACTIVE);
    } else {
        node = node.with_state(StateFlags::BACKDROP);
    }
    let mut view = ThemeNode::new("treeview.view").with_class("view").with_class("file-list").with_class("details");
    view.ancestors.push(window_node(window_active));
    node.ancestors.push(view);
    node
}

fn window_node(window_active: bool) -> ThemeNode {
    let mut node = ThemeNode::new("window").with_class("background").with_class("rf-window").with_class("main");
    if window_active {
        node = node.with_state(StateFlags::ACTIVE);
    } else {
        node = node.with_state(StateFlags::BACKDROP);
    }
    node
}

fn hsla(c: rf_theme_engine::model::Color) -> gpui_kit::Hsla {
    gpui_kit::hsla(c.r as f32 / 255., c.g as f32 / 255., c.b as f32 / 255., c.a as f32 / 255.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_nodes_carry_gtk_names_and_states() {
        let title = titlebar_node(true);
        assert_eq!(title.name, "headerbar");
        assert!(title.classes.iter().any(|c| c == "rf-titlebar"));
        assert!(title.states.contains(StateFlags::ACTIVE));
        assert_eq!(title.ancestors[0].name, "window");

        let row = file_row_node(true, true, false, false, true);
        assert_eq!(row.name, "row");
        assert!(row.classes.iter().any(|c| c == "folder"));
        assert!(row.states.contains(StateFlags::SELECTED));
        assert!(row.states.contains(StateFlags::ACTIVE));

        let backdrop = sidebar_row_node(true, false, false, false);
        assert!(backdrop.states.contains(StateFlags::BACKDROP));
        assert!(!backdrop.states.contains(StateFlags::ACTIVE));
    }
}
