use super::{GridState, clamp_scroll};
use crate::commands::{CommandOutput, GridCommand};

impl GridState {
    pub(super) fn cmd_scroll(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
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
                self.model.recalculate_flex_widths(width);
                self.model.rebuild_offsets();
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_scroll"),
        }
    }

    /// Scrolls the viewport, only when needed, so the cell at `(row, col)`
    /// is fully visible. Pinned columns are never scrolled horizontally
    /// since they are always on screen. The selection is not modified.
    pub(super) fn reveal_cell(&mut self, row: u64, col: usize) {
        // Vertical: keep the row between the data band and the bottom edge.
        let ry = self.model.row_top(row);
        let cy = ry - self.viewport.scroll_y;
        let hh = self.model.data_top();
        if cy < hh {
            self.viewport.scroll_y = ry - hh;
        } else if cy + self.model.row_height > self.viewport.height {
            self.viewport.scroll_y =
                ry + self.model.row_height - self.viewport.height;
        }
        // Horizontal: keep the column clear of the pinned band and the
        // right edge.
        if col < self.model.columns.len() {
            let off = self.model.column_offsets.offsets[col];
            let w = self.model.columns[col].width;
            let lgw = self.model.row_number_width
                + self.model.effective_checkbox_column_width();
            if col >= self.model.pinned_count {
                let cx = off - self.viewport.scroll_x + lgw;
                if cx < lgw + self.model.pinned_width() {
                    self.viewport.scroll_x = off - self.model.pinned_width();
                } else if cx + w > self.viewport.width {
                    self.viewport.scroll_x =
                        off + w - self.viewport.width + lgw;
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
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, state::GridState,
    };

    fn make_state() -> GridState {
        // Wide content (5 × 300 = 1500 px) + many rows so that
        // scroll positions well above 0 are reachable.
        let cols = (0..5)
            .map(|i| ColumnDef::new(format!("c{i}"), format!("C{i}"), 300.0))
            .collect();
        let rows = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("c0", format!("v{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 400.0, 300.0)
    }

    #[test]
    fn scroll_to_sets_position() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollTo { x: 10.0, y: 20.0 });
        assert_eq!(s.viewport.scroll_x, 10.0);
        assert_eq!(s.viewport.scroll_y, 20.0);
    }

    #[test]
    fn scroll_to_clamps_negative() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollTo {
            x: -50.0,
            y: -100.0,
        });
        assert_eq!(s.viewport.scroll_x, 0.0);
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

    #[test]
    fn scroll_to_clamps_beyond_max() {
        let mut s = make_state();
        // Scroll to a position far beyond content size — must be clamped.
        s.apply(GridCommand::ScrollTo {
            x: 99999.0,
            y: 99999.0,
        });
        assert!(s.viewport.scroll_x < 99999.0);
        assert!(s.viewport.scroll_y < 99999.0);
    }

    #[test]
    fn scroll_by_accumulates() {
        let mut s = make_state();
        s.apply(GridCommand::ScrollBy { dx: 10.0, dy: 5.0 });
        s.apply(GridCommand::ScrollBy { dx: 10.0, dy: 5.0 });
        assert_eq!(s.viewport.scroll_x, 20.0);
        assert_eq!(s.viewport.scroll_y, 10.0);
    }

    #[test]
    fn scroll_by_clamps_to_zero() {
        let mut s = make_state();
        // Scroll forward then back past origin — must not go negative.
        s.apply(GridCommand::ScrollBy { dx: 20.0, dy: 10.0 });
        s.apply(GridCommand::ScrollBy {
            dx: -200.0,
            dy: -200.0,
        });
        assert_eq!(s.viewport.scroll_x, 0.0);
        assert_eq!(s.viewport.scroll_y, 0.0);
    }

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
}
