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
| `filter` | `FilterOp`/`FilterCondition` — per-column filter operators and conditions |

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

## Row predicates (`RowPredicate<T>`)

`EditablePredicate`, `CellDecorator`, and `CellButtonsVisible` (below) are
type aliases over one generic `RowPredicate<T>` (`column.rs`) — an
`Rc<dyn Fn(u64, &GridModel) -> T>` wrapper with `new`/`evaluate`, a manual
`Clone` (no `T: Clone` bound — `Rc::clone` never touches `T`) and a manual
`Debug` (`dyn Fn` has no `Debug` impl for any `T`, so `#[derive(Debug)]`
isn't just unnecessarily bounded here, it's impossible). `EditablePredicate =
RowPredicate<bool>`, `CellDecorator = RowPredicate<Option<CellDecoration>>`,
`CellButtonsVisible = RowPredicate<bool>` — **this is why
`EditablePredicate` and `CellButtonsVisible` are the literal same
monomorphized type** and print identical `Debug` output
(`"RowPredicate(..)"`). **Do not add a 4th hand-rolled copy of this
`Rc<dyn Fn(u64, &GridModel) -> T>` wrapper for a future per-row callback** —
alias it onto `RowPredicate<T>` instead, the way the three below do. Each
alias below still owns its own resolver method on `ColumnDef`
(`is_cell_editable`/`cell_decoration`/`are_cell_buttons_visible`) — those stay
bespoke, since their gating logic differs (`is_cell_editable` alone has a
2-layer static gate before the predicate is even consulted).

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
  Same `RowPredicate<T>` alias family as `EditablePredicate` (see above,
  different `T`): attach via
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

## Cell button visibility (`CellButtonsVisible`)

- `ColumnDef.cell_buttons` is column-level by default — the same buttons
  draw on every row (an empty `Vec` already means "no buttons"). For
  per-row visibility (e.g. hide an "Open" button on rows with no known
  URL), attach a dynamic predicate via
  `ColumnDef::cell_buttons_visible_when(f: impl Fn(u64, &GridModel) -> bool)`,
  stored as `ColumnDef.cell_buttons_visible: Option<CellButtonsVisible>`.
  Same `RowPredicate<T>` alias family as `EditablePredicate`/`CellDecorator`
  (see above) — in fact the exact same concrete type as `EditablePredicate`
  (`RowPredicate<bool>`, `CellDecorator` differs only by its `T`) — the
  closure receives the row index and the full `GridModel`, so it can
  implement cross-column logic.
