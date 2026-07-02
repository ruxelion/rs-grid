# rs-grid — Claude Code guide

## Documentation

User docs → <https://rs-grid.com> (repo `rs-grid-site` — do not edit here).
Internal: `docs/skill.md`, `docs/row-count-limits.md`, `docs/RUSTDOC_HISTORY.md`.

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

# Formatting (rustfmt.toml uses nightly-only options — nightly is required,
# or local formatting will diverge from CI)
cargo +nightly fmt --all

# Linting (--all-targets also covers tests, benches and examples)
cargo clippy --workspace --all-targets -- -D warnings

# Architecture invariant — fail if rs-grid-core gains a WASM/web dependency
# (keeps the crate natively testable). Underlying check:
#   cargo tree -p rs-grid-core -e normal   # must list no wasm-bindgen/web-sys/js-sys
just check-arch                            # local (tools/check-arch.ps1); CI uses grep

# Benchmarks (HTML reports → target/criterion/)
just bench        # core + scene
just wasm-size    # bundle WASM + estimation gzip

# Inspect the rendered scene as JSON (no browser) — lets an agent verify
# primitive positions/colors/clips after a change. Needs the `serde` feature.
# Scenarios: basic | selection | pinned | scrolled
cargo run -p rs-grid-scene --features serde --bin scene-dump -- selection

# WASM build (e2e fixture — minimal Leptos app, no Tailwind)
cd e2e/fixture-leptos
trunk build

# Dev server (hot-reload)
just dev
# equivalent to: cd e2e/fixture-leptos && trunk serve
# → http://localhost:9079  (config dans e2e/fixture-leptos/Trunk.toml)
#
# The framework demos moved to standalone repos:
#   github.com/ruxelion/rs-grid-example-{leptos,dioxus,yew,js}

# Release preview (release-plz) — applies version bumps + per-crate CHANGELOG.md
# to the working tree for review. Inspect with `git diff`, then `git checkout .`.
# One-time: cargo install release-plz
release-plz update

