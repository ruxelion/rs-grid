//! Headless grid logic: data model, viewport, selection,
//! hit-testing, and command processing. No WASM dependency.

/// Column definitions, offsets, and cell formatting.
pub mod column;
/// Command enum and output type for all grid mutations.
pub mod commands;
/// Trait abstraction for row data backends.
pub mod datasource;
/// O(log n) hit-testing on cells, headers, and gutters.
pub mod hit_test;
/// Grid data model: columns, datasource, and sizing.
pub mod model;
/// Page-based cache for lazy/virtual data sources.
pub mod page_cache;
/// Row record storage and metadata.
pub mod row;
/// Scrollbar geometry and drag state.
pub mod scrollbar;
/// Selection state: anchor, focus, and clipboard helpers.
pub mod selection;
/// Sort direction and per-column sort state.
pub mod sort;
/// Central mutable grid state combining model, viewport,
/// and selection.
pub mod state;
/// Viewport scroll position and visible-range computation.
pub mod viewport;
