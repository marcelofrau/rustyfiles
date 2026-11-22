# M1 parse report — 2026-11-22

## Corpus

Local GTK theme archives live in `H:\Downloads\themes\gtk` as `tar.gz` files.
For this report, `Agata-v1.0.6.tar.gz`, `Grass.tar.gz` and
`human-theme-3.2.0.tar.gz` were extracted to
`C:\Users\marcelo\AppData\Local\Temp\opencode\gtk-corpus`.
The integration test `crates/rf-theme-engine/tests/parse_real_themes.rs` skips
when no extracted corpus exists, and accepts `GTK_THEME_CORPUS` as an override.

## Reproduce

```powershell
$env:GTK_THEME_CORPUS = 'C:\Users\marcelo\AppData\Local\Temp\opencode\gtk-corpus'
cargo test -p rf-theme-engine --test parse_real_themes -- --nocapture
```

## Result

`parsed 17 files: 1892 rules, 16 imports/keyframes, 680 issues, 40 unique unknown properties`

No panic and no fatal parser error. Unknown declarations are reported with
`file:line` and reported once per property per file; later rules still parse.

Unique unknown properties:

`-NaTrayApplet-icon-padding`, `-NaTrayApplet-icon-size`,
`-UnityDecoration-active-shadow-color`, `-UnityDecoration-active-shadow-radius`,
`-UnityDecoration-extents`, `-UnityDecoration-glow-color`,
`-UnityDecoration-glow-size`, `-UnityDecoration-inactive-shadow-color`,
`-UnityDecoration-inactive-shadow-radius`, `-UnityDecoration-input-extents`,
`-UnityDecoration-shadow-offset-x`, `-UnityDecoration-shadow-offset-y`,
`-UnityDecoration-title-alignment`, `-UnityDecoration-title-fade`,
`-UnityDecoration-title-indent`, `-XfwmTabwinWidget-icon-size`,
`-XfwmTabwinWidget-preview-size`, `-active-window-background`,
`-active-window-border`, `-cinnamon-caption-spacing`,
`-cinnamon-close-overlap`, `-gradient-direction`,
`-inactive-window-background`, `-inactive-window-border`,
`-treeview-vertical-separator`, `appearance`, `border-inline-start`,
`border-radius-bottomleft`, `border-radius-bottomright`, `bottom`,
`caret-size`, `left`, `link-color`, `margin-from-right-edge-of-screen`,
`margin-inline`, `padding-block`, `padding-inline`, `right`,
`text-decoration`, `z-index`.

Most names are shell-specific GTK extensions. They stay as `Value::Raw` in the
AST, so M2 can still inherit and hand them to a future renderer or ignore them
explicitly.

## M1 validation

- `cargo test -p rf-theme-engine`: 17 library tests + 1 integration test, all pass.
- `cargo clippy -p rf-theme-engine --all-targets -- -D warnings`: clean, apart
  from the pre-existing workspace `natord` manifest warning.
- No app behavior changed: the setting remains `off` and no GPUI code is linked.
