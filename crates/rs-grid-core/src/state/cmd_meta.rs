use super::GridState;
use crate::commands::{CommandOutput, GridCommand};

impl GridState {
    pub(super) fn cmd_meta(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::SetHoveredRow(row) => {
                self.hovered_row = row;
                CommandOutput::None
            }
            GridCommand::SetHeaderHeight(h) => {
                if h > 0.0 {
                    self.model.header_height = h;
                }
                CommandOutput::None
            }
            GridCommand::SetFilterRowHeight(h) => {
                if h > 0.0 {
                    self.model.filter_row_height = h;
                }
                CommandOutput::None
            }
            GridCommand::SetRowHeight(h) => {
                if h > 0.0 {
                    self.model.row_height = h;
                }
                CommandOutput::None
            }
            GridCommand::SetRowNumberWidth(w) => {
                if w >= 0.0 {
                    self.model.row_number_width = w;
                    // A manual width is a deliberate override — stop
                    // auto-recomputing it on the next total-count update.
                    self.model.row_number_width_auto = false;
                }
                CommandOutput::None
            }
            GridCommand::SetCheckboxColumnWidth(w) => {
                if w >= 0.0 {
                    self.model.checkbox_column_width = w;
                }
                CommandOutput::None
            }
            GridCommand::NotifyPageLoaded => {
                // No-op — triggers a re-render via dispatch.
                CommandOutput::None
            }
            GridCommand::SetTotalRowCount(n) => {
                // The data source's own row count is updated
                // externally (e.g. `PageCacheDataSource::set_total_rows`)
                // — `GridState` has no generic way to mutate the boxed
                // `dyn DataSource`. What this command *can* and does own:
                // keeping the gutter width in sync, unless the caller
                // fixed it manually via `SetRowNumberWidth`.
                if self.model.row_number_width_auto {
                    self.model.row_number_width =
                        crate::model::GridModel::compute_row_number_width(n);
                }
                // The row set may have shrunk below the current page.
                self.model.clamp_page();
                CommandOutput::None
            }
            GridCommand::SetShowFooter(v) => {
                self.model.show_footer = v;
                self.sync_viewport_height();
                CommandOutput::None
            }
            GridCommand::SetFooterHeight(h) => {
                if h > 0.0 {
                    self.model.footer_height = h;
                    self.sync_viewport_height();
                }
                CommandOutput::None
            }
            GridCommand::SetFooterLabels(labels) => {
                self.model.footer_labels = labels;
                CommandOutput::None
            }
            GridCommand::SetPaginationEnabled(v) => {
                if self.model.pagination_enabled != v {
                    self.model.pagination_enabled = v;
                    // The rows' vertical origin moves with the window.
                    self.viewport.scroll_y = 0.0;
                    self.model.clamp_page();
                    self.sync_viewport_height();
                }
                CommandOutput::None
            }
            GridCommand::SetPage(p) => {
                if self.model.pagination_enabled {
                    let p = p.min(self.model.page_count() - 1);
                    if p != self.model.current_page() {
                        self.viewport.scroll_y = 0.0;
                    }
                    self.model.page = p;
                }
                CommandOutput::None
            }
            GridCommand::SetPageSize(n) => {
                let n = n.max(1);
                // Keep the first row currently shown on screen.
                let first = self.model.page_start();
                self.model.page_size = n;
                self.model.page = first / n;
                self.viewport.scroll_y = 0.0;
                self.model.clamp_page();
                CommandOutput::None
            }
            GridCommand::SetPageSizeOptions(options) => {
                let options: Vec<u64> =
                    options.into_iter().filter(|&n| n > 0).collect();
                if !options.is_empty() {
                    self.model.page_size_options = options;
                }
                CommandOutput::None
            }
            GridCommand::SetShowHeader(v) => {
                self.model.show_header = v;
                CommandOutput::None
            }
            GridCommand::SetShowRowNumbers(v) => {
                self.model.show_row_numbers = v;
                CommandOutput::None
            }
            GridCommand::SetShowCheckboxColumn(v) => {
                self.model.show_checkbox_column = v;
                CommandOutput::None
            }
            GridCommand::SetShowFilterRow(v) => {
                self.model.show_filter_row = v;
                CommandOutput::None
            }
            GridCommand::SetEditable(v) => {
                self.model.editable = v;
                CommandOutput::None
            }
            GridCommand::SetSelectable(v) => {
                self.model.selectable = v;
                if !v {
                    self.selection.clear();
                }
                CommandOutput::None
            }
            GridCommand::SetColumnReorderable(v) => {
                self.model.column_reorderable = v;
                CommandOutput::None
            }
            GridCommand::SetInvalidEditMode(v) => {
                self.model.invalid_edit_mode = v;
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_meta"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows = (0..5)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn set_hovered_row_updates_field() {
        let mut s = make_state();
        s.apply(GridCommand::SetHoveredRow(Some(3)));
        assert_eq!(s.hovered_row, Some(3));
        s.apply(GridCommand::SetHoveredRow(None));
        assert!(s.hovered_row.is_none());
    }

    #[test]
    fn set_header_height_positive_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetHeaderHeight(60.0));
        assert_eq!(s.model.header_height, 60.0);
    }

    #[test]
    fn set_header_height_zero_is_ignored() {
        let mut s = make_state();
        let before = s.model.header_height;
        s.apply(GridCommand::SetHeaderHeight(0.0));
        assert_eq!(s.model.header_height, before);
    }

    #[test]
    fn set_filter_row_height_positive_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetFilterRowHeight(30.0));
        assert_eq!(s.model.filter_row_height, 30.0);
    }

    #[test]
    fn set_filter_row_height_zero_is_ignored() {
        let mut s = make_state();
        let before = s.model.filter_row_height;
        s.apply(GridCommand::SetFilterRowHeight(0.0));
        assert_eq!(s.model.filter_row_height, before);
    }

    #[test]
    fn set_row_height_positive_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetRowHeight(50.0));
        assert_eq!(s.model.row_height, 50.0);
    }

    #[test]
    fn set_row_height_zero_is_ignored() {
        let mut s = make_state();
        let before = s.model.row_height;
        s.apply(GridCommand::SetRowHeight(0.0));
        assert_eq!(s.model.row_height, before);
    }

    #[test]
    fn set_row_number_width_positive_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetRowNumberWidth(120.0));
        assert_eq!(s.model.row_number_width, 120.0);
    }

    #[test]
    fn set_row_number_width_zero_updates() {
        // Unlike header/row height, 0 is a valid gutter width (hidden).
        let mut s = make_state();
        s.apply(GridCommand::SetRowNumberWidth(0.0));
        assert_eq!(s.model.row_number_width, 0.0);
    }

    #[test]
    fn set_row_number_width_negative_is_ignored() {
        let mut s = make_state();
        let before = s.model.row_number_width;
        s.apply(GridCommand::SetRowNumberWidth(-10.0));
        assert_eq!(s.model.row_number_width, before);
    }

    #[test]
    fn set_checkbox_column_width_positive_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetCheckboxColumnWidth(60.0));
        assert_eq!(s.model.checkbox_column_width, 60.0);
    }

    #[test]
    fn set_checkbox_column_width_zero_updates() {
        let mut s = make_state();
        s.apply(GridCommand::SetCheckboxColumnWidth(0.0));
        assert_eq!(s.model.checkbox_column_width, 0.0);
    }

    #[test]
    fn set_checkbox_column_width_negative_is_ignored() {
        let mut s = make_state();
        let before = s.model.checkbox_column_width;
        s.apply(GridCommand::SetCheckboxColumnWidth(-10.0));
        assert_eq!(s.model.checkbox_column_width, before);
    }

    #[test]
    fn notify_page_loaded_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::NotifyPageLoaded);
    }

    #[test]
    fn set_total_row_count_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SetTotalRowCount(9999));
    }

    #[test]
    fn set_show_header_false() {
        let mut s = make_state();
        s.apply(GridCommand::SetShowHeader(false));
        assert!(!s.model.show_header);
    }

    #[test]
    fn set_show_row_numbers_false() {
        let mut s = make_state();
        s.apply(GridCommand::SetShowRowNumbers(false));
        assert!(!s.model.show_row_numbers);
    }

    #[test]
    fn set_show_checkbox_column_true() {
        let mut s = make_state();
        assert!(!s.model.show_checkbox_column);
        s.apply(GridCommand::SetShowCheckboxColumn(true));
        assert!(s.model.show_checkbox_column);
    }

    #[test]
    fn set_show_filter_row_true() {
        let mut s = make_state();
        assert!(!s.model.show_filter_row);
        s.apply(GridCommand::SetShowFilterRow(true));
        assert!(s.model.show_filter_row);
    }

    #[test]
    fn set_editable_false() {
        let mut s = make_state();
        s.apply(GridCommand::SetEditable(false));
        assert!(!s.model.editable);
    }

    #[test]
    fn set_selectable_false_clears_selection() {
        let mut s = make_state();
        use crate::selection::CellCoord;
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        assert!(s.selection.has_selection());
        s.apply(GridCommand::SetSelectable(false));
        assert!(!s.selection.has_selection());
        assert!(!s.model.selectable);
    }

    #[test]
    fn set_selectable_true_does_not_clear() {
        let mut s = make_state();
        use crate::selection::CellCoord;
        s.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
        s.apply(GridCommand::SetSelectable(true));
        assert!(s.selection.has_selection());
    }

    #[test]
    fn set_column_reorderable_false() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnReorderable(false));
        assert!(!s.model.column_reorderable);
    }

    #[test]
    fn set_invalid_edit_mode_toggles() {
        use crate::validation::InvalidEditMode;
        let mut s = make_state();
        assert_eq!(s.model.invalid_edit_mode, InvalidEditMode::Revert);
        s.apply(GridCommand::SetInvalidEditMode(InvalidEditMode::Block));
        assert_eq!(s.model.invalid_edit_mode, InvalidEditMode::Block);
    }

    // ── Footer & pagination ─────────────────────────────────

    /// `rows` rows, row_height 30, header 40, 800×600 canvas.
    fn make_rows_state(rows: u64) -> GridState {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let data = (0..rows)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r
            })
            .collect();
        GridState::new(GridModel::new(cols, data, 30.0, 40.0), 800.0, 600.0)
    }

    fn paged(rows: u64, page_size: u64) -> GridState {
        let mut s = make_rows_state(rows);
        s.apply(GridCommand::SetPaginationEnabled(true));
        s.apply(GridCommand::SetPageSize(page_size));
        s
    }

    #[test]
    fn footer_height_is_reserved_out_of_the_viewport() {
        let mut s = make_rows_state(10);
        assert_eq!(s.viewport.height, 600.0);
        s.apply(GridCommand::SetShowFooter(true));
        assert_eq!(s.viewport.height, 600.0 - s.model.footer_height);
        s.apply(GridCommand::Resize {
            width: 800.0,
            height: 700.0,
        });
        assert_eq!(s.viewport.canvas_height, 700.0);
        assert_eq!(s.viewport.height, 700.0 - s.model.footer_height);
        s.apply(GridCommand::SetShowFooter(false));
        assert_eq!(s.viewport.height, 700.0);
    }

    #[test]
    fn footer_height_zero_is_ignored_and_positive_resizes_body() {
        let mut s = make_rows_state(10);
        s.apply(GridCommand::SetShowFooter(true));
        let before = s.model.footer_height;
        s.apply(GridCommand::SetFooterHeight(0.0));
        assert_eq!(s.model.footer_height, before);
        s.apply(GridCommand::SetFooterHeight(60.0));
        assert_eq!(s.viewport.height, 540.0);
    }

    #[test]
    fn footer_shown_at_construction_reserves_height_from_the_start() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let model = crate::model::GridModelBuilder::new(
            cols,
            Box::new(crate::datasource::VecDataSource::new(Vec::new())),
        )
        .show_footer(true)
        .build();
        let s = GridState::new(model, 800.0, 600.0);
        assert_eq!(s.viewport.height, 600.0 - s.model.footer_height);
    }

    #[test]
    fn pagination_off_leaves_geometry_untouched() {
        let s = make_rows_state(100);
        assert_eq!(s.model.page_window(), (0, 100));
        assert_eq!(s.model.page_count(), 1);
        assert_eq!(s.model.total_height(), 40.0 + 100.0 * 30.0);
        assert_eq!(s.model.row_top(99), 40.0 + 99.0 * 30.0);
    }

    #[test]
    fn page_window_and_geometry_follow_the_current_page() {
        let mut s = paged(100, 25);
        assert_eq!(s.model.page_count(), 4);
        assert_eq!(s.model.page_window(), (0, 25));
        s.apply(GridCommand::SetPage(2));
        assert_eq!(s.model.page_window(), (50, 75));
        // Rows are laid out relative to the page's first row.
        assert_eq!(s.model.row_top(50), 40.0);
        assert_eq!(s.model.row_top(51), 70.0);
        assert_eq!(s.model.total_height(), 40.0 + 25.0 * 30.0);
    }

    #[test]
    fn hit_test_returns_global_rows_of_the_current_page() {
        let mut s = paged(100, 25);
        s.apply(GridCommand::SetPage(2));
        // vx 100 = first data column (past the 50px gutter); vy 55 =
        // middle of the first data row (header is 40px).
        assert_eq!(s.hit_test(100.0, 55.0).map(|c| c.row), Some(50));
        assert_eq!(s.hit_test(100.0, 85.0).map(|c| c.row), Some(51));
    }

    #[test]
    fn hit_test_below_the_last_row_of_a_short_last_page_is_none() {
        let mut s = paged(60, 25);
        s.apply(GridCommand::SetPage(2));
        assert_eq!(s.model.page_window(), (50, 60));
        // 10 rows on the page: row index 10 is empty space.
        let vy = 40.0 + 10.0 * 30.0 + 5.0;
        assert!(s.hit_test(100.0, vy).is_none());
        let vy = 40.0 + 9.0 * 30.0 + 5.0;
        assert_eq!(s.hit_test(100.0, vy).map(|c| c.row), Some(59));
    }

    #[test]
    fn set_page_is_clamped_and_resets_scroll() {
        let mut s = paged(100, 25);
        s.apply(GridCommand::ScrollTo { x: 0.0, y: 100.0 });
        assert!(s.viewport.scroll_y > 0.0);
        s.apply(GridCommand::SetPage(99));
        assert_eq!(s.model.current_page(), 3);
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    #[test]
    fn set_page_is_a_noop_without_pagination() {
        let mut s = make_rows_state(100);
        s.apply(GridCommand::SetPage(3));
        assert_eq!(s.model.page, 0);
        assert_eq!(s.model.page_window(), (0, 100));
    }

    #[test]
    fn set_page_size_keeps_the_first_visible_row_on_screen() {
        let mut s = paged(1_000, 25);
        s.apply(GridCommand::SetPage(5)); // rows 125..150
        s.apply(GridCommand::SetPageSize(50)); // 125 lies on page 2 (100..150)
        assert_eq!(s.model.page_window(), (100, 150));
        s.apply(GridCommand::SetPageSize(0)); // clamped to 1 row per page
        assert_eq!(s.model.page_size, 1);
        assert_eq!(s.model.page_window(), (100, 101));
    }

    #[test]
    fn filtering_clamps_the_page() {
        use crate::filter::{FilterCondition, FilterOp};
        let mut s = paged(100, 10);
        s.apply(GridCommand::SetPage(9));
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".to_owned(),
            condition: FilterCondition {
                op: FilterOp::Equals,
                value: "a3".to_owned(),
            },
        });
        assert_eq!(s.model.display_row_count(), 1);
        assert_eq!(s.model.page, 0);
        assert_eq!(s.model.page_window(), (0, 1));
    }

    #[test]
    fn empty_grid_has_one_empty_page() {
        let s = paged(0, 25);
        assert_eq!(s.model.page_count(), 1);
        assert_eq!(s.model.page_window(), (0, 0));
        assert_eq!(s.model.page_row_count(), 0);
    }

    #[test]
    fn reveal_cell_switches_to_the_page_holding_the_row() {
        let mut s = paged(100, 25);
        s.reveal_cell(60, 0);
        assert_eq!(s.model.current_page(), 2);
        assert!(s.model.page_window().0 <= 60 && 60 < s.model.page_window().1);
        s.reveal_cell(3, 0);
        assert_eq!(s.model.current_page(), 0);
    }

    #[test]
    fn page_size_options_drop_zeros_and_ignore_empty() {
        let mut s = make_rows_state(10);
        s.apply(GridCommand::SetPageSizeOptions(vec![0, 5, 20]));
        assert_eq!(s.model.page_size_options, vec![5, 20]);
        s.apply(GridCommand::SetPageSizeOptions(vec![0]));
        assert_eq!(s.model.page_size_options, vec![5, 20]);
    }

    #[test]
    fn selected_row_count_uses_checked_rows_when_checkboxes_shown() {
        use crate::selection::CellCoord;
        let mut s = make_rows_state(10);
        // No checkbox column: the selection range height.
        assert_eq!(s.selected_row_count(), 0);
        s.apply(GridCommand::SelectCell(CellCoord { row: 2, col: 0 }));
        assert_eq!(s.selected_row_count(), 1);
        s.apply(GridCommand::ExtendSelection(CellCoord { row: 5, col: 0 }));
        assert_eq!(s.selected_row_count(), 4);
        // Checkbox column: checked rows win over the range.
        s.apply(GridCommand::SetShowCheckboxColumn(true));
        assert_eq!(s.selected_row_count(), 0);
        s.apply(GridCommand::ToggleRowChecked(1));
        s.apply(GridCommand::ToggleRowChecked(7));
        assert_eq!(s.selected_row_count(), 2);
    }
}
