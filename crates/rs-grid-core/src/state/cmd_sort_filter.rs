use crate::{
    commands::{CommandOutput, GridCommand},
    sort::{SortDir, SortState},
};

use super::GridState;

impl GridState {
    pub(super) fn cmd_sort_filter(
        &mut self,
        cmd: GridCommand,
    ) -> CommandOutput {
        match cmd {
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
                let sorted = match &new_sort {
                    Some(s) => self.model.apply_sort(&s.col_key, &s.dir),
                    None => {
                        self.model.sort_order.clear();
                        self.model.invalidate_sort_cache();
                        true
                    }
                };
                self.sort = new_sort;
                // Reapply filter with updated sort order.
                if !self.model.filters.is_empty() {
                    self.model.apply_filter();
                }
                self.viewport.scroll_y = 0.0;
                if sorted {
                    CommandOutput::None
                } else {
                    CommandOutput::SortWarning {
                        row_count: self.model.data.row_count(),
                        limit: crate::model::GridModel::MAX_CLIENT_SORT_ROWS,
                    }
                }
            }
            GridCommand::SetSort { col_key, dir } => {
                self.sort = Some(SortState {
                    col_key: col_key.clone(),
                    dir: dir.clone(),
                });
                let sorted = self.model.apply_sort(&col_key, &dir);
                if !self.model.filters.is_empty() {
                    self.model.apply_filter();
                }
                self.viewport.scroll_y = 0.0;
                if sorted {
                    CommandOutput::None
                } else {
                    CommandOutput::SortWarning {
                        row_count: self.model.data.row_count(),
                        limit: crate::model::GridModel::MAX_CLIENT_SORT_ROWS,
                    }
                }
            }
            GridCommand::ClearSort => {
                self.sort = None;
                self.model.sort_order.clear();
                self.model.invalidate_sort_cache();
                if !self.model.filters.is_empty() {
                    self.model.apply_filter();
                }
                self.viewport.scroll_y = 0.0;
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
            _ => {
                debug_assert!(false, "cmd_sort_filter: unsupported variant");
                CommandOutput::None
            }
        }
    }
}
