# rs-grid-core

Headless grid logic crate. **Zero WASM dependency, zero web dependency.**
It must remain testable with standard native `cargo test`.

## Modules

| Module | Role |
|---|---|
| `model` | `GridModel`: columns + data source |
| `state` | `GridState`: central structure combining model + viewport + selection |
| `viewport` | `ViewportState`: scroll_x, scroll_y, visible dimensions, row virtualisation |
| `selection` | `SelectionState`: anchor/focus, TSV copy, TSV paste |
| `hit_test` | O(log n) hit-testing on cells, row headers, and column headers |
| `commands` | `GridCommand` (enum) + `CommandOutput` — all mutations go through here |
| `datasource` | `DataSource` trait for data abstraction |
| `column` | Column definitions (`ColumnDef`) |
| `row` | Row metadata |
| `scrollbar` | Scrollbar state (geometry, dragging) |
| `validation` | `ValidationRule` (declarative per-column rules) and `InvalidEditMode` (revert vs. block policy) |

## Critical invariants

- **No `wasm-bindgen` here.** If you need WASM, it belongs in `rs-grid-web`.
- Row indices are **`u64`** (not `usize`) to support >4B rows on WASM32.
- `GridState` mutations go **exclusively** through `GridState::apply(GridCommand)`.
- Hit-testing must remain O(log n) — column offsets are precomputed.

## Behaviour flags & cell buttons

- `GridModel.editable: bool` (default `true`) — global edit toggle.
  Per-column `ColumnDef.editable` can opt individual columns out. See
  *Per-cell editability* below for how this combines with per-column and
  per-cell overrides. Set at build time via `GridModelBuilder::editable(bool)`
  (symmetric with `selectable()` and `column_reorderable()`); toggle at
  runtime via `GridCommand::SetEditable(bool)`.
- `GridModel.selectable: bool` (default `true`) — when `false`,
  selection commands are silently ignored. Toggle at runtime via
  `GridCommand::SetEditable(bool)` / `GridCommand::SetSelectable(bool)`.
- `GridModel.column_reorderable: bool` (default `true`) — when `false`,
  header drag-to-reorder is suppressed in `rs-grid-web`. Programmatic
  `GridCommand::MoveColumn` is unaffected. Toggle via
  `GridCommand::SetColumnReorderable(bool)`.
- `ColumnDef::with_cell_buttons(Vec<ButtonDef>)` — declares interactive
  buttons rendered inside each cell of that column. Clicks bubble up
  through `rs-grid-web` as a callback (`on_cell_button_click` in the
  framework wrappers) carrying `(row, col_key, button_id)`.

## Validation (`ValidationRule`, `InvalidEditMode`)

- `ColumnDef.rules: Vec<ValidationRule>` — declarative checks run in order
  (first failure wins) before every `CommitEdit`. Built-ins: `Required`,
  `MinLength`, `MaxLength`, `Range`, `OneOf` (allowed-value list),
  `Custom(CellValidator)` (arbitrary closure). Sugar builders on
  `ColumnDef`: `.required()`, `.with_min_length(n)`, `.with_max_length(n)`,
  `.with_range(min, max)`, `.with_allowed_values(values)`, `.with_rules(vec)`.
  `ColumnDef.validator: Option<CellValidator>` (legacy, pre-dates `rules`)
  is still checked afterwards for backward compatibility —
  `ColumnDef::validate_value(&str)` runs both in order.
- **Validation is enforced inside `GridState::apply(CommitEdit)`**
  (`state/cmd_edit.rs`), not just at the `rs-grid-web` dispatch layer —
  this guarantees the invariant "mutations go exclusively through `apply`"
  actually blocks invalid data, for every consumer (native tests, any
  future renderer, framework wrappers).
