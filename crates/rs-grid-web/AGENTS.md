# rs-grid-web

Browser integration. Manages the full lifecycle of a grid instance in the DOM:
mouse/keyboard events, rAF loop, resize, DPR, CSS theme, localisation.

## Modules

| Module | Role |
|---|---|
| `canvas` | `GridCanvas`: mounts the grid on an `HtmlCanvasElement`, manages rAF and events |
| `css_theme` | `theme_from_css_vars()`: reads CSS variables to build a `Theme` |
| `locale` | `Locale`: UI string translations (15 built-in languages, TOML-based) |
| `storage` | `get_item` / `set_item` / `remove_item`: graceful `localStorage` helpers for persisting small demo state (e.g. a column layout). No-op when storage is unavailable. Pair with `example_common::layout::LayoutSnapshot`. |

## Responsibilities of `GridCanvas`

- Resize via `ResizeObserver` (viewport update)
- `requestAnimationFrame` loop: `SceneBuilder` → `SceneFrame` → `CanvasRenderer`
- Event handling: `mousemove`, `mousedown`, `mouseup`, `wheel`, `keydown`,
  `copy`, `paste`
- Canvas DPR adjustment for HiDPI screens
- Auto-scroll during selection drag

## Critical invariants

- `GridCanvas::mount()` is the only public entry point — one canvas = one instance.
- `mount()` calls `console_error_panic_hook::set_once()` (idempotent across
  mounts) so a boundary panic surfaces a readable message + stack in the
  browser console instead of `RuntimeError: unreachable`. Embedders that
  install their own panic hook can disregard it.
- Events are converted to `GridCommand` before being applied to `GridState`.
  **Do not manipulate `GridState` directly from event handlers.**
- DPR is read once at mount and on each resize. Do not re-read it every frame.
- `theme_from_css_vars()` reads the DOM — call only at mount, not every frame.

## Editing contract (`show_edit_input`)

A `dblclick` on a cell dispatches `StartEdit` then calls `show_edit_input()`.
The DOM overlay opened depends on `ColumnDef.editor`:

| `column.editor` value | Result |
|---|---|
| `Some(CellEditor::Text)` | `<input type="text">` positioned over the cell |
| `Some(CellEditor::Select { .. })` | Custom `<select>` dropdown with optional icons |
| `None` | `CancelEdit` is dispatched, no DOM overlay is created |

`None` means "this column is not user-editable via the overlay". Callers who
want plain-text editing must set `column.editor = Some(CellEditor::Text)`
explicitly — the grid does not fall back to a text input automatically.

## Public callbacks

Callbacks fired during `dispatch()` after `GridState::apply()` returns:

| Callback | Triggers |
|---|---|
| `set_on_change` | `PasteAt`, `CommitEdit` (cell data mutations) |
| `set_on_columns_changed` | `CommitColumnResize`, `MoveColumn`, `AutoFitColumn`, `AutoFitAllColumns`, `SetPinnedColumnCount` (layout mutations — **not** sort/filter) |
| `set_on_validation_error` | A `ColumnDef.validator` returned `Err` |
| `set_on_cell_button_click` | User clicked a `ColumnDef.cell_buttons[i]` |

**Re-entrancy**: callbacks fire *after* the dispatch path has released its
borrow on `state` — `apply()` returns its output by value, and each callback
`Rc` is cloned out of its own cell before invocation. So a callback may both
read grid state (e.g. `selected_row_indices()`, `cell_at_logical()`) **and**
synchronously dispatch another `GridCommand` without panicking. (Guard against
unbounded recursion if a dispatch re-triggers the same callback.)

Layout getters callable from `on_columns_changed`:
`column_widths()`, `column_order()`, `pinned_count()`.

## CSS theme

CSS variables are prefixed `--rs-grid-*`. `light.css`, `dark.css`, and
`dimmed.css` in `examples/example-common/themes/` are **auto-generated**
— do not edit them directly. The variable ↔ `Theme` field mapping (both
directions) is the single source of truth in `rs-grid-scene/src/css_vars.rs`;
`css_theme.rs` here is only a thin DOM wrapper around its reader. The
progress-bar cell renderer adds `--rs-grid-progress-track`,
`--rs-grid-progress-fill`, `--rs-grid-progress-height`, and
`--rs-grid-progress-radius`.

### Adding a CSS variable to an existing theme

1. Add the field in `Theme` (`rs-grid-scene/src/theme.rs`) with a value
   in every constructor: `light()`, `dark()`, `dimmed()`
2. Wire the variable **both ways** in `rs-grid-scene/src/css_vars.rs`:
   `theme_to_css_vars` (writer) **and** `theme_from_css_vars_with` (reader).
   The `round_trips_every_field` test fails if you forget either side.
3. `cargo run -p rs-grid-scene --bin generate-theme`

### Adding a new theme (e.g. `solarized`)

1. **`theme.rs`** — add `Theme::solarized() -> Self` with all fields
2. **`generate_theme.rs`** — add `CTX_SOLARIZED` + call
   `render_overlay("solarized", &light_vars, &solarized_vars, CTX_SOLARIZED)`
   and write the output file
3. **`solarized-shell.css`** — create in `example-common/themes/` with
   `:root.solarized` overrides for `.app-*` and `body`
4. **3× `index.html`** — add links for `solarized.css` and
   `solarized-shell.css` after the existing theme links
5. **3× `src/lib.rs`** (Leptos, Dioxus, Yew) — add
   `<option value="solarized">Solarized</option>` in the theme select
6. `cargo run -p rs-grid-scene --bin generate-theme`
7. `cargo check --workspace`
