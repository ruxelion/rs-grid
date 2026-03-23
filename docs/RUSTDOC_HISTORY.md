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

## 2026-03-23T00:01:00Z

- **Area**: `rs_grid_core` (column, viewport, model, selection, datasource, row, sort, page_cache, lib crate doc)
- **Items documented**: 57
- **Coverage**: 126 undocumented public items remaining (was 183); `rs_grid_core` down to 29 (all in `commands.rs`)
- **Summary**:
  - Added `//!` crate-level doc to `rs_grid_core::lib`
  - Added `///` to `CellAlign` variants, `FormattedCell` fields, `CellFormat` variant fields
  - Added `///` to `ViewportState::scroll_x`/`scroll_y`, `GridModel::columns`/`data`
  - Added `///` to `CellCoord` fields, `SelectionState` fields/methods, `CopyError` enum/variants, `MAX_COPY_ROWS`
  - Added `///` to `DataSource` trait + methods, `VecDataSource`, `FnDataSource`
  - Added `///` to `RowRecord` fields/methods
  - Added `///` to `SortDir` variants, `SortState` fields
  - Added `///` to all `PageCacheDataSource` public methods
- **Validation**: pass

## 2026-03-23T00:02:00Z

- **Area**: `rs_grid_core::commands`
- **Items documented**: 29
- **Coverage**: 97 undocumented public items remaining (was 126); `rs_grid_core` now at 0
- **Summary**:
  - Added `///` to all `GridCommand` struct-variant fields (ScrollTo, ScrollBy, Resize, MoveSelection, PasteAt, ResizeColumn, ToggleSort, SetPinnedColumnCount, SetColumnFilter, MoveColumn, StartEdit, CommitEdit, Search, AutoFitColumn)
  - Added `///` to `CommandOutput` enum and its 3 variants
- **Validation**: pass

## 2026-03-23T00:03:00Z

- **Area**: `rs_grid_scene` (lib.rs, primitives, theme, frame, builder)
- **Items documented**: 62
- **Coverage**: 35 undocumented public items remaining (was 97); `rs_grid_core` and `rs_grid_scene` both at 0
- **Summary**:
  - Added `//!` crate-level doc and `///` to all 4 `pub mod` declarations
  - Added `///` to `Color` fields/constructors, `RectPrimitive` fields, `TextAlign` variants
  - Added `///` to `TextPrimitive`, `LinePrimitive`, `PolygonPrimitive` fields
  - Added `///` to `ScenePrimitive` variants
  - Added `///` to all 13 `Theme` fields without docs
  - Added `///` to `SceneFrame` fields/methods
  - Added `///` to `SceneBuilder` field/constructors
- **Validation**: pass

## 2026-03-23T00:04:00Z

- **Area**: `rs_grid_render_canvas`, `rs_grid_web`, `rs_grid_leptos`, `examples/basic-leptos`
- **Items documented**: 29
- **Coverage**: 6 undocumented public items remaining (was 35)
  - 2 build scripts (`build.rs`) — skipped per workflow rules
  - 4 Leptos `#[component]` macro-generated items — cannot be documented directly
- **Summary**:
  - Added `//!` crate docs to `rs_grid_render_canvas`, `rs_grid_web`, `examples/basic-leptos`
  - Added `///` to `CanvasRenderer::new()`, `pub mod renderer`
  - Added `///` to all 6 `BuiltinAction` variants, `ContextMenuItem::action` field
  - Added `///` to 7 `ContextMenuItem` convenience constructors
  - Added `///` to `PageFetchResponse`, `PageFetchRequest`, `FetchConfig` fields
  - Added `///` to `examples/basic-leptos::main()`
- **Validation**: pass