- `GridModel.invalid_edit_mode: InvalidEditMode` (default `Revert`) — set at
  build time via `GridModelBuilder::invalid_edit_mode(...)`, toggled at
  runtime via `GridCommand::SetInvalidEditMode`. On a failing `CommitEdit`:
  `Revert` drops the edit session and reverts the cell (today's behaviour);
  `Block` keeps `GridState.edit` active with `EditCell.validation_error`
  set, so the caller can keep the editor open. Both return
  `CommandOutput::ValidationError { row, col_key, message }`.
- `GridCommand::ValidateEdit { value }` re-checks the in-progress edit's
  pending value **without committing**, updating
  `EditCell.validation_error` for live (per-keystroke) feedback. No-op
  without an active edit; produces no undo entry.
- `GridCommand::PasteAt` (`state/cmd_clipboard.rs`) also calls
  `ColumnDef::validate_value` per target cell and silently skips writing
  ones that fail (`continue`, not `break` — one invalid cell in a tiled
  paste doesn't imply its neighbours are invalid too), leaving the rest
  of the paste to apply normally — same silent-skip precedent as the
  `is_cell_editable` check in the same loop. `PasteAt` returns
  `CommandOutput::PasteApplied { cells }` — the coordinates actually
  written, a subset of the target rectangle. `rs-grid-web` uses this
  (not the selection, which still covers the full target area) to scope
  the paste-flash animation to cells that were genuinely written, so a
  skipped cell doesn't get a misleading "success" flash.
- `GridCommand::CutSelection` clears cells by writing an empty string —
  still a write, so it validates `validate_value("")` per cell exactly
  like `PasteAt` validates its pasted value, and skips (`continue`) any
  cell a rule like `.required()` would reject empty. The copy side
  (`to_tsv`, placed on the clipboard) is unaffected — it always copies
  the full original values regardless of what the clear side skips.
- `GridCommand::ClearCells` (Delete/Backspace) shares `CutSelection`'s
  clearing logic via a private `clear_cell_range` helper
  (`state/cmd_clipboard.rs`) — same `is_cell_editable` +
  `validate_value("")` skip, same full-column-selection guard (a header
  click carries positional intent, not "clear this entire column of
  potentially billions of rows"). It never touches the clipboard, unlike
  `CutSelection`. Returns `CommandOutput::CellsCleared { cells }` — the
  coordinates actually cleared, a subset of the selection — when at
  least one cell was written; `rs-grid-web` passes this to
  `flash_cells` (same success-flash mechanism as `PasteApplied`) so a
  skipped cell doesn't get a misleading "success" flash.
- Validation is also evaluated **at rest**, not just during an edit
  session: `rs-grid-scene`'s `emit_cell` calls `validate_value` against
  every rendered cell's current value and draws a themed border
  (`Theme::invalid_cell_border`) when it fails — so a cell that was
  already invalid when loaded from the data source is flagged without
  requiring the user to click into it first. See
  `rs-grid-scene/AGENTS.md`.

## Per-cell editability (`EditablePredicate`)

- `ColumnDef.editable: bool` locks an entire column statically. For
  per-cell (row+col) locking — AG Grid's `colDef.editable` callback
  equivalent — attach a dynamic predicate via
  `ColumnDef::editable_when(f: impl Fn(u64, &GridModel) -> bool)`, stored
  as `ColumnDef.editable_predicate: Option<EditablePredicate>`. The
  closure receives the row index and the full `GridModel`, so it can
  implement cross-column logic, not just this column's own value.
- `ColumnDef::is_cell_editable(row, model) -> bool` is the single source
  of truth, combining all three layers in order (each short-circuits the
  next, mirroring the `rules` → `validator` layering above):
  `GridModel.editable` (grid-wide) → `ColumnDef.editable` (static
  per-column) → `editable_predicate` (dynamic per-cell, not even called
  if either static layer is `false`).
- Consumed by every mutation/render path that needs to know whether a
  cell can be edited: `GridState::apply(StartEdit)` and `CommitEdit`
  (`state/cmd_edit.rs` — `CommitEdit` re-checks it, since the predicate's
  result can change between `StartEdit` and `CommitEdit`),
  `PasteAt`/`CutSelection` (`state/cmd_clipboard.rs` — locked cells are
  silently skipped, `continue`, not `break`, since a locked cell says
  nothing about its neighbours), `rs-grid-web`'s `hit_locked_cell`
  (`not-allowed` cursor), and `rs-grid-scene`'s `emit_cell` (themed
  locked-cell overlay — see `rs-grid-scene/AGENTS.md`).

## Per-cell decoration (`CellDecorator`)

- Persistent, at-rest visual annotation (border color / background tint
  / reserved message) for a single cell — purely cosmetic, never affects
  whether a value can be written (contrast with `rules`/`validator`).
  Mirrors `EditablePredicate` exactly in shape: attach via
  `ColumnDef::decorated_when(f: impl Fn(u64, &GridModel) -> Option<CellDecoration>)`,
  stored as `ColumnDef.decorator: Option<CellDecorator>`. The closure
  receives the row index and the full `GridModel`, so it can implement
  cross-column logic (e.g. flag a cell when a paired column is
  inconsistent).
- `ColumnDef::cell_decoration(row, model) -> Option<CellDecoration>` is
  the resolver. Unlike `is_cell_editable`, there is no static
  short-circuit layer — a decoration is purely cosmetic and not gated by
  `editable`/`model.editable`.
- `CellDecoration` is `#[non_exhaustive]`; build one from
  `CellDecoration::default()` chained with `.with_border_color(...)`,
  `.with_background_tint(...)`, `.with_message(...)`. `border_color`/
  `background_tint` are consumer-supplied `[u8; 4]` RGBA (same convention
  as `FormattedCell::color`), not read from the theme. `message` is
  reserved for a future native hover tooltip — not rendered anywhere yet.
- Consumed only by `rs-grid-scene`'s `emit_cell` (themed border width,
  consumer-supplied colors — see `rs-grid-scene/AGENTS.md`). No other
  crate needs to call it — there is no hit-test/cursor semantics for
  decoration, unlike `editable_predicate`'s `hit_locked_cell`.

## Cell formats (`CellFormat`)

`ColumnDef.format` selects how a cell is drawn. The enum is
`#[non_exhaustive]`; the scene layer interprets each variant. Besides
`Number`/`Percent`/`Currency`/`Boolean`/`Custom`/`Image`/`Styled`/`ImageText`,
there is:

- `ProgressBar { min, max, show_label, class_of }` — a value-driven progress
  bar. The raw value is parsed as `f64` and mapped to `[0, 1]` via
  `(value - min) / (max - min)`. `class_of: Option<Rc<dyn Fn(&str) -> String>>`
  maps the value to a class string (e.g. `"progress progress-success"`),
  resolved by the registered `ClassResolver` to pick the fill colour per value;
  `None` uses `Theme::progress_fill`. Stays daisyUI-agnostic — class strings are
  opaque here.

## Useful commands

```sh
cargo test -p rs-grid-core
cargo clippy -p rs-grid-core -- -D warnings
```