# Publish to crates.io only the crates whose version is not yet published
# (dependency order, 30 s waits). Unchanged crates are skipped automatically.
# Requires: cargo login + owner rights on each crate.
just publish
```

### Justfile shortcuts

Préférer `just <recipe>` aux commandes cargo brutes — les recettes sont
synchronisées avec `.vscode/tasks.json`.

| Recette | Action |
|---|---|
| `just ci` | fmt + lint + check-arch + tests (gate pré-PR complet) |
| `just check-arch` | invariant archi : rs-grid-core sans dépendance WASM/web |
| `just test` | nextest, WASM crates exclus |
| `just coverage` | rapport HTML couverture (ouvre le navigateur) |
| `just coverage-lcov` | format lcov pour CI |
| `just bench` | tous les benchmarks (core + scene) |
| `just wasm-size` | taille du bundle WASM + estimation gzip |
| `just scene-dump <scenario>` | sérialise un SceneFrame en JSON (basic/selection/pinned/scrolled) pour inspection IA |
| `just gen-scene-fixtures` | régénère les fixtures de scène servies par le MCP (`mcp/scenes/`) |
| `just e2e` | trunk build + Playwright |
| `just e2e-update-snapshots` | régénérer les captures de référence Playwright |
| `just dev` | serveur de dev hot-reload pour la fixture e2e (`localhost:9079`) |
| `just mcp-build` | compiler le serveur MCP TypeScript → `dist/` |
| `just mcp-publish` | publier le serveur MCP sur npm |
| `just release-preview` | aperçu local release-plz (bumps + CHANGELOG par crate) |
| `just publish` | publier sur crates.io les crates dont la version n'est pas encore publiée (ordre de dépendances) |

## Code conventions

- **Edition**: Rust 2024
- **Max line width**: 80 characters (rustfmt.toml)
- **Imports**: grouped by `StdExternalCrate`, granularity `Crate`
- **Comments**: wrapped at 80 chars, formatted in doc-comments
- **Language**: English (US) everywhere in code files — comments, doc-strings,
  inline strings, task labels, error messages (Rust, TOML, Justfile, JSON,
  TypeScript…). This includes GitHub Actions workflow files (`.github/workflows/`):
  step `name:`, `description:`, inline comments, and all string values. Existing
  French text is legacy and must be converted to English when a file is edited.
- No `unwrap()` in production code — use `expect("reason")` or error propagation

## Invariants non-négociables

1. **Pas de wasm-bindgen dans `rs-grid-core`** — le crate doit rester testable en natif
2. **Indices de ligne en `u64`**, jamais `usize` (limite WASM32 à 4 Go)
3. **Hit-testing en O(log n)** via offsets précompilés — ne pas introduire de O(n)
4. **Toutes les mutations via `GridState::apply(GridCommand)`** — jamais directement

## Slash commands

| Commande | Action |
|---|---|
| `/test` | `cargo nextest run -p rs-grid-core` — après chaque changement core |
| `/e2e` | `trunk build` + Playwright — avant toute PR |
| `/publish` | Publication manuelle crates.io + tags per-crate (après merge de la PR release-plz) |

## Versioning (SemVer)

The workspace follows **Semantic Versioning** with **independent per-crate
versions** (each crate bumps on its own). Initial released version: `0.1.0`.

Version bumps and changelogs are automated by **release-plz** (config
`release-plz.toml`): on every push to `main` it opens/updates a *release PR* that
bumps the changed crates and writes `crates/*/CHANGELOG.md` from the
[Conventional Commits](https://www.conventionalcommits.org/). Merging that PR is
the version bump — do not edit `version` in `Cargo.toml` by hand. Publishing to
crates.io + tagging stays manual via `/publish` (see also `.github/workflows/
release-plz.yml`).

For every feature request, bug fix, or refactor, reason about the version impact
**before** proposing an implementation:

| Change type | Version bump | Examples |
|---|---|---|
| Bug fix, no API change | `0.x.Y+1` (patch) | wrong scroll offset, render glitch |
| New public API, backward-compatible | `0.X+1.0` (minor) | new column type, new callback prop |
| Breaking public API change | `0.X+1.0` (minor, pre-1.0 exception) | rename/remove/reorder public fn |
| Stable API commitment | `1.0.0` | deliberate stability milestone |
| Breaking post-1.0 | `X+1.0.0` (major) | only after 1.0 is released |

**Pre-1.0 rule**: while the version is `0.x`, breaking API changes bump the
minor (not major). Users of `0.x` crates accept this instability.

When the user asks for a change, always include in your reasoning:

1. **Classify**: is this a patch, minor, or major bump?
2. **Propose**: state the new version (e.g. "this is a minor bump → `0.2.0`").
3. **Remind** (when applicable): release-plz will open a release PR with the bump
   + changelog; after it merges, a crates.io publish and per-crate tags
   (`rs-grid-<crate>-vX.Y.Z`) are needed. Use `/publish` for the manual
   publish + tag checklist.

Do **not** bump version numbers in `Cargo.toml` automatically — release-plz owns
the bump via its release PR. Only classify and propose the impact in your
reasoning.

## MCP servers

| Server | Role | Setup |
|---|---|---|
| **GitHub** (hosted) | Read changelogs / releases of dependency repos before a bump | Local-only `.mcp.json` (gitignored), HTTP → `api.githubcopilot.com/mcp`, read-only fine-grained PAT in `GITHUB_MCP_PAT` |
| **rs-grid** (internal) | Exposes rs-grid docs (`search_rs_grid_docs`, `get_api_type`, `list_doc_pages`), **structured GridCommand variants** (`list_commands`, `get_command`) **and rendered scenes** (`list_scenes`, `get_scene` — serialized `SceneFrame` JSON so agents see the render without a browser) | `mcp/` (TypeScript), published to npm as `rs-grid-mcp` (`just mcp-build` / `just mcp-publish`). Scene fixtures: `just gen-scene-fixtures`. Docs source: local sibling `rs-grid-site/doc_build` if present, else GitHub (`RS_GRID_DOCS_SOURCE=local\|github` to force) |
| **Playwright** | Interactive visual checks during dev | See *End-to-end tests* below |

The **GitHub** server is a personal, local config (the PAT must not be committed
— `.mcp.json` and `.env` are gitignored). To register it:

```sh
claude mcp add --transport http github \
  https://api.githubcopilot.com/mcp/ \
  --header "Authorization: Bearer ${GITHUB_MCP_PAT}"
```

The PAT needs only **Contents: Read** + **Metadata: Read** (read-only). It is
unrelated to the internal `rs-grid-mcp` doc server above.

### How the internal `rs-grid-mcp` server is built

`mcp/` is a **hand-maintained TypeScript project**, not generated from Rust.
Source: `mcp/src/{index.ts,commands.ts,type-map.ts,search.ts,paths.ts}`.

- **Docs content is copied, not authored in `mcp/`.** `just mcp-build` runs
  `tsc && node scripts/copy-docs.mjs`, which copies pre-rendered markdown from
  the sibling repo `rs-grid-site/doc_build/` (or from GitHub if the sibling
  isn't present locally — `RS_GRID_DOCS_SOURCE=local|github` forces one or
  the other). **→ To update MCP doc content, edit `rs-grid-site`, then
  rebuild the MCP.** Never add doc prose directly under `mcp/`.
- **`list_commands` / `get_command`** parse `rs-grid/docs/skill.md` by regex
  at runtime (`mcp/src/commands.ts`) to enumerate `GridCommand` variants. As
  long as `docs/skill.md` documents a variant, it surfaces automatically —
  no MCP code change needed when a `GridCommand` is added/documented there.
- **`get_api_type`** resolves a type name (e.g. `ColumnDef`, `GridCommand`)
  to a doc page via a hand-maintained lookup table,
  `TYPE_TO_DOC_PATH` in `mcp/src/type-map.ts`. **This table does NOT
  auto-discover new types** — adding a new public Rust type that should be
  queryable (e.g. a new enum like `ValidationRule`) requires manually adding
  an entry here pointing at its `rs-grid-site` doc page.
- **`list_scenes` / `get_scene`** serve static JSON from `mcp/scenes/`,
  regenerated by `just gen-scene-fixtures` (wraps the `scene-dump` Rust
  binary) — checked in, not built on the fly. Only needs regenerating if a
  change adds new rendered primitives/scenarios.
- **No Rust→TypeScript codegen exists anywhere in this pipeline.**

**So, when a Rust feature adds new public API surface**, the checklist is:
1. Document it in `rs-grid-site` (source of truth for the public MCP docs).
2. Document it in `docs/skill.md` if it's a `GridCommand` variant (for
   `list_commands`/`get_command`) or general usage guidance.
3. Add an entry to `TYPE_TO_DOC_PATH` in `mcp/src/type-map.ts` if it's a new
   named type that should resolve via `get_api_type`.
4. `just mcp-build` (+ `just mcp-publish` to ship it) to pick up 1–3.

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

**Test structure**:

`e2e/tests/grid.spec.ts`:
- `smoke` — page loads, canvas visible, default values
- `controls` — row/column dropdowns
- `canvas interaction` — clicks, scroll, shift-click (viewport coordinates)
- `visual regression` — pixel-by-pixel screenshot comparison (2% tolerance)

`e2e/tests/editing.spec.ts`:
- `dblclick on Name cell (CellEditor::Text) opens text input`
- `dblclick on Role cell (editor=None) does not open any input` — regression guard for the None-editor fix
- `Escape closes the text editor`

`e2e/tests/validation.spec.ts`:
- `typing an empty value into a required cell shows the invalid style` /
  `typing a valid value clears the invalid style`
- `invalid value sets a native title attribute, valid value clears it`
- `Enter with an invalid value reverts and closes the editor (default Revert mode)`
- `on_validation_state_changed fires live, not just on commit`

`e2e/tests/progress.spec.ts`:
- `scrolling to the Completion column does not crash` — scrolls right to the
  `CellFormat::ProgressBar` column, asserts no page error
- `renders value-driven progress bars` — visual regression of the DaisyUI
  progress bars (per-value error/warning/success colours)

`e2e/tests/editable-predicate.spec.ts` — the "Notes" column (last base
column, locked on even row indices via `ColumnDef::editable_predicate`):
- `dblclick on an unlocked (odd row) Notes cell opens the text input` /
  `dblclick on a locked (even row) Notes cell does not open the text input`
- `hovering a locked cell shows the not-allowed cursor` /
  `hovering an unlocked cell shows the default cursor`

`e2e/tests/csp.spec.ts` — Content Security Policy, no violations on load.

**Canvas note**: the grid is rendered on `<canvas>`, not in the DOM.
Interaction tests use fixed pixel coordinates. If the layout changes, update
the coordinates in `grid.spec.ts` and `editing.spec.ts`.

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

- **Directed compaction** — on a long session, before the context auto-compacts,
  write your own checkpoint summary that explicitly preserves the
  *non-negotiable invariants* (no WASM in core; `u64` row indices; O(log n)
  hit-test; mutations only via `GridState::apply`; theme values round-trip as
  CSS vars) and the current task state. Auto-compaction tends to drop exactly
  these architectural constraints; restate them so they survive.
- After any code change in `rs-grid-core`, always run `/test` to verify tests
  pass.
- If a test fails, fix it before continuing.
- Any visual change or addition (color, size, animation) must be made
  configurable through the theme engine: field in `Theme`, default value in
  `light()`, `dark()`, and `dimmed()`, CSS variable in `css_theme.rs`.
- **When adding any new runnable command** (cargo, just, npm, trunk…), always
  add it in all three places in the same commit:
  1. `Justfile` — a named recipe wrapping the raw command
  2. `.vscode/tasks.json` — a task calling `just <recipe>`
  3. `AGENTS.md` → *Common commands* — the raw command with a short comment

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
