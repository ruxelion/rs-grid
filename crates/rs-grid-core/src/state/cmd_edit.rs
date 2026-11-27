use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    edit::EditCell,
    undo::UndoEntry,
    validation::InvalidEditMode,
};

impl GridState {
    pub(super) fn cmd_edit(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::StartEdit { row, col_key } => {
                // `is_cell_editable` already folds in grid-wide,
                // static per-column, and dynamic per-cell flags.
                let col_editable = self
                    .model
                    .columns
                    .iter()
                    .find(|c| c.key == col_key)
                    .is_none_or(|c| c.is_cell_editable(row, &self.model));
                if !col_editable {
                    return CommandOutput::None;
                }
                let col_idx =
                    self.model.columns.iter().position(|c| c.key == col_key);
                let initial_value =
                    self.model.get_cell(row, &col_key).unwrap_or_default();
                self.edit = Some(EditCell {
                    row,
                    col_key,
                    col_idx: col_idx.unwrap_or(0),
                    initial_value,
                    validation_error: None,
                });
                // Move the selection to the edited cell so the
                // highlight always follows the active editor.
                if self.model.selectable
                    && let Some(col) = col_idx
                {
                    self.selection.select_cell(row, col);
                }
                CommandOutput::None
            }
            GridCommand::ValidateEdit { value } => {
                let Some(edit) = self.edit.as_mut() else {
                    return CommandOutput::None;
                };
                let result = self
                    .model
                    .columns
                    .iter()
                    .find(|c| c.key == edit.col_key)
                    .map(|c| c.validate_value(&value));
                edit.validation_error = match result {
                    Some(Err(msg)) => Some(msg),
                    _ => None,
                };
                CommandOutput::None
            }
            GridCommand::CommitEdit {
                row,
                col_key,
                value,
            } => {
                if !self
                    .edit
                    .as_ref()
                    .is_some_and(|e| e.row == row && e.col_key == col_key)
                {
                    return CommandOutput::None;
                }
                let col = self.model.columns.iter().find(|c| c.key == col_key);
                // Editability can change between `StartEdit` and
                // `CommitEdit` (e.g. a cross-column predicate whose
                // dependency was edited via another command in the
                // meantime) — re-check rather than trusting the
                // `StartEdit`-time result.
                if col.is_some_and(|c| !c.is_cell_editable(row, &self.model)) {
                    let message = "Cell is no longer editable".to_string();
                    return match self.model.invalid_edit_mode {
                        InvalidEditMode::Revert => {
                            self.edit = None;
                            CommandOutput::ValidationError {
                                row,
                                col_key,
                                message,
                            }
                        }
                        InvalidEditMode::Block => {
                            if let Some(edit) = self.edit.as_mut() {
                                edit.validation_error = Some(message.clone());
                            }
                            CommandOutput::ValidationError {
                                row,
                                col_key,
                                message,
                            }
                        }
                    };
                }
                let validation = col.map(|c| c.validate_value(&value));
                if let Some(Err(message)) = validation {
                    return match self.model.invalid_edit_mode {
                        InvalidEditMode::Revert => {
                            self.edit = None;
                            CommandOutput::ValidationError {
                                row,
                                col_key,
                                message,
                            }
                        }
                        InvalidEditMode::Block => {
                            if let Some(edit) = self.edit.as_mut() {
                                edit.validation_error = Some(message.clone());
                            }
                            CommandOutput::ValidationError {
                                row,
                                col_key,
                                message,
                            }
                        }
                    };
                }
                let old_value = self.model.get_cell(row, &col_key);
                self.model.set_cell(row, &col_key, value);
                self.edit = None;
                self.history.push(UndoEntry::SetCell {
                    row,
                    col_key,
                    old_value,
                });
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

    // ── validation ────────────────────────────────────────

    fn make_validated_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0).required(),
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
    fn commit_edit_invalid_revert_mode_drops_edit() {
        use crate::commands::CommandOutput;
        let mut s = make_validated_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        let out = s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "".into(),
        });
        assert!(s.edit.is_none(), "revert mode should end the edit session");
        assert_eq!(s.model.get_cell(0, "a"), Some("val0".into()));
        assert!(matches!(out, CommandOutput::ValidationError { .. }));
    }

    #[test]
    fn commit_edit_invalid_block_mode_keeps_edit() {
        use crate::{commands::CommandOutput, validation::InvalidEditMode};
        let mut s = make_validated_state();
        s.apply(GridCommand::SetInvalidEditMode(InvalidEditMode::Block));
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        let out = s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "".into(),
        });
        let edit = s.edit.as_ref().expect("block mode keeps the edit active");
        assert!(edit.validation_error.is_some());
        assert_eq!(s.model.get_cell(0, "a"), Some("val0".into()));
        assert!(matches!(out, CommandOutput::ValidationError { .. }));
    }

    #[test]
    fn commit_edit_valid_clears_validation() {
        let mut s = make_validated_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "new".into(),
        });
        assert!(s.edit.is_none());
        assert_eq!(s.model.get_cell(0, "a"), Some("new".into()));
    }

    #[test]
    fn validate_edit_sets_error_without_committing() {
        let mut s = make_validated_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::ValidateEdit { value: "".into() });
        assert!(s.edit.as_ref().unwrap().validation_error.is_some());
        // Value is not committed by ValidateEdit.
        assert_eq!(s.model.get_cell(0, "a"), Some("val0".into()));
    }

    #[test]
    fn validate_edit_clears_error_when_valid() {
        let mut s = make_validated_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::ValidateEdit { value: "".into() });
        assert!(s.edit.as_ref().unwrap().validation_error.is_some());
        s.apply(GridCommand::ValidateEdit { value: "ok".into() });
        assert!(s.edit.as_ref().unwrap().validation_error.is_none());
    }

    #[test]
    fn validate_edit_without_active_edit_is_noop() {
        let mut s = make_validated_state();
        s.apply(GridCommand::ValidateEdit { value: "".into() });
        assert!(s.edit.is_none());
    }

    // ── per-cell editable predicate ──────────────────────────

    fn make_predicate_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0).editable_when(|row, _| row != 0),
            ColumnDef::new("ro", "ReadOnly", 100.0)
                .read_only()
                .editable_when(|_, _| panic!("predicate must not be called")),
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
    fn start_edit_predicate_false_is_noop() {
        let mut s = make_predicate_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        assert!(s.edit.is_none());
    }

    #[test]
    fn start_edit_predicate_true_allows_edit() {
        let mut s = make_predicate_state();
        s.apply(GridCommand::StartEdit {
            row: 1,
            col_key: "a".into(),
        });
        assert!(s.edit.is_some());
    }

    #[test]
    fn start_edit_static_false_short_circuits_predicate() {
        let mut s = make_predicate_state();
        // The "ro" column's predicate panics if called — this must
        // not panic, proving the static `editable=false` check
        // short-circuits before the predicate is ever evaluated.
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "ro".into(),
        });
        assert!(s.edit.is_none());
    }

    #[test]
    fn start_edit_predicate_reads_cross_column_value() {
        let cols = vec![
            ColumnDef::new("status", "Status", 100.0),
            ColumnDef::new("notes", "Notes", 100.0).editable_when(
                |row, model| {
                    model.get_cell(row, "status").as_deref() != Some("locked")
                },
            ),
        ];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("status", "locked");
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("status", "open");
                r
            },
        ];
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);

        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "notes".into(),
        });
        assert!(s.edit.is_none(), "row 0 is locked via the status column");

        s.apply(GridCommand::StartEdit {
            row: 1,
            col_key: "notes".into(),
        });
        assert!(s.edit.is_some(), "row 1 is not locked");
    }

    #[test]
    fn commit_edit_rejects_cell_locked_mid_edit() {
        let cols = vec![
            ColumnDef::new("status", "Status", 100.0),
            ColumnDef::new("notes", "Notes", 100.0).editable_when(
                |row, model| {
                    model.get_cell(row, "status").as_deref() != Some("locked")
                },
            ),
        ];
        let rows = vec![{
            let mut r = RowRecord::new(0);
            r.set("status", "open");
            r
        }];
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);

        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "notes".into(),
        });
        assert!(s.edit.is_some(), "row 0 starts unlocked");

        // Simulate another command changing the predicate's
        // dependency while the edit session is still open.
        s.model.set_cell(0, "status", "locked".to_string());

        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "notes".into(),
            value: "new value".into(),
        });

        assert_eq!(
            s.model.get_cell(0, "notes").unwrap_or_default(),
            "",
            "the write must not go through on a now-locked cell"
        );
        assert!(s.edit.is_none(), "Revert mode drops the edit session");
    }
}
