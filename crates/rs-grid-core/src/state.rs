use crate::{
    commands::{CommandOutput, GridCommand},
    edit::EditCell,
    format::{format_cell, CellFormat},
    hit_test,
    model::GridModel,
    search::SearchState,
    selection::{CellCoord, SelectionState},
    sort::{SortDir, SortState},
    undo::{UndoEntry, UndoHistory},
    viewport::ViewportState,
};

/// The complete mutable state of a grid instance.
///
/// # Undo history
///
/// Cell edits, pastes, column resizes and column moves are recorded in an
/// internal undo stack capped at **100 entries**. Once the cap is reached
/// the oldest entry is evicted (FIFO). Selection and scroll changes are
/// not undoable.
#[derive(Debug)]
pub struct GridState {
    /// Column definitions, data source, and sizing constants.
    pub model: GridModel,
    /// Scroll position and canvas dimensions.
    pub viewport: ViewportState,
    /// Anchor/focus selection and clipboard state.
    pub selection: SelectionState,
    /// Row index currently under the mouse cursor, for hover highlighting.
    pub hovered_row: Option<u64>,
    /// Active sort column and direction (`None` = natural order).
    pub sort: Option<SortState>,
    /// Cell currently being edited (`None` = no edit in progress).
    pub edit: Option<EditCell>,
    /// Active search (empty query = inactive).
    pub search: SearchState,
    /// Undo/redo history.
    history: UndoHistory,
}

/// Clamp `(x, y)` scroll coordinates to the valid range
/// for the given model and viewport.
fn clamp_scroll(
    x: f64,
    y: f64,
    model: &GridModel,
    vp: &ViewportState,
) -> (f64, f64) {
    let rnw = model.row_number_width;
    let sb = model.scrollbar_size;
    let max_x = (model.total_width() - (vp.width - rnw - sb)).max(0.0);
    let max_y = (model.total_height() - vp.height + sb).max(0.0);
    (x.clamp(0.0, max_x), y.clamp(0.0, max_y))
}

