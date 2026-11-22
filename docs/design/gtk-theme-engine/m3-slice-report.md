# M3 slice report — 2026-11-22

## Scope

M3 connects the pure engine to four real GPUI surfaces:

- main titlebar (`workspace.rs`)
- tab strip (`workspace.rs`)
- one sidebar tree row (`sidebar_tree.rs`)
- one details/file row (`folder_view.rs`)

No menus, dialogs, grids, cards or custom painting are in this milestone.
The built-in palette remains the default; the engine stays `off` unless the
user selects a version in **Settings → Appearance → GTK theme engine**.

## Bridge

`crates/rf-app/src/theme_engine.rs` owns a process-wide handle. It loads one
`EngineTheme` from the selected theme folder and rebuilds only when the folder,
preference or effective light/dark appearance changes. State changes such as
selection, focus and window activation only rerun the M2 cascade. `off` and
`legacy` build no handle, so no CSS is read and every `themed()` call returns
the original element.

`crates/rf-app/src/theme_nodes.rs` maps GPUI state to GTK names, classes and
`StateFlags`:

| Surface | Node | States |
|---|---|---|
| Titlebar | `headerbar.rf-titlebar` | active / backdrop |
| Tab strip | `header.rf-tabstrip` inside `notebook` | active |
| Sidebar row | `row.sidebar-row` inside `placessidebar` | selected, focus, disabled, active / backdrop |
| File row | `row.file-row` inside `treeview.view` | selected, focus, disabled, active / backdrop |

The wrapper applies only properties the theme declared: background, text
color, radius, padding, font size and opacity. Anything else keeps the current
GPUI style and interaction code.

## Fixture

`fixtures/slice/gtk-3.0/gtk.css` and `fixtures/slice/theme.toml` are a small
hand-written check theme. Expected colors:

- titlebar: `#00695c` in light mode (`theme.toml` override beats CSS purple)
- tab strip: `#1565c0`
- selected sidebar row: `#ff8f00`
- selected file row: `#ef6c00`

## Manual check

```powershell
$src = 'docs\design\gtk-theme-engine\fixtures\slice'
$dst = Join-Path $env:APPDATA 'rustyfiles\themes\slice'
Copy-Item -Recurse -Force $src $dst
```

Then, in the app:

1. Set **Settings → Appearance → Tema** to Light.
2. Set **Theme folder** to `slice`.
3. Set **GTK theme engine** to `GTK 3` (or `Auto`).
4. Check the four surfaces above. Select a sidebar row and a file row.
5. Set **GTK theme engine** back to `Off`: the original look must return.

The engine is experimental and applies only the mapped slice. Missing theme
properties fall back to the app's existing styles.

## Parser fix found during M3

The fixture exposed an M2 parser bug: a descendant combinator consumed the
first token of the following compound. `window notebook header` parsed as
`window header`, so intermediate ancestors never matched. Fixed in
`css/selector.rs`; see the addendum in `m2-cascade-report.md`.

## Validation

- `cargo test -p rf-theme-engine` — 35 tests pass
- `cargo test -p rf-app --lib theme_engine` — 6 tests pass
- `cargo check -p rf-app --lib --profile fast` — clean apart from known warnings
- `python scripts/i18n_source.py` — 4 new source texts

## Gate

Human visual check of the four surfaces, off and on, before M4.
