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
    ///
    /// **Prefer [`apply(GridCommand)`](Self::apply) for
    /// mutations.** Direct field mutation bypasses undo
    /// history and may leave state inconsistent.
    pub model: GridModel,
    /// Scroll position and canvas dimensions.
    ///
    /// **Prefer [`apply(GridCommand)`](Self::apply) for
    /// mutations** (e.g. `ScrollTo`, `ScrollBy`, `Resize`).
    pub viewport: ViewportState,
    /// Anchor/focus selection and clipboard state.
    ///
    /// **Prefer [`apply(GridCommand)`](Self::apply) for
    /// mutations** (e.g. `SelectCell`, `ClearSelection`).
    pub selection: SelectionState,
    /// Row index currently under the mouse cursor, for hover
    /// highlighting.
    pub hovered_row: Option<u64>,
    /// Active sort column and direction (`None` = natural
    /// order).
    pub sort: Option<SortState>,
    /// Cell currently being edited (`None` = no edit in
    /// progress).
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
        mut model: GridModel,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Self {
        model.recalculate_flex_widths(viewport_width);
        model.rebuild_offsets();
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
            | GridCommand::CommitColumnResize { .. }
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
            | GridCommand::SetShowHeader(_)
            | GridCommand::SetShowRowNumbers(_)
            | GridCommand::SetEditable(_)
            | GridCommand::SetSelectable(_)
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
            header_right_reserve: 0.0,
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
            header_right_reserve: 0.0,
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
            header_right_reserve: 0.0,
        });
        assert_eq!(s.model.columns[0].width, old_width);
    }

    #[test]
    fn auto_fit_image_text_ignores_base64() {
        let cols = vec![ColumnDef {
            key: "country".into(),
            label: "Country".into(),
            width: 100.0,
            min_width: None,
            max_width: None,
            flex: None,
            format: Some(CellFormat::ImageText {
                base_url: String::new(),
                suffix: String::new(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 6.0,
            }),
            editor: None,
            validator: None,
        bold: false,
editable: true,
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
            header_right_reserve: 0.0,
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
            header_right_reserve: 0.0,
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

    // ── CutSelection ─────────────────────────────────────────────────────────

    #[test]
    fn cut_selection_returns_tsv_and_clears_cells() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 0, col: 1 }));
        let out = s.apply(GridCommand::CutSelection);
        assert!(matches!(out, CommandOutput::CopyText(_)));
        // Cells should be cleared.
        assert_eq!(s.model.get_cell(0, "a"), Some(String::new()),);
        assert_eq!(s.model.get_cell(0, "b"), Some(String::new()),);
    }

    #[test]
    fn cut_selection_is_undoable() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::CutSelection);
        assert_eq!(s.model.get_cell(0, "a"), Some(String::new()));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".to_string()),);
    }

    // ── ToggleSort ───────────────────────────────────────────────────────────

    #[test]
    fn toggle_sort_cycles_none_asc_desc_none() {
        use crate::sort::{SortDir, SortState};
        let mut s = make_state();
        // None → Asc
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert_eq!(
            s.sort,
            Some(SortState {
                col_key: "a".into(),
                dir: SortDir::Asc,
            }),
        );
        // Asc → Desc
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert_eq!(s.sort.as_ref().unwrap().dir, SortDir::Desc);
        // Desc → None
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert!(s.sort.is_none());
    }

    #[test]
    fn toggle_sort_resets_scroll_y() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = (0..100).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 500.0 });
        assert!(s.viewport.scroll_y > 0.0);
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    // ── SetSort ──────────────────────────────────────────────────────────────

    #[test]
    fn set_sort_explicit_direction() {
        use crate::sort::{SortDir, SortState};
        let mut s = make_state();
        s.apply(GridCommand::SetSort {
            col_key: "b".into(),
            dir: SortDir::Desc,
        });
        assert_eq!(
            s.sort,
            Some(SortState {
                col_key: "b".into(),
                dir: SortDir::Desc,
            }),
        );
    }

    // ── ResizeColumn ─────────────────────────────────────────────────────────

    #[test]
    fn resize_column_updates_width() {
        let mut s = make_state();
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 200.0,
        });
        assert_eq!(s.model.columns[0].width, 200.0);
    }

    #[test]
    fn resize_column_clamps_to_min_width() {
        let mut s = make_state();
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 5.0,
        });
        assert_eq!(s.model.columns[0].width, 20.0);
    }

    #[test]
    fn resize_column_out_of_bounds_noop() {
        let mut s = make_state();
        let w = s.model.columns[0].width;
        s.apply(GridCommand::ResizeColumn {
            col_idx: 99,
            new_width: 200.0,
        });
        assert_eq!(s.model.columns[0].width, w);
    }

    #[test]
    fn resize_column_respects_min_max_width() {
        let mut s = make_state();
        s.model.columns[0].min_width = Some(60.0);
        s.model.columns[0].max_width = Some(300.0);
        // Below min
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 30.0,
        });
        assert_eq!(s.model.columns[0].width, 60.0);
        // Above max
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 500.0,
        });
        assert_eq!(s.model.columns[0].width, 300.0);
        // Within range
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 150.0,
        });
        assert_eq!(s.model.columns[0].width, 150.0);
    }

    // ── ExtendRowSelection / ExtendColSelection ──────────────────────────────

    #[test]
    fn extend_row_selection_spans_all_columns() {
        let mut s = make_state();
        s.apply(GridCommand::SelectRow(2));
        s.apply(GridCommand::ExtendRowSelection(5));
        let (tl, br) = s.selection.range().unwrap();
        assert_eq!(tl.row, 2);
        assert_eq!(br.row, 5);
        assert_eq!(tl.col, 0);
        assert_eq!(br.col, 2); // 3 columns
    }

    #[test]
    fn extend_col_selection_spans_all_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCol(0));
        s.apply(GridCommand::ExtendColSelection(2));
        let (tl, br) = s.selection.range().unwrap();
        assert_eq!(tl.row, 0);
        assert_eq!(br.row, 9); // 10 rows
        assert_eq!(tl.col, 0);
        assert_eq!(br.col, 2);
    }

    // ── Meta commands ────────────────────────────────────────────────────────

    #[test]
    fn set_hovered_row() {
        let mut s = make_state();
        assert!(s.hovered_row.is_none());
        s.apply(GridCommand::SetHoveredRow(Some(5)));
        assert_eq!(s.hovered_row, Some(5));
        s.apply(GridCommand::SetHoveredRow(None));
        assert!(s.hovered_row.is_none());
    }

    #[test]
    fn set_header_height_positive() {
        let mut s = make_state();
        s.apply(GridCommand::SetHeaderHeight(60.0));
        assert_eq!(s.model.header_height, 60.0);
    }

    #[test]
    fn set_header_height_zero_ignored() {
        let mut s = make_state();
        let old = s.model.header_height;
        s.apply(GridCommand::SetHeaderHeight(0.0));
        assert_eq!(s.model.header_height, old);
    }

    #[test]
    fn set_row_height_positive() {
        let mut s = make_state();
        s.apply(GridCommand::SetRowHeight(50.0));
        assert_eq!(s.model.row_height, 50.0);
    }

    #[test]
    fn set_row_height_negative_ignored() {
        let mut s = make_state();
        let old = s.model.row_height;
        s.apply(GridCommand::SetRowHeight(-10.0));
        assert_eq!(s.model.row_height, old);
    }

    // ── SetShowHeader / SetShowRowNumbers ────────────────────────────────────

    #[test]
    fn set_show_header_false_effective_height_is_zero() {
        let mut s = make_state();
        let original_h = s.model.header_height;
        assert!(original_h > 0.0);
        s.apply(GridCommand::SetShowHeader(false));
        assert_eq!(s.model.effective_header_height(), 0.0);
        // stored value unchanged
        assert_eq!(s.model.header_height, original_h);
    }

    #[test]
    fn set_show_row_numbers_false_effective_width_is_zero() {
        let mut s = make_state();
        let original_w = s.model.row_number_width;
        assert!(original_w > 0.0);
        s.apply(GridCommand::SetShowRowNumbers(false));
        assert_eq!(s.model.effective_row_number_width(), 0.0);
        // stored value unchanged
        assert_eq!(s.model.row_number_width, original_w);
    }

    #[test]
    fn toggle_show_header_restores_original_height() {
        let mut s = make_state();
        let original_h = s.model.header_height;
        s.apply(GridCommand::SetShowHeader(false));
        assert_eq!(s.model.effective_header_height(), 0.0);
        s.apply(GridCommand::SetShowHeader(true));
        assert_eq!(s.model.effective_header_height(), original_h);
    }

    // ── SetSelectable ────────────────────────────────────────────────────────

    #[test]
    fn set_selectable_false_clears_selection() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(s.selection.has_selection());
        s.apply(GridCommand::SetSelectable(false));
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn set_selectable_false_ignores_select_cell() {
        let mut s = make_state();
        s.apply(GridCommand::SetSelectable(false));
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn set_selectable_true_allows_select_cell() {
        let mut s = make_state();
        s.apply(GridCommand::SetSelectable(false));
        s.apply(GridCommand::SetSelectable(true));
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(s.selection.has_selection());
    }

    // ── Edit guard edge cases ────────────────────────────────────────────────

    #[test]
    fn commit_edit_without_start_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "a".into(),
            value: "new".into(),
        });
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".to_string()),);
    }

    #[test]
    fn cancel_edit_without_start_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::CancelEdit);
        assert!(s.edit.is_none());
    }

    // ── MoveColumn same index ────────────────────────────────────────────────

    #[test]
    fn move_column_same_index_noop() {
        let mut s = make_state();
        let keys: Vec<_> =
            s.model.columns.iter().map(|c| c.key.clone()).collect();
        s.apply(GridCommand::MoveColumn {
            from_idx: 1,
            to_idx: 1,
        });
        let after: Vec<_> =
            s.model.columns.iter().map(|c| c.key.clone()).collect();
        assert_eq!(keys, after);
    }

    // ── Paste empty text ─────────────────────────────────────────────────────

    #[test]
    fn paste_empty_text_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::PasteAt {
            text: String::new(),
        });
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".to_string()),);
    }

    // ── MoveSelection no selection ───────────────────────────────────────────

    #[test]
    fn move_selection_no_selection_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::MoveSelection {
            delta_row: 1,
            delta_col: 0,
            extend: false,
        });
        assert!(!s.selection.has_selection());
    }

    // ── SearchNext/Prev on empty matches ─────────────────────────────────────

    #[test]
    fn search_next_empty_matches_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SearchNext);
        assert_eq!(s.search.current, 0);
    }

    #[test]
    fn search_prev_empty_matches_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SearchPrev);
        assert_eq!(s.search.current, 0);
    }

    // ── MoveSelection extend ─────────────────────────

    #[test]
    fn move_selection_extend() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 2, col: 1 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: 2,
            delta_col: 1,
            extend: true,
        });
        // Anchor stays at (2,1), focus moves to (4,2)
        assert!(s.selection.is_selected(2, 1));
        assert!(s.selection.is_selected(4, 2));
        assert!(s.selection.is_selected(3, 1));
    }

    #[test]
    fn move_selection_negative_clamped() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::MoveSelection {
            delta_row: -5,
            delta_col: -5,
            extend: false,
        });
        assert!(s.selection.is_selected(0, 0));
    }

    // ── Copy no selection ────────────────────────────

    #[test]
    fn copy_no_selection_returns_error() {
        let mut s = make_state();
        let out = s.apply(GridCommand::CopySelection);
        assert!(matches!(out, CommandOutput::CopyError(_)));
    }

    #[test]
    fn cut_no_selection_returns_error() {
        let mut s = make_state();
        let out = s.apply(GridCommand::CutSelection);
        assert!(matches!(out, CommandOutput::CopyError(_)));
    }

    // ── Paste tiling ─────────────────────────────────

    #[test]
    fn paste_tiles_into_larger_selection() {
        let mut s = make_state();
        // Select a 2x2 area
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        // Paste a 1x1 value — should tile into 2x2
        s.apply(GridCommand::PasteAt { text: "Z\n".into() });
        assert_eq!(s.model.get_cell(0, "a"), Some("Z".into()));
        assert_eq!(s.model.get_cell(0, "b"), Some("Z".into()));
        assert_eq!(s.model.get_cell(1, "a"), Some("Z".into()));
        assert_eq!(s.model.get_cell(1, "b"), Some("Z".into()));
    }

    #[test]
    fn paste_clamps_to_grid_bounds() {
        let mut s = make_state();
        // Select bottom-right cell
        s.apply(GridCommand::SelectCell(CellCoord { row: 9, col: 2 }));
        // Paste 3x3 — should only write 1 cell
        s.apply(GridCommand::PasteAt {
            text: "X\tY\tZ\nA\tB\tC\nD\tE\tF\n".into(),
        });
        assert_eq!(s.model.get_cell(9, "c"), Some("X".into()));
        // Row 10 doesn't exist, col 3 doesn't exist
    }

    #[test]
    fn paste_no_selection_is_noop() {
        let mut s = make_state();
        let before = s.model.get_cell(0, "a");
        s.apply(GridCommand::PasteAt { text: "X\n".into() });
        assert_eq!(s.model.get_cell(0, "a"), before);
    }

    // ── ToggleSort switching columns ─────────────────

    #[test]
    fn toggle_sort_different_col_resets_to_asc() {
        use crate::sort::SortDir;
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert_eq!(s.sort.as_ref().unwrap().dir, SortDir::Asc);
        // Switch to different column → starts at Asc
        s.apply(GridCommand::ToggleSort {
            col_key: "b".into(),
        });
        assert_eq!(s.sort.as_ref().unwrap().col_key, "b");
        assert_eq!(s.sort.as_ref().unwrap().dir, SortDir::Asc);
    }

    // ── ClearSort ────────────────────────────────────

    #[test]
    fn clear_sort_removes_sort_state() {
        let mut s = make_state();
        s.apply(GridCommand::SetSort {
            col_key: "a".into(),
            dir: crate::sort::SortDir::Asc,
        });
        assert!(s.sort.is_some());
        s.apply(GridCommand::ClearSort);
        assert!(s.sort.is_none());
        assert!(s.model.sort_order.is_empty());
    }

    #[test]
    fn clear_sort_resets_scroll_y() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = (0..100).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 500.0 });
        s.apply(GridCommand::ClearSort);
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    // ── Filter edge cases ────────────────────────────

    #[test]
    fn filter_case_insensitive() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "A3".into(),
        });
        // Data has "a3" (lowercase) — should match
        assert_eq!(s.model.display_row_count(), 1);
    }

    #[test]
    fn filter_clears_selection() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(s.selection.has_selection());
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a1".into(),
        });
        assert!(!s.selection.has_selection());
    }

    #[test]
    fn filter_resets_scroll_y() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows: Vec<RowRecord> = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("v{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 200.0, 200.0);
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 500.0 });
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "v1".into(),
        });
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    #[test]
    fn clear_all_filters_resets_scroll_and_selection() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a5".into(),
        });
        s.apply(GridCommand::ClearAllFilters);
        assert!(!s.selection.has_selection());
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    // ── CommitEdit guard ─────────────────────────────

    #[test]
    fn commit_edit_wrong_cell_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "a".into(),
        });
        // Commit for a different cell → no-op
        s.apply(GridCommand::CommitEdit {
            row: 1,
            col_key: "a".into(),
            value: "wrong".into(),
        });
        assert!(s.edit.is_some());
        assert_eq!(s.model.get_cell(1, "a"), Some("a1".into()));
    }

    // ── ResizeColumn does not push undo ────────────────

    #[test]
    fn resize_column_not_undoable() {
        let mut s = make_state();
        assert_eq!(s.model.columns[0].width, 100.0);
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 250.0,
        });
        assert_eq!(s.model.columns[0].width, 250.0);
        // ResizeColumn doesn't push undo — Undo is a no-op
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[0].width, 250.0);
    }

    #[test]
    fn commit_column_resize_enables_undo() {
        let mut s = make_state();
        let orig = s.model.columns[0].width;
        assert_eq!(orig, 100.0);
        // Simulate drag: many ResizeColumn then one Commit
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 200.0,
        });
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 250.0,
        });
        s.apply(GridCommand::CommitColumnResize {
            col_idx: 0,
            old_width: orig,
            old_flex: None,
        });
        assert_eq!(s.model.columns[0].width, 250.0);
        // Undo restores the width before the drag
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[0].width, orig);
    }

    #[test]
    fn commit_column_resize_noop_when_unchanged() {
        let mut s = make_state();
        let orig = s.model.columns[0].width;
        // Commit without any actual resize — no undo entry
        s.apply(GridCommand::CommitColumnResize {
            col_idx: 0,
            old_width: orig,
            old_flex: None,
        });
        // Undo should be a no-op (nothing was pushed)
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[0].width, orig);
    }

    // ── Meta: NotifyPageLoaded / SetTotalRowCount ────

    #[test]
    fn notify_page_loaded_is_noop() {
        let mut s = make_state();
        let out = s.apply(GridCommand::NotifyPageLoaded);
        assert!(matches!(out, CommandOutput::None));
    }

    #[test]
    fn set_total_row_count_is_noop_for_vec() {
        let mut s = make_state();
        let count_before = s.model.data.row_count();
        s.apply(GridCommand::SetTotalRowCount(9999));
        assert_eq!(s.model.data.row_count(), count_before);
    }

    // ── AutoFitAllColumns ────────────────────────────

    #[test]
    fn auto_fit_all_columns_adjusts_all() {
        let mut s = make_state();
        let widths_before: Vec<f64> =
            s.model.columns.iter().map(|c| c.width).collect();
        s.apply(GridCommand::AutoFitAllColumns {
            char_width: 8.4,
            header_char_width: 8.45,
            cell_padding: 10.0,
            header_right_reserve: 0.0,
        });
        for (i, old_w) in widths_before.iter().enumerate() {
            assert_ne!(
                s.model.columns[i].width, *old_w,
                "column {i} should have changed"
            );
        }
    }

    // ── Search + SearchNext selects match ──────────────

    #[test]
    fn search_next_selects_match() {
        let mut s = make_state();
        s.apply(GridCommand::Search { query: "a5".into() });
        // Search alone doesn't select; SearchNext does
        s.apply(GridCommand::SearchNext);
        assert!(s.selection.is_selected(5, 0));
    }

    // ── Sort + Filter interaction ────────────────────

    #[test]
    fn sort_then_filter_works() {
        let mut s = make_state();
        s.apply(GridCommand::SetSort {
            col_key: "a".into(),
            dir: crate::sort::SortDir::Desc,
        });
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a1".into(),
        });
        assert_eq!(s.model.display_row_count(), 1);
        assert_eq!(s.model.get_cell(0, "a"), Some("a1".into()));
    }

    #[test]
    fn filter_then_sort_works() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a".into(),
        });
        // All rows match "a"
        assert_eq!(s.model.display_row_count(), 10);
        s.apply(GridCommand::SetSort {
            col_key: "a".into(),
            dir: crate::sort::SortDir::Desc,
        });
        // a9 should be first after desc sort
        assert_eq!(s.model.get_cell(0, "a"), Some("a9".into()));
    }

    // ── Redo undo redo cycle ─────────────────────────

    #[test]
    fn redo_undo_redo_cycle() {
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
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
        s.apply(GridCommand::Redo);
        assert_eq!(s.model.get_cell(0, "a"), Some("v1".into()));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
    }

    // ── Undo cut restores values ─────────────────────

    #[test]
    fn undo_cut_restores_multiple_cells() {
        let mut s = make_state();
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 1, col: 1 }));
        s.apply(GridCommand::CutSelection);
        assert_eq!(s.model.get_cell(0, "a"), Some(String::new()));
        assert_eq!(s.model.get_cell(1, "b"), Some(String::new()));
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.get_cell(0, "a"), Some("a0".into()));
        assert_eq!(s.model.get_cell(0, "b"), Some("b0".into()));
        assert_eq!(s.model.get_cell(1, "a"), Some("a1".into()));
        assert_eq!(s.model.get_cell(1, "b"), Some("b1".into()));
    }

    // ── ResizeColumn rebuilds offsets ─────────────────

    #[test]
    fn resize_column_rebuilds_offsets() {
        let mut s = make_state();
        s.apply(GridCommand::ResizeColumn {
            col_idx: 0,
            new_width: 200.0,
        });
        assert_eq!(s.model.column_offsets.offsets[1], 200.0);
    }

    // ── Scroll horizontal ────────────────────────────

    #[test]
    fn scroll_horizontal_clamped() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollTo { x: 99999.0, y: 0.0 });
        // max_x = total_width - (vp.width - rnw - sb)
        // = 450 - (800 - rnw - 14)
        // With rnw ~33 (for 10 rows), max_x is negative → 0
        assert_eq!(s.viewport.scroll_x, 0.0);
    }

    #[test]
    fn scroll_by_negative_clamped() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollBy {
            dx: -100.0,
            dy: -100.0,
        });
        assert_eq!(s.viewport.scroll_x, 0.0);
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    // ── Flex integration ────────────────────────────

    fn make_flex_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::simple("b", "B").with_flex(1.0),
            ColumnDef::simple("c", "C").with_flex(1.0),
        ];
        let rows: Vec<RowRecord> =
            (0..5).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn flex_applied_at_init() {
        let s = make_flex_state();
        // Fixed col unchanged
        assert_eq!(s.model.columns[0].width, 100.0);
        // Flex cols share remaining space equally
        let avail = 800.0
            - s.model.row_number_width
            - s.model.scrollbar_size
            - 100.0;
        let half = avail / 2.0;
        assert!(
            (s.model.columns[1].width - half).abs() < 0.01,
            "flex col b={}, expected {half}",
            s.model.columns[1].width
        );
    }

    #[test]
    fn flex_recalculated_on_resize() {
        let mut s = make_flex_state();
        let w1 = s.model.columns[1].width;
        s.apply(GridCommand::Resize {
            width: 1200.0,
            height: 600.0,
        });
        let w2 = s.model.columns[1].width;
        assert!(
            w2 > w1,
            "flex col should grow on wider viewport: {w2} > {w1}"
        );
    }

    #[test]
    fn flex_cleared_on_resize_column() {
        let mut s = make_flex_state();
        assert!(s.model.columns[1].flex.is_some());
        s.apply(GridCommand::ResizeColumn {
            col_idx: 1,
            new_width: 200.0,
        });
        assert!(s.model.columns[1].flex.is_none());
        assert_eq!(s.model.columns[1].width, 200.0);
    }

    #[test]
    fn flex_cleared_on_autofit() {
        let mut s = make_flex_state();
        assert!(s.model.columns[1].flex.is_some());
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 1,
            char_width: 8.0,
            header_char_width: 8.0,
            cell_padding: 10.0,
            header_right_reserve: 0.0,
        });
        assert!(s.model.columns[1].flex.is_none());
    }

    // ── hit_test / hit_test_row_header / hit_test_col_header ─────────────────
    // Tests for the GridState wrapper methods (delegates to hit_test module).
    // make_state: 3 cols (100+150+200 px), 10 rows, 800×600 viewport,
    //             row_height=30, header=40.
    // row_number_width for 10 rows = max(2*9+24, 40) = 42.

    #[test]
    fn hit_test_data_cell() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vx just past gutter → col 0; vy past header → row 0
        let result = s.hit_test(rnw + 5.0, 45.0);
        assert!(result.is_some(), "expected a hit");
        let c = result.unwrap();
        assert_eq!(c.row, 0);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn hit_test_returns_none_in_gutter() {
        let s = make_state();
        // x=0 is always in the gutter
        assert!(s.hit_test(0.0, 55.0).is_none());
    }

    #[test]
    fn hit_test_returns_none_in_header() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vy < header_height=40
        assert!(s.hit_test(rnw + 5.0, 10.0).is_none());
    }

    #[test]
    fn hit_test_row_header_in_gutter() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vx = rnw/2 (inside gutter), vy = header_height + 15 = row 0
        let row = s.hit_test_row_header(rnw / 2.0, 55.0);
        assert_eq!(row, Some(0));
    }

    #[test]
    fn hit_test_row_header_returns_none_outside_gutter() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vx just past gutter → None
        assert!(s.hit_test_row_header(rnw + 5.0, 55.0).is_none());
    }

    #[test]
    fn hit_test_col_header_in_header_row() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vy=20 < header_height=40; vx=rnw+5 → col 0
        let col = s.hit_test_col_header(rnw + 5.0, 20.0);
        assert_eq!(col, Some(0));
    }

    #[test]
    fn hit_test_col_header_returns_none_below_header() {
        let s = make_state();
        let rnw = s.model.row_number_width;
        // vy=60 > header_height=40
        assert!(s.hit_test_col_header(rnw + 5.0, 60.0).is_none());
    }

    // ── SortWarning when row count exceeds the client-sort limit ─────────────

    #[test]
    fn toggle_sort_returns_sort_warning_for_large_dataset() {
        use crate::commands::CommandOutput;
        use crate::datasource::FnDataSource;
        use crate::model::GridModelBuilder;
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        // Just over the 1_000_000 row limit
        let data = Box::new(FnDataSource::new(
            GridModel::MAX_CLIENT_SORT_ROWS + 1,
            |_r, _c| None,
        ));
        let model = GridModelBuilder::new(cols, data).build();
        let mut s = GridState::new(model, 800.0, 600.0);
        let out = s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert!(
            matches!(out, CommandOutput::SortWarning { .. }),
            "expected SortWarning, got {out:?}"
        );
    }

    #[test]
    fn set_sort_returns_sort_warning_for_large_dataset() {
        use crate::commands::CommandOutput;
        use crate::datasource::FnDataSource;
        use crate::model::GridModelBuilder;
        use crate::sort::SortDir;
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let data = Box::new(FnDataSource::new(
            GridModel::MAX_CLIENT_SORT_ROWS + 1,
            |_r, _c| None,
        ));
        let model = GridModelBuilder::new(cols, data).build();
        let mut s = GridState::new(model, 800.0, 600.0);
        let out = s.apply(GridCommand::SetSort {
            col_key: "a".into(),
            dir: SortDir::Asc,
        });
        assert!(
            matches!(out, CommandOutput::SortWarning { .. }),
            "expected SortWarning, got {out:?}"
        );
    }

    // ── search scroll-into-view ─────────────────────────

    fn make_tall_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let rows = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r.set("c", format!("c{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 300.0, 200.0)
    }

    #[test]
    fn search_next_scrolls_down_to_match() {
        let mut s = make_tall_state();
        // Search for a row far below the viewport
        s.apply(GridCommand::Search { query: "a50".into() });
        assert_eq!(s.search.matches.len(), 1);
        assert_eq!(s.viewport.scroll_y, 0.0);
        // SearchNext scrolls to make row 50 visible
        s.apply(GridCommand::SearchNext);
        assert!(
            s.viewport.scroll_y > 0.0,
            "should have scrolled down, scroll_y={}",
            s.viewport.scroll_y
        );
    }

    #[test]
    fn search_next_scrolls_left_to_match() {
        // Grid with 10 cols × 200px. Viewport=400px.
        // Scroll right, then search for a value in col 0.
        let cols: Vec<ColumnDef> = (0..10)
            .map(|i| ColumnDef::new(&format!("c{i}"), &format!("C{i}"), 200.0))
            .collect();
        let mut row = RowRecord::new(0);
        row.set("c0", "findme");
        for i in 1..10 {
            row.set(&format!("c{i}"), format!("val{i}"));
        }
        let model = GridModel::new(cols, vec![row], 30.0, 40.0);
        let mut s = GridState::new(model, 400.0, 200.0);
        // Scroll far right
        s.apply(GridCommand::ScrollTo { x: 1000.0, y: 0.0 });
        assert!(s.viewport.scroll_x > 0.0);
        s.apply(GridCommand::Search {
            query: "findme".into(),
        });
        s.apply(GridCommand::SearchNext);
        // Should scroll left to show column 0
        assert!(
            s.viewport.scroll_x < 1000.0,
            "should have scrolled left, scroll_x={}",
            s.viewport.scroll_x
        );
    }

    #[test]
    fn search_next_scrolls_right_to_match() {
        // Wide grid: 10 columns × 200px = 2000px, viewport=400px
        let cols: Vec<ColumnDef> = (0..10)
            .map(|i| ColumnDef::new(&format!("c{i}"), &format!("C{i}"), 200.0))
            .collect();
        let mut row = RowRecord::new(0);
        // Put a unique value in the last column
        row.set("c9", "findme");
        for i in 0..9 {
            row.set(&format!("c{i}"), format!("val{i}"));
        }
        let model = GridModel::new(cols, vec![row], 30.0, 40.0);
        let mut s = GridState::new(model, 400.0, 200.0);
        assert_eq!(s.viewport.scroll_x, 0.0);
        s.apply(GridCommand::Search {
            query: "findme".into(),
        });
        assert_eq!(s.search.matches.len(), 1);
        s.apply(GridCommand::SearchNext);
        assert!(
            s.viewport.scroll_x > 0.0,
            "should have scrolled right to find column 9"
        );
    }

    #[test]
    fn search_next_scrolls_up_to_match() {
        let mut s = make_tall_state();
        // Scroll down first
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 1000.0 });
        assert!(s.viewport.scroll_y > 0.0);
        // Search for a row near the top
        s.apply(GridCommand::Search { query: "a0".into() });
        s.apply(GridCommand::SearchNext);
        // Should scroll back up
        assert!(
            s.viewport.scroll_y < 1000.0,
            "should have scrolled up to row 0"
        );
    }

    // ── undo SetCell with None old_value (remove patch) ─

    #[test]
    fn undo_edit_on_nonexistent_cell_removes_patch() {
        // Cell doesn't exist in datasource → get_cell returns None
        // → UndoEntry::SetCell { old_value: None }
        // → undo removes the patch entirely.
        use crate::datasource::FnDataSource;
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        // Datasource only returns values for column "a"
        let data = Box::new(FnDataSource::new(
            5,
            |row, col| {
                if col == "a" {
                    Some(format!("a{row}"))
                } else {
                    None
                }
            },
        ));
        let model = GridModel::with_data_source(
            cols,
            data,
            30.0,
            40.0,
        );
        let mut s = GridState::new(model, 800.0, 600.0);
        // Column "b" has no values in datasource
        assert_eq!(s.model.get_cell(0, "b"), None);
        // Edit column "b": old_value will be None
        s.apply(GridCommand::StartEdit {
            row: 0,
            col_key: "b".into(),
        });
        s.apply(GridCommand::CommitEdit {
            row: 0,
            col_key: "b".into(),
            value: "new_value".into(),
        });
        assert_eq!(
            s.model.get_cell(0, "b"),
            Some("new_value".into())
        );
        // Undo → old_value=None → removes patch
        s.apply(GridCommand::Undo);
        assert_eq!(
            s.model.get_cell(0, "b"),
            None,
            "undo should remove patch entirely"
        );
    }

    // ── undo ResizeColumn with flex recalculation ────────

    #[test]
    fn undo_resize_restores_flex() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::simple("b", "B").with_flex(1.0),
        ];
        let rows: Vec<RowRecord> =
            (0..3).map(|i| RowRecord::new(i)).collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);
        let orig_width = s.model.columns[1].width;
        let orig_flex = s.model.columns[1].flex;
        assert!(orig_flex.is_some());
        // Resize clears flex
        s.apply(GridCommand::ResizeColumn {
            col_idx: 1,
            new_width: 200.0,
        });
        s.apply(GridCommand::CommitColumnResize {
            col_idx: 1,
            old_width: orig_width,
            old_flex: orig_flex,
        });
        assert!(s.model.columns[1].flex.is_none());
        // Undo should restore flex
        s.apply(GridCommand::Undo);
        assert_eq!(s.model.columns[1].flex, orig_flex);
    }

    // ── AutoFit with Image format ────────────────────────

    #[test]
    fn auto_fit_image_column_uses_row_height() {
        let cols = vec![ColumnDef {
            key: "img".into(),
            label: "Img".into(),
            width: 200.0,
            min_width: None,
            max_width: None,
            flex: None,
            format: Some(CellFormat::Image {
                base_url: None,
                border_radius: 0.0,
                padding: 0.0,
            }),
            editor: None,
            validator: None,
        bold: false,
editable: true,
        }];
        let mut row = RowRecord::new(0);
        row.set("img", "photo.png");
        let model = GridModel::new(cols, vec![row], 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 8.0,
            header_char_width: 8.0,
            cell_padding: 10.0,
            header_right_reserve: 0.0,
        });
        // Image: row_height(30) + padding*2(20) = 50
        // Header: "Img".len(3)*8 + 10*2 = 44
        // max(50, 44) = 50
        assert!(
            (s.model.columns[0].width - 50.0).abs() < 0.01,
            "expected 50, got {}",
            s.model.columns[0].width
        );
    }

    // ── AutoFit with formatted (non-Image) column ────────

    #[test]
    fn auto_fit_formatted_column() {
        let cols = vec![ColumnDef {
            key: "v".into(),
            label: "V".into(),
            width: 200.0,
            min_width: None,
            max_width: None,
            flex: None,
            format: Some(CellFormat::Currency {
                symbol: "$".into(),
                decimal_places: 2,
                thousands_sep: Some(','),
                symbol_after: false,
            }),
            editor: None,
            validator: None,
        bold: false,
editable: true,
        }];
        let mut row = RowRecord::new(0);
        row.set("v", "1234.5");
        let model = GridModel::new(cols, vec![row], 30.0, 40.0);
        let mut s = GridState::new(model, 800.0, 600.0);
        s.apply(GridCommand::AutoFitColumn {
            col_idx: 0,
            char_width: 8.0,
            header_char_width: 8.0,
            cell_padding: 10.0,
            header_right_reserve: 0.0,
        });
        // "$1,234.50" = 9 chars → 9*8+10*2 = 92
        // Header: "V" = 1 char → 1*8+10*2 = 28
        // max(92, 28) = 92
        assert!(
            (s.model.columns[0].width - 92.0).abs() < 0.01,
            "expected 92, got {}",
            s.model.columns[0].width
        );
    }

    // ── sort + filter interaction (reapply_filter branch) ─

    #[test]
    fn toggle_sort_reapplies_filter() {
        let mut s = make_state();
        // Apply filter first
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a".into(),
        });
        assert_eq!(s.model.display_row_count(), 10);
        // Toggle sort → cmd_sort_filter should reapply filter
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        assert_eq!(s.model.display_row_count(), 10);
    }

    #[test]
    fn clear_sort_reapplies_filter() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "a1".into(),
        });
        s.apply(GridCommand::ToggleSort {
            col_key: "a".into(),
        });
        s.apply(GridCommand::ClearSort);
        // Filter should still be active after clearing sort
        assert_eq!(s.model.display_row_count(), 1);
    }

    // ── undo paste with SetCells None path ───────────────

    #[test]
    fn undo_paste_on_fn_datasource_removes_patches() {
        use crate::datasource::FnDataSource;
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        // Datasource only returns values for column "a"
        let data = Box::new(FnDataSource::new(5, |row, col| {
            if col == "a" {
                Some(format!("a{row}"))
            } else {
                None
            }
        }));
        let model = GridModel::with_data_source(
            cols,
            data,
            30.0,
            40.0,
        );
        let mut s = GridState::new(model, 800.0, 600.0);
        // Column b has no values → old_value=None for each cell
        assert_eq!(s.model.get_cell(0, "b"), None);
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::ExtendSelection(CellCoord {
            row: 0,
            col: 1,
        }));
        s.apply(GridCommand::PasteAt {
            text: "X\tY\n".into(),
        });
        assert_eq!(s.model.get_cell(0, "a"), Some("X".into()));
        assert_eq!(s.model.get_cell(0, "b"), Some("Y".into()));
        s.apply(GridCommand::Undo);
        assert_eq!(
            s.model.get_cell(0, "a"),
            Some("a0".into()),
            "undo should restore col a"
        );
        assert_eq!(
            s.model.get_cell(0, "b"),
            None,
            "undo should remove patches for col b"
        );
    }
}
