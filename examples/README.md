# rs-grid examples

## Available examples

| Example | Framework | Description |
|---|---|---|
| [basic-leptos](basic-leptos/) | Leptos 0.8 CSR + Trunk | Demo with reactive controls and language selector |
| [basic-dioxus](basic-dioxus/) | Dioxus CSR + Trunk | Demo with reactive controls |
| [basic-yew](basic-yew/) | Yew CSR + Trunk | Demo with reactive controls |

All examples share the same virtual dataset (up to 1 quadrillion rows) and
the same three themes (Light, Dark, Dimmed).

## Shared code

[example-common](example-common/) is a Rust crate containing:

- `build_model()` — creates a `GridModel` backed by a deterministic fake
  data generator (`fake_data.rs`)
- `fmt_rows()` / `fmt_cols()` — display label helpers shared across examples
- `themes/` — CSS theme files (see below)

## Running an example

```sh
cd examples/basic-leptos   # or basic-dioxus, basic-yew
trunk serve
```

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
Trunk). For a Trunk-based example, copy one of the existing examples and
update the crate name and HTML title.
