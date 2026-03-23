# Rustdoc History

## 2026-03-23T00:00:00Z

- **Area**: `rs_grid_core` (lib.rs, column, viewport, state)
- **Items documented**: 26
- **Coverage**: 183 undocumented public items remaining (was 209)
- **Summary**:
  - Added `///` to all 12 `pub mod` declarations in `rs_grid_core::lib`
  - Added `///` to `ColumnDef::new()`, `ColumnOffsets::compute()`, `ColumnOffsets::total_width`
  - Added `///` to `ViewportState::new()`
  - Added `///` to `EditCell` fields (`row`, `col_key`, `initial_value`)
  - Added `///` to `SearchState` fields (`query`, `matches`, `current`)
  - Added `///` to `GridState` fields (`model`, `viewport`, `selection`)
  - Added `///` to `GridState::new()`
- **Validation**: pass (lib clippy clean; 4 pre-existing `redundant_closure` warnings in test code)
