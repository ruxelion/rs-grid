# Callbacks & persistence

## Overview

`WebGridCanvas` exposes 4 callbacks for reacting to user-driven mutations.
They're the primitives you need to persist grid state to localStorage,
a backend, or any external sink. The grid itself never persists — that
choice is left to the caller.

## Available callbacks

| Method                     | Triggers when                                 |
| -------------------------- | --------------------------------------------- |
| `set_on_change`            | A `CommitEdit` or `PasteAt` mutates cell data |
| `set_on_columns_changed`   | Layout changes: width / order / pin count     |
| `set_on_validation_error`  | A column `validator` rejected an edit         |
| `set_on_cell_button_click` | User clicked an in-cell button                |

## Persisting column layout

`set_on_columns_changed` fires after every layout-mutating command.
Combine with the 3 layout readers to snapshot the state.

### Readers

```rust
pub fn column_widths(&self) -> Vec<(String, f64)>;  // (col_key, px)
pub fn column_order(&self) -> Vec<String>;          // col_keys in display order
pub fn pinned_count(&self) -> usize;
```

### Example: localStorage

```rust
use rs_grid_leptos::WebGridCanvas;
use web_sys::window;

let on_mount = move |gc: WebGridCanvas| {
    let gc_save = gc.clone();
    gc.set_on_columns_changed(move || {
        let payload = (
            gc_save.column_widths(),
            gc_save.column_order(),
            gc_save.pinned_count(),
        );
        if let Ok(json) = serde_json::to_string(&payload) {
            if let Some(ls) = window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = ls.set_item("rs-grid-layout", &json);
            }
        }
    });
};
```

On mount, read the stored JSON and apply it by:

- **Widths** — set `ColumnDef.width` before building the `GridModel`
- **Order** — sort `model.columns` to match the persisted order
- **Pinned count** — set `model.pinned_count` directly (or via `set_pinned_count` after mount)

After mutating widths or order in `model.columns`, recompute the precomputed
offsets used by hit-testing:

```rust
model.column_offsets = rs_grid_core::ColumnOffsets::compute(&model.columns);
```

The `examples/basic-leptos/src/lib.rs` example ships a complete
load/save/apply flow you can copy.

## Scope of `set_on_columns_changed`

The callback covers **physical layout only**: widths, order, pin count.
It does **not** fire on `ToggleSort`, `SetSort`, `ClearSort`,
`SetColumnFilter`, or `ClearAllFilters` — those have no dedicated
callback yet. Persist sort/filter state separately if you need it.

It also fires on `CommitColumnResize` (once at mouseup), **not** on every
intermediate `ResizeColumn` during a drag — so writes to localStorage
stay infrequent without any throttling on your side.

## Re-entrancy

⚠️ **Do not dispatch a `GridCommand` synchronously from any callback.**
The callback runs while the grid's internal state is still borrowed
read-only — dispatching would re-borrow and panic. Defer via
`queueMicrotask`, `setTimeout(0)`, or a channel if you need to react
with another command.
