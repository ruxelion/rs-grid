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

## Critical invariants

- **No `wasm-bindgen` here.** If you need WASM, it belongs in `rs-grid-web`.
- Row indices are **`u64`** (not `usize`) to support >4B rows on WASM32.
- `GridState` mutations go **exclusively** through `GridState::apply(GridCommand)`.
- Hit-testing must remain O(log n) — column offsets are precomputed.

## Behaviour flags & cell buttons

- `GridModel.editable: bool` (default `true`) — global edit toggle.
  Per-column `ColumnDef.editable` can opt individual columns out.
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

## Useful commands

```sh
cargo test -p rs-grid-core
cargo clippy -p rs-grid-core -- -D warnings
```
