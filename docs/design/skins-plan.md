# Linux skins: plan and handoff (issue 93)

Working notes for the `feat/93-skins` branch on the fork (marcelofrau/rustyfiles). How the finished parts work is in
[skins.md](skins.md); this file is the plan, where it stands, and what to do next.

## Goal and decisions (user, 2026-11-27)

Read real Linux theme formats, so a theme from gnome-look works unpacked as it is:
window borders from `xfwm4/` first, then `metacity-1/` (MATE Marco, Cinnamon Muffin); widgets from `gtk-2.0/gtkrc`, then
GTK3/4 CSS.

- Skin **rustyfiles' own windows only**: no injection into other apps (the WindowBlinds route was ruled out).
- Skins live in **`<settings>\skins\<Name>\`**, laid out like `~/.themes`, **apart from upstream's `themes\` folders**
  so the two never collide.
- GTK2 = full gtkrc + the pixmap engine. Themes whose engine is a C library (Murrine, Clearlooks, Aurora…) are drawn from
  their gtkrc colours with our own shapes. Engines get rewritten later, one at a time and clean-room: they are GPL,
  rustyfiles is MIT, so their code must not be ported.
- **Fork only**: branch `feat/93-skins`, every commit says `Refs #93` (never `Closes`), no upstream PR for now.
- Work style: the main session plans and reviews, and lighter subagents (scout / worker) do the hands-on work.

## Status

| Commit | What |
|--------|------|
| `a1fb141a` | Phase 1: xfwm4 borders on the main, Open/Save, Properties, HyperDownload and quick look windows |
| `6cd45c58` | The window takes the shape of the theme's corners (`SetWindowRgn`), DWM border hidden, Mica & co. kept, setting moved next to Icon theme |

Code: `crates/rf-core/src/skin/` (xpm, xfwm, colours, tested), `crates/rf-app/src/skin.rs` (loading, drawing, shape),
`rf_win::window::{set_square_corners, set_shape, set_border_hidden}`.

Example skins installed for testing in `%APPDATA%\rustyfiles\skins\`: Greybird, Greybird-Dark, Numix, Xfce-Default,
Xfce-daloa, Xfce-kokodi, Xfce-moheli, Xfce-default-hdpi, Xfce-default-xhdpi.

## Open at handoff

1. **The user saw "something still wrong" after `6cd45c58` and did not say what yet.** Ask first. Suspects:
   - the shape is offset: `set_shape` assumes GPUI's (0,0) is the client top-left and offsets by client origin minus
     window rect;
   - the region is not re-applied after restore from maximized, or after a DPI change;
   - no drop shadow on a shaped window (expected);
   - colours: the app body stays light under a dark border until GTK2.
2. Tela icon packs: a worker was unpacking the user's Tela archives from `Downloads` into
   `%APPDATA%\rustyfiles\icon-themes\` (symlinks turned into real copies, official names such as `Tela-blue`,
   `-dark`, `-light`, existing `Tela-manjaro*` left alone). Check that it finished: each pack has `index.theme` at its root.
   The icon setting itself was only unset (`icon_theme = "windows"`, `icon_theme_pack = ""`); nothing was broken in code.
3. Fork `main` also got `a1fb141a` by mistake. The branch tracked `upstream/main`, so a plain push went to `main`. The user
   decides whether to reset it: `git push --force-with-lease=main:a1fb141a origin 63be76eb:refs/heads/main`.
4. Not checked by hand yet: hover/pressed buttons, drag, double-click maximize, Snap, edge resize, the Properties and
   quick look windows.

## Next: phase 2, GTK2 (after the user's ok on phase 1)

1. `rf-core/src/skin/gtkrc.rs` should handle:
   - `include` and `style "a" = "b" { … }` inheritance;
   - `gtk-color-scheme` and `@name` references;
   - colour expressions (`mix`, `shade`, `lighter`, `darker`, `#rgb`, `{r,g,b}`);
   - `bg/fg/base/text[NORMAL|PRELIGHT|ACTIVE|SELECTED|INSENSITIVE]`, `bg_pixmap`, `xthickness/ythickness`;
   - `engine "pixmap" { image { function detail state shadow orientation gap_side file border stretch overlay_* gap_* } }`;
   - other engine blocks, parsed and kept by name;
   - `class` / `widget_class` / `widget` bindings with GTK glob matching and priority.

   It exposes `resolve(path, function, detail, state, shadow, orientation) -> (colours, Option<image match>)`, with tests
   on synthetic fixtures (no GPL theme files in the repo).
