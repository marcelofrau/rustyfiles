//! Xfce's window-border themes (`<theme>/xfwm4/`): the `themerc` settings, the names of the pieces and
//! where each piece goes for a window of a given size, following xfwm4's own layout rules.
//!
//! A theme is a set of pictures: four corners, three sides (left, right, bottom), five title pieces
//! (`title-1` … `title-5`, left to right) and the buttons, each in an `-active` and `-inactive` form
//! (buttons also `-prelight`, `-pressed` and `-toggled-…`). The title height is `title-3`'s, the side
//! widths are the side pictures'. Pieces are tiled, never scaled.

use std::collections::HashMap;

use super::{Rgba, parse_color};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// What `themerc` (and xfwm4's defaults) say.
#[derive(Clone, Debug)]
pub struct Config {
    pub button_offset: i32,
    pub button_spacing: i32,
    /// Letters left of `|` sit at the left, the rest at the right: `O` menu, `T` stick, `S` shade, `H` hide,
    /// `M` maximize, `C` close.
    pub button_layout: String,
    pub title_alignment: Align,
    pub full_width_title: bool,
    pub title_horizontal_offset: i32,
    /// `[active, inactive]`.
    pub title_vertical_offset: [i32; 2],
    pub maximized_offset: i32,
    /// `[active, inactive]`: the text has a shadow.
    pub title_shadow: [bool; 2],
    /// Every `key=value`, for the colour symbols a theme sets itself.
    values: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        parse_themerc("")
    }
}

/// Reads a `themerc`: `key=value` lines, `#` comments. Unknown keys are kept (colour symbols), bad numbers
/// keep the default.
pub fn parse_themerc(text: &str) -> Config {
    let mut values = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            values.insert(k.trim().to_ascii_lowercase(), v.trim().trim_matches('"').to_string());
        }
    }
    let int = |k: &str, d: i32| values.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
    let yes = |k: &str, d: bool| values.get(k).map_or(d, |v| matches!(v.to_ascii_lowercase().as_str(), "true" | "yes" | "1" | "under" | "frame"));
    let title_alignment = match values.get("title_alignment").map(|v| v.to_ascii_lowercase()) {
        Some(v) if v == "left" => Align::Left,
        Some(v) if v == "right" => Align::Right,
        _ => Align::Center,
    };
    Config {
        button_offset: int("button_offset", 0),
        button_spacing: int("button_spacing", 0),
        // xfwm4's own default has shade too; on Windows the system set is minimize, maximize, close
        button_layout: values.get("button_layout").cloned().unwrap_or_else(|| "O|HMC".into()),
        title_alignment,
        full_width_title: yes("full_width_title", true),
        title_horizontal_offset: int("title_horizontal_offset", 0),
        title_vertical_offset: [int("title_vertical_offset_active", 0), int("title_vertical_offset_inactive", 0)],
        maximized_offset: int("maximized_offset", 0),
        title_shadow: [yes("title_shadow_active", false), yes("title_shadow_inactive", false)],
        values,
    }
}

/// The parts of a GTK style xfwm4 takes its colours from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Fg,
    Bg,
    Light,
    Dark,
    Mid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Normal,
    Selected,
    Insensitive,
}

