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
    MoveSelection { delta_row: i64, delta_col: i64, extend: bool },
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
}

#[derive(Debug, Clone)]
pub enum CommandOutput {
    None,
    CopyText(String),
    CopyError(CopyError),
}
