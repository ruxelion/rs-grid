use crate::{
    selection::{CellCoord, CopyError},
    sort::SortDir,
    validation::InvalidEditMode,
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
/// | **Editing** | `StartEdit`, `CommitEdit`, `CancelEdit`, `ClearCells` |
/// | **Undo** | `Undo`, `Redo` |
/// | **Search** | `Search`, `SearchNext`, `SearchPrev`, `ClearSearch` |
/// | **Row checkboxes** | `ToggleRowChecked`, `ExtendRowChecked`, `ToggleAllFilteredChecked`, `SetShowCheckboxColumn`, `SetCheckboxColumnWidth` |
/// | **Meta** | `SetHoveredRow`, `SetHeaderHeight`, `SetRowHeight`, `SetRowNumberWidth`, `NotifyPageLoaded`, `SetTotalRowCount` |
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
    /// Extend the current column selection to cover a new column (drag in
    /// header).
    ExtendColSelection(usize),
    /// Set the header row height in logical pixels.
    SetHeaderHeight(f64),
    /// Set the data row height in logical pixels.
    SetRowHeight(f64),
    /// Set the row-number gutter width in logical pixels (0 hides it,
    /// same meaning as `SetShowRowNumbers(false)`).
    SetRowNumberWidth(f64),
    /// Set the row-selection checkbox column's width in logical pixels
    /// (negative values are ignored). The checkbox itself stays centered,
    /// so this also controls the visual margin around it.
    SetCheckboxColumnWidth(f64),
    /// Show or hide the column header row.
    SetShowHeader(bool),
    /// Show or hide the row-number gutter.
    SetShowRowNumbers(bool),
    /// Enable or disable inline cell editing grid-wide.
    SetEditable(bool),
    /// Enable or disable cell/row/column selection grid-wide.
    SetSelectable(bool),
    /// Enable or disable header drag-to-reorder of columns.
    /// Does not affect programmatic `MoveColumn` commands.
    SetColumnReorderable(bool),
    /// Set the grid-wide policy applied when a `CommitEdit` fails
    /// validation (revert vs. block).
    SetInvalidEditMode(InvalidEditMode),
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
    /// Clear every editable cell in the current selection to an empty
    /// string (Delete/Backspace) — validated the same way as
    /// `CutSelection`'s clear step (a cell whose rules reject an empty
    /// value, e.g. `.required()`, keeps its original value instead of
    /// being cleared). Unlike `CutSelection`, does not touch the
    /// clipboard. No-op without a selection, or on a full-column
    /// selection (same rationale as `CutSelection`: a header click
    /// carries positional intent, not "clear this entire column of
    /// potentially billions of rows").
    ClearCells,
    /// Re-validate the value currently typed in the active editor,
    /// without committing it. Updates the active `EditCell`'s
    /// `validation_error` for live (per-keystroke) UI feedback.
    /// No-op if there is no active edit.
    ValidateEdit {
        /// Value currently typed in the editor.
        value: String,
    },
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
    /// Toggle whether a single row (logical index) is checked in the
    /// row-selection checkbox column. Checked state is tracked by
    /// physical row id, so it survives sort/filter changes. Starts a
    /// fresh shift+click gesture: records `logical_row` as the range
    /// anchor and the state it toggled *to* as the gesture's fixed
    /// direction, for any following `ExtendRowChecked` calls.
    ToggleRowChecked(u64),
    /// Shift+click on the checkbox column: set every row (logical
    /// index) in `[anchor, logical_row]` to the direction fixed by the
    /// last `ToggleRowChecked` — mirrors `ExtendRowSelection`'s
    /// anchor/focus range. Unlike a plain range-set, this also
    /// reconciles against the *previous* `ExtendRowChecked` call in the
    /// same gesture: rows the previous call touched but the new range
    /// no longer covers are reverted to the opposite state — so
    /// checking 1‑10 then, while still holding shift, clicking row 9
    /// gives row 10 back its earlier (unchecked) state instead of
    /// leaving it checked. With no prior anchor, behaves like a single
    /// `ToggleRowChecked` (range of one, checked).
    ExtendRowChecked(u64),
    /// Toggle the checkbox-column header: if every row currently
    /// passing the active filter (or every row, if unfiltered) is
    /// checked, uncheck them all; otherwise check them all. Never
    /// touches rows filtered out of view.
    ToggleAllFilteredChecked,
    /// Show or hide the row-selection checkbox column.
    SetShowCheckboxColumn(bool),
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
    /// A `CommitEdit` was rejected by the column's validation rules
    /// (or legacy validator). Emitted whether the edit reverted
    /// (`InvalidEditMode::Revert`) or stayed open
    /// (`InvalidEditMode::Block`).
    ValidationError {
        /// Row index of the rejected edit.
        row: u64,
        /// Column key of the rejected edit.
        col_key: String,
        /// Error message from the failing rule/validator.
        message: String,
    },
    /// A `PasteAt` completed. `cells` holds the coordinates actually
    /// written — a subset of the target rectangle, since cells that
    /// are locked (`ColumnDef::is_cell_editable` false) or whose
    /// pasted value fails validation are silently skipped. Consumers
    /// that give paste-success feedback (e.g. a flash animation)
    /// should use this list rather than the selection rectangle,
    /// which still covers the full target area regardless of skips.
    PasteApplied {
        /// Coordinates of cells that were actually written.
        cells: Vec<CellCoord>,
    },
    /// A `ClearCells` completed and wrote at least one cell. `cells`
    /// holds the coordinates actually cleared — a subset of the
    /// selection, since locked or validation-rejecting cells (e.g.
    /// `.required()`) are silently skipped, same as `PasteApplied`.
    /// Not emitted when nothing was cleared (empty/full-column
    /// selection, or every cell in range was skipped) — consumers use
    /// this to scope success feedback (e.g. a flash animation) to
    /// cells that were genuinely written.
    CellsCleared {
        /// Coordinates of cells that were actually cleared.
        cells: Vec<CellCoord>,
    },
    /// A per-cell `CutSelection` completed (not the full-column-header
    /// variant, which still returns plain `CopyText` — see the
    /// `CutSelection` doc comment). `text` is the TSV to place on the
    /// clipboard, always covering the full original selection
    /// regardless of what the clear side skipped. `skipped` holds the
    /// coordinates that were **not** cleared because the cell is locked
    /// or its empty value fails validation (e.g. `.required()`) — a
    /// subset of the selection. Consumers use `skipped` to give
    /// feedback (e.g. an error flash) distinguishing "cut succeeded" from
    /// "copied but couldn't clear", which otherwise look identical.
    CutApplied {
        /// TSV text to place on the clipboard.
        text: String,
        /// Coordinates that were copied but not cleared.
        skipped: Vec<CellCoord>,
    },
}
