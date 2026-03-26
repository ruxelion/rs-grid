# rs-grid — Claude Code guide

## Documentation

The project has a full documentation site built in `site/`. Two files are
particularly useful for AI context:

- `site/doc_build/llms.txt` — index of all documentation pages
- `site/doc_build/llms-full.txt` — full documentation concatenated (use for
  deep API or concept questions)

Individual pages under `site/doc_build/` cover the API reference
(`api/`), architecture concepts (`concepts/`), data sources (`data/`),
and all features (`features/`). Consult these before asking questions about
existing behaviour or before designing a change.

## Global context

This repository is part of a global roadmap centralised in the private repo:
https://github.com/bpodwinski/roadmap

If the local `roadmap/` folder is present in this repo, it must be used as the
primary source of truth.

Before any structural proposal, consult in priority:

- `roadmap/AI_CONTEXT.md`
- `roadmap/docs/00-hub.md`
- `roadmap/docs/02-current-focus.md`
- `roadmap/docs/projects/rs-grid.md`

If `roadmap/` is not available locally, use the private GitHub repo as
reference.

## Position in the roadmap

`rs-grid` is a strategic cross-cutting project, but it is not the top priority
while `FDF` is not yet stable.

Role of `rs-grid` in the overall system:

- high-performance Rust/WASM data grid engine
- reusable UI base for future tools
- potential foundation for a future AG Grid competitor product
- reusable building block for the future `Product Data Editor`

## Strategic rules

- Do not drift the project too early toward full AG Grid feature parity
- Prioritise a solid technical core first
- Avoid business, SaaS, or broad JS expansion work while the core product is
  not sufficiently mature
- If an important decision changes the project direction, propose an update in
  the `roadmap` repo, specifically in `docs/03-decisions.md`

## Current priorities for rs-grid

Preferred:

- viewport virtualisation
- smooth rendering
- selection
- performant hit-testing
- basic editing
- renderer-agnostic architecture
- core stability

Avoid for now:

- feature race against AG Grid
- dispersion on secondary integrations
- premature API complexity
- commercial expansion too early

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

| Crate                   | Role                                                                              |
| ----------------------- | --------------------------------------------------------------------------------- |
| `rs-grid-core`          | Headless logic: model, viewport, selection, hit-testing. **No WASM dependency.** |
| `rs-grid-scene`         | Converts `GridState` into renderer-agnostic primitives (`ScenePrimitive`)         |
| `rs-grid-render-canvas` | Canvas2D backend via wasm-bindgen                                                 |
| `rs-grid-web`           | Browser integration: events, DPR, rAF loop, CSS theme                            |
| `rs-grid-leptos`        | Leptos CSR component wrapper (`<GridCanvas>`)                                     |
| `rs-grid-dioxus`        | Dioxus CSR component wrapper (`GridCanvas`)                                       |
| `examples/basic-leptos` | Demo application using Trunk                                                      |

Dependencies flow in one direction only — never introduce a reverse dependency:
- `leptos → web → render-canvas → scene → core`
- `dioxus → web → render-canvas → scene → core`

## Common commands

```sh
# Quick check (entire workspace)
cargo check --workspace

# Native build (for rs-grid-core unit tests)
cargo build -p rs-grid-core

# Unit tests
cargo test --workspace

# Formatting
cargo fmt --all

# Linting
cargo clippy --workspace -- -D warnings

# WASM build (Leptos example)
cd examples/basic-leptos
trunk build

# Dev server
cd examples/basic-leptos
trunk serve
```

## Code conventions

- **Edition**: Rust 2021
- **Max line width**: 80 characters (rustfmt.toml)
- **Imports**: grouped by `StdExternalCrate`, granularity `Crate`
- **Comments**: wrapped at 80 chars, formatted in doc-comments
- No `unwrap()` in production code — use `expect("reason")` or error propagation

## Important limits

- **Row count**: `u64` (max ~9×10¹⁴ with f64 precision). See `docs/row-count-limits.md`.
- **WASM32**: 32-bit address space, `usize` = 4 GB max. Row indices are `u64`, not `usize`.
- **Hit-testing**: O(log n) thanks to precomputed column offsets. Do not introduce O(n) on this path.

## Data model

`GridState` is the central structure:

- `model: GridModel` — columns + data
- `viewport: ViewportState` — scroll_x, scroll_y, width, height
- `selection: SelectionState` — anchor + focus (cell, row, or column)

All mutations go exclusively through `GridState::apply(GridCommand)`.

## Theme

The theme is read from CSS variables (`rs-grid-web::theme_from_css_vars`).
The reference file is `examples/basic-leptos/rs-grid-theme.css`.

**Rule**: any color or visual value introduced by a change must be exposed in
`Theme` (`rs-grid-scene/src/theme.rs`) with a default value in both `light()`
and `dark()`, read from a CSS variable `--rs-grid-<name>` in `css_theme.rs`,
and documented in the `css_theme.rs` table. Never hardcode a color or size
in `builder.rs`.

## End-to-end tests (Playwright)

Visual and functional tests are in `e2e/`.

```sh
# 1. Install Playwright (once)
cd e2e && npm install && npx playwright install chromium

# 2. Build the app (required before each run)
cd examples/basic-leptos && trunk build

# 3. Run the tests
cd e2e && npm test

# 4. Generate / regenerate reference screenshots
cd e2e && npm run update-snapshots
```

**Test structure** (`e2e/tests/grid.spec.ts`):

- `smoke` — page loads, canvas visible, default values
- `controls` — row/column dropdowns
- `canvas interaction` — clicks, scroll, shift-click (viewport coordinates)
- `visual regression` — pixel-by-pixel screenshot comparison (2% tolerance)

**Canvas note**: the grid is rendered on `<canvas>`, not in the DOM.
Interaction tests use fixed pixel coordinates. If the layout changes, update
the coordinates in `grid.spec.ts`.

**Claude command**: `/e2e` runs `trunk build` then `npm test` automatically.

## Claude working rules

- After any code change in `rs-grid-core`, always run `/test` to verify tests
  pass.
- If a test fails, fix it before continuing.
- Any visual change or addition (color, size, animation) must be made
  configurable through the theme engine: field in `Theme`, default value in
  `light()` and `dark()`, CSS variable in `css_theme.rs`.

## Adding a new renderer

1. Create a new crate depending on `rs-grid-scene`
2. Consume `SceneFrame` and iterate over `ScenePrimitive`
3. Do not modify `rs-grid-core` or `rs-grid-scene`
