use crate::{
    commands::{CommandOutput, GridCommand},
    edit::EditCell,
    undo::UndoEntry,
};

use super::GridState;

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
                    .map_or(true, |c| c.editable);
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
            _ => unreachable!(),
        }
    }
}
