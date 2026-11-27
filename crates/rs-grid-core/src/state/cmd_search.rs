use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    search::SearchState,
};

impl GridState {
    pub(super) fn cmd_search(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
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
            _ => super::unreachable_cmd("cmd_search"),
        }
    }
}

impl GridState {
    fn run_search(&mut self, query: &str) {
        self.search = SearchState::run(&self.model, query);
    }

    fn scroll_to_search_match(&mut self) {
        let coord = match self.search.matches.get(self.search.current) {
            Some(c) => c.clone(),
            None => return,
        };
        // Select the matched cell, then scroll it into view.
        self.selection.select_cell(coord.row, coord.col);
        self.reveal_cell(coord.row, coord.col);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![ColumnDef::new("name", "Name", 150.0)];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("name", "Alice");
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("name", "Bob");
                r
            },
            {
                let mut r = RowRecord::new(2);
                r.set("name", "Alice2");
                r
            },
        ];
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn search_finds_matches() {
        let mut s = make_state();
        s.apply(GridCommand::Search {
            query: "Alice".into(),
        });
        assert_eq!(s.search.matches.len(), 2);
    }

    #[test]
    fn search_next_advances_current() {
        let mut s = make_state();
        s.apply(GridCommand::Search {
            query: "Alice".into(),
        });
        let before = s.search.current;
        s.apply(GridCommand::SearchNext);
        assert_ne!(s.search.current, before);
    }

    #[test]
    fn search_next_wraps_around() {
        let mut s = make_state();
        s.apply(GridCommand::Search {
            query: "Alice".into(),
        });
        // Advance past the last match — should wrap to 0.
        s.apply(GridCommand::SearchNext);
        s.apply(GridCommand::SearchNext);
        assert_eq!(s.search.current, 0);
    }

    #[test]
    fn search_prev_wraps_around() {
        let mut s = make_state();
        s.apply(GridCommand::Search {
            query: "Alice".into(),
        });
        // Going prev from 0 should wrap to last index.
        s.apply(GridCommand::SearchPrev);
        assert_eq!(s.search.current, 1);
    }

    #[test]
    fn search_next_empty_results_is_noop() {
        let mut s = make_state();
        // No search active — SearchNext must not panic.
        s.apply(GridCommand::SearchNext);
        assert_eq!(s.search.current, 0);
    }

    #[test]
    fn search_prev_empty_results_is_noop() {
        let mut s = make_state();
        s.apply(GridCommand::SearchPrev);
        assert_eq!(s.search.current, 0);
    }

    #[test]
    fn clear_search_resets_state() {
        let mut s = make_state();
        s.apply(GridCommand::Search {
            query: "Alice".into(),
        });
        assert!(!s.search.matches.is_empty());
        s.apply(GridCommand::ClearSearch);
        assert!(s.search.matches.is_empty());
    }
}
