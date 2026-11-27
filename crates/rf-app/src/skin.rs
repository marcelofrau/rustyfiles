//! Linux skins in rustyfiles' own windows (issue 93). A skin is a folder in `<settings folder>\skins\`, laid
//! out like `~/.themes` on Linux, so a theme from gnome-look works unpacked as it is. This first part reads the
//! window borders in its `xfwm4\` folder (Xfce's format): Settings > Appearance > Window borders picks one.
//! Upstream's own theme folders (`themes.rs`, `<settings folder>\themes\`) are a separate thing and stay as they are.
//!
//! The pictures are read when the setting changes; their colour symbols (`active_color_1` …) are filled from the
//! window's palette when a frame is first drawn with that palette, and kept.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use gpui_kit::*;
use rf_core::skin::xfwm::{self, Button, ButtonState, Part, Piece, State};
use rf_core::skin::{Image, Rgba as Rgb8, xpm};

use crate::theme::Palette;

/// A theme's picture is a few KB; this only stops a bad file from eating memory.
const MAX_FILE_BYTES: u64 = 1024 * 1024;
/// Tiled pieces are drawn from strips at least this long, so a 1 px picture needs a handful of elements, not a thousand.
const STRIP_MIN: i32 = 256;

/// `<settings folder>\skins`.
pub fn dir() -> Option<PathBuf> {
    crate::settings::app_dir().map(|d| d.join("skins"))
}

/// The skins that have window borders (`xfwm4\themerc`), by name.
pub fn list_borders() -> Vec<String> {
    let Some(d) = dir() else { return Vec::new() };
    let mut v: Vec<String> = std::fs::read_dir(d)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("xfwm4").join("themerc").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort_by_key(|s| s.to_lowercase());
    v
}

struct Source {
    xpm: Option<String>,
    png: Option<Image>,
}

#[derive(Clone)]
struct Pic {
    /// The picture as it is, and (for tiled pieces) a strip of it repeated along the tiling direction.
    img: Arc<RenderImage>,
    strip: Arc<RenderImage>,
    w: i32,
    h: i32,
    strip_len: i32,
}

struct Borders {
    name: String,
    cfg: xfwm::Config,
    sources: HashMap<String, Source>,
    /// (file stem, tiling, colours) -> picture; `None`: the theme has none.
    pics: Mutex<HashMap<(String, Tile, u64), Option<Pic>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    None,
    Across,
    Down,
}

static CURRENT: RwLock<Option<Arc<Borders>>> = RwLock::new(None);

/// The window borders in use (`""`: Windows' own). Called when the settings change.
pub fn set_borders(name: &str) {
    let now = current().map(|b| b.name.clone()).unwrap_or_default();
    if now == name {
        return;
    }
    let b = (!name.is_empty()).then(|| dir().and_then(|d| load(&d.join(name).join("xfwm4"), name))).flatten().map(Arc::new);
    if let Ok(mut c) = CURRENT.write() {
        *c = b;
    }
}

fn current() -> Option<Arc<Borders>> {
    CURRENT.read().ok()?.clone()
}

fn load(folder: &Path, name: &str) -> Option<Borders> {
    let cfg = match std::fs::read_to_string(folder.join("themerc")) {
        Ok(t) => xfwm::parse_themerc(&t),
        Err(e) => {
            log::warn!("skin {name}: themerc: {e}");
            return None;
        }
    };
    let mut sources: HashMap<String, Source> = HashMap::new();
    for e in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let p = e.path();
        let ext = p.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
        let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().to_lowercase()) else { continue };
        if !matches!(ext.as_str(), "xpm" | "png") || e.metadata().map_or(true, |m| !m.is_file() || m.len() > MAX_FILE_BYTES) {
            continue;
        }
        let p = follow_link(folder, p);
        let src = sources.entry(stem).or_insert(Source { xpm: None, png: None });
        match ext.as_str() {
            "xpm" => src.xpm = std::fs::read_to_string(&p).ok(),
            _ => {
                src.png = std::fs::read(&p).ok().and_then(|b| image::load_from_memory(&b).ok()).map(|i| {
                    let i = i.to_rgba8();
                    Image { w: i.width(), h: i.height(), rgba: i.into_raw() }
                })
            }
        }
    }
    Some(Borders { name: name.to_string(), cfg, sources, pics: Mutex::new(HashMap::new()) })
}

