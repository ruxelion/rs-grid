use crate::{
    commands::{CommandOutput, GridCommand},
    hit_test,
    model::GridModel,
    selection::{CellCoord, SelectionState},
    sort::{SortDir, SortState},
    viewport::ViewportState,
};

/// The complete mutable state of a grid instance.
#[derive(Debug)]
pub struct GridState {
    pub model: GridModel,
    pub viewport: ViewportState,
    pub selection: SelectionState,
    /// Row index currently under the mouse cursor, for hover highlighting.
    pub hovered_row: Option<u64>,
    /// Active sort column and direction (`None` = natural order).
    pub sort: Option<SortState>,
}

impl GridState {
    pub fn new(model: GridModel, viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            model,
            viewport: ViewportState::new(viewport_width, viewport_height),
            selection: SelectionState::default(),
            hovered_row: None,
            sort: None,
        }
    }

    /// Apply a command, mutating state in place.
    pub fn apply(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::SelectCell(coord) => {
                self.selection.select_cell(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ExtendSelection(coord) => {
                self.selection.extend_to(coord.row, coord.col);
                CommandOutput::None
            }
            GridCommand::ScrollTo { x, y } => {
                let rnw = self.model.row_number_width;
                let max_x = (self.model.total_width() - (self.viewport.width - rnw)).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
                CommandOutput::None
            }
            GridCommand::ScrollBy { dx, dy } => {
                let x = self.viewport.scroll_x + dx;
                let y = self.viewport.scroll_y + dy;
                let rnw = self.model.row_number_width;
                let max_x = (self.model.total_width() - (self.viewport.width - rnw)).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
                CommandOutput::None
            }
            GridCommand::Resize { width, height } => {
                self.viewport.width = width;
                self.viewport.height = height;
                CommandOutput::None
            }
            GridCommand::ClearSelection => {
                self.selection.clear();
                CommandOutput::None
            }
            GridCommand::CopySelection => {
                match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e)   => CommandOutput::CopyError(e),
                }
            }
            GridCommand::CutSelection => {
                let result = match self.selection.to_tsv(&self.model) {
                    Ok(text) => CommandOutput::CopyText(text),
                    Err(e)   => return CommandOutput::CopyError(e),
                };
                if let Some((tl, br)) = self.selection.range() {
                    for r in tl.row..=br.row {
                        for ci in tl.col..=br.col {
                            let key = self.model.columns[ci].key.clone();
                            self.model.set_cell(r, key, String::new());
                        }
                    }
                }
                result
            }
            GridCommand::PasteAt { text } => {
                let origin = self.selection.anchor.clone()
                    .or_else(|| self.selection.focus.clone());
                if let Some(orig) = origin {
                    let rows = crate::selection::parse_tsv(&text);
                    let col_count = self.model.columns.len();
                    let row_count = self.model.data.row_count();
                    for (dr, row_vals) in rows.iter().enumerate() {
                        let r = orig.row + dr as u64;
                        if r >= row_count { break; }
                        for (dc, val) in row_vals.iter().enumerate() {
                            let c = orig.col + dc;
                            if c >= col_count { break; }
                            let key = self.model.columns[c].key.clone();
                            self.model.set_cell(r, key, val.clone());
                        }
                    }
                    // Update selection to cover the pasted rectangle
                    if !rows.is_empty() && !rows[0].is_empty() {
                        let last_r = (orig.row + rows.len() as u64 - 1).min(row_count - 1);
                        let last_c = (orig.col + rows[0].len() - 1).min(col_count - 1);
                        self.selection.anchor = Some(CellCoord { row: orig.row, col: orig.col });
                        self.selection.focus  = Some(CellCoord { row: last_r,   col: last_c });
                    }
                }
                CommandOutput::None
            }
            GridCommand::SelectRow(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row, col: 0 });
                self.selection.focus  = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::ExtendRowSelection(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                // Clamp anchor column to 0 so the range always spans all columns.
                if let Some(ref mut a) = self.selection.anchor { a.col = 0; }
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::SelectCol(col) => {
                let last_row = self.model.data.row_count().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row: 0, col });
                self.selection.focus  = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::ExtendColSelection(col) => {
                let last_row = self.model.data.row_count().saturating_sub(1);
                // Clamp anchor row to 0 so the range always spans all rows.
                if let Some(ref mut a) = self.selection.anchor { a.row = 0; }
                self.selection.focus = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::SetHoveredRow(row) => {
                self.hovered_row = row;
                CommandOutput::None
            }
            GridCommand::ResizeColumn { col_idx, new_width } => {
                const MIN_COL_WIDTH: f64 = 20.0;
                if col_idx < self.model.columns.len() {
                    self.model.columns[col_idx].width = new_width.max(MIN_COL_WIDTH);
                    self.model.rebuild_offsets();
                }
                CommandOutput::None
            }
            GridCommand::ToggleSort { col_key } => {
                let new_sort = match &self.sort {
                    None => Some(SortState {
                        col_key: col_key.clone(),
                        dir: SortDir::Asc,
                    }),
                    Some(s) if s.col_key == col_key && s.dir == SortDir::Asc => {
                        Some(SortState {
                            col_key: col_key.clone(),
                            dir: SortDir::Desc,
                        })
                    }
                    Some(s) if s.col_key == col_key => None,
                    _ => Some(SortState {
                        col_key: col_key.clone(),
                        dir: SortDir::Asc,
                    }),
                };
                match &new_sort {
                    Some(s) => self.model.apply_sort(&s.col_key, &s.dir),
                    None => self.model.sort_order.clear(),
                }
                self.sort = new_sort;
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::MoveSelection { delta_row, delta_col, extend } => {
                let row_count = self.model.data.row_count();
                let col_count = self.model.columns.len();
                let base = self.selection.focus.clone()
                    .or_else(|| self.selection.anchor.clone());
                if let Some(b) = base {
                    let new_row = (b.row as i64 + delta_row)
                        .clamp(0, row_count.saturating_sub(1) as i64) as u64;
                    let new_col = (b.col as i64 + delta_col)
                        .clamp(0, col_count.saturating_sub(1) as i64) as usize;
                    if extend { self.selection.extend_to(new_row, new_col); }
                    else      { self.selection.select_cell(new_row, new_col); }
                }
                CommandOutput::None
            }
        }
    }

    /// Hit-test a viewport-space pointer position against the data cells.
    pub fn hit_test(&self, vx: f64, vy: f64) -> Option<CellCoord> {
        hit_test::hit_test(
            vx,
            vy,
            &self.model,
            self.viewport.scroll_x,
            self.viewport.scroll_y,
        )
    }

    /// Hit-test the sticky row-number gutter. Returns the row index or `None`.
    pub fn hit_test_row_header(&self, vx: f64, vy: f64) -> Option<u64> {
        hit_test::hit_test_row_header(vx, vy, &self.model, self.viewport.scroll_y)
    }

    /// Hit-test a column header. Returns the column index or `None`.
    pub fn hit_test_col_header(&self, vx: f64, vy: f64) -> Option<usize> {
        hit_test::hit_test_col_header(vx, vy, &self.model, self.viewport.scroll_x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        column::ColumnDef,
        commands::{CommandOutput, GridCommand},
        model::GridModel,
        row::RowRecord,
        selection::CellCoord,
    };

    /// 3 columns (100+150+200=450 px total), 10 rows, viewport 800×600.
    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let rows = (0..10).map(|i| {
            let mut r = RowRecord::new(i);
            r.set("a", format!("a{i}"));
            r.set("b", format!("b{i}"));
            r.set("c", format!("c{i}"));
            r
        }).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    #[test]
    fn resize_updates_viewport() {
        let mut s = make_state();
        s.apply(GridCommand::Resize { width: 1024.0, height: 768.0 });
        assert_eq!(s.viewport.width, 1024.0);
        assert_eq!(s.viewport.height, 768.0);
    }

    // ── ScrollTo ──────────────────────────────────────────────────────────────

    #[test]
    fn scroll_to_basic() {
        let mut s = make_state();
        // total_height = 40 + 10*30 = 340; max_y = (340 - 600).max(0) = 0
        // viewport is larger than content → clamped to 0
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 100.0 });
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    #[test]
    fn scroll_to_small_viewport() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = (0..100).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        // total_height = 40 + 100*30 = 3040; max_y = 3040 - 200 = 2840
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 500.0 });
        assert_eq!(s.viewport.scroll_y, 500.0);
    }

    #[test]
    fn scroll_to_clamped_above_max() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = (0..100).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        // max_y = 2840
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 99_999.0 });
        assert_eq!(s.viewport.scroll_y, 2840.0);
    }

    #[test]
    fn scroll_to_negative_clamped_to_zero() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollTo { x: 0.0, y: -50.0 });
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    // ── ScrollBy ──────────────────────────────────────────────────────────────

    #[test]
    fn scroll_by_accumulates() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = (0..100).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        s.apply(GridCommand::ScrollBy { dx: 0.0, dy: 100.0 });
        s.apply(GridCommand::ScrollBy { dx: 0.0, dy: 50.0 });
        assert_eq!(s.viewport.scroll_y, 150.0);
    }

    // ── SelectCell / ClearSelection ───────────────────────────────────────────

    #[test]
    fn select_cell_and_clear() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 2, col: 1 }));
        assert!(s.selection.is_selected(2, 1));
        s.apply(GridCommand::ClearSelection);
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn extend_selection() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 3, col: 2 }));
        assert!(s.selection.is_selected(0, 0));
        assert!(s.selection.is_selected(3, 2));
        assert!(s.selection.is_selected(1, 1));
    }

    // ── SelectRow / ExtendRowSelection ────────────────────────────────────────

    #[test]
    fn select_row_spans_all_columns() {
        let mut s = make_state();
        s.apply(GridCommand::SelectRow(2));
        assert!(s.selection.is_selected(2, 0));
        assert!(s.selection.is_selected(2, 2)); // last col
        assert!(!s.selection.is_selected(1, 0));
    }

    // ── SelectCol / ExtendColSelection ────────────────────────────────────────

    #[test]
    fn select_col_spans_all_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCol(1));
        assert!(s.selection.is_selected(0, 1));
        assert!(s.selection.is_selected(9, 1)); // last row
        assert!(!s.selection.is_selected(0, 0));
    }

    // ── MoveSelection ─────────────────────────────────────────────────────────

    #[test]
    fn move_selection_down() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: 2,
            delta_col: 1,
            extend: false,
        });
        assert!(s.selection.is_selected(2, 1));
        assert!(!s.selection.is_selected(0, 0));
    }

    #[test]
    fn move_selection_clamped_to_bounds() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 9, col: 2 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: 100,
            delta_col: 100,
            extend: false,
        });
        // row_count=10 → max row=9; col_count=3 → max col=2
        assert!(s.selection.is_selected(9, 2));
    }

    // ── CopySelection ─────────────────────────────────────────────────────────

    #[test]
    fn copy_selection_returns_tsv() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::CopySelection);
        match out {
            CommandOutput::CopyText(t) => assert_eq!(t, "a0\tb0\n"),
            other => panic!("expected CopyText, got {other:?}"),
        }
    }

    // ── PasteAt ───────────────────────────────────────────────────────────────

    #[test]
    fn paste_at_origin() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 1, col: 0 }));
        s.apply(GridCommand::PasteAt { text: "X\tY\n".into() });
        assert_eq!(s.model.get_cell(1, "a"), Some("X".into()));
        assert_eq!(s.model.get_cell(1, "b"), Some("Y".into()));
    }
}
