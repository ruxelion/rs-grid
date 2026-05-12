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
            GridCommand::SetRowHeight(h) => {
                if h > 0.0 {
                    self.model.row_height = h;
                }
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
            GridCommand::SetShowHeader(v) => {
                self.model.show_header = v;
                CommandOutput::None
            }
            GridCommand::SetShowRowNumbers(v) => {
                self.model.show_row_numbers = v;
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
}
