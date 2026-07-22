use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    sort::{SortDir, SortState},
};

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
                if !self.model.filters.is_empty()
                    || !self.model.value_filters.is_empty()
                {
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
                if !self.model.filters.is_empty()
                    || !self.model.value_filters.is_empty()
                {
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
                if !self.model.filters.is_empty()
                    || !self.model.value_filters.is_empty()
                {
                    self.model.apply_filter();
                }
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::SetColumnFilter { col_key, condition } => {
                if condition.is_empty() {
                    self.model.filters.remove(&col_key);
                } else {
                    self.model.filters.insert(col_key, condition);
                }
                self.model.apply_filter();
                self.selection.clear();
                // Filtering reshuffles which physical row a logical index
                // maps to — a checkbox shift+click gesture anchored to a
                // pre-filter logical row would otherwise resume against
                // rows it never visually spanned. Same precedent as
                // `selection.clear()` above.
                self.checked_row_anchor = None;
                self.checked_row_last_extend = None;
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::SetColumnValueFilter { col_key, values } => {
                self.model.value_filters.insert(col_key, values);
                self.model.apply_filter();
                self.selection.clear();
                // Same reshuffle rationale as `SetColumnFilter` above.
                self.checked_row_anchor = None;
                self.checked_row_last_extend = None;
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::ClearColumnValueFilter { col_key } => {
                self.model.value_filters.remove(&col_key);
                self.model.apply_filter();
                self.selection.clear();
                self.checked_row_anchor = None;
                self.checked_row_last_extend = None;
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            GridCommand::ClearAllFilters => {
                self.model.filters.clear();
                self.model.value_filters.clear();
                // Goes through apply_filter() rather than a direct
                // `filtered_indices.clear()` so its private
                // `filter_computed` bookkeeping resets too — otherwise
                // `is_filter_applied()` would keep reporting `true` from
                // before this clear, misreading the now-meaningless
                // empty `filtered_indices` as "0 rows match" instead of
                // "no filter active."
                self.model.apply_filter();
                self.selection.clear();
                self.checked_row_anchor = None;
                self.checked_row_last_extend = None;
                self.viewport.scroll_y = 0.0;
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_sort_filter"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, filter::FilterCondition,
        model::GridModel, row::RowRecord, sort::SortDir, state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("name", "Name", 150.0),
            ColumnDef::new("val", "Val", 100.0),
        ];
        let rows = vec!["Bob", "Alice", "Charlie"]
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                let mut r = RowRecord::new(i as u64);
                r.set("name", name);
                r.set("val", i.to_string());
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn toggle_sort_first_call_sets_asc() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        let sort = s.sort.as_ref().expect("sort must be set");
        assert_eq!(sort.dir, SortDir::Asc);
        assert_eq!(sort.col_key, "name");
    }

    #[test]
    fn toggle_sort_second_call_sets_desc() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        let sort = s.sort.as_ref().expect("sort must be set");
        assert_eq!(sort.dir, SortDir::Desc);
    }

    #[test]
    fn toggle_sort_third_call_clears_sort() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        assert!(s.sort.is_none());
    }

    #[test]
    fn toggle_sort_different_column_resets_to_asc() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        // Sort on a different column → reset to Asc.
        s.apply(GridCommand::ToggleSort {
            col_key: "val".into(),
        });
        let sort = s.sort.as_ref().expect("sort must be set");
        assert_eq!(sort.col_key, "val");
        assert_eq!(sort.dir, SortDir::Asc);
    }

    #[test]
    fn set_sort_applies_direction() {
        let mut s = make_state();
        s.apply(GridCommand::SetSort {
            col_key: "name".into(),
            dir: SortDir::Desc,
        });
        let sort = s.sort.as_ref().expect("sort must be set");
        assert_eq!(sort.dir, SortDir::Desc);
    }

    #[test]
    fn clear_sort_removes_sort() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        s.apply(GridCommand::ClearSort);
        assert!(s.sort.is_none());
    }

    #[test]
    fn set_column_filter_filters_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains("Alice"),
        });
        assert_eq!(s.model.display_row_count(), 1);
    }

    #[test]
    fn set_column_filter_empty_value_removes_filter() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains("Alice"),
        });
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains(""),
        });
        assert_eq!(s.model.display_row_count(), 3);
    }

    #[test]
    fn clear_all_filters_restores_all_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains("Alice"),
        });
        s.apply(GridCommand::ClearAllFilters);
        assert_eq!(s.model.display_row_count(), 3);
    }

    #[test]
    fn set_column_value_filter_restricts_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnValueFilter {
            col_key: "name".into(),
            values: std::collections::HashSet::from(["Alice".to_string()]),
        });
        assert_eq!(s.model.display_row_count(), 1);
    }

    #[test]
    fn clear_column_value_filter_restores_rows() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnValueFilter {
            col_key: "name".into(),
            values: std::collections::HashSet::from(["Alice".to_string()]),
        });
        s.apply(GridCommand::ClearColumnValueFilter {
            col_key: "name".into(),
        });
        assert_eq!(s.model.display_row_count(), 3);
    }

    #[test]
    fn set_column_value_filter_and_condition_filter_and_combine() {
        let mut s = make_state();
        // "Bob"/"Alice"/"Charlie" — case-insensitive "a" matches all
        // three; narrow with a value filter to just "Alice".
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains("a"),
        });
        s.apply(GridCommand::SetColumnValueFilter {
            col_key: "name".into(),
            values: std::collections::HashSet::from(["Alice".to_string()]),
        });
        assert_eq!(s.model.display_row_count(), 1);
    }

    #[test]
    fn clear_all_filters_also_clears_value_filters() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnValueFilter {
            col_key: "name".into(),
            values: std::collections::HashSet::from(["Alice".to_string()]),
        });
        s.apply(GridCommand::ClearAllFilters);
        assert_eq!(s.model.display_row_count(), 3);
        assert!(s.model.value_filters.is_empty());
    }

    #[test]
    fn set_column_filter_numeric_greater_than() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "val".into(),
            condition: FilterCondition {
                op: crate::filter::FilterOp::GreaterThan,
                value: "0".into(),
            },
        });
        // rows have val = 0, 1, 2 — only 1 and 2 pass > 0.
        assert_eq!(s.model.display_row_count(), 2);
    }

    #[test]
    fn set_column_filter_reapplies_after_sort_change() {
        let mut s = make_state();
        s.apply(GridCommand::SetColumnFilter {
            col_key: "name".into(),
            condition: FilterCondition::contains("a"),
        });
        // "Bob" doesn't contain "a"; "Alice" and "Charlie" do.
        assert_eq!(s.model.display_row_count(), 2);
        s.apply(GridCommand::ToggleSort {
            col_key: "name".into(),
        });
        assert_eq!(s.model.display_row_count(), 2);
    }
}
