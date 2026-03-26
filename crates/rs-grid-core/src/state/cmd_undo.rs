use crate::{
    commands::{CommandOutput, GridCommand},
    undo::UndoEntry,
};

use super::GridState;

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
            _ => unreachable!(),
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
            UndoEntry::ResizeColumn { col_idx, old_width } => {
                let current_width = self.model.columns[*col_idx].width;
                self.model.columns[*col_idx].width = *old_width;
                self.model.rebuild_offsets();
                UndoEntry::ResizeColumn {
                    col_idx: *col_idx,
                    old_width: current_width,
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
