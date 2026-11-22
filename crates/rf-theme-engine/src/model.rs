//! Engine-neutral value types. No GPUI, no Windows, no filesystem.

/// sRGB color with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// `#RGB`, `#RRGGBB` or `#RRGGBBAA`, with or without the leading `#`.
    pub fn parse_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        let hex = |i: usize| -> Option<u8> { s.as_bytes().get(i).and_then(|c| hex_digit(*c)) };
        match s.len() {
            3 => Some(Self::new(hex(0)? * 17, hex(1)? * 17, hex(2)? * 17, 255)),
            6 => Some(Self::new((hex(0)? << 4) | hex(1)?, (hex(2)? << 4) | hex(3)?, (hex(4)? << 4) | hex(5)?, 255)),
            8 => Some(Self::new((hex(0)? << 4) | hex(1)?, (hex(2)? << 4) | hex(3)?, (hex(4)? << 4) | hex(5)?, (hex(6)? << 4) | hex(7)?)),
            _ => None,
        }
    }

    pub fn parse_named(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "transparent" => Self::new(0, 0, 0, 0),
            "white" => Self::new(255, 255, 255, 255),
            "black" => Self::new(0, 0, 0, 255),
            "red" => Self::new(255, 0, 0, 255),
            "lime" | "green" => Self::new(0, 128, 0, 255),
            "blue" => Self::new(0, 0, 255, 255),
            "yellow" => Self::new(255, 255, 0, 255),
            "orange" => Self::new(255, 165, 0, 255),
            "gray" | "grey" => Self::new(128, 128, 128, 255),
            "silver" => Self::new(192, 192, 192, 255),
            _ => return None,
        })
    }

    /// GTK color functions use 0..=1 floats or 0..=100 percentages.
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        let conv = |v: f32, percent: bool| -> u8 {
            let v = if percent { v / 100.0 } else { v };
            (v.clamp(0.0, 1.0) * 255.0).round() as u8
        };
        Self::new(conv(r, r > 1.0), conv(g, g > 1.0), conv(b, b > 1.0), conv(a, a > 1.0))
    }
}

fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    Px,
    Pt,
    Em,
    Rem,
    Percent,
    Mm,
    Cm,
    In,
    Vh,
    Vw,
    Deg,
    Ms,
    S,
    None,
}

