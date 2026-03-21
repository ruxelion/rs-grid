use crate::selection::{CellCoord, CopyError};

/// All mutations that can be applied to a `GridState`.
#[derive(Debug, Clone)]
pub enum GridCommand {
    /// Set a new single-cell selection.
    SelectCell(CellCoord),
    /// Extend the current selection to a new focus (shift-click).
    ExtendSelection(CellCoord),
    /// Scroll to an absolute position.
    ScrollTo { x: f64, y: f64 },
    /// Scroll by a delta (wheel event).
    ScrollBy { dx: f64, dy: f64 },
    /// Update canvas dimensions (resize).
    Resize { width: f64, height: f64 },
    /// Remove the current selection.
    ClearSelection,
    /// Copy the current selection to clipboard (returns TSV text).
    CopySelection,
    /// Copy the current selection to clipboard and clear the selected cells.
    CutSelection,
    /// Move or extend the selection by a row/col delta.
    MoveSelection {
        delta_row: i64,
        delta_col: i64,
        extend: bool,
    },
    /// Paste TSV text starting at the current selection anchor.
    PasteAt { text: String },
    /// Select all cells in a row (click on row-number gutter).
    SelectRow(u64),
    /// Extend the current row selection to cover a new row (drag in gutter).
    ExtendRowSelection(u64),
    /// Select all cells in a column (click on column header).
    SelectCol(usize),
    /// Extend the current column selection to cover a new column (drag in header).
    ExtendColSelection(usize),
    /// Set the width of a column (column resize drag).
    ResizeColumn { col_idx: usize, new_width: f64 },
    /// Update the hovered row (mousemove / mouseleave).
    SetHoveredRow(Option<u64>),
    /// Cycle sort state for a column: None → Asc → Desc → None.
    ToggleSort { col_key: String },
    /// Set the number of leading columns pinned (frozen) during
    /// horizontal scroll.
    SetPinnedColumnCount { count: usize },
    /// Set a text filter on a column (case-insensitive contains).
    /// Empty text clears the filter for that column.
    SetColumnFilter { col_key: String, text: String },
    /// Clear all column filters at once.
    ClearAllFilters,
    /// Move a column from one position to another (drag & drop).
    MoveColumn { from_idx: usize, to_idx: usize },
    /// Start editing a cell (double-click).
    StartEdit { row: u64, col_key: String },
    /// Commit the current cell edit with a new value.
    CommitEdit {
        row: u64,
        col_key: String,
        value: String,
    },
    /// Cancel the current cell edit.
    CancelEdit,
    /// Undo the last undoable action.
    Undo,
    /// Redo the last undone action.
    Redo,
    /// Search all visible cells for a query (case-insensitive).
    Search { query: String },
    /// Jump to the next search match.
    SearchNext,
    /// Jump to the previous search match.
    SearchPrev,
    /// Clear the search state.
    ClearSearch,
    /// Notify the grid that a page of data has been loaded into the
    /// cache. This is a no-op; it exists solely to trigger a re-render.
    NotifyPageLoaded,
    /// Update the total row count (used by async data sources after the
    /// first server response).
    SetTotalRowCount(u64),
    /// Auto-fit a column width to its content (double-click separator).
    AutoFitColumn {
        col_idx: usize,
        /// Average character width in logical pixels, provided by the
        /// renderer (derived from `font_size`).
        char_width: f64,
        /// Average character width for the header font (may be bold).
        header_char_width: f64,
        /// Horizontal cell padding (both sides).
        cell_padding: f64,
    },
}

#[derive(Debug, Clone)]
pub enum CommandOutput {
    None,
    CopyText(String),
    CopyError(CopyError),
}
