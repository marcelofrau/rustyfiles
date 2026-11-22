//! Cascade and computed style. Pure data: no GPUI, no files, no app state.

use std::collections::BTreeMap;

use super::selector;
use super::value::{Function, Value};
use crate::model::{Border, BorderEdges, BorderStyle, Color, Corners, Edges, FontSpec, Length, Paint, Shadow, ThemeNode};
use crate::style::ComputedStyle;
use crate::theme::EngineTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Rank {
    important: bool,
    ids: u32,
    classes: u32,
    elements: u32,
    order: usize,
}

#[derive(Debug, Clone)]
struct Winner {
    rank: Rank,
    value: Value,
}

/// Resolve a root node with no inherited parent.
pub fn resolve(node: &ThemeNode, theme: &EngineTheme) -> ComputedStyle {
    resolve_with_parent(node, None, theme)
}

/// Resolve a node against a parent style. Inherited properties come from the
/// parent; all other properties start from the engine fallback.
pub fn resolve_with_parent(node: &ThemeNode, parent: Option<&ComputedStyle>, theme: &EngineTheme) -> ComputedStyle {
    let fallback = ComputedStyle::from_palette(theme.fallback.clone());
    let mut winners: BTreeMap<String, Winner> = BTreeMap::new();
    let mut order = 0usize;

    for sheet in &theme.stylesheets {
        for rule in &sheet.rules {
            for selector in &rule.selectors {
                if !selector::matches(node, selector) {
                    continue;
                }
                let (ids, classes, elements) = selector.specificity();
                for declaration in &rule.declarations {
                    order += 1;
                    let rank = Rank { important: declaration.important, ids, classes, elements, order };
                    let replace = winners
                        .get(&declaration.property)
                        .is_none_or(|winner| rank > winner.rank);
                    if replace {
                        winners.insert(
                            declaration.property.clone(),
                            Winner { rank, value: declaration.value.clone() },
                        );
                    }
                }
            }
        }
    }

    let mut out = parent.cloned().unwrap_or_else(|| fallback.clone());
    if parent.is_some() {
        out.background = fallback.background.clone();
        out.border = fallback.border.clone();
        out.radius = fallback.radius;
        out.shadow = fallback.shadow.clone();
        out.text_shadow = fallback.text_shadow.clone();
        out.padding = fallback.padding;
        out.margin = fallback.margin;
        out.opacity = fallback.opacity;
    }

    // Longhands and shorthands can touch the same field, so declaration order
    // must survive the property-keyed winner map.
    let mut ordered: Vec<_> = winners.into_iter().collect();
    ordered.sort_by_key(|(_, winner)| winner.rank.order);
    for (property, winner) in ordered {
        apply(&mut out, &property, &winner.value, parent, &fallback);
    }

    out.palette.apply_overrides(&theme.overrides);
    if theme.overrides.contains_key("text") {
        out.color = out.palette.text;
    }
    out
}

/// GTK inherits typography, foreground and icon style, but not surfaces.
pub fn is_inherited(property: &str) -> bool {
    matches!(
        property,
        "color"
            | "font"
            | "font-family"
            | "font-size"
            | "font-style"
            | "font-weight"
            | "-gtk-icon-style"
            | "-gtk-icon-shadow"
            | "caret-color"
            | "selection-background-color"
            | "text-decoration-color"
    )
}

fn apply(
    out: &mut ComputedStyle,
    property: &str,
    value: &Value,
    parent: Option<&ComputedStyle>,
    fallback: &ComputedStyle,
) {
    match value {
        Value::Inherit => {
            if let Some(parent) = parent {
                copy_property(out, parent, property);
            }
        }
        Value::Initial => copy_property(out, fallback, property),
        Value::Unset => {
            let source = if is_inherited(property) { parent } else { Some(fallback) };
            if let Some(source) = source {
                copy_property(out, source, property);
            }
        }
        _ => apply_value(out, property, value),
    }
}

