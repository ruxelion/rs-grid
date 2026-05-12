use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    selection::CellCoord,
    undo::UndoEntry,
};

impl GridState {
    pub(super) fn cmd_clipboard(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::CopySelection => {
                match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e) => CommandOutput::CopyError(e),
                }
            }
            GridCommand::CutSelection => {
                let result = match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e) => return CommandOutput::CopyError(e),
                };
                if let Some((tl, br)) = self.selection.range() {
                    let mut old_cells = Vec::new();
                    for r in tl.row..=br.row {
                        for ci in tl.col..=br.col {
                            let key = self.model.columns[ci].key.clone();
                            let old = self.model.get_cell(r, &key);
                            old_cells.push((r, key.clone(), old));
                            self.model.set_cell(r, key, String::new());
                        }
                    }
                    self.history.push(UndoEntry::SetCells(old_cells));
                }
                result
            }
            GridCommand::PasteAt { text } => {
                let sel_range = self.selection.range();
                // Use the top-left of the normalized selection so
                // that paste always starts at the visual top-left,
                // regardless of selection direction.
                let origin = sel_range
                    .as_ref()
                    .map(|(tl, _)| tl.clone())
                    .or_else(|| self.selection.anchor.clone())
                    .or_else(|| self.selection.focus.clone());
                if let Some(orig) = origin {
                    let clip = crate::selection::parse_tsv(&text);
                    if clip.is_empty() || clip[0].is_empty() {
                        return CommandOutput::None;
                    }
                    let col_count = self.model.columns.len();
                    let row_count = self.model.display_row_count();
                    let clip_rows = clip.len();
                    let clip_cols = clip[0].len();

                    // Determine target rectangle.
                    // Single-cell selection → paste clipboard
                    // as-is. Multi-cell selection → tile
                    // clipboard to fill the target range
                    // (Excel-like behavior).
                    let (target_rows, target_cols) = match sel_range {
                        Some((ref tl, ref br))
                            if tl.row != br.row || tl.col != br.col =>
                        {
                            let tr = (br.row - tl.row + 1) as usize;
                            let tc = br.col - tl.col + 1;
                            (tr, tc)
                        }
                        _ => (clip_rows, clip_cols),
                    };

                    let mut old_cells = Vec::new();
                    for dr in 0..target_rows {
                        let r = orig.row.saturating_add(dr as u64);
                        if r >= row_count {
                            break;
                        }
                        let src_row = &clip[dr % clip_rows];
                        for dc in 0..target_cols {
                            let c = orig.col + dc;
                            if c >= col_count {
                                break;
                            }
                            let val = &src_row[dc % clip_cols];
                            let key = self.model.columns[c].key.clone();
                            let old = self.model.get_cell(r, &key);
                            old_cells.push((r, key.clone(), old));
                            self.model.set_cell(r, key, val.clone());
                        }
                    }
                    if !old_cells.is_empty() {
                        self.history.push(UndoEntry::SetCells(old_cells));
                    }
                    // Update selection to cover pasted area.
                    let last_r =
                        (orig.row + target_rows as u64 - 1).min(row_count - 1);
                    let last_c =
                        (orig.col + target_cols - 1).min(col_count - 1);
                    self.selection.anchor = Some(CellCoord {
                        row: orig.row,
                        col: orig.col,
                    });
                    self.selection.focus = Some(CellCoord {
                        row: last_r,
                        col: last_c,
                    });
                }
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_clipboard"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef,
        commands::{CommandOutput, GridCommand},
        model::GridModel,
        row::RowRecord,
        selection::CellCoord,
        state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows = (0..4)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn copy_selection_returns_tsv() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        let out = s.apply(GridCommand::CopySelection);
        assert!(matches!(out, CommandOutput::CopyText(_)));
    }

    #[test]
    fn paste_without_selection_is_noop() {
        let mut s = make_state();
        // No anchor, no focus → paste must not panic or modify cells.
        let out = s.apply(GridCommand::PasteAt {
            text: "new\n".into(),
        });
        assert!(matches!(out, CommandOutput::None));
    }

    #[test]
    fn paste_at_anchor_updates_cell() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::PasteAt { text: "X\n".into() });
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("X"));
    }

    #[test]
    fn paste_tiling_fills_selection() {
        let mut s = make_state();
        // Select a 2×2 range, paste a single cell → tiled to fill.
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        s.apply(GridCommand::PasteAt { text: "Z\n".into() });
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("Z"));
        assert_eq!(s.model.get_cell(1, "a").as_deref(), Some("Z"));
        assert_eq!(s.model.get_cell(0, "b").as_deref(), Some("Z"));
        assert_eq!(s.model.get_cell(1, "b").as_deref(), Some("Z"));
    }

    #[test]
    fn cut_clears_cells_and_returns_tsv() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        let out = s.apply(GridCommand::CutSelection);
        assert!(matches!(out, CommandOutput::CopyText(_)));
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some(""));
    }
}
