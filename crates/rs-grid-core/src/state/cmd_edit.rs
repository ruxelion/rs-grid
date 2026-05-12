use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    edit::EditCell,
    undo::UndoEntry,
};

impl GridState {
    pub(super) fn cmd_edit(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::StartEdit { row, col_key } => {
                // Respect grid-wide and per-column editable flags.
                let col_editable = self
                    .model
                    .columns
                    .iter()
                    .find(|c| c.key == col_key)
                    .is_none_or(|c| c.editable);
                if !self.model.editable || !col_editable {
                    return CommandOutput::None;
                }
                let initial_value =
                    self.model.get_cell(row, &col_key).unwrap_or_default();
                self.edit = Some(EditCell {
                    row,
                    col_key,
                    initial_value,
                });
                CommandOutput::None
            }
            GridCommand::CommitEdit {
                row,
                col_key,
                value,
            } => {
                if self
                    .edit
                    .as_ref()
                    .is_some_and(|e| e.row == row && e.col_key == col_key)
                {
                    let old_value = self.model.get_cell(row, &col_key);
                    self.model.set_cell(row, &col_key, value);
                    self.edit = None;
                    self.history.push(UndoEntry::SetCell {
                        row,
                        col_key,
                        old_value,
                    });
                }
                CommandOutput::None
            }
            GridCommand::CancelEdit => {
                self.edit = None;
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_edit"),
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
            ColumnDef::new("ro", "ReadOnly", 100.0).read_only(),
        ];
        let rows = (0..3)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("val{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn start_edit_sets_edit_cell() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        assert!(s.edit.is_some());
        let e = s.edit.as_ref().expect("edit should be set");
        assert_eq!(e.row, 0);
        assert_eq!(e.col_key, "a");
        assert_eq!(e.initial_value, "val0");
    }

    #[test]
    fn start_edit_read_only_column_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "ro".into(),
        });
        assert!(s.edit.is_none());
    }

    #[test]
    fn start_edit_grid_not_editable_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SetEditable(false));
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        assert!(s.edit.is_none());
    }

    #[test]
    fn cancel_edit_clears_edit() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CancelEdit);
        assert!(s.edit.is_none());
    }

    #[test]
    fn commit_edit_without_active_edit_is_noop() {
        let mut s = make_state();
        // No StartEdit — CommitEdit should not panic.
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "new".into(),
        });
        assert!(s.edit.is_none());
    }

    #[test]
    fn commit_edit_wrong_row_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        // Commit for a different row — should not apply.
        s.apply(GridCommand::CommitEdit {
            row: 1,
            col_key: "a".into(),
            value: "new".into(),
        });
        // Edit remains active because the commit didn't match.
        assert!(s.edit.is_some());
    }
}
