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
    /// Restore a column width (and flex factor).
    ResizeColumn {
        col_idx: usize,
        old_width: f64,
        old_flex: Option<f64>,
    },
    /// Reverse a column move.
    MoveColumn { from_idx: usize, to_idx: usize },
}

/// Maximum number of entries in the undo stack. Once
/// reached, the oldest entry is evicted (FIFO).
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy() -> UndoEntry {
        UndoEntry::SetCell {
            row: 0,
            col_key: "a".into(),
            old_value: None,
        }
    }

    fn numbered(n: u64) -> UndoEntry {
        UndoEntry::SetCell {
            row: n,
            col_key: "a".into(),
            old_value: None,
        }
    }

    #[test]
    fn push_clears_redo_stack() {
        let mut h = UndoHistory::default();
        h.push_redo(dummy());
        assert_eq!(h.redo_stack.len(), 1);
        h.push(dummy());
        assert!(h.redo_stack.is_empty());
    }

    #[test]
    fn push_fifo_eviction_at_capacity() {
        let mut h = UndoHistory::default();
        for i in 0..=MAX_UNDO as u64 {
            h.push(numbered(i));
        }
        assert_eq!(h.undo_stack.len(), MAX_UNDO);
        // First entry (row=0) was evicted; oldest is row=1.
        match &h.undo_stack[0] {
            UndoEntry::SetCell { row, .. } => assert_eq!(*row, 1),
            _ => panic!("expected SetCell"),
        }
    }

    #[test]
    fn push_undo_keep_redo_preserves_redo() {
        let mut h = UndoHistory::default();
        h.push_redo(dummy());
        h.push_undo_keep_redo(dummy());
        assert_eq!(h.redo_stack.len(), 1);
    }

    #[test]
    fn push_undo_keep_redo_also_evicts() {
        let mut h = UndoHistory::default();
        for i in 0..=MAX_UNDO as u64 {
            h.push_undo_keep_redo(numbered(i));
        }
        assert_eq!(h.undo_stack.len(), MAX_UNDO);
        match &h.undo_stack[0] {
            UndoEntry::SetCell { row, .. } => assert_eq!(*row, 1),
            _ => panic!("expected SetCell"),
        }
    }

    #[test]
    fn pop_undo_empty_returns_none() {
        let mut h = UndoHistory::default();
        assert!(h.pop_undo().is_none());
    }

    #[test]
    fn pop_redo_empty_returns_none() {
        let mut h = UndoHistory::default();
        assert!(h.pop_redo().is_none());
    }
}