/// Linux themes reuse pictures through symlinks (`title-3-active.xpm` -> `title-1-active.xpm`); git and many unpackers on Windows
/// write such a link as a small file holding the target's name. That name is followed, a few hops, within the same folder only.
fn follow_link(folder: &Path, mut p: PathBuf) -> PathBuf {
    for _ in 0..4 {
        let Ok(m) = std::fs::metadata(&p) else { break };
        if m.len() > 255 {
            break;
        }
        let Ok(text) = std::fs::read_to_string(&p) else { break };
        let name = text.trim().trim_start_matches("./");
        let plain = !name.is_empty() && !name.contains(['/', '\\', ':', '\n']) && name != "." && name != "..";
        let target = folder.join(name);
        if !plain || !target.is_file() {
            break;
        }
        p = target;
    }
    p
}

/// The GTK colours xfwm4 fills its symbols from, taken from the palette: `[normal, selected, insensitive]`, each `(fg, bg)`.
fn gtk_colors(p: &Palette) -> [(Rgb8, Rgb8); 3] {
    let bg = opaque(p.window_bg, Rgb8::rgb(0, 0, 0));
    [
        (opaque(p.text, bg), bg),
        (opaque(p.accent_text, opaque(p.accent, bg)), opaque(p.accent, bg)),
        (opaque(p.text_tertiary, bg), bg),
    ]
}

/// `c` laid over `under`, as plain 8-bit colour.
fn opaque(c: Hsla, under: Rgb8) -> Rgb8 {
    let c = c.to_rgb();
    let mix = |v: f32, u: u8| ((v * c.a + u as f32 / 255. * (1. - c.a)) * 255.).round().clamp(0., 255.) as u8;
    Rgb8::rgb(mix(c.r, under.0[0]), mix(c.g, under.0[1]), mix(c.b, under.0[2]))
}

impl Borders {
    fn pic(&self, stem: &str, tile: Tile, colors: &[(Rgb8, Rgb8); 3]) -> Option<Pic> {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        colors.hash(&mut h);
        let key = (stem.to_string(), tile, h.finish());
        if let Some(hit) = self.pics.lock().ok()?.get(&key) {
            return hit.clone();
        }
        let pic = self.make(stem, tile, colors);
        if let Ok(mut m) = self.pics.lock() {
            m.insert(key, pic.clone());
        }
        pic
    }

    fn make(&self, stem: &str, tile: Tile, colors: &[(Rgb8, Rgb8); 3]) -> Option<Pic> {
        let src = self.sources.get(stem)?;
        let gtk = |part: Part, state: State| {
            let (fg, bg) = colors[match state {
                State::Normal => 0,
                State::Selected => 1,
                State::Insensitive => 2,
            }];
            xfwm::derived(part, fg, bg)
        };
        let symbol = |name: &str| self.cfg.symbol(name, &gtk);
        // xfwm4: the XPM first, the PNG of the same name laid over it
        let xpm = src.xpm.as_deref().and_then(|t| xpm::decode(t, &symbol).map_err(|e| log::warn!("skin {}: {stem}.xpm: {e}", self.name)).ok());
        let img = match (xpm, &src.png) {
            (Some(mut x), Some(p)) => {
                x.compose(p);
                x
            }
            (Some(x), None) => x,
            (None, Some(p)) => p.clone(),
            (None, None) => return None,
        };
        if img.w == 0 || img.h == 0 {
            return None;
        }
        let (w, h) = (img.w as i32, img.h as i32);
        let one = render_image(img.w, img.h, img.rgba.clone())?;
        let (strip, strip_len) = match tile {
            Tile::None => (one.clone(), w),
            Tile::Across => {
                let n = (STRIP_MIN + w - 1) / w;
                let mut rgba = Vec::with_capacity((w * n * h * 4) as usize);
                for row in img.rgba.chunks_exact((w * 4) as usize) {
                    for _ in 0..n {
                        rgba.extend_from_slice(row);
                    }
                }
                (render_image((w * n) as u32, img.h, rgba)?, w * n)
            }
            Tile::Down => {
                let n = (STRIP_MIN + h - 1) / h;
                (render_image(img.w, (h * n) as u32, img.rgba.repeat(n as usize))?, h * n)
            }
        };
        Some(Pic { img: one, strip, w, h, strip_len })
    }
}

/// GPUI takes BGRA.
fn render_image(w: u32, h: u32, mut rgba: Vec<u8>) -> Option<Arc<RenderImage>> {
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buf = image::RgbaImage::from_raw(w, h, rgba)?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(buf)])))
}

/// A window's skinned frame for one render.
pub struct Frame {
    layout: xfwm::Layout,
    pieces: Vec<(xfwm::Rect, Pic, Tile)>,
    buttons: Vec<(Button, xfwm::Rect, [Option<Pic>; 3])>,
    title: SharedString,
    text: Hsla,
    shadow: Option<Hsla>,
}

