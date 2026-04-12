use crate::{
    selection::{CellCoord, CopyError},
    sort::SortDir,
};

/// All mutations that can be applied to a
/// [`GridState`](crate::state::GridState) via
/// [`GridState::apply`](crate::state::GridState::apply).
///
/// # Index type convention
///
/// Row indices are `u64` (supports >4 billion rows on
/// WASM32). Column indices are `usize` (columns are always
/// a small count). See [`CellCoord`] for details.
///
/// # Variant categories
///
/// | Category | Variants |
/// |---|---|
/// | **Selection** | `SelectCell`, `ExtendSelection`, `SelectRow`, `ExtendRowSelection`, `SelectCol`, `ExtendColSelection`, `ClearSelection`, `MoveSelection` |
/// | **Scroll** | `ScrollTo`, `ScrollBy`, `Resize` |
/// | **Clipboard** | `CopySelection`, `CutSelection`, `PasteAt` |
/// | **Sort & filter** | `ToggleSort`, `SetSort`, `ClearSort`, `SetColumnFilter`, `ClearAllFilters` |
/// | **Columns** | `ResizeColumn`, `CommitColumnResize`, `SetPinnedColumnCount`, `MoveColumn`, `AutoFitColumn`, `AutoFitAllColumns` |
/// | **Editing** | `StartEdit`, `CommitEdit`, `CancelEdit` |
/// | **Undo** | `Undo`, `Redo` |
/// | **Search** | `Search`, `SearchNext`, `SearchPrev`, `ClearSearch` |
/// | **Meta** | `SetHoveredRow`, `SetHeaderHeight`, `SetRowHeight`, `NotifyPageLoaded`, `SetTotalRowCount` |
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GridCommand {
    /// Set a new single-cell selection.
    SelectCell(CellCoord),
    /// Extend the current selection to a new focus (shift-click).
    ExtendSelection(CellCoord),
    /// Scroll to an absolute position.
    ScrollTo {
        /// Horizontal offset in logical pixels.
        x: f64,
        /// Vertical offset in logical pixels.
        y: f64,
    },
    /// Scroll by a delta (wheel event).
    ScrollBy {
        /// Horizontal delta in logical pixels.
        dx: f64,
        /// Vertical delta in logical pixels.
        dy: f64,
    },
    /// Update canvas dimensions (resize).
    Resize {
        /// New canvas width in logical pixels.
        width: f64,
        /// New canvas height in logical pixels.
        height: f64,
    },
    /// Remove the current selection.
    ClearSelection,
    /// Copy the current selection to clipboard (returns TSV text).
    CopySelection,
    /// Copy the current selection to clipboard and clear the selected cells.
    CutSelection,
    /// Move or extend the selection by a row/col delta.
    MoveSelection {
        /// Row offset (positive = down).
        delta_row: i64,
        /// Column offset (positive = right).
        delta_col: i64,
        /// If `true`, extend rather than move the selection.
        extend: bool,
    },
    /// Paste TSV text starting at the current selection anchor.
    PasteAt {
        /// Tab-separated text (RFC 4180).
        text: String,
    },
    /// Select all cells in a row (click on row-number gutter).
    SelectRow(u64),
    /// Extend the current row selection to cover a new row (drag in gutter).
    ExtendRowSelection(u64),
    /// Select all cells in a column (click on column header).
    SelectCol(usize),
    /// Extend the current column selection to cover a new column (drag in header).
    ExtendColSelection(usize),
    /// Set the header row height in logical pixels.
    SetHeaderHeight(f64),
    /// Set the data row height in logical pixels.
    SetRowHeight(f64),
    /// Show or hide the column header row.
    SetShowHeader(bool),
    /// Show or hide the row-number gutter.
    SetShowRowNumbers(bool),
    /// Enable or disable inline cell editing grid-wide.
    SetEditable(bool),
    /// Enable or disable cell/row/column selection grid-wide.
    SetSelectable(bool),
    /// Set the width of a column (column resize drag).
    ResizeColumn {
        /// Index of the column to resize.
        col_idx: usize,
        /// New width in logical pixels.
        new_width: f64,
    },
    /// Update the hovered row (mousemove / mouseleave).
    SetHoveredRow(Option<u64>),
    /// Cycle sort state for a column: None → Asc → Desc → None.
    ToggleSort {
        /// Column key to toggle.
        col_key: String,
    },
    /// Set an explicit sort direction for a column.
    SetSort {
        /// Column key to sort.
        col_key: String,
        /// Direction to apply.
        dir: SortDir,
    },
    /// Remove the active sort (restore natural row order).
    ClearSort,
    /// Set the number of leading columns pinned (frozen) during
    /// horizontal scroll.
    SetPinnedColumnCount {
        /// Number of leading columns to pin.
        count: usize,
    },
    /// Set a text filter on a column (case-insensitive contains).
    /// Empty text clears the filter for that column.
    SetColumnFilter {
        /// Column key to filter.
        col_key: String,
        /// Filter text (empty = clear filter for this column).
        text: String,
    },
    /// Clear all column filters at once.
    ClearAllFilters,
    /// Move a column from one position to another (drag & drop).
    MoveColumn {
        /// Original column index.
        from_idx: usize,
        /// Destination column index.
        to_idx: usize,
    },
    /// Start editing a cell (double-click).
    StartEdit {
        /// Row index of the cell to edit.
        row: u64,
        /// Column key of the cell to edit.
        col_key: String,
    },
    /// Commit the current cell edit with a new value.
    CommitEdit {
        /// Row index of the edited cell.
        row: u64,
        /// Column key of the edited cell.
        col_key: String,
        /// New cell value to commit.
        value: String,
    },
    /// Cancel the current cell edit.
    CancelEdit,
    /// Undo the last undoable action.
    Undo,
    /// Redo the last undone action.
    Redo,
    /// Search all visible cells for a query (case-insensitive).
    Search {
        /// Case-insensitive search text.
        query: String,
    },
    /// Jump to the next search match.
    SearchNext,
    /// Jump to the previous search match.
    SearchPrev,
    /// Clear the search state.
    ClearSearch,
    /// Notify the grid that a page of data has been loaded into the
    /// cache. This is a no-op command — it exists solely to trigger a
    /// re-render after the `PageCacheDataSource` has been mutated
    /// externally. Has no effect on other data source types.
    NotifyPageLoaded,
    /// Update the total row count for an async data source.
    ///
    /// Intended for use with `PageCacheDataSource` after the first
    /// server response returns the real row count. Has no effect on
    /// `VecDataSource` or `FnDataSource`.
    SetTotalRowCount(u64),
    /// Record an undo entry after a column-resize drag ends.
    ///
    /// During a resize drag the web layer sends many
    /// [`ResizeColumn`] commands (one per mousemove) which
    /// intentionally do **not** push undo entries. At mouseup
    /// the web layer dispatches this command once to record
    /// the resize as a single undoable action.
    CommitColumnResize {
        /// Index of the resized column.
        col_idx: usize,
        /// Width before the drag started.
        old_width: f64,
        /// Flex factor before the drag started (`None` if fixed).
        old_flex: Option<f64>,
    },
    /// Auto-fit a column width to its content (double-click separator).
    AutoFitColumn {
        /// Index of the column to auto-fit.
        col_idx: usize,
        /// Average character width in logical pixels, provided by the
        /// renderer (derived from `font_size`).
        char_width: f64,
        /// Average character width for the header font (may be bold).
        header_char_width: f64,
        /// Horizontal cell padding (both sides).
        cell_padding: f64,
        /// Extra space reserved at the right of the header for the
        /// menu icon button, sort arrow, and their margins.
        header_right_reserve: f64,
    },
    /// Auto-fit all column widths to their content.
    AutoFitAllColumns {
        /// Average character width in logical pixels.
        char_width: f64,
        /// Average character width for the header font (may be bold).
        header_char_width: f64,
        /// Horizontal cell padding (both sides).
        cell_padding: f64,
        /// Extra space reserved at the right of the header for the
        /// menu icon button, sort arrow, and their margins.
        header_right_reserve: f64,
    },
}

/// Value returned by [`crate::state::GridState::apply`]
/// after processing a command.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum CommandOutput {
    /// Command produced no output.
    None,
    /// TSV text ready for the clipboard.
    CopyText(String),
    /// Copy/cut failed.
    CopyError(CopyError),
    /// Sort was requested but skipped because the dataset exceeds
    /// the client-side sort limit. The grid remains unsorted.
    SortWarning {
        /// Actual number of rows in the dataset.
        row_count: u64,
        /// Maximum rows supported for client-side sort.
        limit: u64,
    },
}
