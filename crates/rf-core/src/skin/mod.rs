//! Linux skins (issue 93): window-border themes (`xfwm4/`) and, later, GTK widget themes, read from
//! `<settings>\skins\<Name>\` in the same layout as `~/.themes` on Linux. This module holds the OS-free
//! parts: the file formats and the frame geometry. Loading pictures and drawing them is rf-app's job.

pub mod xfwm;
pub mod xpm;

/// Straight (not premultiplied) RGBA.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba([0, 0, 0, 0]);

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Rgba([r, g, b, 255])
    }

    /// GTK2's `shade`: lightness and saturation scaled by `k` in HLS, each clamped to 0..1.
    pub fn shade(self, k: f32) -> Rgba {
        let [r, g, b, a] = self.0.map(|c| c as f32 / 255.);
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let l = (max + min) / 2.;
        let d = max - min;
        let (mut h, mut s) = (0., 0.);
        if d > 0. {
            s = if l <= 0.5 { d / (max + min) } else { d / (2. - max - min) };
            h = if r == max {
                (g - b) / d
            } else if g == max {
                2. + (b - r) / d
            } else {
                4. + (r - g) / d
            } * 60.;
            if h < 0. {
                h += 360.;
            }
        }
        let l = (l * k).clamp(0., 1.);
        let s = (s * k).clamp(0., 1.);
        let m2 = if l <= 0.5 { l * (1. + s) } else { l + s - l * s };
        let m1 = 2. * l - m2;
        let ch = |hue: f32| {
            let hue = hue.rem_euclid(360.);
            let v = if hue < 60. {
                m1 + (m2 - m1) * hue / 60.
            } else if hue < 180. {
                m2
            } else if hue < 240. {
                m1 + (m2 - m1) * (240. - hue) / 60.
            } else {
                m1
            };
            (v * 255.).round().clamp(0., 255.) as u8
        };
        if s == 0. {
            let v = (l * 255.).round() as u8;
            return Rgba([v, v, v, (a * 255.) as u8]);
        }
        Rgba([ch(h + 120.), ch(h), ch(h - 120.), (a * 255.) as u8])
    }

    pub fn mix(self, other: Rgba, t: f32) -> Rgba {
        let mut o = [0u8; 4];
        for i in 0..4 {
            o[i] = (self.0[i] as f32 * (1. - t) + other.0[i] as f32 * t).round() as u8;
        }
        Rgba(o)
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
    }
}

/// A decoded picture: straight RGBA rows, `w * h * 4` bytes.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    /// `top` drawn over `self` at the origin with alpha blending, clipped to `self`'s size
    /// (xfwm4 lays a theme's PNG over the XPM of the same name this way).
    pub fn compose(&mut self, top: &Image) {
        for y in 0..self.h.min(top.h) {
            for x in 0..self.w.min(top.w) {
                let d = ((y * self.w + x) * 4) as usize;
                let s = ((y * top.w + x) * 4) as usize;
                let sa = top.rgba[s + 3] as f32 / 255.;
                let da = self.rgba[d + 3] as f32 / 255.;
                let oa = sa + da * (1. - sa);
                for c in 0..3 {
                    let v = if oa > 0. {
                        (top.rgba[s + c] as f32 * sa + self.rgba[d + c] as f32 * da * (1. - sa)) / oa
                    } else {
                        0.
                    };
                    self.rgba[d + c] = v.round() as u8;
                }
                self.rgba[d + 3] = (oa * 255.).round() as u8;
            }
        }
    }
}

/// An X11 colour: `#` and 1-4 hex digits per channel, or a name from the common subset.
pub fn parse_color(s: &str) -> Option<Rgba> {
    let s = s.trim().trim_matches('"');
    if let Some(hex) = s.strip_prefix('#') {
        let n = hex.len();
        if n == 0 || n % 3 != 0 || n > 12 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let d = n / 3;
        let max = (16u32.pow(d as u32) - 1) as f32;
        let ch = |i: usize| (u32::from_str_radix(&hex[i * d..(i + 1) * d], 16).unwrap() as f32 * 255. / max).round() as u8;
        return Some(Rgba::rgb(ch(0), ch(1), ch(2)));
    }
    let name: String = s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_lowercase();
    for prefix in ["gray", "grey"] {
        if let Some(n) = name.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).filter(|n| *n <= 100) {
            let v = (n as f32 * 2.55).round() as u8;
            return Some(Rgba::rgb(v, v, v));
        }
    }
    // shortcut: only the X11 names seen in window-border themes; add more when a theme needs them.
    let rgb = match name.as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "gray" | "grey" => (190, 190, 190),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "red" => (255, 0, 0),
        "green" => (0, 255, 0),
        "blue" => (0, 0, 255),
        "yellow" => (255, 255, 0),
        "cyan" => (0, 255, 255),
        "magenta" => (255, 0, 255),
        "orange" => (255, 165, 0),
        "navy" => (0, 0, 128),
        _ => return None,
    };
    Some(Rgba::rgb(rgb.0, rgb.1, rgb.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors() {
        assert_eq!(parse_color("#fff"), Some(Rgba::rgb(255, 255, 255)));
        assert_eq!(parse_color("#336699"), Some(Rgba::rgb(0x33, 0x66, 0x99)));
        assert_eq!(parse_color("#ffff00000000"), Some(Rgba::rgb(255, 0, 0)));
        assert_eq!(parse_color("gray50"), Some(Rgba::rgb(128, 128, 128)));
        assert_eq!(parse_color("Light Grey"), Some(Rgba::rgb(211, 211, 211)));
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("nonsense"), None);
    }

    #[test]
    fn shade_keeps_hue() {
        let c = Rgba::rgb(0x40, 0x80, 0xc0);
        assert_eq!(c.shade(1.0), c);
        let light = c.shade(1.3);
        assert!(light.0[2] > c.0[2] && light.0[0] > c.0[0]);
        assert_eq!(Rgba::rgb(100, 100, 100).shade(0.5), Rgba::rgb(50, 50, 50));
    }

    #[test]
    fn compose_blends_over() {
        let mut base = Image { w: 2, h: 1, rgba: vec![0, 0, 255, 255, 0, 0, 255, 255] };
        let top = Image { w: 1, h: 1, rgba: vec![255, 0, 0, 255] };
        base.compose(&top);
        assert_eq!(base.rgba, vec![255, 0, 0, 255, 0, 0, 255, 255]);
    }
}
