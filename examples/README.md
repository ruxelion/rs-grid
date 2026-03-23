# rs-grid examples

## Available examples

| Example | Stack | Description |
|---|---|---|
| [basic-leptos](basic-leptos/) | Leptos 0.8 CSR + Trunk | Full-featured demo with reactive controls |
| [basic-js](basic-js/) | Vanilla JS + wasm-pack | Minimal integration — no framework, pure ES modules |

Both examples share the same virtual dataset (up to 1 quadrillion rows),
theme CSS, and feature set: row/column sizing, cell editing, export/import,
pinned columns, filtering, and four themes (Light, Dark, Material 3,
Material 3 Dark).

## Shared code

[example-common](example-common/) is a Rust crate containing:

- `build_model()` — creates a `GridModel` backed by a deterministic fake
  data generator (`fake_data.rs`, ~950 lines of names, roles, departments…)
- `rs-grid-theme.css` — canonical theme file with Light, Dark, Material 3
  light, and Material 3 dark variants

Each example has a `build.rs` that copies the CSS from `example-common/` at
compile time, so themes stay in sync.

## Running an example

### Leptos (basic-leptos)

```sh
just serve          # trunk serve on port 9080
```

### Vanilla JS (basic-js)

```sh
just serve-js       # wasm-pack build + http.server on port 8081
```

### Any example by name

```sh
just serve-example basic-js
```

## Creating a new example

```sh
just new-example my-demo
```

This copies the [_template-wasm](_template-wasm/) scaffold, replaces
`{{NAME}}` / `{{TITLE}}` placeholders, and prints next steps:

1. Add `"examples/my-demo"` to `[workspace] members` in the root
   `Cargo.toml`
2. `just build-example my-demo`
3. `just serve-example my-demo`

The template produces a vanilla JS + wasm-pack example with
`example-common` already wired in.

## Themes

The theme selector in each example switches between four presets by
setting a CSS class on `<html>`:

| Class | Theme |
|---|---|
| *(none)* | Light (default) |
| `dark` | Dark |
| `material` | Material Design 3 Light |
| `material-dark` | Material Design 3 Dark |

To add a new theme, add a `:root.my-theme { ... }` block in
`example-common/rs-grid-theme.css` defining all `--rs-grid-*` variables,
then add the option to each example's theme selector.