- `ColumnDef::are_cell_buttons_visible(row, model) -> bool` is the
  resolver. Like `cell_decoration`, there is no static gate layer — an
  empty `cell_buttons` vec already serves that role, so this is `true`
  whenever no predicate is set (today's behaviour unchanged).
- Consumed only by `rs-grid-scene`'s `emit_cell_buttons`, which now takes
  `model: &GridModel` — when the resolver is `false`, nothing is drawn
  and no `ButtonZone` hit-test entry is registered for that row (see
  `rs-grid-scene/AGENTS.md`).

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

## Row-number gutter width (`row_number_width` / `row_number_width_auto`)

`GridModel::compute_row_number_width(row_count)` sizes the gutter from the
digit count of `row_count` (~9px/digit + 24px padding, 40px floor) — unlike
AG Grid's row-number column, which is a fixed 60px regardless of scale (see
`docs/row-count-limits.md`: rs-grid's supported range spans ~1 to ~15
digits, not a handful, so a fixed width doesn't translate here).

- `GridModel.row_number_width_auto: bool` (default `true`) tracks whether
  `row_number_width` should keep following the data source's row count.
  Set from `data.row_count()` at construction (`with_data_source`).
- `GridCommand::SetTotalRowCount(n)` recomputes `row_number_width` from
  `n` when `row_number_width_auto` is `true` — its intended use is
  `PageCacheDataSource` learning its real total from the first server
  response, when the model was built from a placeholder count (`0` per
  the server-pagination how-to). It cannot update the data source's own
  count generically (`GridState` only holds a `Box<dyn DataSource>`) —
  that's still `PageCacheDataSource::set_total_rows`, dispatched
  alongside this command, not instead of it (`rs-grid-web`'s
  `FetchConfig` fetcher does both, see `rs-grid-web/AGENTS.md`).
- `GridCommand::SetRowNumberWidth(w)` sets `row_number_width_auto =
  false` — an explicit manual width is a deliberate override that a
  later `SetTotalRowCount` must not clobber. There is no command to
  re-enable auto mode once disabled (not needed yet — YAGNI).

## Per-column filtering (`FilterOp`, `FilterCondition`)

`model.filters: HashMap<String, FilterCondition>` maps a column key to an
active filter condition — an operator (`FilterOp`) plus a comparison
`value: String`. `FilterOp` covers text operators (`Contains`,
`NotContains`, `StartsWith`, `EndsWith`, case-insensitive),
`Blank`/`NotBlank` (ignore `value`, check `cell.trim().is_empty()`), and
four always-numeric operators (`GreaterThan`, `GreaterThanOrEqual`,
`LessThan`, `LessThanOrEqual`) that parse both sides as `f64` — a
non-numeric cell never matches, it never panics (same precedent as
`CellFormat::ProgressBar`'s value parsing). `Equals`/`NotEquals` branch on
`CellFormat::is_numeric_like()` (true for `Number`/`Percent`/`Currency`/
`ProgressBar`): numeric compare for numeric-like columns, case-insensitive
string equality otherwise.

`GridCommand::SetColumnFilter { col_key, condition }` sets/clears a
column's filter (`condition.is_empty()` — an operator that needs a value
with none — clears it); `GridCommand::ClearAllFilters` clears every
column (both this and `value_filters` below). `GridModel::apply_filter()`
rebuilds `filtered_indices` by AND-combining every active condition
across columns — it resolves each filtered column's `is_numeric_like()`
**once per call, not once per row**, hoisted out of the per-row loop (an
easy way to accidentally regress from O(n_rows·n_filters) to
O(n_rows·n_filters·n_columns)).

### Value-set filter (`value_filters`, `unique_values`)

`model.value_filters: HashMap<String, HashSet<String>>` is a second,
independent filter dimension — AG-Grid's "Set Filter" checklist. A
present entry restricts that column to exactly those cell values; an
absent entry means no restriction. An empty set is a valid, deliberately
restrictive state ("matches no rows"), distinct from an absent entry —
this is why `apply_filter()` can't use `filtered_indices.is_empty()` as
a "no filter" sentinel (see `is_filter_applied()` below).
`GridCommand::SetColumnValueFilter { col_key, values }` /
`ClearColumnValueFilter { col_key }` set/clear one column's entry;
`apply_filter()` AND-combines `value_filters` with `filters` and across
columns, in the same per-row pass (not a second O(n) scan).

`GridModel::unique_values(col_key, cap) -> UniqueValues` (`filter.rs`)
computes the distinct values for a column, scanning up to
`MAX_CLIENT_SORT_ROWS` rows — backs the checklist's own value list.
`UniqueValues::Values(Vec<String>)` (sorted, via `BTreeSet`) or
`UniqueValues::TooMany { cap }` once the running distinct count exceeds
`cap`, returned immediately rather than continuing the scan — bounds
memory to `cap + 1` entries regardless of row count, so a fully-unique
column (e.g. an email column) returns fast instead of scanning to
completion.

### Floating filter row (`show_filter_row`, `filter_row_height`)

`GridModel.show_filter_row: bool` (opt-in, default `false`, same
precedent as `show_checkbox_column`) and `filter_row_height: f64`
(default `36.0`) add a second sticky row directly under the column
headers — AG-Grid's floating filter row. `effective_filter_row_height()`
mirrors `effective_header_height()`: `filter_row_height` when shown, `0.0`
otherwise. `GridModel::data_top() -> f64` is
`effective_header_height() + effective_filter_row_height()` — the single
accessor for "where does the data/gutter/scrollbar band start,"
factored out because inserting this row means auditing every existing
`effective_header_height()` call site individually:

- **Extended to `data_top()`** (they measure "where does data start,"
  which the new row pushes down): `row_top()`, `total_height()`,
  `hit_test()`, `hit_test_row_header()`, `hit_test_checkbox_row()`,
  `logical_row_at_vy()` (`hit_test.rs`), and `cmd_search.rs`'s
  scroll-into-view positioning.
- **Left unchanged** (they measure the header's own band, which does
  not grow): `hit_test_col_header()`, `hit_test_checkbox_header()`.

`GridState::hit_test_filter_row_cell(vx, vy) -> Option<usize>`
(`hit_test.rs`) resolves a column index when the pointer is over the
filter row's own vertical band `[effective_header_height(),
effective_header_height() + effective_filter_row_height())` — same
column resolution (pinned/scroll/checkbox-column math) as
`hit_test_col_header`, `None` immediately when the row is hidden
(`effective_filter_row_height() <= 0.0`).

Two `Meta` (non-undoable) commands drive it, mirroring
`SetHeaderHeight`/`SetShowCheckboxColumn` exactly:
`GridCommand::SetFilterRowHeight(f64)` (ignored if `<= 0.0`) and
`GridCommand::SetShowFilterRow(bool)`.

This row is **additive to, not a replacement for**, the header funnel
icon + popup below — AND-combined, confirmed via `AskUserQuestion` when
this was designed. The row itself dispatches the same
`GridCommand::SetColumnFilter` with `FilterOp::Contains` (a fast
"contains" path); the popup remains for advanced (operator/checklist)
filtering. `rs-grid-web` renders the row (canvas-drawn when closed) and
opens a transient DOM `<input>` overlay on click — see
`rs-grid-web/AGENTS.md`'s "Floating filter row" section.

### `is_filter_applied()` — the empty-`Vec` ambiguity

`filtered_indices` being empty is ambiguous on its own: it means either
"no filter is active" (show every row) or "a filter is active and
genuinely matches zero rows" (show none) — both produce the identical
empty `Vec`. `GridModel::is_filter_applied()` disambiguates: `true` only
on the `apply_filter()` branch that actually performs a full row scan
and assigns a definitive (possibly empty) result; `false` when no filter
is active, filtering is delegated to the server, or the row count
exceeds the client-side cap and filtering was skipped as a safety
measure. **Every reader of `filtered_indices` must check this accessor
first** — `logical_to_physical()`, `display_row_count()`,
`checkbox_header_state()` (`state.rs`), and `ToggleAllFilteredChecked`'s
scope computation (`state/cmd_row_check.rs`) all do. A direct
`filtered_indices.clear()` that bypasses `apply_filter()` (as
`ClearAllFilters`'s handler used to do) leaves the private
`filter_computed` flag stale — go through `apply_filter()` instead of
duplicating its clearing logic.

`rs-grid-web` builds a header filter icon + popup (canvas-drawn icon,
DOM popup for the operator/value form plus the value checklist) on top
of this — see `rs-grid-web/AGENTS.md`'s "Header filter icon + popup"
section.

## Useful commands

```sh
cargo test -p rs-grid-core
cargo clippy -p rs-grid-core -- -D warnings
```
