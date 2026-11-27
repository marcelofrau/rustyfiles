//! XPM3 pictures (the C-source form every xfwm4 theme uses), with symbolic colours: a colour entry
//! `s active_color_1` takes its value from the theme's colour scheme, so one picture follows the
//! accent and the focus state.

use std::collections::HashMap;

use super::{Image, Rgba, parse_color};

const MAX_SIDE: u32 = 4096;

/// Decodes an XPM. `symbol` resolves a symbolic colour name (`active_color_1` ...) to a colour, `None`
/// when unknown (the entry's other keys decide then).
pub fn decode(text: &str, symbol: &dyn Fn(&str) -> Option<Rgba>) -> Result<Image, String> {
    let strings = c_strings(text);
    let mut it = strings.iter();
    let header = it.next().ok_or("no XPM header")?;
    let nums: Vec<u32> = header.split_whitespace().take(4).map(|n| n.parse().unwrap_or(0)).collect();
    let [w, h, ncolors, cpp] = nums[..] else { return Err("bad XPM header".into()) };
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE || ncolors == 0 || !(1..=31).contains(&cpp) {
        return Err(format!("unsupported XPM header {header:?}"));
    }
    let cpp = cpp as usize;
    let mut table: HashMap<&str, Rgba> = HashMap::new();
    let mut first = None;
    for _ in 0..ncolors {
        let line = it.next().ok_or("XPM colour table too short")?;
        let Some(code) = line.get(..cpp) else { continue };
        let color = entry_color(&line[cpp..], symbol);
        first.get_or_insert(color);
        table.insert(code, color);
    }
    let first = first.unwrap_or(Rgba::TRANSPARENT);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h as usize {
        // a missing or short row stays transparent, as in xfwm4
        let Some(row) = it.next() else { break };
        for x in 0..w as usize {
            let Some(code) = row.get(x * cpp..(x + 1) * cpp) else { break };
            let c = table.get(code).copied().unwrap_or(first);
            let i = (y * w as usize + x) * 4;
            rgba[i..i + 4].copy_from_slice(&c.0);
        }
    }
    Ok(Image { w, h, rgba })
}

/// The colour of one colour-table entry: `s` (when the symbol is known) beats `c`, then `g`, `g4`;
/// `m` alone, `None` or a name that cannot be read is transparent.
fn entry_color(spec: &str, symbol: &dyn Fn(&str) -> Option<Rgba>) -> Rgba {
    let mut keys: Vec<(&str, String)> = Vec::new();
    for word in spec.split_whitespace() {
        if matches!(word, "c" | "m" | "g" | "g4" | "s") {
            keys.push((word, String::new()));
        } else if let Some((_, v)) = keys.last_mut() {
            if !v.is_empty() {
                v.push(' ');
            }
            v.push_str(word);
        } else {
            return Rgba::TRANSPARENT;
        }
    }
    let value = |k: &str| keys.iter().find(|(key, _)| *key == k).map(|(_, v)| v.as_str());
    if let Some(c) = value("s").and_then(|name| symbol(&name.to_ascii_lowercase())) {
        return c;
    }
    let chosen = value("c").or_else(|| value("g")).or_else(|| value("g4"));
    match chosen {
        Some(v) if !v.eq_ignore_ascii_case("none") => parse_color(v).unwrap_or(Rgba::TRANSPARENT),
        _ => Rgba::TRANSPARENT,
    }
}

/// The `"..."` literals of a C source, comments skipped.
fn c_strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
            }
            '"' => {
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => {
                            if let Some(e) = chars.next() {
                                s.push(e);
                            }
                        }
                        c => s.push(c),
                    }
                }
                out.push(s);
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const XPM: &str = r#"/* XPM */
static char * close_xpm[] = {
/* columns rows colors chars-per-pixel */
"3 2 4 1",
"  c None",
". c #FF0000",
"+ s active_color_1 c #00FF00",
"@ m white",
" .+",
"@+"
};"#;

    fn px(img: &Image, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * img.w + x) * 4) as usize;
        img.rgba[i..i + 4].try_into().unwrap()
    }

    #[test]
    fn decodes_with_symbols() {
        let blue = Rgba::rgb(0, 0, 255);
        let img = decode(XPM, &|s| (s == "active_color_1").then_some(blue)).unwrap();
        assert_eq!((img.w, img.h), (3, 2));
        assert_eq!(px(&img, 0, 0), [0, 0, 0, 0]);
        assert_eq!(px(&img, 1, 0), [255, 0, 0, 255]);
        assert_eq!(px(&img, 2, 0), [0, 0, 255, 255]);
        // `m` alone is transparent; the short second row leaves its last pixel transparent
        assert_eq!(px(&img, 0, 1), [0, 0, 0, 0]);
        assert_eq!(px(&img, 1, 1), [0, 0, 255, 255]);
        assert_eq!(px(&img, 2, 1), [0, 0, 0, 0]);
    }

    #[test]
    fn unknown_symbol_falls_back_to_c() {
        let img = decode(XPM, &|_| None).unwrap();
        assert_eq!(px(&img, 2, 0), [0, 255, 0, 255]);
    }

    #[test]
    fn rejects_bad_headers() {
        assert!(decode(r#""0 2 1 1", "  c None""#, &|_| None).is_err());
        assert!(decode("", &|_| None).is_err());
    }
}
