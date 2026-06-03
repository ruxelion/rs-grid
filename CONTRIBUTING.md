# Contributing to rs-grid

Thanks for your interest in contributing! rs-grid is a high-performance,
renderer-agnostic Rust/WASM data grid engine. It is in **early development**,
so the public API may still change.

## Getting set up

```sh
# Rust (2021 edition) + the WASM target
rustup target add wasm32-unknown-unknown

# One-time dev tools
cargo install cargo-nextest --locked
cargo install trunk --locked
```

## Workspace layout

```
leptos / dioxus / yew  →  web  →  render-canvas  →  scene  →  core
```

Dependencies flow in **one direction only** — never introduce a reverse
dependency. `rs-grid-core` is headless pure Rust with **no WASM dependency**
and is unit-testable with plain `cargo`.

| Crate | Role |
| ----- | ---- |
| `rs-grid-core` | Model, viewport, selection, hit-testing (no WASM) |
| `rs-grid-scene` | `GridState` → renderer-agnostic `ScenePrimitive`s |
| `rs-grid-render-canvas` | Canvas2D backend via `wasm-bindgen` |
| `rs-grid-web` | Browser integration: events, DPR, rAF loop, CSS theme |
| `rs-grid-leptos` / `-dioxus` / `-yew` | Framework component wrappers |
| `rs-grid-icons` | Embedded SVG icons |

## Before you open a PR

Run the full local check suite — CI runs the same gates:

```sh
# Format (rustfmt.toml uses nightly-only options)
cargo +nightly fmt --all

# Lint — warnings are errors
cargo clippy --workspace -- -D warnings

# Unit tests (WASM crates excluded — they need a browser)
cargo nextest run --workspace \
  --exclude rs-grid-web --exclude rs-grid-leptos \
  --exclude rs-grid-dioxus --exclude rs-grid-yew \
  --exclude rs-grid-render-canvas \
  --exclude fixture-leptos --exclude example-common
```

End-to-end (Playwright) tests live in `e2e/` — see the README.

## Code conventions

- Rust 2021 edition, **max line width 80** (`rustfmt.toml`).
- Imports grouped by `StdExternalCrate`, granularity `Crate`.
- **No `unwrap()` in production code** — use `expect("reason")` or propagate
  errors. (`unwrap()` is fine in tests.)
- All `GridState` mutations go through `GridState::apply(GridCommand)` — never
  mutate state directly.
- Hit-testing must stay **O(log n)** — do not introduce O(n) on that path.
- Any new color/size/animation must be exposed through the theme engine
  (`Theme` field + `light()`/`dark()`/`dimmed()` defaults + a `--rs-grid-*`
  CSS variable), never hardcoded.

## Commits & PRs

- Keep PRs focused and reasonably small.
- Update the relevant docs in the same change (if behaviour changed, docs
  change too).
- Describe **what** changed and **why** in the PR body.
- Make sure `fmt`, `clippy`, and tests are green before requesting review.

## Adding a new renderer

1. Create a crate depending on `rs-grid-scene`.
2. Consume `SceneFrame` and iterate over `ScenePrimitive`.
3. Do **not** modify `rs-grid-core` or `rs-grid-scene`.

## Reporting bugs / requesting features

Use the GitHub issue templates. For security issues, **do not** open a public
issue — see [SECURITY.md](SECURITY.md).

By contributing you agree your contributions are licensed under the project's
[MIT License](LICENSE).
