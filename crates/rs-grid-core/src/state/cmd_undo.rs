use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    undo::UndoEntry,
};

impl GridState {
    pub(super) fn cmd_undo(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::Undo => {
                if let Some(entry) = self.history.pop_undo() {
                    let redo = self.apply_undo_entry(&entry);
                    self.history.push_redo(redo);
                }
                CommandOutput::None
            }
            GridCommand::Redo => {
                if let Some(entry) = self.history.pop_redo() {
                    let undo = self.apply_undo_entry(&entry);
                    self.history.push_undo_keep_redo(undo);
                }
                CommandOutput::None
            }
            _ => {
                debug_assert!(false, "cmd_undo: unsupported variant");
                CommandOutput::None
            }
        }
    }

    /// Apply an undo entry and return the inverse entry for redo.
    pub(super) fn apply_undo_entry(&mut self, entry: &UndoEntry) -> UndoEntry {
        match entry {
            UndoEntry::SetCell {
                row,
                col_key,
                old_value,
            } => {
                let current = self.model.get_cell(*row, col_key);
                if let Some(v) = old_value {
                    self.model.set_cell(*row, col_key, v.clone());
                } else {
                    // Remove patch to restore datasource value.
                    let physical = self.model.logical_to_physical(*row);
                    self.model.patches.remove(&(physical, col_key.clone()));
                }
                UndoEntry::SetCell {
                    row: *row,
                    col_key: col_key.clone(),
                    old_value: current,
                }
            }
            UndoEntry::SetCells(cells) => {
                let mut inverse = Vec::with_capacity(cells.len());
                for (row, col_key, old_value) in cells {
                    let current = self.model.get_cell(*row, col_key);
                    if let Some(v) = old_value {
                        self.model.set_cell(*row, col_key, v.clone());
                    } else {
                        let physical = self.model.logical_to_physical(*row);
                        self.model.patches.remove(&(physical, col_key.clone()));
                    }
                    inverse.push((*row, col_key.clone(), current));
                }
                UndoEntry::SetCells(inverse)
            }
            UndoEntry::ResizeColumn {
                col_idx,
                old_width,
                old_flex,
            } => {
                let cur_width = self.model.columns[*col_idx].width;
                let cur_flex = self.model.columns[*col_idx].flex;
                self.model.columns[*col_idx].width = *old_width;
                self.model.columns[*col_idx].flex = *old_flex;
                if old_flex.is_some() {
                    self.model.recalculate_flex_widths(self.viewport.width);
                }
                self.model.rebuild_offsets();
                UndoEntry::ResizeColumn {
                    col_idx: *col_idx,
                    old_width: cur_width,
                    old_flex: cur_flex,
                }
            }
            UndoEntry::MoveColumn { from_idx, to_idx } => {
                let col = self.model.columns.remove(*from_idx);
                self.model.columns.insert(*to_idx, col);
                self.model.rebuild_offsets();
                UndoEntry::MoveColumn {
                    from_idx: *to_idx,
                    to_idx: *from_idx,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows = (0..5)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("v{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn undo_on_empty_history_is_noop() {
        let mut s = make_state();
        // No prior edits — Undo must not panic.
        s.apply(GridCommand::Undo);
    }

    #[test]
    fn redo_on_empty_history_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::Redo);
    }

    #[test]
    fn undo_restores_previous_value() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "edited".into(),
        });
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("edited"));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("v0"));
    }

    #[test]
    fn redo_reapplies_after_undo() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "new".into(),
        });
        s.apply(GridCommand::Undo);
        s.apply(GridCommand::Redo);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("new"));
    }

    #[test]
    fn undo_cell_with_no_prior_value_removes_patch() {
        let mut s = make_state();
        // Set a cell that had no initial value (col "b" has no data).
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "b".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "b".into(),
            value: "added".into(),
        });
        assert!(s.model.get_cell(0, "b").is_some());
        s.apply(GridCommand::Undo);
        // Original state had no value for "b" — patch should be removed.
        assert!(s.model.get_cell(0, "b").is_none());
    }
}