/// xfwm4's colour symbols and the GTK colour each starts from.
pub const SYMBOLS: &[(&str, Part, State)] = {
    use Part::*;
    use State::*;
    &[
        ("active_text_color", Fg, Selected),
        ("inactive_text_color", Fg, Insensitive),
        ("active_text_shadow_color", Dark, Selected),
        ("inactive_text_shadow_color", Dark, Insensitive),
        ("active_border_color", Fg, Normal),
        ("inactive_border_color", Fg, Normal),
        ("active_color_1", Bg, Selected),
        ("active_hilight_1", Light, Selected),
        ("active_shadow_1", Dark, Selected),
        ("active_mid_1", Mid, Selected),
        ("active_text_color_2", Fg, Normal),
        ("active_color_2", Bg, Normal),
        ("active_hilight_2", Light, Normal),
        ("active_shadow_2", Dark, Normal),
        ("active_mid_2", Mid, Normal),
        ("inactive_color_1", Bg, Insensitive),
        ("inactive_hilight_1", Light, Insensitive),
        ("inactive_shadow_1", Dark, Insensitive),
        ("inactive_mid_1", Mid, Insensitive),
        ("inactive_text_color_2", Fg, Insensitive),
        ("inactive_color_2", Bg, Normal),
        ("inactive_hilight_2", Light, Normal),
        ("inactive_shadow_2", Dark, Normal),
        ("inactive_mid_2", Mid, Normal),
    ]
};

/// GTK2's derived colours: `light` and `dark` are `bg` shaded by 1.3 and 0.7, `mid` lies between them.
pub fn derived(part: Part, fg: Rgba, bg: Rgba) -> Rgba {
    match part {
        Part::Fg => fg,
        Part::Bg => bg,
        Part::Light => bg.shade(1.3),
        Part::Dark => bg.shade(0.7),
        Part::Mid => bg.shade(1.3).mix(bg.shade(0.7), 0.5),
    }
}