fn copy_property(out: &mut ComputedStyle, source: &ComputedStyle, property: &str) {
    match property {
        "color" => out.color = source.color,
        "background" | "background-color" | "background-image" => out.background = source.background.clone(),
        "border" | "border-color" | "border-width" | "border-style" => out.border = source.border.clone(),
        p if p.starts_with("border-") && !p.ends_with("-radius") && !p.contains("-radius-") => {
            out.border = source.border.clone()
        }
        p if p.starts_with("border-radius") || p.starts_with("border-top-left-radius")
            || p.starts_with("border-top-right-radius") || p.starts_with("border-bottom-left-radius")
            || p.starts_with("border-bottom-right-radius") =>
        {
            out.radius = source.radius
        }
        "box-shadow" => out.shadow = source.shadow.clone(),
        "text-shadow" => out.text_shadow = source.text_shadow.clone(),
        "font" | "font-family" | "font-size" | "font-style" | "font-weight" => out.font = source.font.clone(),
        "padding" | "padding-left" | "padding-right" | "padding-top" | "padding-bottom" => out.padding = source.padding,
        "margin" | "margin-left" | "margin-right" | "margin-top" | "margin-bottom" => out.margin = source.margin,
        "opacity" => out.opacity = source.opacity,
        "-gtk-icon-style" => out.icon_style = source.icon_style.clone(),
        _ => {}
    }
}

