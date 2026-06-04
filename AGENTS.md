# rs-grid — Claude Code guide

## Documentation

The user-facing documentation lives in the **separate `rs-grid-site` repo**
(rspress, bilingual EN/FR) and is published at <https://ruxelion.com>. Edit the
docs there — not in this repo.

The in-repo `docs/` folder holds only internal reference files:

- `docs/skill.md` — MCP server skill definition (served as `rs-grid://skill.md`)
- `docs/row-count-limits.md` — row-count / f64-precision reference, cited from
  `rs-grid-core/src/selection.rs`
- `docs/RUSTDOC_HISTORY.md` — rustdoc history notes

Consult the published docs before asking questions about existing behaviour or
before designing a change.

## Project focus

`rs-grid` is a high-performance, renderer-agnostic Rust/WASM data grid engine.
The current focus is the technical core:

- viewport virtualisation
- smooth rendering
- selection
- performant hit-testing
- basic editing
- renderer-agnostic architecture
- core stability

## Architecture

```
GridState  ──►  SceneBuilder  ──►  SceneFrame  ──►  CanvasRenderer  ──►  <canvas>
```

| Crate                   | Role                                                                             |
| ----------------------- | -------------------------------------------------------------------------------- |
| `rs-grid-core`          | Headless logic: model, viewport, selection, hit-testing. **No WASM dependency.** |
| `rs-grid-scene`         | Converts `GridState` into renderer-agnostic primitives (`ScenePrimitive`)        |
| `rs-grid-render-canvas` | Canvas2D backend via wasm-bindgen                                                |
| `rs-grid-web`           | Browser integration: events, DPR, rAF loop, CSS theme                            |
| `rs-grid-leptos`        | Leptos CSR component wrapper (`<GridCanvas>`)                                    |
| `rs-grid-dioxus`        | Dioxus CSR component wrapper (`GridCanvas`)                                      |
| `rs-grid-yew`           | Yew CSR component wrapper (`GridCanvas`)                                         |
| `e2e/fixture-leptos`    | Minimal Leptos app — the e2e / CI / Pages target                                 |

Dependencies flow in one direction only — never introduce a reverse dependency:

- `leptos → web → render-canvas → scene → core`
- `dioxus → web → render-canvas → scene → core`
- `yew    → web → render-canvas → scene → core`

## Common commands

```sh
# Quick check (entire workspace)
cargo check --workspace

# Native build (for rs-grid-core unit tests)
cargo build -p rs-grid-core

# Unit tests (nextest — WASM crates excluded)
cargo nextest run --workspace \
  --exclude rs-grid-web --exclude rs-grid-leptos \
  --exclude rs-grid-dioxus --exclude rs-grid-yew \
  --exclude rs-grid-render-canvas \
  --exclude fixture-leptos --exclude example-common

# Unit tests — core only
cargo nextest run -p rs-grid-core

# Code coverage — HTML report (opens browser)
cargo llvm-cov nextest \
  -p rs-grid-core -p rs-grid-scene -p rs-grid-icons \
  --html --open

# Code coverage — lcov format (CI)
cargo llvm-cov nextest \
  -p rs-grid-core -p rs-grid-scene -p rs-grid-icons \
  --lcov --output-path target/llvm-cov/lcov.info

# Formatting
cargo fmt --all

# Linting
cargo clippy --workspace -- -D warnings

# WASM build (e2e fixture — minimal Leptos app, no Tailwind)
cd e2e/fixture-leptos
trunk build

# Dev server (hot-reload)
cd e2e/fixture-leptos
trunk serve
# → http://localhost:9079  (config dans e2e/fixture-leptos/Trunk.toml)
#
# The framework demos moved to standalone repos:
#   github.com/ruxelion/rs-grid-example-{leptos,dioxus,yew,js}
```

### One-time tool installation

```sh
cargo install cargo-nextest --locked
cargo install cargo-llvm-cov --locked
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
The reference files are in `examples/example-common/themes/` (`light.css`,
`dark.css`, `dimmed.css`, + shell overrides).

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

# 2. Build the fixture app (required before each run)
cd e2e/fixture-leptos && trunk build

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

### Playwright MCP — tests interactifs en cours de développement

Pour vérifier visuellement un changement avec les outils Playwright MCP
(`mcp__playwright__browser_*`) **sans relancer la suite complète** :

```sh
# 1. Démarrer le dev server (une seule fois par session)
cd e2e/fixture-leptos && trunk serve
# Écoute sur localhost:9079 — hot-reload automatique à chaque cargo build

# 2. Après chaque modification, recompiler
cd e2e/fixture-leptos && trunk build
```

Puis dans les outils MCP :

```
mcp__playwright__browser_navigate → http://localhost:9079
```

**Règle** : utiliser `http://localhost:9079` (dev server trunk) pour les
vérifications MCP interactives. Les tests formels `/e2e` utilisent
`http://localhost:4173` (serveur statique sur le `dist/` pré-compilé).

## Claude working rules

- After any code change in `rs-grid-core`, always run `/test` to verify tests
  pass.
- If a test fails, fix it before continuing.
- Any visual change or addition (color, size, animation) must be made
  configurable through the theme engine: field in `Theme`, default value in
  `light()`, `dark()`, and `dimmed()`, CSS variable in `css_theme.rs`.

### Documentation sync

After every code change, update the relevant CLAUDE.md files in the same
commit. The rule: **if the code changed, the docs change too.**

| What changed                   | Which CLAUDE.md to update                   |
| ------------------------------ | ------------------------------------------- |
| Public API of a crate          | The crate's own `CLAUDE.md`                 |
| New feature or workflow step   | Root `CLAUDE.md` (+ crate if needed)        |
| New theme / theme variable     | `rs-grid-web/CLAUDE.md` → CSS theme section |
| New primitive or scene concept | `rs-grid-scene/CLAUDE.md`                   |
| New command, shortcut, or tool | Root `CLAUDE.md` → Common commands          |
| New invariant or constraint    | The crate's own `CLAUDE.md`                 |

Do not update CLAUDE.md for internal refactors that don't change
observable behaviour or usage.

## Adding a new renderer

1. Create a new crate depending on `rs-grid-scene`
2. Consume `SceneFrame` and iterate over `ScenePrimitive`
3. Do not modify `rs-grid-core` or `rs-grid-scene`
