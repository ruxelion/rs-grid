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

## Useful commands

```sh
cargo test -p rs-grid-core
cargo clippy -p rs-grid-core -- -D warnings
```
