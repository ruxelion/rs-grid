# Hit-Testing

## Overview

Hit-testing converts a pointer position (in viewport coordinates) to the
cell, column header, or row header under the cursor. It is the foundation
for click selection, hover, editing, and context menus.

## Functions

### `hit_test(vx, vy, model, scroll_x, scroll_y) → Option<CellCoord>`

Resolves a data cell from viewport coordinates:

1. **Gutter check** — if `vx < row_number_width`, returns `None`
2. **Header check** — if `vy < header_height`, returns `None`
3. **Column lookup** — uses `ColumnOffsets` for O(1) lookup
4. **Row lookup** — O(1) thanks to uniform row height

Returns `None` if the pointer is below the last row or right of the last column.

### `hit_test_col_header(vx, vy, model, scroll_x) → Option<usize>`

Returns the column index when clicking a column header. Used for:

- Sort toggle
- Column selection
- Column resize detection

### `hit_test_row_header(vx, vy, model, scroll_y) → Option<u64>`

Returns the row index when clicking the row number gutter. Used for:

- Row selection
- Row selection extension (drag)

## Pinned columns

Hit-testing accounts for pinned columns:

```
if vx_data < pinned_width:
    abs_x = vx_data          // pinned: no scroll offset
else:
    abs_x = vx_data + scroll_x  // scrollable: add scroll
```

This ensures clicks on pinned columns resolve correctly regardless of
horizontal scroll position.

## Precision at extreme scroll positions

For very large datasets (billions of rows), naive `(vy + scroll_y - header_height) / row_height`
loses f64 precision because it subtracts two large numbers.

The hit-testing code uses a **precision-preserving decomposition**:

```rust
let sy_content = scroll_y - header_height;
let first_row = (sy_content / row_height) as u64;
let frac = sy_content % row_height;  // sub-row offset
let offset = ((vy + frac) / row_height) as u64;
let row = first_row + offset;
```

This keeps intermediate values small, preserving precision even at row
indices near the u64 limit.

## Performance

- Column lookup: O(1) via precomputed `ColumnOffsets`
- Row lookup: O(1) with uniform row height
- Total: O(1) per hit-test call

:::warning
Never introduce O(n) algorithms in the hit-testing path. The column
offsets are precomputed once when columns change, not on every pointer event.
:::