/// Title text as xfwm4 draws it by default ("Sans Bold 9").
const TITLE_PX: f32 = 12.;

/// The frame of `window` when window borders are skinned, with `title` in its title bar. The title row has the theme's own
/// height (its pictures are drawn at their size, never stretched); what the window shows in its own top bar (tabs) goes below it.
pub fn frame(window: &Window, p: &Palette, title: &str) -> Option<Frame> {
    let b = current();
    sync_corners(window, b.is_some());
    let b = b?;
    let colors = gtk_colors(p);
    let active = window.is_window_active();
    let maximized = window.is_maximized();
    let size = window.viewport_size();
    let (width, height) = (f32::from(size.width) as i32, f32::from(size.height) as i32);
    let tile_of = |piece: Piece| match piece {
        Piece::Title(_) | Piece::Bottom => Tile::Across,
        Piece::Left | Piece::Right => Tile::Down,
        _ => Tile::None,
    };
    // (an inactive piece the theme lacks is drawn active)
    let piece_pic = |piece: Piece| b.pic(&piece.file(active), tile_of(piece), &colors).or_else(|| b.pic(&piece.file(true), tile_of(piece), &colors));
    let pics: HashMap<Piece, Pic> = Piece::ALL.iter().filter_map(|&pc| piece_pic(pc).map(|pic| (pc, pic))).collect();
    if pics.get(&Piece::Title(3)).is_none_or(|p| p.h == 0) {
        return None;
    }
    let size_of = |pc: Piece| pics.get(&pc).map_or((0, 0), |p| (p.w, p.h));
    let state = if active { ButtonState::Active } else { ButtonState::Inactive };
    // (Windows has no menu, shade or stick button to stand for)
    let mut bpics: HashMap<Button, [Option<Pic>; 3]> = HashMap::new();
    for btn in [Button::Hide, Button::Maximize, Button::Close] {
        let get = |st: ButtonState| xfwm::button_files(btn, btn == Button::Maximize && maximized, st).iter().find_map(|f| b.pic(f, Tile::None, &colors));
        bpics.insert(btn, [get(state), get(ButtonState::Prelight), get(ButtonState::Pressed)]);
    }
    let button_size = |btn: Button| bpics.get(&btn)?[0].as_ref().map(|p| (p.w, p.h));
    let title: SharedString = title.to_string().into();
    let mut style = window.text_style();
    style.font_weight = FontWeight::BOLD;
    let text_w = f32::from(window.text_system().shape_line(title.clone(), px(TITLE_PX), &[style.to_run(title.len())], None).width).ceil() as i32;
    let layout = xfwm::layout(&b.cfg, &size_of, &button_size, (width, height), Some(text_w), active, maximized);
    let pieces = layout.pieces.iter().filter_map(|(pc, r)| Some((*r, pics.get(pc)?.clone(), tile_of(*pc)))).collect();
    let buttons = layout.buttons.iter().filter_map(|(btn, r)| Some((*btn, *r, bpics.get(btn)?.clone()))).collect();
    let gtk = |part: Part, state: State| {
        let (fg, bg) = colors[match state {
            State::Normal => 0,
            State::Selected => 1,
            State::Insensitive => 2,
        }];
        xfwm::derived(part, fg, bg)
    };
    let (text_key, shadow_key) = if active { ("active_text_color", "active_text_shadow_color") } else { ("inactive_text_color", "inactive_text_shadow_color") };
    let hsla = |c: Rgb8| Hsla::from(gpui_kit::Rgba { r: c.0[0] as f32 / 255., g: c.0[1] as f32 / 255., b: c.0[2] as f32 / 255., a: 1. });
    let text = b.cfg.symbol(text_key, &gtk).map_or(p.text, hsla);
    let shadow = b.cfg.title_shadow[usize::from(!active)].then(|| b.cfg.symbol(shadow_key, &gtk).map(hsla)).flatten();
    Some(Frame { layout, pieces, buttons, title, text, shadow })
}

/// A skinned window has square corners (the theme's pictures are square, or round their own); one that is no longer skinned gets
/// the system's back. Only windows this squared are touched.
// shortcut: keyed by HWND, so a new window that reuses a closed one's handle may keep round corners; key by window id if seen.
fn sync_corners(window: &Window, square: bool) {
    static SQUARED: Mutex<Option<HashMap<isize, bool>>> = Mutex::new(None);
    let hwnd = crate::look::window::hwnd_of(window);
    let Ok(mut m) = SQUARED.lock() else { return };
    let m = m.get_or_insert_with(HashMap::new);
    let was = m.get(&hwnd).copied();
    if hwnd != 0 && was != Some(square) && (square || was.is_some()) {
        rf_win::window::set_square_corners(hwnd, square);
        m.insert(hwnd, square);
    }
}

