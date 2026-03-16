/// A (row, col) address of a cell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellCoord {
    pub row: usize,
    pub col: usize,
}

/// Rectangular selection defined by an anchor and a focus cell.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub anchor: Option<CellCoord>,
    pub focus: Option<CellCoord>,
}

impl SelectionState {
    /// Start a new single-cell selection.
    pub fn select_cell(&mut self, row: usize, col: usize) {
        let coord = CellCoord { row, col };
        self.anchor = Some(coord.clone());
        self.focus = Some(coord);
    }

    /// Extend the selection to cover a new focus cell (shift-click / shift-arrow).
    pub fn extend_to(&mut self, row: usize, col: usize) {
        self.focus = Some(CellCoord { row, col });
    }

    /// Returns `true` if the given cell falls inside the selected rectangle.
    pub fn is_selected(&self, row: usize, col: usize) -> bool {
        match (&self.anchor, &self.focus) {
            (Some(a), Some(f)) => {
                let r_min = a.row.min(f.row);
                let r_max = a.row.max(f.row);
                let c_min = a.col.min(f.col);
                let c_max = a.col.max(f.col);
                row >= r_min && row <= r_max && col >= c_min && col <= c_max
            }
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }

    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }
}
