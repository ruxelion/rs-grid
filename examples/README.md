# rs-grid examples

The framework demo apps now live in standalone repos under the `ruxelion` org:

| Demo | Framework | Repo |
|---|---|---|
| Leptos | Leptos 0.8 CSR + Trunk | [rs-grid-example-leptos](https://github.com/ruxelion/rs-grid-example-leptos) |
| Dioxus | Dioxus 0.7 CSR + Trunk | [rs-grid-example-dioxus](https://github.com/ruxelion/rs-grid-example-dioxus) |
| Yew | Yew 0.23 CSR + Trunk | [rs-grid-example-yew](https://github.com/ruxelion/rs-grid-example-yew) |
| Vanilla JS | wasm-pack library | [rs-grid-example-js](https://github.com/ruxelion/rs-grid-example-js) |

Clone one and run `trunk serve` (or `wasm-pack build` for js). They all share
the same virtual dataset (up to 1 quadrillion rows) and three themes
(Light, Dark, Dimmed).

## Shared code

This directory keeps [example-common](example-common/) — the Rust crate the demo
repos depend on (via git tag) — containing:

- `build_model()` — creates a `GridModel` backed by a deterministic fake
  data generator (`fake_data.rs`)
- `fmt_rows()` / `fmt_cols()` — display label helpers shared across the demos
- `themes/` — CSS theme files (see below)

## Themes

The theme selector switches between three presets by setting a CSS class on
`<html>`:

| Class | Theme |
|---|---|
| *(none)* | Light (AG Grid Quartz-inspired) |
| `dark` | Dark (iOS gray) |
| `dimmed` | Dimmed (GitHub Dimmed-inspired) |

Theme files in `example-common/themes/`:

```
base.css          # reset + app shell layout
light.css         # :root — auto-generated from Theme::light()
dark.css          # :root.dark — auto-generated (diff vs light)
dark-shell.css    # :root.dark — app shell overrides (hand-written)
dimmed.css        # :root.dimmed — auto-generated (diff vs light)
dimmed-shell.css  # :root.dimmed — app shell overrides (hand-written)
```

`light.css`, `dark.css`, and `dimmed.css` are **auto-generated** from
`Theme::light()`, `Theme::dark()`, and `Theme::dimmed()` in
`crates/rs-grid-scene/src/theme.rs`.

To add or change a theme variable, see `crates/rs-grid-web/CLAUDE.md`.

## Creating a new example

Copy `_template-wasm/` for a minimal vanilla JS + wasm-pack scaffold (no
Trunk). For a Trunk-based example, fork one of the standalone
`rs-grid-example-*` repos and update the crate name and HTML title.
