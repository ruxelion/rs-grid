use crate::{
    commands::{CommandOutput, GridCommand},
    hit_test,
    model::GridModel,
    selection::{CellCoord, SelectionState},
    viewport::ViewportState,
};

/// The complete mutable state of a grid instance.
#[derive(Debug)]
pub struct GridState {
    pub model: GridModel,
    pub viewport: ViewportState,
    pub selection: SelectionState,
}

impl GridState {
    pub fn new(model: GridModel, viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            model,
            viewport: ViewportState::new(viewport_width, viewport_height),
            selection: SelectionState::default(),
        }
    }

    /// Apply a command, mutating state in place.
    pub fn apply(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::SelectCell(coord) => {
                self.selection.select_cell(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ExtendSelection(coord) => {
                self.selection.extend_to(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ScrollTo { x, y } => {
                let rnw = self.model.row_number_width;
                let max_x = (self.model.total_width() - (self.viewport.width - rnw)).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
                CommandOutput::None
            }
            GridCommand::ScrollBy { dx, dy } => {
                let x = self.viewport.scroll_x + dx;
                let y = self.viewport.scroll_y + dy;
                let rnw = self.model.row_number_width;
                let max_x = (self.model.total_width() - (self.viewport.width - rnw)).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
                CommandOutput::None
            }
            GridCommand::Resize { width, height } => {
                self.viewport.width = width;
                self.viewport.height = height;
                CommandOutput::None
            }
            GridCommand::ClearSelection => {
                self.selection.clear();
                CommandOutput::None
            }
            GridCommand::CopySelection => {
                match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e)   => CommandOutput::CopyError(e),
                }
            }
            GridCommand::PasteAt { text } => {
                let origin = self.selection.anchor.clone()
                    .or_else(|| self.selection.focus.clone());
                if let Some(orig) = origin {
                    let rows = crate::selection::parse_tsv(&text);
                    let col_count = self.model.columns.len();
                    let row_count = self.model.data.row_count();
                    for (dr, row_vals) in rows.iter().enumerate() {
                        let r = orig.row + dr as u64;
                        if r >= row_count { break; }
                        for (dc, val) in row_vals.iter().enumerate() {
                            let c = orig.col + dc;
                            if c >= col_count { break; }
                            let key = self.model.columns[c].key.clone();
                            self.model.set_cell(r, key, val.clone());
                        }
                    }
                    // Update selection to cover the pasted rectangle
                    if !rows.is_empty() && !rows[0].is_empty() {
                        let last_r = (orig.row + rows.len() as u64 - 1).min(row_count - 1);
                        let last_c = (orig.col + rows[0].len() - 1).min(col_count - 1);
                        self.selection.anchor = Some(CellCoord { row: orig.row, col: orig.col });
                        self.selection.focus  = Some(CellCoord { row: last_r,   col: last_c });
                    }
                }
                CommandOutput::None
            }
            GridCommand::SelectRow(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row, col: 0 });
                self.selection.focus  = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::ExtendRowSelection(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                // Clamp anchor column to 0 so the range always spans all columns.
                if let Some(ref mut a) = self.selection.anchor { a.col = 0; }
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::SelectCol(col) => {
                let last_row = self.model.data.row_count().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row: 0, col });
                self.selection.focus  = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::ExtendColSelection(col) => {
                let last_row = self.model.data.row_count().saturating_sub(1);
                // Clamp anchor row to 0 so the range always spans all rows.
                if let Some(ref mut a) = self.selection.anchor { a.row = 0; }
                self.selection.focus = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::MoveSelection { delta_row, delta_col, extend } => {
                let row_count = self.model.data.row_count();
                let col_count = self.model.columns.len();
                let base = self.selection.focus.clone()
                    .or_else(|| self.selection.anchor.clone());
                if let Some(b) = base {
                    let new_row = (b.row as i64 + delta_row)
                        .clamp(0, row_count.saturating_sub(1) as i64) as u64;
                    let new_col = (b.col as i64 + delta_col)
                        .clamp(0, col_count.saturating_sub(1) as i64) as usize;
                    if extend { self.selection.extend_to(new_row, new_col); }
                    else      { self.selection.select_cell(new_row, new_col); }
                }
                CommandOutput::None
            }
        }
    }

    /// Hit-test a viewport-space pointer position against the data cells.
    pub fn hit_test(&self, vx: f64, vy: f64) -> Option<CellCoord> {
        hit_test::hit_test(
            vx,
            vy,
            &self.model,
            self.viewport.scroll_x,
            self.viewport.scroll_y,
        )
    }

    /// Hit-test the sticky row-number gutter. Returns the row index or `None`.
    pub fn hit_test_row_header(&self, vx: f64, vy: f64) -> Option<u64> {
        hit_test::hit_test_row_header(vx, vy, &self.model, self.viewport.scroll_y)
    }

    /// Hit-test a column header. Returns the column index or `None`.
    pub fn hit_test_col_header(&self, vx: f64, vy: f64) -> Option<usize> {
        hit_test::hit_test_col_header(vx, vy, &self.model, self.viewport.scroll_x)
    }
}
