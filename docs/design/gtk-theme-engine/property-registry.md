# GTK property registry (M0.2)

Generated from the compiled `*.css` files in the extracted Agata, Grass, and Human theme archives.
The archives live outside the repository; only this classification is committed.

Distinct properties: **312**. Supported: **78**. Parse-only: **77**. Ignored: **157**.

`supported` means the engine will parse and paint the property in M4. `parse-only` means the value is kept in the AST with no visual effect until a later milestone. `ignored` means the token is either a selector false positive from the scan or an extension with no RustyFiles node.

| Property | Occurrences | Status | Reason |
|---|---:|---|---|
| `background-color` | 9297 | supported | parse and paint in M4 |
| `color` | 7869 | supported | parse and paint in M4 |
| `padding` | 5240 | supported | parse and paint in M4 |
| `box-shadow` | 4848 | supported | parse and paint in M4 |
| `border-radius` | 4738 | supported | parse and paint in M4 |
| `margin` | 3608 | supported | parse and paint in M4 |
| `border` | 2777 | supported | parse and paint in M4 |
| `background` | 2261 | supported | parse and paint in M4 |
| `min-height` | 2019 | supported | parse and paint in M4 |
| `min-width` | 1734 | supported | parse and paint in M4 |
| `background-image` | 1590 | supported | parse and paint in M4 |
| `transition` | 1213 | parse-only | value stored; visual effect deferred to M7 |
| `-gtk-icon-source` | 1051 | parse-only | icon slots stay in the separate icon-theme work |
| `margin-right` | 1002 | supported | parse and paint in M4 |
| `margin-left` | 966 | supported | parse and paint in M4 |
| `font-weight` | 869 | supported | parse and paint in M4 |
| `border-color` | 751 | supported | parse and paint in M4 |
| `margin-bottom` | 741 | supported | parse and paint in M4 |
| `margin-top` | 737 | supported | parse and paint in M4 |
| `padding-left` | 709 | supported | parse and paint in M4 |
| `font-size` | 674 | supported | parse and paint in M4 |
| `border-style` | 654 | supported | parse and paint in M4 |
| `padding-right` | 628 | supported | parse and paint in M4 |
| `outline` | 592 | supported | parse and paint in M4 |
| `background-size` | 567 | supported | parse and paint in M4 |
| `spacing` | 550 | supported | parse and paint in M4 |
| `border-bottom` | 445 | supported | parse and paint in M4 |
| `background-position` | 437 | supported | parse and paint in M4 |
| `border-spacing` | 432 | supported | parse and paint in M4 |
| `border-top` | 428 | supported | parse and paint in M4 |
| `border-bottom-left-radius` | 397 | supported | parse and paint in M4 |
| `width` | 389 | supported | parse and paint in M4 |
| `padding-top` | 384 | supported | parse and paint in M4 |
| `background-repeat` | 365 | supported | parse and paint in M4 |
| `padding-bottom` | 346 | supported | parse and paint in M4 |
| `border-bottom-right-radius` | 336 | supported | parse and paint in M4 |
| `height` | 330 | supported | parse and paint in M4 |
| `opacity` | 321 | supported | parse and paint in M4 |
| `border-width` | 311 | supported | parse and paint in M4 |
| `border-top-left-radius` | 303 | supported | parse and paint in M4 |
| `radio` | 296 | ignored | selector token captured by the scan, not a declaration |
| `background-clip` | 289 | supported | parse and paint in M4 |
| `border-right` | 282 | supported | parse and paint in M4 |
| `border-top-right-radius` | 282 | supported | parse and paint in M4 |
| `border-left` | 255 | supported | parse and paint in M4 |
| `animation` | 251 | parse-only | value stored; visual effect deferred to M7 |
| `check` | 249 | ignored | selector token captured by the scan, not a declaration |
| `border-image` | 236 | supported | parse and paint in M4 |
| `icon-size` | 234 | supported | parse and paint in M4 |
| `-gtk-icon-transform` | 226 | parse-only | icon slots stay in the separate icon-theme work |
| `text-align` | 205 | supported | parse and paint in M4 |
| `outline-offset` | 185 | supported | parse and paint in M4 |
| `button` | 166 | ignored | selector token captured by the scan, not a declaration |
| `text-shadow` | 163 | supported | parse and paint in M4 |
| `transition-duration` | 150 | parse-only | value stored; visual effect deferred to M7 |
| `visibility` | 144 | parse-only | value stored; visual effect deferred to M7 |
| `outline-color` | 131 | supported | parse and paint in M4 |
| `toolbar` | 125 | ignored | selector token captured by the scan, not a declaration |
| `background-gradient-direction` | 121 | parse-only | value stored; visual effect deferred to M7 |
| `spinbutton` | 117 | ignored | selector token captured by the scan, not a declaration |
| `row` | 112 | ignored | selector token captured by the scan, not a declaration |
| `--menuitem-icon` | 112 | ignored | theme-specific extension; no mapped RustyFiles node |
| `entry` | 96 | ignored | selector token captured by the scan, not a declaration |
| `caret-color` | 95 | supported | parse and paint in M4 |
| `-gtk-icon-size` | 92 | parse-only | icon slots stay in the separate icon-theme work |
| `border-left-width` | 88 | supported | parse and paint in M4 |
| `switch` | 84 | ignored | selector token captured by the scan, not a declaration |
| `background-gradient-start` | 81 | parse-only | value stored; visual effect deferred to M7 |
| `background-gradient-end` | 81 | parse-only | value stored; visual effect deferred to M7 |
| `border-right-width` | 65 | supported | parse and paint in M4 |
| `border-image-source` | 64 | supported | parse and paint in M4 |
| `border-bottom-style` | 62 | supported | parse and paint in M4 |
| `max-width` | 61 | supported | parse and paint in M4 |
| `filter` | 60 | parse-only | value stored; visual effect deferred to M7 |
| `border-bottom-color` | 60 | supported | parse and paint in M4 |
| `border-bottom-width` | 57 | supported | parse and paint in M4 |
| `border-left-color` | 52 | supported | parse and paint in M4 |
| `-gtk-icon-shadow` | 51 | parse-only | icon slots stay in the separate icon-theme work |
| `spinner` | 50 | ignored | selector token captured by the scan, not a declaration |
| `calendar` | 50 | ignored | selector token captured by the scan, not a declaration |
| `border-top-style` | 49 | supported | parse and paint in M4 |
| `border-right-style` | 49 | supported | parse and paint in M4 |
| `border-top-width` | 48 | supported | parse and paint in M4 |
| `border-top-color` | 48 | supported | parse and paint in M4 |
| `display` | 48 | parse-only | value stored; visual effect deferred to M7 |
| `selection-background-color` | 47 | supported | parse and paint in M4 |
| `-st-icon-style` | 44 | parse-only | icon slots stay in the separate icon-theme work |
| `menuitem` | 44 | ignored | selector token captured by the scan, not a declaration |
| `selected-color` | 43 | supported | parse and paint in M4 |
| `-gtk-icon-filter` | 40 | parse-only | icon slots stay in the separate icon-theme work |
| `tabbar` | 40 | ignored | selector token captured by the scan, not a declaration |
| `monthcell` | 40 | ignored | selector token captured by the scan, not a declaration |
| `spacing-rows` | 38 | supported | parse and paint in M4 |
| `spacing-columns` | 38 | supported | parse and paint in M4 |
| `iconview` | 38 | ignored | selector token captured by the scan, not a declaration |
| `-gtk-icon-effect` | 37 | parse-only | icon slots stay in the separate icon-theme work |
| `colorswatch` | 35 | ignored | selector token captured by the scan, not a declaration |
| `font-style` | 34 | supported | parse and paint in M4 |
| `font-family` | 34 | supported | parse and paint in M4 |
| `outline-style` | 33 | supported | parse and paint in M4 |
| `outline-width` | 33 | supported | parse and paint in M4 |
| `font` | 33 | supported | parse and paint in M4 |
| `border-left-style` | 33 | supported | parse and paint in M4 |
| `expander` | 32 | ignored | selector token captured by the scan, not a declaration |
| `checkbutton` | 32 | ignored | selector token captured by the scan, not a declaration |
| `menubar` | 29 | ignored | selector token captured by the scan, not a declaration |
| `-GtkWidget-window-dragging` | 28 | ignored | theme-specific extension; no mapped RustyFiles node |
| `wnck-pager` | 26 | ignored | selector token captured by the scan, not a declaration |
| `headerbar` | 26 | ignored | selector token captured by the scan, not a declaration |
| `-arrow-rise` | 25 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-natural-hpadding` | 25 | ignored | theme-specific extension; no mapped RustyFiles node |
| `transition-property` | 25 | parse-only | value stored; visual effect deferred to M7 |
| `-GtkArrow-arrow-scaling` | 24 | ignored | theme-specific extension; no mapped RustyFiles node |
| `all` | 24 | ignored | selector token captured by the scan, not a declaration |
| `appearance` | 24 | ignored | selector token captured by the scan, not a declaration |
| `border-image-slice` | 24 | supported | parse and paint in M4 |
| `border-image-repeat` | 24 | supported | parse and paint in M4 |
| `max-height` | 23 | supported | parse and paint in M4 |
| `-arrow-border-radius` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-arrow-background-color` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-arrow-border-width` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-arrow-border-color` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-arrow-base` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-minimum-hpadding` | 21 | ignored | theme-specific extension; no mapped RustyFiles node |
| `background-origin` | 21 | supported | parse and paint in M4 |
| `border-right-color` | 21 | supported | parse and paint in M4 |
| `stroke-width` | 20 | parse-only | value stored; visual effect deferred to M7 |
| `stroke` | 20 | parse-only | value stored; visual effect deferred to M7 |
| `fill` | 20 | parse-only | value stored; visual effect deferred to M7 |
| `position` | 20 | parse-only | value stored; visual effect deferred to M7 |
| `image` | 19 | ignored | selector token captured by the scan, not a declaration |
| `-st-vfade-offset` | 18 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-boxpointer-gap` | 17 | ignored | theme-specific extension; no mapped RustyFiles node |
| `label` | 17 | ignored | selector token captured by the scan, not a declaration |
| `text-decoration-line` | 17 | supported | parse and paint in M4 |
| `font-feature-settings` | 16 | supported | parse and paint in M4 |
| `-PnlDockBin-handle-size` | 16 | ignored | theme-specific extension; no mapped RustyFiles node |
| `docktab` | 16 | ignored | selector token captured by the scan, not a declaration |
| `transform` | 16 | parse-only | value stored; visual effect deferred to M7 |
| `placessidebar` | 16 | ignored | selector token captured by the scan, not a declaration |
| `stackswitcher` | 16 | ignored | selector token captured by the scan, not a declaration |
| `expander-widget` | 16 | ignored | selector token captured by the scan, not a declaration |
| `windowcontrols` | 16 | ignored | selector token captured by the scan, not a declaration |
| `toast` | 16 | ignored | selector token captured by the scan, not a declaration |
| `right` | 16 | ignored | selector token captured by the scan, not a declaration |
| `html` | 16 | ignored | selector token captured by the scan, not a declaration |
| `-gtk-icon-style` | 16 | parse-only | icon slots stay in the separate icon-theme work |
| `scale` | 16 | ignored | selector token captured by the scan, not a declaration |
| `progressbar` | 16 | ignored | selector token captured by the scan, not a declaration |
| `border-radius-topleft` | 15 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gtk-outline-radius` | 15 | ignored | theme-specific extension; no mapped RustyFiles node |
| `border-radius-topright` | 14 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-st-hfade-offset` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gradient-height` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gradient-start` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gradient-end` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-margin-horizontal` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-height` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-background-color` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-border-color` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-active-background-color` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-active-border-color` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-border-width` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-handle-radius` | 13 | ignored | theme-specific extension; no mapped RustyFiles node |
| `text-decoration` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `row-spacing` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `column-spacing` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `page-padding-top` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `page-padding-bottom` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `page-padding-left` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `page-padding-right` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-y-offset` | 12 | ignored | theme-specific extension; no mapped RustyFiles node |
| `StEntry` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-XfwmTabwinWidget-icon-size` | 12 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-XfwmTabwinWidget-preview-size` | 12 | ignored | theme-specific extension; no mapped RustyFiles node |
| `margin-inline` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `terminal-window` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `line-height` | 12 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-active-window-background` | 10 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-active-window-border` | 10 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-inactive-window-background` | 10 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-inactive-window-border` | 10 | ignored | theme-specific extension; no mapped RustyFiles node |
| `decoration` | 10 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-NaTrayApplet-icon-padding` | 10 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-cinnamon-caption-spacing` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTextView-error-underline-color` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkToolItemGroup-expander-size` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkWidget-text-handle-width` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkWidget-text-handle-height` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkDialog-button-spacing` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkDialog-action-area-border` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrollbar-has-backward-stepper` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrollbar-has-forward-stepper` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-extents` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-input-extents` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-shadow-offset-x` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-shadow-offset-y` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-active-shadow-color` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-active-shadow-radius` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-inactive-shadow-color` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-inactive-shadow-radius` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-glow-size` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-glow-color` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-title-indent` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-title-fade` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-UnityDecoration-title-alignment` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NaTrayApplet-icon-size` | 9 | ignored | theme-specific extension; no mapped RustyFiles node |
| `separator` | 9 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `filechooserbutton` | 9 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `warning-color` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `icon-shadow` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-x-offset` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-height` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-background-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-active-background-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-overdrive-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-overdrive-separator-width` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-border-width` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-border-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-panel-corner-radius` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-panel-corner-background-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-panel-corner-border-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `offset-y` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-GtkToolButton-icon-spacing` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrolledWindow-scrollbar-spacing` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gtk-secondary-caret-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-horizontal-separator` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-grid-line-width` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-grid-line-pattern` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-tree-line-width` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-tree-line-pattern` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeView-expander-size` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `junction` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `stack` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `omnibar` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-GdMainIconView-icon-size` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `UnityPanelWidget` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-PanelMenuBar-icon-visible` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-bg-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-fg-color` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-bar-width` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-bar-radius` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-bottom-padding` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-NemoPlacesTreeView-disk-full-max-length` | 8 | ignored | theme-specific extension; no mapped RustyFiles node |
| `arrow` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `scrollbar` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `link` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `tabview` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `tabbox` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `scrolledwindow` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `listview` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `list` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `emoji-completion-row` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `calendar-view` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `event` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `weekgrid` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `padding-block` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `border-inline-start` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `notebook` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `levelbar` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `box` | 8 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `border-radius-bottomleft` | 7 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-cinnamon-close-overlap` | 6 | ignored | theme-specific extension; no mapped RustyFiles node |
| `border-radius-bottomright` | 6 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `caret-size` | 5 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-st-background-image-shadow` | 5 | ignored | theme-specific extension; no mapped RustyFiles node |
| `margin-from-right-edge-of-screen` | 5 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `z-index` | 5 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-slider-handle-border-color` | 5 | ignored | theme-specific extension; no mapped RustyFiles node |
| `link-color` | 5 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-pie-border-width` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-pie-border-color` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-pie-background-color` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-panel-corner-border-width` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-panel-corner-opacity` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-arrow-box-shadow` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `weight` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-slider-handle-border-width` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-active-border-color` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-barlevel-overdrive-border-color` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `max-row-spacing` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `max-column-spacing` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `visible-width` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `--tab-min-height` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--urlbar-min-height` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--urlbar-height` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--urlbar-container-height` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-padding` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-shadow-margin` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-box-shadow-margin` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-box-shadow` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-border-radius` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-border-color` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--panel-background` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--inactive-window-transition` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `--inactive-titlebar-opacity` | 4 | ignored | theme-specific extension; no mapped RustyFiles node |
| `padding-inline` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `bottom` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `left` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `treeview` | 4 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `-GtkToolbar-button-relief` | 2 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-slider-border-radius` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-gradient-direction` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrollbar-has-secondary-backward-stepper` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrollbar-has-secondary-forward-stepper` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrolledwindow-scrollbar-spacing` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkScrolledwindow-scrollbars-within-bevel` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkToolbutton-icon-spacing` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-GtkTreeview-expander-size` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `-treeview-vertical-separator` | 1 | ignored | theme-specific extension; no mapped RustyFiles node |
| `actionbar` | 1 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
| `XfdesktopIconView` | 1 | parse-only | value parsed; effect not mapped to a RustyFiles node yet |
