# Linux skins: window borders (xfwm4), then GTK

Issue 93 asked for full visual styles. The theme folders (`themes.rs`, [docs/themes.md](../themes.md)) recolour rustyfiles and
change its icons. Skins go further: they read the **real Linux theme formats**, so a theme downloaded from gnome-look or
similar sites works unpacked as it is.

## Where skins live

`<settings folder>\skins\<Name>\`, in the same layout as `~/.themes` on Linux:

```
skins\
  Greybird\
    xfwm4\        window borders (Xfce)      <- read now
    gtk-2.0\      GTK2 widgets               <- next phase
    metacity-1\   window borders (MATE, Cinnamon)  <- later
```

Skins are kept apart from `themes\` so the two features never collide: a theme folder and a skin can be used together.
Settings > Appearance > **Window borders** lists the skins that have `xfwm4\themerc`.

## Window borders (xfwm4)

| Part | Where |
|------|-------|
| `themerc` parser, XPM decoder (with colour symbols), piece names, frame geometry | `crates/rf-core/src/skin/` (OS-free, tested) |
| Loading the pictures, colour symbols from the palette, drawing, caption buttons | `crates/rf-app/src/skin.rs` |
| Square window corners while skinned | `rf_win::window::set_square_corners` |

How the frame is drawn follows xfwm4:

- The title height is `title-3`'s height. The side widths come from `left`/`right`, the bottom height from `bottom`. When
  the window is maximized only the title row is left, and `maximized_offset` replaces `button_offset`.
- `title-2` and `title-4` keep their width. `title-1`, `title-3` and `title-5` fill the rest. `full_width_title`,
  `title_alignment` and `title_horizontal_offset` place the text. Pieces are tiled, never scaled.
- Buttons come from `button_layout` (default `O|HMC`). Only `H` (minimize), `M` (maximize) and `C` (close) are drawn,
  because Windows has nothing for the menu, shade and stick buttons. Each button is tagged as a `WindowControlArea`, so
  Windows does the clicks and snap layouts. `-prelight` shows on hover and `-pressed` while pressed. A maximized window
  uses the `-toggled-` pictures. A missing state falls back to `-active`.
- Pictures: the `.xpm` is loaded first, and a `.png` of the same name is laid over it. XPM colour symbols
  (`active_color_1`, `active_text_color`, … 24 of them) start from the GTK colours they stand for. Until the GTK phase
  those come from the palette (`bg[SELECTED]` = accent, `bg[NORMAL]` = window colour, `fg[NORMAL]` = text …). A
  `themerc` entry overrides a symbol, and may name another symbol.
- Linux themes reuse pictures through symlinks. On Windows, git and many unpackers write a symlink as a small file that
  holds the target's name; such a name is followed within the same folder.
- The title row keeps the theme's height. The main window's tab strip and quick look's bar become a row under it, as a
  GTK application looks under Xfce. Properties and HyperDownload need no row of their own: the skin's title says what
  their bar said.
- While skinned, the material is Solid (`Settings::backdrop_in_use`): the pictures are opaque.

## Limits

- Not read yet: `-stretch` and `top-N` pieces, `-mask` pictures, `title_font`, SVG pieces, `show_app_icon`.
- The pictures are drawn at their pixel size in logical pixels: on a scaled display they are enlarged with the window.
  Themes that ship `xfwm4-hdpi` are not picked automatically.
- The quick look window in its overlay mode (a picture or video filling the window) keeps its own bar.
- Rounded top corners show the colour under the window's frame. Windows themselves get square corners.

## Next: GTK2

`gtk-2.0/gtkrc` (styles, colour scheme, `class` / `widget_class` bindings, the pixmap engine) for the widgets. Its colours
also feed the xfwm4 symbols, as on Linux. Themes whose engine is a C library (Murrine, Clearlooks…) are drawn from their
gtkrc colours.