fn apply_value(out: &mut ComputedStyle, property: &str, value: &Value) {
    match property {
        "color" => {
            if let Some(color) = color_of(value) {
                out.color = color;
            }
        }
        "background-color" => {
            if let Some(color) = color_of(value) {
                let paint = Paint::Color(color);
                if out.background.is_empty() {
                    out.background.push(paint);
                } else if matches!(out.background[0], Paint::Color(_)) {
                    out.background[0] = paint;
                } else {
                    out.background.insert(0, paint);
                }
            }
        }
        "background" | "background-image" => {
            let paints = paints_of(value);
            if !paints.is_empty() {
                if property == "background-image" {
                    out.background.extend(paints);
                } else {
                    out.background = paints;
                }
            }
        }
        "border-radius" | "border-radius-topleft" | "border-radius-topright" | "border-radius-bottomleft"
        | "border-radius-bottomright" | "border-top-left-radius" | "border-top-right-radius"
        | "border-bottom-left-radius" | "border-bottom-right-radius" => apply_radius(&mut out.radius, property, value),
        "border" => {
            let parts = parts_of(value);
            for border in [&mut out.border.top, &mut out.border.right, &mut out.border.bottom, &mut out.border.left] {
                apply_border_parts(border, &parts);
            }
        }
        "border-color" => assign_borders(&mut out.border, value, BorderPart::Color),
        "border-width" => assign_borders(&mut out.border, value, BorderPart::Width),
        "border-style" => assign_borders(&mut out.border, value, BorderPart::Style),
        p if p.starts_with("border-") && !p.contains("radius") => apply_single_border(&mut out.border, p, value),
        "padding" | "padding-left" | "padding-right" | "padding-top" | "padding-bottom" => {
            apply_edges(&mut out.padding, property, value)
        }
        "margin" | "margin-left" | "margin-right" | "margin-top" | "margin-bottom" => {
            apply_edges(&mut out.margin, property, value)
        }
        "box-shadow" => out.shadow = shadows_of(value),
        "text-shadow" => out.text_shadow = shadows_of(value),
        "font-size" => {
            if let Some(length) = length_of(value) {
                out.font.size = Some(length);
            }
        }
        "font-family" => out.font.family = text_of(value),
        "font-weight" => out.font.weight = text_of(value),
        "font-style" => out.font.style = text_of(value),
        "font" => apply_font(&mut out.font, value),
        "opacity" => {
            if let Value::Number(n) = value {
                out.opacity = *n;
            }
        }
        "-gtk-icon-style" => out.icon_style = text_of(value),
        "selection-background-color" => {
            if let Some(color) = color_of(value) {
                out.palette.selection = color;
            }
        }
        "selected-color" => {
            if let Some(color) = color_of(value) {
                out.palette.accent_text = color;
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Copy)]
enum BorderPart {
    Color,
    Width,
    Style,
}

fn assign_borders(border: &mut BorderEdges, value: &Value, part: BorderPart) {
    let values = match part {
        BorderPart::Color => colors_of(value).into_iter().map(Value::Color).collect(),
        _ => parts_of(value),
    };
    let values = if values.is_empty() { parts_of(value) } else { values };
    let n = values.len().clamp(1, 4);
    let sides: [&mut Border; 4] = [&mut border.top, &mut border.right, &mut border.bottom, &mut border.left];
    for (index, side) in sides.into_iter().enumerate() {
        if index >= n {
            break;
        }
        let value_index = match n {
            1 => 0,
            2 => index % 2,
            3 => [0, 1, 2, 1][index],
            _ => index,
        };
        assign_border_part(side, &values[value_index], part);
    }
}

fn apply_single_border(border: &mut BorderEdges, property: &str, value: &Value) {
    let (side, part) = if let Some(rest) = property.strip_prefix("border-top-") {
        (0, rest)
    } else if let Some(rest) = property.strip_prefix("border-right-") {
        (1, rest)
    } else if let Some(rest) = property.strip_prefix("border-bottom-") {
        (2, rest)
    } else if let Some(rest) = property.strip_prefix("border-left-") {
        (3, rest)
    } else {
        return;
    };
    let target = match side {
        0 => &mut border.top,
        1 => &mut border.right,
        2 => &mut border.bottom,
        _ => &mut border.left,
    };
    match part {
        "color" => assign_border_part(target, value, BorderPart::Color),
        "width" => assign_border_part(target, value, BorderPart::Width),
        "style" => assign_border_part(target, value, BorderPart::Style),
        _ => apply_border_parts(target, &parts_of(value)),
    }
}

fn assign_border_part(border: &mut Border, value: &Value, part: BorderPart) {
    match part {
        BorderPart::Color => {
            if let Some(color) = color_of(value) {
                border.color = Some(color);
            }
        }
        BorderPart::Width => {
            if let Some(length) = length_of(value) {
                border.width = length;
            }
        }
        BorderPart::Style => {
            if let Some(style) = text_of(value).and_then(|s| BorderStyle::parse(&s)) {
                border.style = style;
            }
        }
    }
}

fn apply_border_parts(border: &mut Border, parts: &[Value]) {
    for part in parts {
        if let Some(color) = color_of(part) {
            border.color = Some(color);
        } else if let Some(length) = length_of(part) {
            border.width = length;
        } else if let Some(style) = text_of(part).and_then(|s| BorderStyle::parse(&s)) {
            border.style = style;
        }
    }
}

fn apply_radius(radius: &mut Corners, property: &str, value: &Value) {
    let lengths = lengths_of(value);
    if lengths.is_empty() {
        return;
    }
    match property {
        "border-radius" => {
            if let Some(corners) = Corners::from_lengths(&lengths) {
                *radius = corners;
            }
        }
        "border-radius-topleft" | "border-top-left-radius" => radius.top_left = lengths[0],
        "border-radius-topright" | "border-top-right-radius" => radius.top_right = lengths[0],
        "border-radius-bottomleft" | "border-bottom-left-radius" => radius.bottom_left = lengths[0],
        "border-radius-bottomright" | "border-bottom-right-radius" => radius.bottom_right = lengths[0],
        _ => {}
    }
}

fn apply_edges(edges: &mut Edges, property: &str, value: &Value) {
    let Some(lengths) = edges_of(value) else {
        return;
    };
    match property {
        "padding" | "margin" => *edges = lengths,
        "padding-left" | "margin-left" => edges.left = lengths.left,
        "padding-right" | "margin-right" => edges.right = lengths.right,
        "padding-top" | "margin-top" => edges.top = lengths.top,
        "padding-bottom" | "margin-bottom" => edges.bottom = lengths.bottom,
        _ => {}
    }
}

fn apply_font(font: &mut FontSpec, value: &Value) {
    for part in parts_of(value) {
        if let Some(size) = length_of(&part) {
            font.size = Some(size);
        } else if let Some(style) = text_of(&part) {
            let lower = style.to_ascii_lowercase();
            if BorderStyle::parse(&lower).is_none() && matches!(lower.as_str(), "normal" | "italic" | "oblique") {
                font.style = Some(lower);
            } else if matches!(lower.as_str(), "bold" | "lighter" | "bolder") {
                font.weight = Some(lower);
            } else if font.family.is_none() {
                font.family = Some(style);
            }
        }
    }
}

fn parts_of(value: &Value) -> Vec<Value> {
    match value {
        Value::List(items) => items.clone(),
        _ => vec![value.clone()],
    }
}

fn colors_of(value: &Value) -> Vec<Color> {
    parts_of(value).iter().filter_map(color_of).collect()
}

fn lengths_of(value: &Value) -> Vec<Length> {
    parts_of(value).iter().filter_map(length_of).collect()
}

fn color_of(value: &Value) -> Option<Color> {
    match value {
        Value::Color(color) => Some(*color),
        Value::Function(function) => eval_color_function(function),
        Value::Raw(text) => Color::parse_hex(text).or_else(|| Color::parse_named(text)),
        _ => None,
    }
}

fn eval_color_function(function: &Function) -> Option<Color> {
    let mut args = function.args.iter();
    let first = color_of(args.next()?)?;
    match (function.name.as_str(), args.as_slice()) {
        ("shade", [Value::Number(factor)]) => Some(scale(first, *factor)),
        ("lighter" | "darker", []) => {
            let target = if function.name == "lighter" { Color::new(255, 255, 255, 255) } else { Color::new(0, 0, 0, 255) };
            Some(mix(first, target, 0.25))
        }
        ("alpha", [Value::Number(a)]) => Some(Color::new(first.r, first.g, first.b, unit_alpha(*a))),
        ("mix", [other, Value::Number(rate)]) => Some(mix(first, color_of(other)?, *rate)),
        _ => None,
    }
}

fn scale(color: Color, factor: f32) -> Color {
    let chan = |v: u8| ((v as f32 * factor).round().clamp(0.0, 255.0)) as u8;
    Color::new(chan(color.r), chan(color.g), chan(color.b), color.a)
}

fn mix(a: Color, b: Color, rate: f32) -> Color {
    let rate = rate.clamp(0.0, 1.0);
    let chan = |x: u8, y: u8| ((x as f32 * (1.0 - rate) + y as f32 * rate).round()) as u8;
    Color::new(chan(a.r, b.r), chan(a.g, b.g), chan(a.b, b.b), chan(a.a, b.a))
}

fn unit_alpha(a: f32) -> u8 {
    let a = if a > 1.0 { a / 100.0 } else { a };
    (a.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn length_of(value: &Value) -> Option<Length> {
    match value {
        Value::Length(length) => Some(*length),
        Value::Number(number) => Some(Length::new(*number, crate::model::LengthUnit::Px)),
        _ => None,
    }
}

fn edges_of(value: &Value) -> Option<Edges> {
    match value {
        Value::Edges(edges) => Some(*edges),
        _ => Edges::from_lengths(&lengths_of(value)),
    }
}

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Ident(text) | Value::String(text) | Value::Raw(text) => Some(text.clone()),
        _ => None,
    }
}

fn shadows_of(value: &Value) -> Vec<Shadow> {
    match value {
        Value::List(items) => items.iter().filter_map(shadow_of).collect(),
        _ => shadow_of(value).into_iter().collect(),
    }
}

fn shadow_of(value: &Value) -> Option<Shadow> {
    match value {
        Value::Shadow(shadow) => Some(shadow.clone()),
        _ => None,
    }
}

fn paints_of(value: &Value) -> Vec<Paint> {
    match value {
        Value::Color(color) => vec![Paint::Color(*color)],
        Value::Gradient(gradient) => vec![Paint::Gradient(gradient.clone())],
        Value::Url(path) => vec![Paint::Image(path.clone())],
        Value::List(items) => items.iter().flat_map(paints_of).collect(),
        _ => Vec::new(),
    }
}