impl LengthUnit {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "" => Self::None,
            "px" => Self::Px,
            "pt" => Self::Pt,
            "em" => Self::Em,
            "rem" => Self::Rem,
            "%" => Self::Percent,
            "mm" => Self::Mm,
            "cm" => Self::Cm,
            "in" => Self::In,
            "vh" => Self::Vh,
            "vw" => Self::Vw,
            "deg" => Self::Deg,
            "ms" => Self::Ms,
            "s" => Self::S,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length {
    pub value: f32,
    pub unit: LengthUnit,
}

impl Length {
    pub const fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, LengthUnit::Px)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edges {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

impl Edges {
    pub const fn zero() -> Self {
        let zero = Length::zero();
        Self { top: zero, right: zero, bottom: zero, left: zero }
    }

    /// CSS 1/2/3/4-value shorthand.
    pub fn from_lengths(v: &[Length]) -> Option<Self> {
        match v.len() {
            1 => Some(Self { top: v[0], right: v[0], bottom: v[0], left: v[0] }),
            2 => Some(Self { top: v[0], right: v[1], bottom: v[0], left: v[1] }),
            3 => Some(Self { top: v[0], right: v[1], bottom: v[2], left: v[1] }),
            4 => Some(Self { top: v[0], right: v[1], bottom: v[2], left: v[3] }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientKind {
    Linear,
    Radial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub color: Option<Color>,
    pub position: Option<Length>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub angle: Option<Length>,
    pub stops: Vec<ColorStop>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shadow {
    pub offset_x: Length,
    pub offset_y: Length,
    pub blur: Option<Length>,
    pub spread: Option<Length>,
    pub color: Option<Color>,
    pub inset: bool,
}

/// GTK state pseudo-classes. The adapter sets `FOCUS_WITHIN` on ancestors and
/// `BACKDROP` only on the active window; `SELECTED` comes from app state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StateFlags(u32);

impl StateFlags {
    pub const HOVER: Self = Self(1 << 0);
    pub const ACTIVE: Self = Self(1 << 1);
    pub const SELECTED: Self = Self(1 << 2);
    pub const CHECKED: Self = Self(1 << 3);
    pub const DISABLED: Self = Self(1 << 4);
    pub const FOCUS: Self = Self(1 << 5);
    pub const FOCUS_WITHIN: Self = Self(1 << 6);
    pub const FOCUS_VISIBLE: Self = Self(1 << 7);
    pub const BACKDROP: Self = Self(1 << 8);
    pub const INDETERMINATE: Self = Self(1 << 9);
    /// GTK alias for `:hover`.
    pub const PRELIGHT: Self = Self::HOVER;
    /// GTK alias for `:disabled`.
    pub const INSENSITIVE: Self = Self::DISABLED;

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for StateFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// One node in the themed tree. Ancestors are nearest last; `previous` holds
/// preceding siblings, also nearest last. The adapter fills those lists so the
/// matcher can handle child, descendant and sibling combinators without a DOM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeNode {
    pub name: String,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub states: StateFlags,
    pub ancestors: Vec<ThemeNode>,
    pub previous: Vec<ThemeNode>,
    pub index: usize,
    pub sibling_count: usize,
}

impl ThemeNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            classes: Vec::new(),
            id: None,
            states: StateFlags::default(),
            ancestors: Vec::new(),
            previous: Vec::new(),
            index: 0,
            sibling_count: 1,
        }
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    pub fn with_state(mut self, state: StateFlags) -> Self {
        self.states = self.states.union(state);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Hidden,
    Solid,
    Dashed,
    Dotted,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl BorderStyle {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "none" => Self::None,
            "hidden" => Self::Hidden,
            "solid" => Self::Solid,
            "dashed" => Self::Dashed,
            "dotted" => Self::Dotted,
            "double" => Self::Double,
            "groove" => Self::Groove,
            "ridge" => Self::Ridge,
            "inset" => Self::Inset,
            "outset" => Self::Outset,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    pub width: Length,
    pub color: Option<Color>,
    pub style: BorderStyle,
}

impl Border {
    pub const fn none() -> Self {
        Self { width: Length::zero(), color: None, style: BorderStyle::None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderEdges {
    pub top: Border,
    pub right: Border,
    pub bottom: Border,
    pub left: Border,
}

impl BorderEdges {
    pub const fn none() -> Self {
        let none = Border::none();
        Self { top: none, right: none, bottom: none, left: none }
    }

    pub fn from_borders(v: &[Border]) -> Option<Self> {
        let get = |i: usize| v.get(i).copied().unwrap_or_else(Border::none);
        match v.len() {
            1 => Some(Self { top: get(0), right: get(0), bottom: get(0), left: get(0) }),
            2 => Some(Self { top: get(0), right: get(1), bottom: get(0), left: get(1) }),
            3 => Some(Self { top: get(0), right: get(1), bottom: get(2), left: get(1) }),
            4 => Some(Self { top: get(0), right: get(1), bottom: get(2), left: get(3) }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corners {
    pub top_left: Length,
    pub top_right: Length,
    pub bottom_right: Length,
    pub bottom_left: Length,
}

impl Corners {
    pub const fn zero() -> Self {
        let zero = Length::zero();
        Self { top_left: zero, top_right: zero, bottom_right: zero, bottom_left: zero }
    }

    pub fn from_lengths(v: &[Length]) -> Option<Self> {
        let get = |i: usize| v.get(i).copied().unwrap_or_else(Length::zero);
        match v.len() {
            1 => Some(Self { top_left: get(0), top_right: get(0), bottom_right: get(0), bottom_left: get(0) }),
            2 => Some(Self { top_left: get(0), top_right: get(1), bottom_right: get(0), bottom_left: get(1) }),
            3 => Some(Self { top_left: get(0), top_right: get(1), bottom_right: get(2), bottom_left: get(1) }),
            4 => Some(Self { top_left: get(0), top_right: get(1), bottom_right: get(2), bottom_left: get(3) }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Color(Color),
    Gradient(Gradient),
    Image(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontSpec {
    pub family: Option<String>,
    pub size: Option<Length>,
    pub weight: Option<String>,
    pub style: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_forms() {
        assert_eq!(Color::parse_hex("#abc"), Some(Color::new(170, 187, 204, 255)));
        assert_eq!(Color::parse_hex("#112233"), Some(Color::new(17, 34, 51, 255)));
        assert_eq!(Color::parse_hex("11223380"), Some(Color::new(17, 34, 51, 128)));
        assert_eq!(Color::parse_hex("nope"), None);
    }

    #[test]
    fn edges_follow_css_shorthand() {
        let a = Length::new(1.0, LengthUnit::Px);
        let b = Length::new(2.0, LengthUnit::Px);
        let e = Edges::from_lengths(&[a, b]).unwrap();
        assert_eq!((e.top, e.right, e.bottom, e.left), (a, b, a, b));
        assert!(Edges::from_lengths(&[]).is_none());
    }
}