impl GridState {
    /// Create a grid state from a model and initial viewport
    /// dimensions.
    pub fn new(
        model: GridModel,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Self {
        Self {
            model,
            viewport: ViewportState::new(viewport_width, viewport_height),
            selection: SelectionState::default(),
            hovered_row: None,
            sort: None,
            edit: None,
            search: SearchState::default(),
            history: UndoHistory::default(),
        }
    }

    /// Apply an undo entry and return the inverse entry for redo.
    fn apply_undo_entry(&mut self, entry: &UndoEntry) -> UndoEntry {
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

    fn run_search(&mut self, query: &str) {
        self.search = SearchState::run(&self.model, query);
    }

    fn scroll_to_search_match(&mut self) {
        let coord = match self.search.matches.get(self.search.current) {
            Some(c) => c.clone(),
            None => return,
        };
        // Select the matched cell.
        self.selection.select_cell(coord.row, coord.col);
        // Scroll to make the cell visible.
        let ry = self.model.row_top(coord.row);
        let cy = ry - self.viewport.scroll_y;
        if cy < self.model.header_height {
            self.viewport.scroll_y = ry - self.model.header_height;
        } else if cy + self.model.row_height > self.viewport.height {
            self.viewport.scroll_y =
                ry + self.model.row_height - self.viewport.height;
        }
        if coord.col < self.model.columns.len() {
            let off = self.model.column_offsets.offsets[coord.col];
            let w = self.model.columns[coord.col].width;
            let rnw = self.model.row_number_width;
            // Don't scroll for pinned columns.
            if coord.col >= self.model.pinned_count {
                let cx = off - self.viewport.scroll_x + rnw;
                if cx < rnw + self.model.pinned_width() {
                    self.viewport.scroll_x = off - self.model.pinned_width();
                } else if cx + w > self.viewport.width {
                    self.viewport.scroll_x =
                        off + w - self.viewport.width + rnw;
                }
            }
        }
        let (sx, sy) = clamp_scroll(
            self.viewport.scroll_x,
            self.viewport.scroll_y,
            &self.model,
            &self.viewport,
        );
        self.viewport.scroll_x = sx;
        self.viewport.scroll_y = sy;
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
                let (sx, sy) = clamp_scroll(x, y, &self.model, &self.viewport);
                self.viewport.scroll_x = sx;
                self.viewport.scroll_y = sy;
                CommandOutput::None
            }
            GridCommand::ScrollBy { dx, dy } => {
                let (sx, sy) = clamp_scroll(
                    self.viewport.scroll_x + dx,
                    self.viewport.scroll_y + dy,
                    &self.model,
                    &self.viewport,
                );
                self.viewport.scroll_x = sx;
                self.viewport.scroll_y = sy;
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
                    // Update selection to cover pasted area
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
            GridCommand::SelectRow(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row, col: 0 });
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::ExtendRowSelection(row) => {
                let last_col = self.model.columns.len().saturating_sub(1);
                // Clamp anchor column to 0 so the range always spans all columns.
                if let Some(ref mut a) = self.selection.anchor {
                    a.col = 0;
                }
                self.selection.focus = Some(CellCoord { row, col: last_col });
                CommandOutput::None
            }
            GridCommand::SelectCol(col) => {
                let last_row = self.model.display_row_count().saturating_sub(1);
                self.selection.anchor = Some(CellCoord { row: 0, col });
                self.selection.focus = Some(CellCoord { row: last_row, col });
                CommandOutput::None
            }
            GridCommand::ExtendColSelection(col) => {
                let last_row = self.model.display_row_count().saturating_sub(1);
                // Clamp anchor row to 0 so the range always spans all rows.
                if let Some(ref mut a) = self.selection.anchor {
                    a.row = 0;
                }
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
                    self.model.columns[col_idx].width =
                        new_width.max(MIN_COL_WIDTH);
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
                    Some(s)
                        if s.col_key == col_key && s.dir == SortDir::Asc =>
                    {
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
                    None => {
                        self.model.sort_order.clear();
                        self.model.invalidate_sort_cache();
                    }
                }
                self.sort = new_sort;
                // Reapply filter with updated sort order.
                if !self.model.filters.is_empty() {
                    self.model.apply_filter();
                }
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::SetPinnedColumnCount { count } => {
                self.model.pinned_count = count.min(self.model.columns.len());
                CommandOutput::None
            }
            GridCommand::SetColumnFilter { col_key, text } => {
                if text.is_empty() {
                    self.model.filters.remove(&col_key);
                } else {
                    self.model.filters.insert(col_key, text);
                }
                self.model.apply_filter();
                self.selection.clear();
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::ClearAllFilters => {
                self.model.filters.clear();
                self.model.filtered_indices.clear();
                self.selection.clear();
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::MoveColumn { from_idx, to_idx } => {
                let len = self.model.columns.len();
                if from_idx < len && to_idx < len && from_idx != to_idx {
                    let col = self.model.columns.remove(from_idx);
                    self.model.columns.insert(to_idx, col);
                    self.model.rebuild_offsets();
                    self.history.push(UndoEntry::MoveColumn {
                        from_idx: to_idx,
                        to_idx: from_idx,
                    });
                }
                CommandOutput::None
            }
            GridCommand::StartEdit { row, col_key } => {
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
            GridCommand::Search { query } => {
                self.run_search(&query);
                CommandOutput::None
            }
            GridCommand::SearchNext => {
                if !self.search.matches.is_empty() {
                    self.search.current =
                        (self.search.current + 1) % self.search.matches.len();
                    self.scroll_to_search_match();
                }
                CommandOutput::None
            }
            GridCommand::SearchPrev => {
                if !self.search.matches.is_empty() {
                    let len = self.search.matches.len();
                    self.search.current = (self.search.current + len - 1) % len;
                    self.scroll_to_search_match();
                }
                CommandOutput::None
            }
            GridCommand::ClearSearch => {
                self.search = SearchState::default();
                CommandOutput::None
            }
            GridCommand::NotifyPageLoaded => {
                // No-op — triggers a re-render via dispatch.
                CommandOutput::None
            }
            GridCommand::SetTotalRowCount(n) => {
                // Update the underlying data source row count.
                // For PageCacheDataSource this is done
                // externally; here we just trigger re-render.
                let _ = n;
                CommandOutput::None
            }
            GridCommand::AutoFitColumn {
                col_idx,
                char_width,
                header_char_width,
                cell_padding,
            } => {
                const MIN_COL_WIDTH: f64 = 20.0;
                const MAX_SAMPLE_ROWS: u64 = 1_000;
                if col_idx < self.model.columns.len() {
                    let old_width = self.model.columns[col_idx].width;
                    let col_key = self.model.columns[col_idx].key.clone();
                    let label = &self.model.columns[col_idx].label;
                    let header_w = label.len() as f64 * header_char_width
                        + cell_padding * 2.0;
                    let col_format =
                        self.model.columns[col_idx].format.clone();
                    let row_count =
                        self.model.display_row_count().min(MAX_SAMPLE_ROWS);
                    let mut max_w = header_w;
                    for r in 0..row_count {
                        if let Some(val) = self.model.get_cell(r, &col_key) {
                            let w = match &col_format {
                                Some(CellFormat::Image { .. }) => {
                                    self.model.row_height
                                        + cell_padding * 2.0
                                }
                                Some(CellFormat::ImageText {
                                    image_size,
                                    gap,
                                    ..
                                }) => {
                                    let label_len = val
                                        .find(' ')
                                        .map(|i| val[i + 1..].len())
                                        .unwrap_or(val.len());
                                    image_size
                                        + gap
                                        + label_len as f64 * char_width
                                        + cell_padding * 2.0
                                }
                                Some(fmt) => {
                                    let formatted =
                                        format_cell(&val, fmt);
                                    formatted.text.len() as f64
                                        * char_width
                                        + cell_padding * 2.0
                                }
                                None => {
                                    val.len() as f64 * char_width
                                        + cell_padding * 2.0
                                }
                            };
                            if w > max_w {
                                max_w = w;
                            }
                        }
                    }
                    self.model.columns[col_idx].width =
                        max_w.max(MIN_COL_WIDTH);
                    self.model.rebuild_offsets();
                    self.history
                        .push(UndoEntry::ResizeColumn { col_idx, old_width });
                }
                CommandOutput::None
            }
            GridCommand::MoveSelection {
                delta_row,
                delta_col,
                extend,
            } => {
                let row_count = self.model.display_row_count();
                let col_count = self.model.columns.len();
                let base = self
                    .selection
                    .focus
                    .clone()
                    .or_else(|| self.selection.anchor.clone());
                if let Some(b) = base {
                    // Cast to i64 first so negative deltas don't
                    // underflow the unsigned row/col indices, then
                    // clamp to valid bounds before casting back.
                    let new_row = (b.row as i64 + delta_row)
                        .clamp(0, row_count.saturating_sub(1) as i64)
                        as u64;
                    let new_col = (b.col as i64 + delta_col)
                        .clamp(0, col_count.saturating_sub(1) as i64)
                        as usize;
                    if extend {
                        self.selection.extend_to(new_row, new_col);
                    } else {
                        self.selection.select_cell(new_row, new_col);
                    }
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
        hit_test::hit_test_row_header(
            vx,
            vy,
            &self.model,
            self.viewport.scroll_y,
        )
    }

    /// Hit-test a column header. Returns the column index or `None`.
    pub fn hit_test_col_header(&self, vx: f64, vy: f64) -> Option<usize> {
        hit_test::hit_test_col_header(
            vx,
            vy,
            &self.model,
            self.viewport.scroll_x,
        )
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
        let rows = (0..10)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r.set("c", format!("c{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    #[test]
    fn resize_updates_viewport() {
        let mut s = make_state();
        s.apply(GridCommand::Resize {
            width: 1024.0,
            height: 768.0,
        });
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
        // max_y = 3040 - 200 + 14 (scrollbar) = 2854
        s.apply(GridCommand::ScrollTo {
            x: 0.0,
            y: 99_999.0,
        });
        assert_eq!(s.viewport.scroll_y, 2854.0);
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
        s.apply(GridCommand::PasteAt {
            text: "X\tY\n".into(),
        });
        assert_eq!(s.model.get_cell(1, "a"), Some("X".into()));
        assert_eq!(s.model.get_cell(1, "b"), Some("Y".into()));
    }

    #[test]
    fn paste_with_upward_selection() {
        let mut s = make_state();
        // Select row 3, then extend upward to row 1 (anchor=3, focus=1).
        s.apply(GridCommand::SelectCell(CellCoord { row: 3, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 0 }));
        // Paste should fill rows 1..=3 (top-left of selection),
        // NOT rows 3..=5 (anchor).
        s.apply(GridCommand::PasteAt {
            text: "X\nY\nZ\n".into(),
        });
        assert_eq!(s.model.get_cell(1, "a"), Some("X".into()));
        assert_eq!(s.model.get_cell(2, "a"), Some("Y".into()));
        assert_eq!(s.model.get_cell(3, "a"), Some("Z".into()));
        // Row 0 untouched
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
    }

    // ── SetColumnFilter ──────────────────────────────────────────────────────

    #[test]
    fn filter_reduces_display_row_count() {
        let mut s = make_state();
        // Only rows containing "a3" in column "a"
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a3".into(),
        });
        assert_eq!(s.model.display_row_count(), 1);
        // Logical row 0 maps to physical row 3
        assert_eq!(s.model.get_cell(0, "a"), Some("a3".into()));
    }

    #[test]
    fn filter_empty_text_clears() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a1".into(),
        });
        assert_eq!(s.model.display_row_count(), 1);
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "".into(),
        });
        assert_eq!(s.model.display_row_count(), 10);
    }

    #[test]
    fn clear_all_filters() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a5".into(),
        });
        s.apply(GridCommand::ClearAllFilters);
        assert_eq!(s.model.display_row_count(), 10);
    }

