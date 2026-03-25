use crate::{
    commands::{CommandOutput, GridCommand},
    search::SearchState,
};

use super::{clamp_scroll, GridState};

impl GridState {
    pub(super) fn cmd_search(
        &mut self,
        cmd: GridCommand,
    ) -> CommandOutput {
        match cmd {
            GridCommand::Search { query } => {
                self.run_search(&query);
                CommandOutput::None
            }
            GridCommand::SearchNext => {
                if !self.search.matches.is_empty() {
                    self.search.current = (self.search.current + 1)
                        % self.search.matches.len();
                    self.scroll_to_search_match();
                }
                CommandOutput::None
            }
            GridCommand::SearchPrev => {
                if !self.search.matches.is_empty() {
                    let len = self.search.matches.len();
                    self.search.current =
                        (self.search.current + len - 1) % len;
                    self.scroll_to_search_match();
                }
                CommandOutput::None
            }
            GridCommand::ClearSearch => {
                self.search = SearchState::default();
                CommandOutput::None
            }
            _ => unreachable!(),
        }
    }

    fn run_search(&mut self, query: &str) {
        self.search = SearchState::run(&self.model, query);
    }

    fn scroll_to_search_match(&mut self) {
        let coord =
            match self.search.matches.get(self.search.current) {
                Some(c) => c.clone(),
                None => return,
            };
        // Select the matched cell.
        self.selection.select_cell(coord.row, coord.col);
        // Scroll to make the cell visible vertically.
        let ry = self.model.row_top(coord.row);
        let cy = ry - self.viewport.scroll_y;
        if cy < self.model.header_height {
            self.viewport.scroll_y =
                ry - self.model.header_height;
        } else if cy + self.model.row_height > self.viewport.height {
            self.viewport.scroll_y =
                ry + self.model.row_height - self.viewport.height;
        }
        // Scroll to make the cell visible horizontally.
        if coord.col < self.model.columns.len() {
            let off =
                self.model.column_offsets.offsets[coord.col];
            let w = self.model.columns[coord.col].width;
            let rnw = self.model.row_number_width;
            // Don't scroll for pinned columns.
            if coord.col >= self.model.pinned_count {
                let cx = off - self.viewport.scroll_x + rnw;
                if cx < rnw + self.model.pinned_width() {
                    self.viewport.scroll_x =
                        off - self.model.pinned_width();
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
}
