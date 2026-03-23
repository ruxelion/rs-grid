/// A reversible action stored in the undo/redo stack.
#[derive(Debug, Clone)]
pub(crate) enum UndoEntry {
    /// Restore a single cell to its previous value.
    SetCell {
        row: u64,
        col_key: String,
        old_value: Option<String>,
    },
    /// Restore multiple cells (batch paste/cut).
    SetCells(Vec<(u64, String, Option<String>)>),
    /// Restore a column width.
    ResizeColumn { col_idx: usize, old_width: f64 },
    /// Reverse a column move.
    MoveColumn { from_idx: usize, to_idx: usize },
}

const MAX_UNDO: usize = 100;

/// Undo/redo stack with a fixed capacity.
///
/// All mutations to the grid that should be reversible push
/// an [`UndoEntry`] via [`UndoHistory::push`]. Undo pops from
/// the undo stack and pushes the inverse onto the redo stack;
/// redo does the reverse.
#[derive(Debug, Default)]
pub(crate) struct UndoHistory {
    pub(crate) undo_stack: Vec<UndoEntry>,
    pub(crate) redo_stack: Vec<UndoEntry>,
}

impl UndoHistory {
    /// Push a new undo entry, capping the stack at `MAX_UNDO`.
    /// Clears the redo stack (new action breaks the redo chain).
    pub(crate) fn push(&mut self, entry: UndoEntry) {
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    /// Pop the top undo entry, if any.
    pub(crate) fn pop_undo(&mut self) -> Option<UndoEntry> {
        self.undo_stack.pop()
    }

    /// Push an entry onto the redo stack (used after applying
    /// an undo to store its inverse).
    pub(crate) fn push_redo(&mut self, entry: UndoEntry) {
        self.redo_stack.push(entry);
    }

    /// Pop the top redo entry, if any.
    pub(crate) fn pop_redo(&mut self) -> Option<UndoEntry> {
        self.redo_stack.pop()
    }

    /// Push directly onto the undo stack without clearing the
    /// redo stack. Used when re-applying a redo entry.
    pub(crate) fn push_undo_keep_redo(&mut self, entry: UndoEntry) {
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(entry);
    }
}
