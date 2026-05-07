use crate::{
    commands::{CommandOutput, GridCommand},
    selection::CellCoord,
};

use super::GridState;

impl GridState {
    pub(super) fn cmd_selection(&mut self, cmd: GridCommand) -> CommandOutput {
        if !self.model.selectable {
            return CommandOutput::None;
        }
        match cmd {
            GridCommand::SelectCell(coord) => {
                self.selection.select_cell(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ExtendSelection(coord) => {
                self.selection.extend_to(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ClearSelection => {
                self.selection.clear();
                CommandOutput::None
            }
            GridCommand::SelectRow(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row, col: 0 });
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::ExtendRowSelection(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                // Clamp anchor column to 0 so the range always
                // spans all columns.
                if let Some(ref mut a) = self.selection.anchor {
                    a.col = 0;
                }
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::SelectCol(col) => {
                let last_row = self.model.display_row_count().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row: 0, col });
                self.selection.focus = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::ExtendColSelection(col) => {
                let last_row = self.model.display_row_count().saturating_sub(1);
                // Clamp anchor row to 0 so the range always
                // spans all rows.
                if let Some(ref mut a) = self.selection.anchor {
                    a.row = 0;
                }
                self.selection.focus = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::MoveSelection {
                delta_row,
                delta_col,
                extend,
            } => {
                let row_count = self.model.display_row_count();
                let col_count = self.model.columns.len();
                let base = self
                    .selection
                    .focus
                    .clone()
                    .or_else(|| self.selection.anchor.clone());
                if let Some(b) = base {
                    // Use i128 for the row arithmetic so a u64 row
                    // index near 2^63 cannot wrap when cast through
                    // i64. Columns stay in i64: usize counts are
                    // bounded well below 2^63 in practice.
                    let max_row = row_count.saturating_sub(1);
                    let new_row = ((b.row as i128) + (delta_row as i128))
                        .clamp(0, max_row as i128)
                        as u64;
                    let new_col = (b.col as i64 + delta_col)
                        .clamp(0, col_count.saturating_sub(1) as i64)
                        as usize;
                    if extend {
                        self.selection.extend_to(new_row, new_col);
                    } else {
                        self.selection.select_cell(new_row, new_col);
                    }
                }
                CommandOutput::None
            }
            _ => {
                debug_assert!(false, "cmd_selection: unsupported variant");
                CommandOutput::None
            }
        }
    }
}
