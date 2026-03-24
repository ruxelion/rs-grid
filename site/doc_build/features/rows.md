# Rows

## RowRecord

Each row is a `RowRecord` — a thin wrapper around a `HashMap<String, String>`
with a numeric ID:

```rust
pub struct RowRecord {
    pub id: u64,
    fields: HashMap<String, String>,
}

let mut row = RowRecord::new(0);
row.set("name", "Alice");
row.set("age", "30");
```

## Row height

All rows share the same height (`model.row_height`), set at creation time:

```rust
let model = GridModel::new(columns, rows, 32.0, 36.0);
//                                        ^^^^
//                                     row_height
```

Uniform row height enables O(1) row lookup: `row_index = floor((y - header_height) / row_height)`.

## Row number gutter

A sticky column on the left displays row numbers (1-based). The gutter width
auto-adjusts based on digit count:

```
width = max(digits × 9px + 24px, 40px)
```

| Row count   | Digits | Width          |
| ----------- | ------ | -------------- |
| 1–9         | 1      | 40px (minimum) |
| 10–99       | 2      | 42px           |
| 1,000–9,999 | 4      | 60px           |
| 1,000,000+  | 7+     | 87px+          |

## Row hover

`SetHoveredRow` highlights the row under the cursor:

```rust
state.apply(GridCommand::SetHoveredRow(Some(42)));
state.apply(GridCommand::SetHoveredRow(None)); // mouse left the grid
```

The hover color is controlled by `--rs-grid-row-hover-bg` (default: semi-transparent overlay).

## Alternating row backgrounds

Odd rows use the `row_alt_bg` theme color for subtle zebra striping.
Customize with `--rs-grid-row-alt-bg`.

## Row count limits

Row indices are `u64` (not `usize`), supporting up to \~9×10^14 rows
with full f64 precision. On WASM32, `usize` is only 32 bits (4 GB),
which is why row indices use `u64` throughout.

:::warning
Beyond \~9×10^14 rows, f64 precision degrades (consecutive integers
become indistinguishable). The hit-testing code uses a precision-preserving
decomposition to mitigate this at extreme scroll positions.
:::
