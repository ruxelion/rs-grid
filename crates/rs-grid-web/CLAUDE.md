# rs-grid-web

Browser integration. Manages the full lifecycle of a grid instance in the DOM:
mouse/keyboard events, rAF loop, resize, DPR, CSS theme, localisation.

## Modules

| Module | Role |
|---|---|
| `canvas` | `GridCanvas`: mounts the grid on an `HtmlCanvasElement`, manages rAF and events |
| `css_theme` | `theme_from_css_vars()`: reads CSS variables to build a `Theme` |
| `locale` | `Locale`: UI string translations (15 built-in languages, TOML-based) |

## Responsibilities of `GridCanvas`

- Resize via `ResizeObserver` (viewport update)
- `requestAnimationFrame` loop: `SceneBuilder` → `SceneFrame` → `CanvasRenderer`
- Event handling: `mousemove`, `mousedown`, `mouseup`, `wheel`, `keydown`,
  `copy`, `paste`
- Canvas DPR adjustment for HiDPI screens
- Auto-scroll during selection drag

## Critical invariants

- `GridCanvas::mount()` is the only public entry point — one canvas = one instance.
- Events are converted to `GridCommand` before being applied to `GridState`.
  **Do not manipulate `GridState` directly from event handlers.**
- DPR is read once at mount and on each resize. Do not re-read it every frame.
- `theme_from_css_vars()` reads the DOM — call only at mount, not every frame.

## CSS theme

CSS variables are prefixed `--rs-grid-*`. `light.css` and `dark.css` in
`examples/example-common/themes/` are **auto-generated** — do not edit them.

To add a theme variable:
1. Add the field in `Theme` (`rs-grid-scene/src/theme.rs`) with a default
   in both `light()` and `dark()`
2. Add the mapping in `css_theme.rs` (reads the CSS variable at runtime)
3. Add the entry in `generate_theme.rs` (`examples/example-common/src/bin/`)
4. Run `cargo run -p example-common --bin generate-theme` to regenerate CSS
