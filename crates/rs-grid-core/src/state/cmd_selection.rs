use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    selection::CellCoord,
};

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

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, selection::CellCoord, state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let rows = (0..10)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn select_cell_not_selectable_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SetSelectable(false));
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn extend_selection_not_selectable_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::SetSelectable(false));
        // extend while not selectable — must not change selection
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 5, col: 2 }));
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn extend_row_selection_without_anchor_sets_focus() {
        let mut s = make_state();
        // No prior selection — anchor is None.
        s.apply(GridCommand::ExtendRowSelection(3));
        // Focus should be set even with no prior anchor.
        assert!(s.selection.focus.is_some());
    }

    #[test]
    fn extend_row_selection_clamps_anchor_col_to_zero() {
        let mut s = make_state();
        // Anchor on col 2 — ExtendRowSelection must clamp anchor col to 0.
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 2 }));
        s.apply(GridCommand::ExtendRowSelection(4));
        let anchor = s.selection.anchor.expect("anchor must be set");
        assert_eq!(anchor.col, 0);
    }

    #[test]
    fn extend_col_selection_without_anchor_sets_focus() {
        let mut s = make_state();
        s.apply(GridCommand::ExtendColSelection(1));
        assert!(s.selection.focus.is_some());
    }

    #[test]
    fn extend_col_selection_clamps_anchor_row_to_zero() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 5, col: 0 }));
        s.apply(GridCommand::ExtendColSelection(2));
        let anchor = s.selection.anchor.expect("anchor must be set");
        assert_eq!(anchor.row, 0);
    }

    #[test]
    fn move_selection_without_existing_selection_is_noop() {
        let mut s = make_state();
        // No selection — MoveSelection should not panic.
        s.apply(GridCommand::MoveSelection {
            delta_row: 1,
            delta_col: 0,
            extend: false,
        });
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn move_selection_extend_true_extends_range() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: 2,
            delta_col: 1,
            extend: true,
        });
        // After extend the anchor stays at (0,0) and focus moves.
        let anchor = s.selection.anchor.expect("anchor must be set");
        let focus = s.selection.focus.expect("focus must be set");
        assert_eq!(anchor.row, 0);
        assert_eq!(focus.row, 2);
        assert_eq!(focus.col, 1);
    }

    #[test]
    fn move_selection_clamps_to_bounds() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 9, col: 2 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: 100,
            delta_col: 100,
            extend: false,
        });
        let focus = s.selection.focus.expect("focus must be set");
        assert_eq!(focus.row, 9);
        assert_eq!(focus.col, 2);
    }
}
