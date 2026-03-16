use crate::selection::CellCoord;

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
}