impl Frame {
    /// `root` (the window's outermost element) padded by the borders, with the border pictures as its first children: call it
    /// before adding the window's own content, which then lies over them.
    pub fn frame_root<E: Styled + ParentElement>(&self, root: E) -> E {
        let (l, r, b) = self.insets();
        let mut root = root.pl(l).pr(r).pb(b);
        root.extend(self.pieces());
        root
    }

    /// The room the side and bottom borders take: the window's content is padded by it.
    pub fn insets(&self) -> (Pixels, Pixels, Pixels) {
        (px(self.layout.left as f32), px(self.layout.right as f32), px(self.layout.bottom as f32))
    }

    /// The border pictures, as absolute elements over the window (put them under the content, over the backdrop).
    pub fn pieces(&self) -> Vec<AnyElement> {
        self.pieces.iter().map(|(r, pic, tile)| tiled(*r, pic, *tile)).collect()
    }

    /// The title row: the title text (its free part also moves the window) and the buttons. It spans the window's whole
    /// width, so in a parent padded by [`Self::insets`] it reaches out over the side borders.
    pub fn title_bar(&self) -> AnyElement {
        let (left, right, _) = self.insets();
        let (t, top) = (self.layout.title, self.layout.top as f32);
        let text_at = |d: f32, color: Hsla| {
            div()
                .absolute()
                .left(px((self.layout.text_x - t.x) as f32 + d))
                .top(px(self.layout.text_dy as f32 + d))
                .h(px(top))
                .flex()
                .items_center()
                .whitespace_nowrap()
                .text_size(px(TITLE_PX))
                .font_weight(FontWeight::BOLD)
                .text_color(color)
                .child(self.title.clone())
        };
        let mut bar = div().relative().flex_shrink_0().h(px(top)).ml(-left).mr(-right).child(
            div()
                .id("skin-title")
                .absolute()
                .left(px(t.x as f32))
                .top_0()
                .w(px(t.w as f32))
                .h_full()
                .overflow_hidden()
                .window_control_area(WindowControlArea::Drag)
                .children(self.shadow.map(|c| text_at(1., c)))
                .child(text_at(0., self.text)),
        );
        for (btn, r, [normal, hover, pressed]) in &self.buttons {
            let (area, group) = match btn {
                Button::Hide => (WindowControlArea::Min, "skin-hide"),
                Button::Maximize => (WindowControlArea::Max, "skin-max"),
                _ => (WindowControlArea::Close, "skin-close"),
            };
            let layer = |pic: &Pic| img(pic.img.clone()).absolute().top_0().left_0().w(px(pic.w as f32)).h(px(pic.h as f32));
            bar = bar.child(
                div()
                    .id(group)
                    .group(group)
                    .absolute()
                    .left(px(r.x as f32))
                    .top(px(r.y as f32))
                    .w(px(r.w as f32))
                    .h(px(r.h as f32))
                    .window_control_area(area)
                    .children(normal.as_ref().map(layer))
                    .children(hover.as_ref().map(|p| layer(p).opacity(0.).group_hover(group, |s| s.opacity(1.))))
                    .children(pressed.as_ref().map(|p| layer(p).opacity(0.).group_active(group, |s| s.opacity(1.)))),
            );
        }
        bar.into_any_element()
    }
}

/// A picture tiled over `r`.
fn tiled(r: xfwm::Rect, pic: &Pic, tile: Tile) -> AnyElement {
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
    let holder = div().absolute().left(px(x)).top(px(y)).w(px(w)).h(px(h)).overflow_hidden();
    let len = pic.strip_len.max(1) as f32;
    match tile {
        Tile::None => holder.child(img(pic.img.clone()).size_full().object_fit(ObjectFit::Fill)).into_any_element(),
        Tile::Across => holder
            .flex()
            .flex_row()
            .children((0..(w / len).ceil() as usize).map(|_| img(pic.strip.clone()).flex_shrink_0().w(px(len)).h(px(h)).object_fit(ObjectFit::Fill)))
            .into_any_element(),
        Tile::Down => holder
            .flex()
            .flex_col()
            .children((0..(h / len).ceil() as usize).map(|_| img(pic.strip.clone()).flex_shrink_0().w(px(w)).h(px(len)).object_fit(ObjectFit::Fill)))
            .into_any_element(),
    }
}
