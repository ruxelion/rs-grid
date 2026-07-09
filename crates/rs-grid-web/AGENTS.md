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
| `Some(CellEditor::Text)` | `<input>` or `<textarea>` positioned over the cell (see *Single-line vs. multiline* below) |
| `Some(CellEditor::Select { .. })` | Custom `<select>` dropdown with optional icons |
| `None` | `CancelEdit` is dispatched, no DOM overlay is created |

`None` means "this column is not user-editable via the overlay". Callers who
want plain-text editing must set `column.editor = Some(CellEditor::Text)`
explicitly — the grid does not fall back to a text input automatically.

### Single-line `<input>` vs. multiline `<textarea>`

`show_text_editor` (`canvas/edit.rs`) is a dispatcher, not an editor itself.
It decides once, from the value the cell already holds when the edit
opens, which of two sibling functions to call:

- **`show_single_line_editor`** — a classic `<input type="text">`, used
  when the value has no `\n`/`\r` and its measured width (see below) fits
  within `MAX_EDITOR_WIDTH` (520px) without wrapping. Behaves exactly like
  the editor did before multiline support existed: `Enter` (no modifier)
  always commits, height stays the cell's own height, vertical centering
  is the input's native rendering.
- **`show_multiline_editor`** — a `<textarea>`, used when the value
  already contains a line break or is too long to fit on one line. Supports
  wrapping, `Alt+Enter` for manual line breaks, and dynamic height (see
  *sizing* below). `Shift+Enter` is **not** a newline shortcut — it commits
  like plain `Enter`.

The initial choice is made once, from the value the cell already holds
when the edit opens — a value that grows past the single-line cap mid-edit
(by typing) keeps scrolling horizontally in its `<input>` rather than
morphing into a `<textarea>` under the user's cursor. The one exception is
an explicit `Alt+Enter` keypress: `show_single_line_editor`'s keydown
handler special-cases this (before its plain-`Enter`-commits arm) to splice
exactly one `\n` into the current value at the cursor (`selectionStart`,
clamped to the nearest UTF-8 char boundary — `selectionStart` is a UTF-16
code-unit offset, so this guards non-ASCII text against a slice panic),
then calls `remove_edit_input()` + `show_multiline_editor()` with that
spliced value and the same `EditorGeom`, and finally restores the cursor
right after the inserted `\n` on the new `<textarea>` (`select()` inside
`show_multiline_editor` would otherwise leave the whole value selected).
No new `StartEdit` is dispatched — it's the same edit session, only the
DOM overlay changes. `TEXT_PADDING` (24px) and `MAX_EDITOR_WIDTH` (520px)
are module-level consts in `canvas/edit.rs`, shared by both paths.

### Content-based sizing of the text editor

Neither editor opens smaller than the cell's own size, and both grow to
fit the actual value:

