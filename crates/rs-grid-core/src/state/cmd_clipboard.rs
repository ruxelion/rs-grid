use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    model::GridModel,
    selection::CellCoord,
    undo::UndoEntry,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// `(row, col_key, old_value)` triples ready for `UndoEntry::SetCells`.
type UndoCells = Vec<(u64, String, Option<String>)>;

/// Returns `true` when the selection spans every row of the model —
/// i.e. the user clicked a column header rather than dragging a cell range.
/// Full-column selections carry positional/sort intent, not bulk data.
fn is_full_col_sel(tl: &CellCoord, br: &CellCoord, row_count: u64) -> bool {
    row_count > 0 && tl.row == 0 && br.row == row_count - 1
}

/// Builds a single-row TSV of column header labels for columns
/// `col_start..=col_end`.
fn header_tsv(model: &GridModel, col_start: usize, col_end: usize) -> String {
    let row: Vec<&str> = (col_start..=col_end)
        .map(|ci| model.columns[ci].label.as_str())
        .collect();
    format!("{}\n", row.join("\t"))
}

// ── command handler ──────────────────────────────────────────────────────────

impl GridState {
    /// Clears every editable cell in `tl..=br` to an empty string,
    /// skipping (not writing) any cell that's locked or whose empty
    /// value would fail validation (e.g. `.required()`) — same
    /// silent-skip precedent as `PasteAt`. Returns the `(row, col_key,
    /// old_value)` triples for the undo history alongside the
    /// coordinates actually cleared; shared by `CutSelection` (clear
    /// step, after copying — ignores the coordinates, it has no flash
    /// feedback) and `ClearCells` (Delete/Backspace, no clipboard
    /// involved, reports the coordinates as `CommandOutput::
    /// CellsCleared` for success-flash feedback).
    fn clear_cell_range(
        &mut self,
        tl: &CellCoord,
        br: &CellCoord,
    ) -> (UndoCells, Vec<CellCoord>, Vec<CellCoord>) {
        let mut old_cells = Vec::new();
        let mut cleared = Vec::new();
        let mut skipped = Vec::new();
        for r in tl.row..=br.row {
            for ci in tl.col..=br.col {
                let coord = CellCoord { row: r, col: ci };
                if !self.model.columns[ci].is_cell_editable(r, &self.model) {
                    skipped.push(coord);
                    continue;
                }
                if self.model.columns[ci].validate_value("").is_err() {
                    skipped.push(coord);
                    continue;
                }
                let key = self.model.columns[ci].key.clone();
                let old = self.model.get_cell(r, &key);
                old_cells.push((r, key.clone(), old));
                self.model.set_cell(r, key, String::new());
                cleared.push(coord);
            }
        }
        (old_cells, cleared, skipped)
    }

