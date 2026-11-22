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

#[derive(Debug, Clone, PartialEq)]
pub struct Edges {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

impl Edges {
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