- **Width** (both editors) — `measure_text_width` (`canvas/edit.rs`)
  renders the initial text on an offscreen `<canvas>` 2D context with the
  same font string as `rs-grid-render-canvas`'s `draw_text` (`400 {size}px
  system-ui, sans-serif`, `theme.font_size`). The box width becomes
  `cell_width.max(measured_width + TEXT_PADDING)`, capped at
  `MAX_EDITOR_WIDTH` for the `<textarea>` path (the `<input>` path never
  needs the cap — that's precisely the condition that keeps it on the
  `<input>` path in the first place).
- **Height** (`<textarea>` only) — `resize_multiline_editor` re-measures
  the *current* value (not just the initial one) and re-applies the box
  height on every `input` event — typing, pasting, or an `Alt+Enter`
  newline insertion (which re-dispatches a synthetic `input` event
  precisely so this runs) — so the box grows as lines are added and
  shrinks back down as they're removed, live, not just once at open time.
  It calls `measure_wrapped_height`, which lays the text out in a
  throwaway offscreen `<textarea>` at the chosen width (`white-space:
  pre-wrap`, `word-break: break-word` — matching the real editor's CSS)
  and reads its `scrollHeight`, since the browser's own line-wrapping is
  the only reliable way to get this without reimplementing text shaping.
  The offscreen element sets `rows="1"` — a bare `<textarea>` defaults to
  `rows="2"`, which would floor `scrollHeight` at two lines even for
  single-line text — and its horizontal padding matches `EDITOR_H_PADDING`
  so wrap points line up with what's actually rendered. The overlay
  height becomes `cell_height.max(wrapped_height + 2.0 *
  EDITOR_V_PADDING)` — never below the cell's own height, even after
  deleting back down to one line — capped at 60% of the window's inner
  height (past which it scrolls internally, `overflow-y: auto`, rather
  than growing further). `show_multiline_editor`'s own setup calls the
  same method once, so there's a single code path for both the initial
  size and every live resize after.

Both editors override `apply_edit_style`'s shorthand `padding` with the
same explicit `padding-top`/`padding-bottom`/`padding-left`/`padding-right`
longhands, deliberately wider than the themed `--rs-grid-editor-padding`
default (4px) — so a value that flips between `<input>` and `<textarea>`
across edits (see *dispatcher* above) doesn't visibly shift position
within the box:

- **`show_single_line_editor`** — fixed at `EDITOR_V_PADDING`/
  `EDITOR_H_PADDING` (8px each). The `<input>`'s native rendering
  centers its one line regardless of the exact top/bottom value.
- **`show_multiline_editor`** — horizontal is the same fixed
  `EDITOR_H_PADDING`; vertical is `(box_height - wrapped_height) / 2`
  (real centering, since a `<textarea>` always top-aligns otherwise),
  floored at `EDITOR_V_PADDING` so it never collapses to ~0 when the box
  height already matches the content almost exactly (which reads as
  text jammed against the border).

`apply_edit_style` then clamps the `left` position so the box never runs
past the window's right edge (shifts left, does not shrink), for either
editor. This sizing does not touch the select dropdown, which has its own
independent `--rs-grid-dropdown-min-width` sizing in `show_select_editor`.

### Multiline editing

`show_multiline_editor`'s `<textarea>` accepts newlines: `Alt+Enter`
(Excel's convention) inserts exactly one, done by explicitly splicing
`\n` into the value at the cursor and manually re-dispatching an `input`
event (`set_value()` doesn't fire one on its own, and the existing
`input` listener needs to run to keep live validation current) — not by
relying on the browser's own default Enter-in-a-textarea behavior, so the
count is deterministic regardless of what a given browser does with the
Alt modifier held. Plain `Enter` (with or without Shift) commits;
`Shift+Enter` is deliberately **not** treated as a newline shortcut.
`white-space: pre-wrap` + `word-break: break-word` preserve manually-typed
line breaks and wrap long words; `resize: none` and `overflow-y: auto`
keep the box a fixed-then-clamped size (see *sizing* above) with a
scrollbar for content past the 60%-viewport-height cap.

`show_single_line_editor`'s `<input>` can never itself hold a newline —
but `Alt+Enter` there doesn't just do nothing: see *Single-line vs.
multiline* above for how it switches to `show_multiline_editor` mid-edit
instead, inserting the same single `\n`.

### Validation feedback on the text editor

Both `show_single_line_editor`'s `<input>` and `show_multiline_editor`'s
`<textarea>` (`canvas/edit.rs`) wire the same three listeners:

| Event | Behaviour |
|---|---|
| `input` | Dispatches `GridCommand::ValidateEdit` on every keystroke (no commit), then restyles the input via `apply_edit_validity_style` |
| `keydown` (`Enter` with no Alt/Shift) / `blur` | Dispatches `GridCommand::CommitEdit`, then calls `keep_or_close`: if `GridState.edit` is still `Some` (`InvalidEditMode::Block` kept it open), restyle as invalid and refocus; otherwise tear the overlay down as before |
| `keydown` (`Escape`) | Dispatches `GridCommand::CancelEdit` unconditionally, always tears the overlay down |

While a cell is being edited, its DOM overlay fully occludes the canvas
underneath (opaque `--rs-grid-editor-bg`), so the invalid-value indicator
for the *in-progress edit* is applied directly to that element's own
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

## Row-selection checkbox column

`GridModel.show_checkbox_column` (opt-in, default `false`) inserts a
column at `GridModel.checkbox_column_width` (default
`GridModel::CHECKBOX_COLUMN_WIDTH`, runtime-configurable via
`GridCommand::SetCheckboxColumnWidth`/`GridCanvas::set_checkbox_column_width`),
as the first slot of the scrollable (unpinned) region — unlike the
row-number gutter, it is
**not** fixed on screen: it scrolls away with `scroll_x` like a real
column, and sits after any pinned real columns. Still outside
`ColumnOffsets`/`hit_column` (rs-grid-core's `hit_test`/
`hit_test_col_header` reserve its width as a scroll-shift term instead).
`attach_mousedown` (`canvas/events.rs`) hit-tests it in a dedicated
cascade block, right before the row-number gutter check:
`hit_test_checkbox_header` (→ `GridCommand::ToggleAllFilteredChecked`)
and `hit_test_checkbox_row` (→ `GridCommand::ToggleRowChecked`, or
`ExtendRowChecked` when `evt.shift_key()` is set — sets every row in
`[anchor, this row]` to the direction fixed by the last
`ToggleRowChecked` (the anchor click's own resulting state),
mirroring `ExtendRowSelection`'s anchor/focus range. A further
shift+click within the same gesture also reconciles against the
*previous* `ExtendRowChecked` call: rows it touched but the new range
no longer covers revert to the opposite state — checking rows 1-10,
then, still holding shift, clicking row 9 gives row 10 back its
earlier state, matching drag-selection behaviour instead of only ever
growing the checked set) — both `pub(super)` wrappers in
`canvas/hittest.rs` delegating to the
`GridState` methods of the same name (rs-grid-core), which now factor
in `scroll_x` since the
checkbox's on-screen position is scroll-dependent. A row-checkbox
click is a discrete toggle, not a drag gesture — unlike the
row-number gutter's `ActiveDrag::Row`.

`GridCanvas::checked_row_indices()` returns **physical** row ids (not
logical/display order, unlike `selected_row_indices()`) — checkbox state
is tracked by row identity so it survives sort/filter. Pair with
`set_on_checked_rows_changed` (see *Public callbacks* below) and
`checkbox_header_state()` (tri-state: `Checked`/`Unchecked`/
`Indeterminate`) to drive a bulk-action toolbar. Toggle the column itself
at runtime with `set_show_checkbox_column(bool)`.

## Server-side page fetcher (`FetchConfig`)

`canvas/fetcher.rs` implements the automatic fetch coordinator behind
`GridCanvas::enable_async_fetch(page_cache, FetchConfig { .. })` — detects
needed pages for the current viewport (`PageCacheDataSource::needed_pages`),
fetches them via `build_url`/`parse_response`, and inserts the response.
Full usage guide: `rs-grid-site`'s `data/page-cache.mdx` and
`howto/server-pagination.mdx` (see root `AGENTS.md`'s doc-sync rule — this
crate's own doc here only covers what a *contributor to this crate* needs,
not the end-user how-to).

On a successful response it dispatches **two** commands, not one:
`GridCommand::SetTotalRowCount(resp.total_rows)` then
`GridCommand::NotifyPageLoaded` — both after `cache_clone.set_total_rows(..)`
already updated the `PageCacheDataSource`'s own count. `SetTotalRowCount`
is what keeps `GridModel.row_number_width` in sync with the server-reported
total when the model was built from a placeholder count (see
`rs-grid-core/AGENTS.md`'s "Row-number gutter width" section) — dropping it
would silently regress the gutter back to whatever digit-count the
placeholder implied, even though `PageCacheDataSource::set_total_rows` on
its own already gives the *data* the right count.

## Success-flash feedback (paste, clear)

`GridCanvas::flash_cells(&[CellCoord])` (`canvas/dispatch.rs`) arms a
400 ms fading yellow overlay on exactly the given cells — **not** a
selection rectangle. Two call sites use it, both via `dispatch_with_output`
so they can read the `CommandOutput`:

- Paste — Ctrl+V in `canvas/keyboard.rs` and context-menu Paste in
  `canvas/context_menu.rs` dispatch `GridCommand::PasteAt { .. }` and pass
  `CommandOutput::PasteApplied { cells }` to `flash_cells`.
- Clear — Delete/Backspace in `canvas/keyboard.rs` dispatch
  `GridCommand::ClearCells` and pass `CommandOutput::CellsCleared { cells }`
  to `flash_cells`.

In both cases this matters because the *selection* still covers the full
target/cleared rectangle regardless of skips, but `cells` only contains
what was actually written — cells skipped for being locked or failing
validation must not get a "success" flash. `FlashState` (`canvas/mod.rs`)
stores the cell set; `compute_flash_hint` (`canvas/animation.rs`) clones it
into `FlashHint` each frame, and `rs-grid-scene`'s `emit_cell` checks
membership directly instead of reusing the selection highlight.

## Public callbacks

Callbacks fired during `dispatch()` after `GridState::apply()` returns:

| Callback | Triggers |
|---|---|
| `set_on_change` | `PasteAt`, `CommitEdit`, `ClearCells` (cell data mutations) — **not** fired when a `CommitEdit` is rejected by validation |
| `set_on_columns_changed` | `CommitColumnResize`, `MoveColumn`, `AutoFitColumn`, `AutoFitAllColumns`, `SetPinnedColumnCount` (layout mutations — **not** sort/filter) |
| `set_on_validation_error` | A `CommitEdit` was rejected by `ColumnDef.rules`/`validator` (`CommandOutput::ValidationError`) — fires for both `InvalidEditMode::Revert` and `::Block` |
| `set_on_validation_state_changed` | `StartEdit`, `ValidateEdit`, `CommitEdit`, `CancelEdit` — fires with the fresh `validation_error()` value on *every keystroke*, not just rejected commits |
| `set_on_cell_button_click` | User clicked a `ColumnDef.cell_buttons[i]` |
| `set_on_checked_rows_changed` | `ToggleRowChecked`, `ExtendRowChecked`, `ToggleAllFilteredChecked` (row-selection checkbox column) |

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
`--rs-grid-locked-cell-text`. The at-rest invalid-cell background/border
(see *Validation feedback on the text editor* above) adds
`--rs-grid-invalid-cell-bg`, `--rs-grid-invalid-cell-border`, and
`--rs-grid-invalid-cell-border-width`.

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
| `--rs-grid-editor-padding` | `0 4px` | Padding shorthand — immediately overridden by both `show_single_line_editor` and `show_multiline_editor`'s explicit `padding-*` longhands (see *Single-line vs. multiline* below), so this var has no visible effect on the text editors today; it still applies as-is to the select dropdown. |
| `--rs-grid-editor-font-size` | `theme.font_size` (not `inherit`) | Font size. Must stay in sync with `theme.font_size` — `measure_text_width`/`measure_wrapped_height` always measure at that value, so overriding this var to something else desyncs the multiline `<textarea>`'s computed width/height from what's actually rendered (manifests as unwanted scrolling or excess whitespace). |
| `--rs-grid-editor-shadow` | `none` | Box shadow |
| *(none — fixed)* | `system-ui, sans-serif` | Font family. Not configurable via CSS var (unlike the rest of this table) — hard-coded in `apply_edit_style` to match `measure_text_width`/`measure_wrapped_height` and the canvas renderer's own `draw_text`. Was `inherit` before multiline support; changing it back would desync the same way an overridden font-size does. |
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