impl Config {
    /// A colour symbol's value: the theme's own `themerc` entry (which may name another symbol) over the GTK
    /// colour `gtk(part, state)`.
    pub fn symbol(&self, name: &str, gtk: &dyn Fn(Part, State) -> Rgba) -> Option<Rgba> {
        let mut name = name.to_ascii_lowercase();
        // (an alias chain is short; the bound only stops a loop)
        for _ in 0..4 {
            let base = SYMBOLS.iter().find(|(n, ..)| *n == name);
            match self.values.get(&name) {
                Some(v) => match parse_color(v) {
                    Some(c) => return Some(c),
                    None if SYMBOLS.iter().any(|(n, ..)| n.eq_ignore_ascii_case(v)) => name = v.to_ascii_lowercase(),
                    None => return base.map(|(_, p, s)| gtk(*p, *s)),
                },
                None => return base.map(|(_, p, s)| gtk(*p, *s)),
            }
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    TopLeft,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
    /// `title-1` … `title-5`.
    Title(u8),
}

impl Piece {
    pub const ALL: [Piece; 12] = [
        Piece::TopLeft,
        Piece::TopRight,
        Piece::Left,
        Piece::Right,
        Piece::BottomLeft,
        Piece::Bottom,
        Piece::BottomRight,
        Piece::Title(1),
        Piece::Title(2),
        Piece::Title(3),
        Piece::Title(4),
        Piece::Title(5),
    ];

    /// The file name without extension, e.g. `title-3-active`.
    pub fn file(self, active: bool) -> String {
        let base = match self {
            Piece::TopLeft => "top-left".to_string(),
            Piece::TopRight => "top-right".into(),
            Piece::Left => "left".into(),
            Piece::Right => "right".into(),
            Piece::BottomLeft => "bottom-left".into(),
            Piece::Bottom => "bottom".into(),
            Piece::BottomRight => "bottom-right".into(),
            Piece::Title(n) => format!("title-{n}"),
        };
        format!("{base}-{}", if active { "active" } else { "inactive" })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    Menu,
    Stick,
    Shade,
    Hide,
    Maximize,
    Close,
}

impl Button {
    pub fn from_letter(c: char) -> Option<Button> {
        Some(match c {
            'O' => Button::Menu,
            'T' => Button::Stick,
            'S' => Button::Shade,
            'H' => Button::Hide,
            'M' => Button::Maximize,
            'C' => Button::Close,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Button::Menu => "menu",
            Button::Stick => "stick",
            Button::Shade => "shade",
            Button::Hide => "hide",
            Button::Maximize => "maximize",
            Button::Close => "close",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Active,
    Inactive,
    Prelight,
    Pressed,
}

impl ButtonState {
    pub const ALL: [ButtonState; 4] = [ButtonState::Active, ButtonState::Inactive, ButtonState::Prelight, ButtonState::Pressed];
}

/// File names to try for a button, best first: the toggled form (a maximized window's maximize button) falls back
/// to the plain one, a missing state to `active`.
pub fn button_files(b: Button, toggled: bool, state: ButtonState) -> Vec<String> {
    let st = match state {
        ButtonState::Active => "active",
        ButtonState::Inactive => "inactive",
        ButtonState::Prelight => "prelight",
        ButtonState::Pressed => "pressed",
    };
    let mut v = Vec::new();
    for t in if toggled { &["-toggled", ""][..] } else { &[""][..] } {
        v.push(format!("{}{t}-{st}", b.name()));
        if state != ButtonState::Active {
            v.push(format!("{}{t}-active", b.name()));
        }
    }
    v
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Where everything goes, in window coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// The frame around the content: title height and the side and bottom widths.
    pub top: i32,
    pub left: i32,
    pub right: i32,
    pub bottom: i32,
    /// Pictures to tile into each rect, in painting order.
    pub pieces: Vec<(Piece, Rect)>,
    pub buttons: Vec<(Button, Rect)>,
    /// The free part of the title between the button groups.
    pub title: Rect,
    /// Where the title text starts (its vertical centre is the title's, moved by the offset).
    pub text_x: i32,
    pub text_dy: i32,
}

/// The frame of a `width` × `height` window. `size` gives each piece's picture size (`(0, 0)` when the theme has
/// none), `button` the size of a button's picture (`None`: not shown). `text_w` is the title text's width; `None`
/// lays the title out full width whatever the theme says (the title bar holds content of its own, e.g. tabs).
pub fn layout(
    cfg: &Config,
    size: &dyn Fn(Piece) -> (i32, i32),
    button: &dyn Fn(Button) -> Option<(i32, i32)>,
    (width, height): (i32, i32),
    text_w: Option<i32>,
    active: bool,
    maximized: bool,
) -> Layout {
    let sz = |p: Piece| if maximized && !matches!(p, Piece::Title(_)) { (0, 0) } else { size(p) };
    let top = size(Piece::Title(3)).1;
    let (left, right, bottom) = (sz(Piece::Left).0, sz(Piece::Right).0, sz(Piece::Bottom).1);
    let mut out = Layout { top, left, right, bottom, ..Default::default() };

    // buttons, from the outside in
    let spacing = cfg.button_spacing;
    let offset = if maximized { cfg.maximized_offset.max(0) } else { cfg.button_offset };
    let (lefts, rights) = cfg.button_layout.split_once('|').unwrap_or((cfg.button_layout.as_str(), ""));
    let mut lx = (left + offset).max(0);
    let limit = width - right - offset;
    for b in lefts.chars().filter_map(Button::from_letter) {
        let Some((bw, bh)) = button(b) else { continue };
        if lx + bw + spacing < limit {
            out.buttons.push((b, Rect { x: lx, y: (top - bh + 1) / 2, w: bw, h: bh }));
            lx += bw + spacing;
        }
    }
    let mut rx = width - right + spacing - offset;
    for b in rights.chars().rev().filter_map(Button::from_letter) {
        let Some((bw, bh)) = button(b) else { continue };
        if rx - bw - spacing > lx {
            rx -= bw + spacing;
            out.buttons.push((b, Rect { x: rx, y: (top - bh + 1) / 2, w: bw, h: bh }));
        }
    }

    // the title strip between the top corners
    let (tl, tr) = (sz(Piece::TopLeft), sz(Piece::TopRight));
    let sx = tl.0;
    let sw = (width - tl.0 - tr.0).max(0);
    let (mut l, mut r) = ((lx - spacing - sx).clamp(0, sw), (rx - sx).clamp(0, sw));
    if l > r {
        std::mem::swap(&mut l, &mut r);
    }
    out.title = Rect { x: sx + l, y: 0, w: r - l, h: top };
    let (w2, w4) = (size(Piece::Title(2)).0, size(Piece::Title(4)).0);
    let hoff = cfg.title_horizontal_offset;
    let (w1, w3, text_in_3) = match text_w {
        Some(tw) if !cfg.full_width_title => {
            let w3 = tw.min(sw - w2 - w4).max(0);
            let w1 = match cfg.title_alignment {
                Align::Left => l + hoff,
                Align::Right => r - (w2 + w3 + w4) - hoff,
                Align::Center => (l + r) / 2 - w3 / 2 - w2,
            };
            (w1.max(l), w3, 0)
        }
        _ => {
            let w3 = (r - l - w2 - w4).max(0);
            let tw = text_w.unwrap_or(0);
            let x = match cfg.title_alignment {
                Align::Left => hoff,
                Align::Right => w3 - tw - hoff,
                Align::Center => w3 / 2 - tw / 2,
            };
            (l, w3, x.max(hoff))
        }
    };
    let mut x = 0;
    let mut put = |p: Piece, from: i32, w: i32| {
        if w > 0 {
            out.pieces.push((p, Rect { x: sx + from, y: 0, w, h: top }));
        }
    };
    put(Piece::Title(1), x, w1);
    x += w1;
    put(Piece::Title(2), x, w2);
    x += w2;
    let x3 = x;
    put(Piece::Title(3), x, w3);
    x = (x + w3).min(r - w4).max(0);
    put(Piece::Title(4), x, w4);
    x += w4;
    put(Piece::Title(5), x, sw - x);
    out.text_x = sx + x3 + text_in_3;
    out.text_dy = cfg.title_vertical_offset[usize::from(!active)];

    if !maximized {
        let (bl, br) = (sz(Piece::BottomLeft), sz(Piece::BottomRight));
        let mut put = |p: Piece, r: Rect| {
            if r.w > 0 && r.h > 0 {
                out.pieces.push((p, r));
            }
        };
        put(Piece::Left, Rect { x: 0, y: top, w: left, h: height - top - bl.1 });
        put(Piece::Right, Rect { x: width - right, y: top, w: right, h: height - top - br.1 });
        put(Piece::Bottom, Rect { x: bl.0, y: height - bottom, w: width - bl.0 - br.0, h: bottom });
        put(Piece::BottomLeft, Rect { x: 0, y: height - bl.1, w: bl.0, h: bl.1 });
        put(Piece::BottomRight, Rect { x: width - br.0, y: height - br.1, w: br.0, h: br.1 });
        put(Piece::TopLeft, Rect { x: 0, y: 0, w: tl.0, h: tl.1 });
        put(Piece::TopRight, Rect { x: width - tr.0, y: 0, w: tr.0, h: tr.1 });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sizes(p: Piece) -> (i32, i32) {
        match p {
            Piece::TopLeft | Piece::TopRight => (6, 24),
            Piece::Left | Piece::Right => (4, 1),
            Piece::Bottom => (1, 4),
            Piece::BottomLeft | Piece::BottomRight => (6, 6),
            Piece::Title(2) | Piece::Title(4) => (5, 24),
            Piece::Title(_) => (1, 24),
        }
    }

    fn buttons(b: Button) -> Option<(i32, i32)> {
        (b != Button::Menu).then_some((20, 18))
    }

    #[test]
    fn themerc_values_and_defaults() {
        let c = parse_themerc("# comment\nbutton_offset=3\nbutton_spacing = 2\ntitle_alignment=left\nfull_width_title=false\nbogus\nbutton_layout=O|SHMC\n");
        assert_eq!((c.button_offset, c.button_spacing), (3, 2));
        assert_eq!(c.title_alignment, Align::Left);
        assert!(!c.full_width_title);
        assert_eq!(c.button_layout, "O|SHMC");
        let d = Config::default();
        assert_eq!((d.title_alignment, d.full_width_title, d.button_layout.as_str()), (Align::Center, true, "O|HMC"));
    }

    #[test]
    fn symbols_follow_themerc_then_gtk() {
        let c = parse_themerc("active_color_1=#ff0000\nactive_text_color=active_text_color_2\n");
        let gtk = |p: Part, s: State| match (p, s) {
            (Part::Fg, State::Normal) => Rgba::rgb(1, 2, 3),
            _ => Rgba::rgb(9, 9, 9),
        };
        assert_eq!(c.symbol("active_color_1", &gtk), Some(Rgba::rgb(255, 0, 0)));
        assert_eq!(c.symbol("ACTIVE_TEXT_COLOR", &gtk), Some(Rgba::rgb(1, 2, 3)));
        assert_eq!(c.symbol("inactive_mid_2", &gtk), Some(Rgba::rgb(9, 9, 9)));
        assert_eq!(c.symbol("no_such_symbol", &gtk), None);
    }

    #[test]
    fn button_fallbacks() {
        assert_eq!(button_files(Button::Close, false, ButtonState::Active), ["close-active"]);
        assert_eq!(
            button_files(Button::Maximize, true, ButtonState::Prelight),
            ["maximize-toggled-prelight", "maximize-toggled-active", "maximize-prelight", "maximize-active"]
        );
    }

    #[test]
    fn full_width_frame() {
        let cfg = Config::default();
        let l = layout(&cfg, &sizes, &buttons, (200, 100), None, true, false);
        assert_eq!((l.top, l.left, l.right, l.bottom), (24, 4, 4, 4));
        // H M C at the right, from the right border inwards, centred vertically; the menu has no picture
        let at: Vec<_> = l.buttons.iter().map(|(b, r)| (*b, r.x, r.y)).collect();
        assert_eq!(at, [(Button::Close, 176, 3), (Button::Maximize, 156, 3), (Button::Hide, 136, 3)]);
        assert_eq!(l.title, Rect { x: 6, y: 0, w: 130, h: 24 });
        // title-2 and -4 keep their width, title-3 fills the free part, title-5 runs under the buttons
        let t: Vec<_> = l.pieces.iter().filter(|(p, _)| matches!(p, Piece::Title(_))).map(|(p, r)| (*p, r.x, r.w)).collect();
        assert_eq!(t, [(Piece::Title(2), 6, 5), (Piece::Title(3), 11, 120), (Piece::Title(4), 131, 5), (Piece::Title(5), 136, 58)]);
        let find = |p| l.pieces.iter().find(|(q, _)| *q == p).unwrap().1;
        assert_eq!(find(Piece::Left), Rect { x: 0, y: 24, w: 4, h: 70 });
        assert_eq!(find(Piece::Bottom), Rect { x: 6, y: 96, w: 188, h: 4 });
        assert_eq!(find(Piece::BottomRight), Rect { x: 194, y: 94, w: 6, h: 6 });
    }

    #[test]
    fn centred_title_text() {
        let cfg = parse_themerc("full_width_title=false");
        let l = layout(&cfg, &sizes, &buttons, (200, 100), Some(40), true, false);
        let t3 = l.pieces.iter().find(|(p, _)| *p == Piece::Title(3)).unwrap().1;
        assert_eq!((t3.x, t3.w), (51, 40));
        assert_eq!(l.text_x, 51);
    }

    #[test]
    fn maximized_drops_the_border() {
        let cfg = parse_themerc("button_offset=8\nmaximized_offset=1");
        let l = layout(&cfg, &sizes, &buttons, (200, 100), None, false, true);
        assert_eq!((l.left, l.right, l.bottom), (0, 0, 0));
        assert!(l.pieces.iter().all(|(p, _)| matches!(p, Piece::Title(_))));
        assert_eq!(l.buttons[0].1.x, 179);
    }
}
