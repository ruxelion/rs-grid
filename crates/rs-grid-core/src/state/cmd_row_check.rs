use super::GridState;
use crate::{
    commands::{CommandOutput, GridCommand},
    row_check::CheckboxTriState,
};

impl GridState {
    pub(super) fn cmd_row_check(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::ToggleRowChecked(logical_row) => {
                let physical = self.model.logical_to_physical(logical_row);
                if !self.checked_rows.remove(&physical) {
                    self.checked_rows.insert(physical);
                }
                CommandOutput::None
            }
            GridCommand::ToggleAllFilteredChecked => {
                let scope: Vec<u64> = if !self.model.filtered_indices.is_empty()
                {
                    self.model.filtered_indices.clone()
                } else {
                    (0..self.model.data.row_count()).collect()
                };
                if self.checkbox_header_state() == CheckboxTriState::Checked {
                    for id in &scope {
                        self.checked_rows.remove(id);
                    }
                } else {
                    self.checked_rows.extend(scope);
                }
                CommandOutput::None
            }
            _ => super::unreachable_cmd("cmd_row_check"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::ColumnDef, commands::GridCommand, model::GridModel,
        row::RowRecord, row_check::CheckboxTriState, sort::SortDir,
        state::GridState,
    };

    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        let rows = (0..5)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", (4 - i).to_string());
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    #[test]
    fn toggle_row_checked_sets_and_clears() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(2));
        assert!(s.checked_rows.contains(&2));
        s.apply(GridCommand::ToggleRowChecked(2));
        assert!(!s.checked_rows.contains(&2));
    }

    #[test]
    fn toggle_row_checked_survives_sort() {
        let mut s = make_state();
        // Logical row 2 → physical row 2 before any sort.
        s.apply(GridCommand::ToggleRowChecked(2));
        assert!(s.checked_rows.contains(&2));

        // Sort by column "a" (values are `4 - i`, so ascending sort
        // reverses physical order entirely).
        s.model.apply_sort("a", &SortDir::Asc);

        // The same physical row must still be checked, regardless of
        // where it now displays.
        assert!(s.checked_rows.contains(&2));
        assert_eq!(s.checked_rows.len(), 1);
    }

    #[test]
    fn toggle_all_filtered_checked_scopes_to_filter() {
        let mut s = make_state();
        // Values of "a" are 4-i, so filtering for "1" keeps only
        // physical row 3.
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "1".into(),
        });
        assert_eq!(s.model.filtered_indices, vec![3]);

        // A row outside the filtered scope, checked beforehand —
        // must never be touched by ToggleAllFilteredChecked.
        s.checked_rows.insert(0);

        s.apply(GridCommand::ToggleAllFilteredChecked);
        assert_eq!(s.checked_rows, [0, 3].into_iter().collect());

        // Header state is now Checked (row 3, the only row in scope,
        // is checked); toggling again unchecks only the filtered
        // scope, leaving row 0 untouched.
        s.apply(GridCommand::ToggleAllFilteredChecked);
        assert_eq!(s.checked_rows, [0].into_iter().collect());
    }

    #[test]
    fn header_state_indeterminate_then_checked() {
        let mut s = make_state();
        assert_eq!(s.checkbox_header_state(), CheckboxTriState::Unchecked);
        s.apply(GridCommand::ToggleRowChecked(0));
        assert_eq!(s.checkbox_header_state(), CheckboxTriState::Indeterminate);
        for i in 1..5 {
            s.apply(GridCommand::ToggleRowChecked(i));
        }
        assert_eq!(s.checkbox_header_state(), CheckboxTriState::Checked);
    }
}
