# Extracting Data

rs-grid does not include a built-in CSV/JSON export. Instead, `GridModel`
exposes all the data you need to build your own export in a few lines of code.

## Key API surface

| Field / Method              | What it gives you                                               |
| --------------------------- | --------------------------------------------------------------- |
| `model.columns`             | `Vec<ColumnDef>` — ordered column definitions                   |
| `model.data.row_count()`    | Total number of physical rows                                   |
| `model.data.get_cell(r, k)` | Cell value by physical row index and column key                 |
| `model.data.get_cell_ref()` | Zero-copy `Cow<str>` variant (for in-memory sources)            |
| `model.sort_order`          | Physical indices in current sort order (empty = natural order)  |
| `model.filtered_indices`    | Physical indices passing all active filters (empty = no filter) |
| `model.patches`             | Edited cell overrides `(row, col_key) → value`                  |

## Basic example: export all rows

```rust
let row_count = model.data.row_count();

for row in 0..row_count {
    for col in &model.columns {
        // Check patches first, then fall back to data source
        let value = model
            .patches
            .get(&(row, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(row, &col.key))
            .unwrap_or_default();

        // Write `value` to your output (CSV writer, JSON array, etc.)
    }
}
```

## Respecting sort and filter

If you want the export to match what the user sees in the grid, iterate over
`filtered_indices` (if active) or `sort_order` (if active) instead of raw
row indices:

```rust
let indices: Vec<u64> = if !model.filtered_indices.is_empty() {
    // Filtered indices are already in sort order
    model.filtered_indices.clone()
} else if !model.sort_order.is_empty() {
    model.sort_order.clone()
} else {
    (0..model.data.row_count()).collect()
};

for &phys in &indices {
    for col in &model.columns {
        let value = model
            .patches
            .get(&(phys, col.key.clone()))
            .cloned()
            .or_else(|| model.data.get_cell(phys, &col.key))
            .unwrap_or_default();
        // ...
    }
}
```

## Column headers

Use `ColumnDef` fields for header labels:

```rust
let headers: Vec<&str> = model
    .columns
    .iter()
    .map(|c| c.label.as_str())
    .collect();
```

## Server-side data sources

For `PageCacheDataSource` or custom server-side sources, only rows that have
been fetched into the local cache are available. Check `cell_status()` before
reading:

```rust
use rs_grid_core::datasource::CellStatus;

match model.data.cell_status(row, &col.key) {
    CellStatus::Ready(val) => { /* use val */ }
    CellStatus::Loading    => { /* page not yet fetched */ }
    CellStatus::Absent     => { /* no value */ }
}
```

For a full export of server-side data, fetch all pages from your backend
directly rather than reading through the grid.