2. A table of virtual widgets maps each rustyfiles surface to a GTK2 path and draw call:
   - toolbar button: `GtkToolbar.GtkButton` BOX "button";
   - tab: `GtkNotebook` EXTENSION "tab";
   - menu and item: `GtkMenu` BOX "menu" / "menuitem";
   - list rows: `GtkTreeView` FLAT_BOX "cell_even/odd";
   - entry: SHADOW "entry";
   - check / radio: CHECK / OPTION;
   - progress: BOX "bar" / "trough";
   - scrollbar: SLIDER / BOX "trough";
   - also the statusbar, the sidebar and the window background.
3. Palette from the gtkrc, applied just before `themes::apply` in `Palette::for_window` (`theme.rs`), so a `theme.toml`
   still wins:

   | gtkrc colour | Palette field |
   |---|---|
   | `bg[NORMAL]` | `window_bg` |
   | `base[NORMAL]` | `content_bg` |
   | `text[NORMAL]` | `text` |
   | `base[SELECTED]` | `selection` / `accent` |
   | `text[SELECTED]` | `accent_text` |
   | `fg[INSENSITIVE]` | `text_tertiary` |
   | `bg[PRELIGHT]` | `hover` |
   | `bg[ACTIVE]` | `pressed` |

   The same colours replace `skin::gtk_colors` for the xfwm4 symbols, as xfwm4 does on Linux.
4. Setting: **GTK theme** = "None" | skins that have `gtk-2.0/gtkrc` (new `settings.rs` field `skin_gtk`,
   `#[serde(default)]`).
5. `rf-app` gets a `nine_slice` element (pre-sliced images, cached) and skinned versions of the shared helpers:
   - `settings_view.rs`: `button`, `switch`, `dropdown`, `slider_for`;
   - `liquid.rs`;
   - the tabs in `Workspace::render_titlebar`;
   - the `fluent_menu` items;
   - the `folder_view` row and selection backgrounds.
6. Phase 2b, last and view by view because it conflicts most with upstream: move the ad-hoc `div().bg(p.hover)` buttons
   to a `skin::button`, and give the gpui-component Scrollbar and Input a skin.

Later phases: Metacity / Marco / Cinnamon XML (geometry, expressions, draw ops rasterised with tiny-skia into the same
frame model), then GTK3/4 CSS (subset, `gtk.gresource` via a GVDB reader). Each gets its own design pass.

## Gotchas learned

- `git push` on this branch: it now tracks `origin/feat/93-skins`. Never let it track `upstream/main` again.
- `tools/shot.ps1` captures the *first* rustyfiles window, which may be the user's own instance. Capture by PID instead
  (copy in `%LOCALAPPDATA%\Temp\rf-skin-test\tools\shotpid.ps1` and `run.ps1`, with a test config in `...\cfg`).
- `directlook.exe` hands the file to a running instance: it cannot be used for a test while the user's copy runs.
- A running `target\debug\rustyfiles.exe` locks the link: use `cargo check` / `cargo test -p rf-core` until it is closed.
- `cargo test --workspace` has one failure unrelated to this work on this machine:
  `quicklook::the_pipe_is_named_after_the_user` expects an `S-1-5-` SID, and the Entra ID account is `S-1-12-1-`.
- Greybird's pictures are symlinks that git writes as text files (`./bottom-active.xpm`); `skin.rs::follow_link`
  handles them. Icon packs have the same problem, but `icon_theme.rs` does not follow such stubs, so unpack packs with a
  script that copies link targets.
- The `!` prompt runs bash, not PowerShell:
  `RUSTYFILES_MULTI_INSTANCE=1 cmd //c start "" "D:\workspace\_non_work_\rustyfiles\target\debug\rustyfiles.exe"`.
