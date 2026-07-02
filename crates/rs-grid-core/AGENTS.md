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