    pub(super) fn cmd_clipboard(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            // ── Copy ─────────────────────────────────────────────────────────
            GridCommand::CopySelection => {
                if let Some((tl, br)) = self.selection.range() {
                    let row_count = self.model.display_row_count();
                    if is_full_col_sel(&tl, &br, row_count) {
                        // Full-column selection: copy the column header
                        // labels, not the cell data. Copying billions of
                        // cells into the clipboard would OOM the tab.
                        return CommandOutput::CopyText(header_tsv(
                            &self.model,
                            tl.col,
                            br.col,
                        ));
                    }
                }
                match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e) => CommandOutput::CopyError(e),
                }
            }

            // ── Cut ──────────────────────────────────────────────────────────
            GridCommand::CutSelection => {
                if let Some((tl, br)) = self.selection.range() {
                    let row_count = self.model.display_row_count();
                    if is_full_col_sel(&tl, &br, row_count) {
                        // Full-column selection: cutting a column definition
                        // makes no sense — behave like copy.
                        return CommandOutput::CopyText(header_tsv(
                            &self.model,
                            tl.col,
                            br.col,
                        ));
                    }
                }
                // Normal cut: copy data then clear cells.
                let text = match self.selection.to_tsv(&self.model) {
                    Ok(text) => text,
                    Err(e) => return CommandOutput::CopyError(e),
                };
                let mut skipped = Vec::new();
                if let Some((tl, br)) = self.selection.range() {
                    let (old_cells, _cleared, cell_skipped) =
                        self.clear_cell_range(&tl, &br);
                    if !old_cells.is_empty() {
                        self.history.push(UndoEntry::SetCells(old_cells));
                    }
                    skipped = cell_skipped;
                }
                CommandOutput::CutApplied { text, skipped }
            }

            // ── Clear (Delete/Backspace) ────────────────────────────────────
            GridCommand::ClearCells => {
                let Some((tl, br)) = self.selection.range() else {
                    return CommandOutput::None;
                };
                let row_count = self.model.display_row_count();
                if is_full_col_sel(&tl, &br, row_count) {
                    // Same rationale as CutSelection's full-column
                    // guard: a header click carries positional intent,
                    // not "clear this entire column of potentially
                    // billions of rows".
                    return CommandOutput::None;
                }
                let (old_cells, cleared, _skipped) =
                    self.clear_cell_range(&tl, &br);
                if old_cells.is_empty() {
                    return CommandOutput::None;
                }
                self.history.push(UndoEntry::SetCells(old_cells));
                CommandOutput::CellsCleared { cells: cleared }
            }

            // ── Paste ────────────────────────────────────────────────────────
            GridCommand::PasteAt { text } => {
                let sel_range = self.selection.range();
                // Use the top-left of the normalized selection so that
                // paste always starts at the visual top-left.
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
                    // Single-cell or full-column selection → paste clipboard
                    // as-is from the anchor (row 0 for column selections).
                    // Multi-cell range → tile clipboard to fill (Excel-like).
                    //
                    // `tr_u64` is computed as u64 and capped before
                    // `as usize` — prevents WASM32 overflow on billion-row
                    // column selections.
                    let (target_rows, target_cols) = match sel_range {
                        Some((ref tl, ref br))
                            if tl.row != br.row || tl.col != br.col =>
                        {
                            let tr_u64 = (br.row - tl.row + 1)
                                .min(crate::selection::MAX_COPY_ROWS);
                            let tc = br.col - tl.col + 1;
                            (tr_u64 as usize, tc)
                        }
                        _ => (clip_rows, clip_cols),
                    };

                    let mut old_cells = Vec::new();
                    let mut written_cells = Vec::new();
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
                            if !self.model.columns[c]
                                .is_cell_editable(r, &self.model)
                            {
                                continue;
                            }
                            let val = &src_row[dc % clip_cols];
                            if self.model.columns[c]
                                .validate_value(val)
                                .is_err()
                            {
                                continue;
                            }
                            let key = self.model.columns[c].key.clone();
                            let old = self.model.get_cell(r, &key);
                            old_cells.push((r, key.clone(), old));
                            self.model.set_cell(r, key, val.clone());
                            written_cells.push(CellCoord { row: r, col: c });
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
                    return CommandOutput::PasteApplied {
                        cells: written_cells,
                    };
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
            ColumnDef::new("a", "Name", 100.0),
            ColumnDef::new("b", "Email", 150.0),
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

    // ── CopySelection — cell range ─────────────────────────────────────────

    #[test]
    fn copy_single_cell_returns_tsv() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        let out = s.apply(GridCommand::CopySelection);
        assert!(matches!(out, CommandOutput::CopyText(_)));
        if let CommandOutput::CopyText(t) = out {
            assert_eq!(t, "a0\n");
        }
    }

    #[test]
    fn copy_range_returns_tsv() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        let out = s.apply(GridCommand::CopySelection);
        if let CommandOutput::CopyText(t) = out {
            assert_eq!(t, "a0\tb0\na1\tb1\n");
        } else {
            panic!("expected CopyText");
        }
    }

    // ── CopySelection — full-column selection ──────────────────────────────

    #[test]
    fn copy_full_col_returns_header_not_data() {
        let mut s = make_state();
        // SelectCol selects all rows for a given column index.
        s.apply(GridCommand::SelectCol(0));
        let out = s.apply(GridCommand::CopySelection);
        if let CommandOutput::CopyText(t) = out {
            assert_eq!(t, "Name\n", "expected header label, got: {t:?}");
        } else {
            panic!("expected CopyText");
        }
    }

    #[test]
    fn copy_full_multi_col_returns_headers() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCol(0));
        s.apply(GridCommand::ExtendColSelection(1));
        let out = s.apply(GridCommand::CopySelection);
        if let CommandOutput::CopyText(t) = out {
            assert_eq!(t, "Name\tEmail\n");
        } else {
            panic!("expected CopyText");
        }
    }

    // ── CutSelection — full-column selection ───────────────────────────────

    #[test]
    fn cut_full_col_returns_header_does_not_clear() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCol(0));
        let out = s.apply(GridCommand::CutSelection);
        // Should return header, not clear any cells.
        if let CommandOutput::CopyText(t) = out {
            assert_eq!(t, "Name\n");
        } else {
            panic!("expected CopyText");
        }
        // Cell data must be untouched.
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("a0"));
        assert_eq!(s.model.get_cell(3, "a").as_deref(), Some("a3"));
    }

    // ── PasteAt ────────────────────────────────────────────────────────────

    #[test]
    fn paste_without_selection_is_noop() {
        let mut s = make_state();
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
        assert!(matches!(out, CommandOutput::CutApplied { .. }));
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some(""));
    }

    // ── locked cells ─────────────────────────────────────────────────────

    fn make_locked_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "Name", 100.0),
            ColumnDef::new("b", "Email", 150.0).read_only(),
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
    fn paste_at_skips_locked_cells() {
        let mut s = make_locked_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        s.apply(GridCommand::PasteAt {
            text: "X\tY\n".into(),
        });
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some("X"),
            "editable column receives the pasted value"
        );
        assert_eq!(
            s.model.get_cell(0, "b").as_deref(),
            Some("b0"),
            "read-only column keeps its original value"
        );
    }

    #[test]
    fn paste_at_reports_only_written_cells() {
        let mut s = make_locked_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::PasteAt {
            text: "X\tY\n".into(),
        });
        match out {
            CommandOutput::PasteApplied { cells } => {
                assert_eq!(
                    cells,
                    vec![CellCoord { row: 0, col: 0 }],
                    "only the editable column's cell is reported written, \
                     not the locked one"
                );
            }
            other => panic!("expected PasteApplied, got {other:?}"),
        }
    }

    #[test]
    fn cut_selection_skips_locked_cells_but_copies_everything() {
        let mut s = make_locked_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::CutSelection);
        if let CommandOutput::CutApplied { text, skipped } = out {
            assert_eq!(
                text, "a0\tb0\n",
                "the copy side is unaffected by editability"
            );
            assert_eq!(
                skipped,
                vec![CellCoord { row: 0, col: 1 }],
                "the locked cell is reported as skipped"
            );
        } else {
            panic!("expected CutApplied");
        }
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some(""),
            "editable column is cleared"
        );
        assert_eq!(
            s.model.get_cell(0, "b").as_deref(),
            Some("b0"),
            "read-only column is left untouched"
        );
    }

    // ── ClearCells (Delete/Backspace) ──────────────────────────────────────

    #[test]
    fn clear_cells_without_selection_is_noop() {
        let mut s = make_state();
        let out = s.apply(GridCommand::ClearCells);
        assert!(matches!(out, CommandOutput::None));
    }

    #[test]
    fn clear_cells_clears_selected_range() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        s.apply(GridCommand::ClearCells);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some(""));
        assert_eq!(s.model.get_cell(0, "b").as_deref(), Some(""));
        assert_eq!(s.model.get_cell(1, "a").as_deref(), Some(""));
        assert_eq!(s.model.get_cell(1, "b").as_deref(), Some(""));
    }

    #[test]
    fn clear_cells_does_not_touch_clipboard() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        let out = s.apply(GridCommand::ClearCells);
        assert!(
            matches!(out, CommandOutput::CellsCleared { .. }),
            "unlike CutSelection, ClearCells never returns CopyText"
        );
    }

    #[test]
    fn clear_cells_reports_cleared_coordinates() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        let out = s.apply(GridCommand::ClearCells);
        match out {
            CommandOutput::CellsCleared { cells } => {
                assert_eq!(
                    cells,
                    vec![
                        CellCoord { row: 0, col: 0 },
                        CellCoord { row: 0, col: 1 },
                        CellCoord { row: 1, col: 0 },
                        CellCoord { row: 1, col: 1 },
                    ]
                );
            }
            other => panic!("expected CellsCleared, got {other:?}"),
        }
    }

    #[test]
    fn clear_cells_reports_only_cleared_cells_when_some_are_skipped() {
        let mut s = make_locked_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::ClearCells);
        match out {
            CommandOutput::CellsCleared { cells } => {
                assert_eq!(
                    cells,
                    vec![CellCoord { row: 0, col: 0 }],
                    "only the editable column's cell is reported cleared, \
                     not the locked one"
                );
            }
            other => panic!("expected CellsCleared, got {other:?}"),
        }
    }

    #[test]
    fn clear_cells_skips_locked_cells() {
        let mut s = make_locked_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        s.apply(GridCommand::ClearCells);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some(""));
        assert_eq!(
            s.model.get_cell(0, "b").as_deref(),
            Some("b0"),
            "read-only column is left untouched"
        );
    }

    #[test]
    fn clear_cells_skips_cells_that_would_become_invalid() {
        let mut s = make_validated_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        s.apply(GridCommand::ClearCells);
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some("a0"),
            "required column keeps its value"
        );
        assert_eq!(s.model.get_cell(0, "b").as_deref(), Some(""));
    }

    #[test]
    fn clear_cells_on_full_column_selection_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCol(0));
        s.apply(GridCommand::ClearCells);
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some("a0"),
            "full-column selection is not bulk-cleared"
        );
        assert_eq!(s.model.get_cell(3, "a").as_deref(), Some("a3"));
    }

    #[test]
    fn clear_cells_is_undoable() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ClearCells);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some(""));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("a0"));
    }

    // ── validation ───────────────────────────────────────────────────────

    fn make_validated_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "Name", 100.0).required(),
            ColumnDef::new("b", "Email", 150.0),
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
    fn paste_at_skips_invalid_values() {
        let mut s = make_validated_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        // Empty string fails "a"'s `required()` rule; "b" has no rules.
        s.apply(GridCommand::PasteAt {
            text: "\tY\n".into(),
        });
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some("a0"),
            "the required column keeps its original value"
        );
        assert_eq!(
            s.model.get_cell(0, "b").as_deref(),
            Some("Y"),
            "the unvalidated column still receives the pasted value"
        );
    }

    #[test]
    fn paste_at_tiled_partial_validity() {
        let mut s = make_validated_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 0 }));
        // Tiled single-column paste: row 0 gets a valid value, row 1
        // gets a whitespace-only (invalid per `required()`) one.
        s.apply(GridCommand::PasteAt {
            text: "Z\n \n".into(),
        });
        assert_eq!(s.model.get_cell(0, "a").as_deref(), Some("Z"));
        assert_eq!(
            s.model.get_cell(1, "a").as_deref(),
            Some("a1"),
            "row 1's invalid pasted value is skipped, not written"
        );
    }

    #[test]
    fn cut_selection_skips_cells_that_would_become_invalid() {
        let mut s = make_validated_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::CutSelection);
        // The copy side is unaffected — the full original values are
        // still placed on the clipboard.
        if let CommandOutput::CutApplied { text, skipped } = out {
            assert_eq!(text, "a0\tb0\n");
            assert_eq!(
                skipped,
                vec![CellCoord { row: 0, col: 0 }],
                "the required cell is reported as skipped"
            );
        } else {
            panic!("expected CutApplied");
        }
        assert_eq!(
            s.model.get_cell(0, "a").as_deref(),
            Some("a0"),
            "required column keeps its value — clearing it would fail \
             validation"
        );
        assert_eq!(
            s.model.get_cell(0, "b").as_deref(),
            Some(""),
            "unvalidated column is cleared normally"
        );
    }

    #[test]
    fn paste_at_reports_only_valid_cells() {
        let mut s = make_validated_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        // Empty string fails "a"'s `required()` rule; "b" has no rules.
        let out = s.apply(GridCommand::PasteAt {
            text: "\tY\n".into(),
        });
        match out {
            CommandOutput::PasteApplied { cells } => {
                assert_eq!(
                    cells,
                    vec![CellCoord { row: 0, col: 1 }],
                    "only the unvalidated column's cell is reported \
                     written, not the one that failed validation"
                );
            }
            other => panic!("expected PasteApplied, got {other:?}"),
        }
    }
}
