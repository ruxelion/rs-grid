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

### Validation feedback on the text editor

The `<input>` created by `show_text_editor` (`canvas/edit.rs`) wires three
listeners:

| Event | Behaviour |
|---|---|
| `input` | Dispatches `GridCommand::ValidateEdit` on every keystroke (no commit), then restyles the input via `apply_edit_validity_style` |
| `keydown` (`Enter`) / `blur` | Dispatches `GridCommand::CommitEdit`, then calls `keep_or_close`: if `GridState.edit` is still `Some` (`InvalidEditMode::Block` kept it open), restyle as invalid and refocus; otherwise tear the overlay down as before |
| `keydown` (`Escape`) | Dispatches `GridCommand::CancelEdit` unconditionally, always tears the overlay down |

While a cell is being edited, its DOM `<input>` fully occludes the canvas
underneath (opaque `--rs-grid-editor-bg`), so the invalid-value indicator for
the *in-progress edit* is applied directly to that `<input>`'s own
border/background instead of a `ScenePrimitive` — see *CSS theme* below. This
is separate from the canvas-rendered `invalid_cell_border` overlay
(`rs-grid-scene`'s `emit_cell`), which flags any cell whose *current* value
fails validation regardless of whether it's being edited — see
`rs-grid-scene/AGENTS.md`'s "At-rest invalid-value border". The two never
overlap: the DOM input hides the canvas cell underneath while editing.

### At-rest validation tooltip (hover)

A cell can be invalid without ever being edited — data loaded already-bad
from the source (see `rs-grid-scene/AGENTS.md`'s "At-rest invalid-value
border"). `refresh_validation_tooltip(vx, vy)` (`canvas/tooltip.rs`), called
from `attach_mousemove`'s hover branch right after `refresh_hover_cursor`,
surfaces this on hover via a single reused DOM `<div>`
(`tooltip_el: RefCell<Option<HtmlElement>>`, mirroring `edit_input`) — **not**
one element per invalid cell, so cost stays O(1) regardless of how many
cells in the grid are invalid. It is:

- Positioned via the existing `cell_client_rect(row, col_key)` (`canvas/
  edit.rs`) — no new geometry.
- Given `pointer-events: none` (the mouse stays over the `<canvas>`, never
  over this element) and `data-tip="<message>"` — the attribute daisyUI
  reads via `content: attr(data-tip)`.
- Change-guarded on `(row, col)` (`tooltip_cell`) so DOM writes only happen
  when the hovered cell actually changes, not on every `mousemove`.
- Hidden (not removed) on wheel/scroll (`attach_wheel`) since its position
  is computed once at hover time in fixed client coordinates and would
  otherwise desync from the cell it targets; it reappears on the next
  `mousemove` if still hovering an invalid cell.

rs-grid renders **no visual of its own** for this tooltip — the wrapper's
`class` attribute is fully caller-controlled via
`set_validation_tooltip_class(Option<String>)`; with no class set, the
element exists but is invisible. This is the same "rs-grid does not pick
where/how validation feedback renders" stance as `cell_client_rect`'s own
doc comment. To reproduce daisyUI's tooltip:
```rust
canvas.set_validation_tooltip_class(Some(
    "tooltip tooltip-open tooltip-error".into(),
));
```
`tooltip-open` (or an equivalent always-open modifier) is required in the
class — daisyUI's tooltip CSS normally shows on native `:hover`, which this
element can never receive (`pointer-events: none`); rs-grid owns
show/hide entirely via its own `display` toggle, not CSS `:hover`.
`GridCanvas::cell_validation_error(row, col_key) -> Option<String>` is also
public standalone (mirrors `validation_error()`, but for at-rest values) —
useful to build a fully custom indicator without the built-in hover
mechanism.

### Consuming validation state generically

rs-grid does not impose a validation-error widget for the *in-progress edit*
case (no built-in styled tooltip component, unlike e.g. AG Grid). Instead it
exposes the raw state so an integrator can build any UI with their own
framework/CSS:

- `GridCanvas::validation_error() -> Option<(u64, String, String)>` — reads
  the live state on demand (`GridState.edit.validation_error`).
- `set_on_validation_state_changed` — push notification on every keystroke
  (see *Public callbacks* above), for reactive UIs.
- `set_native_validation_tooltip(bool)` (default `true`) — toggles the
  zero-config native `title` attribute set by `apply_edit_validity_style`
  in `canvas/edit.rs`. Disable it when building a custom UI to avoid a
  competing browser tooltip.

The three framework wrappers (`rs-grid-leptos`/`dioxus`/`yew`) forward
`on_validation_state_changed` as a plain callback prop, mirroring the
existing `on_validation_error` plumbing exactly — no wrapper-managed
reactive signal exists for this or any other `GridCanvas` state today.

## Locked-cell cursor feedback

The mousemove hover handler (`canvas/events.rs`) sets `cursor: not-allowed`
when hovering a cell that resolves to non-editable via
`ColumnDef::is_cell_editable` — the single source of truth combining the
grid-wide `GridModel.editable` toggle, the column's static `editable` flag,
and a false-resolving `editable_predicate` (all in `rs-grid-core`) — using
the `hit_locked_cell(vx, vy)` helper in `canvas/hittest.rs` (delegates to
`GridState::hit_test` for an O(log n) row+col lookup). Because
`is_cell_editable` folds in `GridModel.editable`, a globally read-only grid
shows `not-allowed` on every cell, consistent with `emit_cell`'s locked
overlay (`rs-grid-scene`). Paired with the `locked_cell_bg`/`locked_cell_text`
`Theme` fields (see *CSS theme* below) for the themed visual.

## Paste-flash feedback

`GridCanvas::flash_cells(&[CellCoord])` (`canvas/dispatch.rs`) arms a
400 ms fading yellow overlay on exactly the given cells — **not** a
selection rectangle. Both paste call sites (Ctrl+V in `canvas/keyboard.rs`,
context-menu Paste in `canvas/context_menu.rs`) call
`dispatch_with_output(GridCommand::PasteAt { .. })` and pass the
`CommandOutput::PasteApplied { cells }` result straight to `flash_cells`.
This matters because `PasteAt` always expands the *selection* to the full
target rectangle regardless of skips, but `cells` only contains what was
actually written — cells skipped for being locked or failing validation
must not get a "success" flash. `FlashState` (`canvas/mod.rs`) stores the
cell set; `compute_flash_hint` (`canvas/animation.rs`) clones it into
`FlashHint` each frame, and `rs-grid-scene`'s `emit_cell` checks membership
directly instead of reusing the selection highlight.

## Public callbacks

Callbacks fired during `dispatch()` after `GridState::apply()` returns:

| Callback | Triggers |
|---|---|
| `set_on_change` | `PasteAt`, `CommitEdit` (cell data mutations) — **not** fired when a `CommitEdit` is rejected by validation |
| `set_on_columns_changed` | `CommitColumnResize`, `MoveColumn`, `AutoFitColumn`, `AutoFitAllColumns`, `SetPinnedColumnCount` (layout mutations — **not** sort/filter) |
| `set_on_validation_error` | A `CommitEdit` was rejected by `ColumnDef.rules`/`validator` (`CommandOutput::ValidationError`) — fires for both `InvalidEditMode::Revert` and `::Block` |
| `set_on_validation_state_changed` | `StartEdit`, `ValidateEdit`, `CommitEdit`, `CancelEdit` — fires with the fresh `validation_error()` value on *every keystroke*, not just rejected commits |
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
`--rs-grid-progress-radius`. The locked-cell overlay (see *Locked-cell
cursor feedback* above) adds `--rs-grid-locked-cell-bg` and
`--rs-grid-locked-cell-text`. The at-rest invalid-cell border (see
*Validation feedback on the text editor* above) adds
`--rs-grid-invalid-cell-border` and `--rs-grid-invalid-cell-border-width`.

### Inline editor overlay variables

`apply_edit_style`/`apply_edit_validity_style` (`canvas/edit.rs`) read a
**separate** set of `--rs-grid-editor-*` variables directly from the DOM
(via `css_theme::get_var`), with hard-coded Rust fallbacks — this is *not*
part of the `Theme`/`css_vars.rs` round-trip system above, since these
variables style a DOM overlay element, not a canvas primitive:

| Variable | Fallback | Applies to |
|---|---|---|
| `--rs-grid-editor-border` | `#2563eb` | Border colour, normal state |
| `--rs-grid-editor-border-width` | `2px` | Border width (both states) |
| `--rs-grid-editor-border-radius` | `0` | Border radius |
| `--rs-grid-editor-bg` | `#ffffff` | Background, normal state |
| `--rs-grid-editor-color` | `#000000` | Text colour |
| `--rs-grid-editor-padding` | `0 4px` | Padding |
| `--rs-grid-editor-font-size` | `inherit` | Font size |
| `--rs-grid-editor-shadow` | `none` | Box shadow |
| `--rs-grid-editor-border-invalid` | `#dc2626` | Border colour while `EditCell.validation_error` is `Some` |
| `--rs-grid-editor-bg-invalid` | `#fef2f2` | Background while `EditCell.validation_error` is `Some` |

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