    // ── MoveColumn ───────────────────────────────────────────────────────────

    #[test]
    fn move_column_reorders() {
        let mut s = make_state();
        // [a, b, c] → move 0 to 2 → [b, c, a]
        s.apply(GridCommand::MoveColumn {
            from_idx: 0,
            to_idx: 2,
        });
        assert_eq!(s.model.columns[0].key, "b");
        assert_eq!(s.model.columns[1].key, "c");
        assert_eq!(s.model.columns[2].key, "a");
    }

    #[test]
    fn move_column_out_of_range_noop() {
        let mut s = make_state();
        s.apply(GridCommand::MoveColumn {
            from_idx: 0,
            to_idx: 99,
        });
        // unchanged
        assert_eq!(s.model.columns[0].key, "a");
    }

    // ── StartEdit / CommitEdit / CancelEdit ──────────────────────────────────

    #[test]
    fn start_and_commit_edit() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        assert!(s.edit.is_some());
        assert_eq!(s.edit.as_ref().unwrap().initial_value, "a0");
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "edited".into(),
        });
        assert!(s.edit.is_none());
        assert_eq!(s.model.get_cell(0, "a"), Some("edited".into()));
    }

    #[test]
    fn cancel_edit_discards() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CancelEdit);
        assert!(s.edit.is_none());
        // Cell unchanged
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
    }

    // ── SetPinnedColumnCount ─────────────────────────────────────────────────

    #[test]
    fn set_pinned_count() {
        let mut s = make_state();
        s.apply(GridCommand::SetPinnedColumnCount { count: 1 });
        assert_eq!(s.model.pinned_count, 1);
        assert_eq!(s.model.pinned_width(), 100.0);
    }

    #[test]
    fn set_pinned_count_clamped() {
        let mut s = make_state();
        s.apply(GridCommand::SetPinnedColumnCount { count: 99 });
        assert_eq!(s.model.pinned_count, 3);
    }

    // ── AutoFitColumn ──────────────────────────────────────────────────────

    #[test]
    fn auto_fit_column_adjusts_width() {
        let mut s = make_state();
        // Column "a" has label "A" (1 char) and values "a0".."a9" (2 chars).
        // Heuristic: max_w = max(header, data) + padding*2
        // data: 2 * 8.4 + 10*2 = 36.8
        // header: 1 * 8.45 + 10*2 = 28.45
        // Expected: 36.8 (data wins)
        let old_width = s.model.columns[0].width;
        assert_eq!(old_width, 100.0);
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 8.4,
            header_char_width: 8.45,
            cell_padding: 10.0,
        });
        let new_width = s.model.columns[0].width;
        assert!(
            (new_width - 36.8).abs() < 0.01,
            "expected ~36.8, got {new_width}"
        );
    }

    #[test]
    fn auto_fit_column_respects_min_width() {
        let mut s = make_state();
        // With very small char_width the result should be at least 20.0
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 0.1,
            header_char_width: 0.1,
            cell_padding: 0.1,
        });
        assert!(
            s.model.columns[0].width >= 20.0,
            "width should be at least 20.0, got {}",
            s.model.columns[0].width
        );
    }

    #[test]
    fn auto_fit_column_out_of_range_noop() {
        let mut s = make_state();
        let old_width = s.model.columns[0].width;
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 99,
            char_width: 8.4,
            header_char_width: 8.45,
            cell_padding: 10.0,
        });
        assert_eq!(s.model.columns[0].width, old_width);
    }

    #[test]
    fn auto_fit_image_text_ignores_base64() {
        let cols = vec![ColumnDef {
            key: "country".into(),
            label: "Country".into(),
            width: 100.0,
            format: Some(CellFormat::ImageText {
                base_url: String::new(),
                suffix: String::new(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 6.0,
            }),
            editor: None,
            validator: None,
        }];
        // base64-like key + short label
        let mut row = RowRecord::new(0);
        row.set(
            "country",
            "data:image/png;base64,AAAA France".to_string(),
        );
        let model = GridModel::new(cols, vec![row], 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 8.0,
            header_char_width: 8.0,
            cell_padding: 10.0,
        });
        let w = s.model.columns[0].width;
        // image_size(20) + gap(6) + "France".len(6)*8 + pad*2(20) = 94
        let expected = 20.0 + 6.0 + 6.0 * 8.0 + 10.0 * 2.0;
        assert!(
            (w - expected).abs() < 0.01,
            "expected {expected}, got {w}"
        );
    }

    // ── Undo / Redo ────────────────────────────────────────────────────────

    #[test]
    fn undo_commit_edit() {
        let mut s = make_state();
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "edited".into(),
        });
        assert_eq!(s.model.get_cell(0, "a"), Some("edited".into()));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
    }

    #[test]
    fn redo_after_undo() {
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
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
        s.apply(GridCommand::Redo);
        assert_eq!(s.model.get_cell(0, "a"), Some("edited".into()));
    }

    #[test]
    fn undo_paste() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::PasteAt {
            text: "X\tY".into(),
        });
        assert_eq!(s.model.get_cell(0, "a"), Some("X".into()));
        assert_eq!(s.model.get_cell(0, "b"), Some("Y".into()));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
        assert_eq!(s.model.get_cell(0, "b"), Some("b0".into()));
    }

    #[test]
    fn undo_move_column() {
        let mut s = make_state();
        assert_eq!(s.model.columns[0].key, "a");
        assert_eq!(s.model.columns[1].key, "b");
        s.apply(GridCommand::MoveColumn {
            from_idx: 0,
            to_idx: 1,
        });
        assert_eq!(s.model.columns[0].key, "b");
        assert_eq!(s.model.columns[1].key, "a");
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[0].key, "a");
        assert_eq!(s.model.columns[1].key, "b");
    }

    #[test]
    fn undo_auto_fit_restores_width() {
        let mut s = make_state();
        assert_eq!(s.model.columns[0].width, 100.0);
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 8.4,
            header_char_width: 8.45,
            cell_padding: 10.0,
        });
        assert_ne!(s.model.columns[0].width, 100.0);
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[0].width, 100.0);
    }

    #[test]
    fn new_action_clears_redo_stack() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "v1".into(),
        });
        s.apply(GridCommand::Undo);
        // Now do a new edit — redo stack should be cleared
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "v2".into(),
        });
        s.apply(GridCommand::Redo); // should be no-op
        assert_eq!(s.model.get_cell(0, "a"), Some("v2".into()));
    }

    #[test]
    fn undo_on_empty_stack_is_noop() {
        let mut s = make_state();
        let val = s.model.get_cell(0, "a");
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), val);
    }

    // ── Search ─────────────────────────────────────────────────────────────

    #[test]
    fn search_finds_matches() {
        let mut s = make_state();
        // Data: column "a" has values "a0".."a9"
        s.apply(GridCommand::Search { query: "a0".into() });
        assert_eq!(s.search.matches.len(), 1);
        assert_eq!(s.search.matches[0].row, 0);
        assert_eq!(s.search.matches[0].col, 0);
    }

    #[test]
    fn search_case_insensitive() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "A0".into() });
        assert_eq!(s.search.matches.len(), 1);
    }

    #[test]
    fn search_multiple_matches() {
        let mut s = make_state();
        // "b" appears in column "b" values: "b0".."b9" (10 matches)
        s.apply(GridCommand::Search { query: "b".into() });
        assert_eq!(s.search.matches.len(), 10);
    }

    #[test]
    fn search_next_cycles() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "a".into() });
        let len = s.search.matches.len();
        assert!(len > 1);
        assert_eq!(s.search.current, 0);
        s.apply(GridCommand::SearchNext);
        assert_eq!(s.search.current, 1);
        // Cycle back to 0
        for _ in 0..len - 1 {
            s.apply(GridCommand::SearchNext);
        }
        assert_eq!(s.search.current, 0);
    }

    #[test]
    fn search_prev_cycles() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "a".into() });
        assert_eq!(s.search.current, 0);
        s.apply(GridCommand::SearchPrev);
        assert_eq!(s.search.current, s.search.matches.len() - 1);
    }

    #[test]
    fn clear_search_resets() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "a".into() });
        assert!(!s.search.matches.is_empty());
        s.apply(GridCommand::ClearSearch);
        assert!(s.search.query.is_empty());
        assert!(s.search.matches.is_empty());
    }

    #[test]
    fn search_empty_query_clears() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "a".into() });
        assert!(!s.search.matches.is_empty());
        s.apply(GridCommand::Search {
            query: String::new(),
        });
        assert!(s.search.matches.is_empty());
    }
}
