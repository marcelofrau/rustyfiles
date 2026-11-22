# M2 cascade report — 2026-11-22

## Scope

M2 turns the M1 AST into one `ComputedStyle` per node. It stays inside
`rf-theme-engine`: no GPUI, no file I/O, no app setting changes.

## Resolved semantics

- **Specificity.** GTK widget names, classes, IDs and state pseudo-classes use
  CSS-like `(ids, classes, elements)` ordering. `:not()` contributes the
  specificity of its most specific argument; every argument is matched.
- **Cascade order.** `!important` wins, then specificity, then source order.
  Source order includes the rule index and declaration index, so equal rules
  are deterministic.
- **Inheritance.** `color`, font properties, `-gtk-icon-style`,
  `-gtk-icon-shadow`, caret/selection colors and text decoration inherit.
  Surfaces, borders, padding, margin, shadows and opacity do not.
  `inherit`, `initial` and `unset` are applied per property.
- **States.** The node adapter sets `StateFlags`: `:hover`, `:active`,
  `:selected`, `:checked`, `:disabled`, `:focus`, `:focus-visible`,
  `:focus-within`, `:backdrop`, plus GTK aliases `:prelight` and
  `:insensitive`. `:focus-within` propagation and `:backdrop` scoping belong to
  the adapter in M3.
- **Combinators.** Descendant, child, adjacent sibling and general sibling use
  the `ancestors` and `previous` lists on `ThemeNode`. Structural
  `:first-child` and `:last-child` use `index`/`sibling_count`.
- **Values.** `color`, `@define-color`, `var()`, `rgb()/rgba()`, GTK
  `shade()/alpha()/mix()/lighter()/darker()`, gradients, images, shadows,
  borders, radii, edges, font and opacity reach the computed style.
  Whitespace-separated shorthands such as `border: 1px solid #fff` are
  `Value::List`; comma-separated layers stay separate. Unknown declarations
  remain `Value::Raw` and are ignored here, not dropped.
- **Variants.** `theme.rs` selects GTK4/3/2 and `gtk.css`/`gtk-dark.css` (or
  `gtkrc`/`gtkrc-dark`) from the preference, available versions and appearance.
- **Legacy overrides.** The 21 `theme.toml` color keys are applied last and
  override GTK colors, including `text` → `ComputedStyle::color`.

## Public API

- `ThemeNode`, `StateFlags`, `ComputedStyle`, `LegacyPalette`, `Paint`
- `EngineTheme::{with_css, add_css, set_override}`
- `resolve(node, theme)` and `resolve_with_parent(node, parent, theme)`
- `theme::{ThemePref, ThemeVersion, Appearance, choose_version, variant_file}`

## Validation

```powershell
$env:GTK_THEME_CORPUS = 'C:\Users\marcelo\AppData\Local\Temp\opencode\gtk-corpus'
cargo test -p rf-theme-engine
cargo clippy -p rf-theme-engine --all-targets -- -D warnings
```

- 21 library tests, 11 cascade tests and 2 real-corpus integration tests pass.
- The real-corpus test resolves five synthetic nodes against all extracted
  stylesheets and asserts identical results on repeated resolution.
- Clippy is clean apart from the pre-existing workspace `natord` warning.
- App behavior unchanged: engine still off, no GPUI bridge yet.

## Known limits

- Attribute selectors and `:nth-child()` are parsed but do not match yet.
- Focus-within and backdrop state propagation is not implemented until the
  GPUI node adapter exists.
- Longhands and shorthands keep declaration order, but shorthand expansion is
  shallow: `background: #fff url(x)` yields two paints, not a layered
  shorthand with size, position and repeat fields.
- GTK2 `gtkrc` cascade is not part of M2; the grammar follows in M6.
