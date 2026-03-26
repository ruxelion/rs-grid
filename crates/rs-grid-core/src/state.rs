mod cmd_clipboard;
mod cmd_column;
mod cmd_edit;
mod cmd_meta;
mod cmd_scroll;
mod cmd_search;
mod cmd_selection;
mod cmd_sort_filter;
mod cmd_undo;

use crate::{
    commands::{CommandOutput, GridCommand},
    edit::EditCell,
    hit_test,
    model::GridModel,
    search::SearchState,
    selection::{CellCoord, SelectionState},
    sort::SortState,
    undo::UndoHistory,
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

    /// Apply a command, mutating state in place.
    pub fn apply(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::SelectCell { .. }
            | GridCommand::ExtendSelection { .. }
            | GridCommand::SelectRow(_)
            | GridCommand::SelectCol(_)
            | GridCommand::ExtendRowSelection(_)
            | GridCommand::ExtendColSelection(_)
            | GridCommand::ClearSelection
            | GridCommand::MoveSelection { .. } => self.cmd_selection(cmd),

            GridCommand::ScrollTo { .. }
            | GridCommand::ScrollBy { .. }
            | GridCommand::Resize { .. } => self.cmd_scroll(cmd),

            GridCommand::CopySelection
            | GridCommand::CutSelection
            | GridCommand::PasteAt { .. } => self.cmd_clipboard(cmd),

            GridCommand::ToggleSort { .. }
            | GridCommand::SetSort { .. }
            | GridCommand::ClearSort
            | GridCommand::SetColumnFilter { .. }
            | GridCommand::ClearAllFilters => self.cmd_sort_filter(cmd),

            GridCommand::ResizeColumn { .. }
            | GridCommand::SetPinnedColumnCount { .. }
            | GridCommand::MoveColumn { .. }
            | GridCommand::AutoFitColumn { .. }
            | GridCommand::AutoFitAllColumns { .. } => self.cmd_column(cmd),

            GridCommand::StartEdit { .. }
            | GridCommand::CommitEdit { .. }
            | GridCommand::CancelEdit => self.cmd_edit(cmd),

            GridCommand::Undo | GridCommand::Redo => self.cmd_undo(cmd),

            GridCommand::Search { .. }
            | GridCommand::SearchNext
            | GridCommand::SearchPrev
            | GridCommand::ClearSearch => self.cmd_search(cmd),

            GridCommand::SetHoveredRow(_)
            | GridCommand::SetHeaderHeight(_)
            | GridCommand::SetRowHeight(_)
            | GridCommand::NotifyPageLoaded
            | GridCommand::SetTotalRowCount(_) => self.cmd_meta(cmd),
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
        format::CellFormat,
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
        row.set("country", "data:image/png;base64,AAAA France".to_string());
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
        assert!((w - expected).abs() < 0.01, "expected {expected}, got {w}");
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
