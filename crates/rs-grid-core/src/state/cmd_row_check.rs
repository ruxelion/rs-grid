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
                let checking = if self.checked_rows.remove(&physical) {
                    false
                } else {
                    self.checked_rows.insert(physical);
                    true
                };
                // Starts a fresh gesture: the anchor's own direction
                // applies to every following ExtendRowChecked, and
                // there is no prior extend to reconcile against yet.
                self.checked_row_anchor = Some((logical_row, checking));
                self.checked_row_last_extend = Some(logical_row);
                CommandOutput::None
            }
            GridCommand::ExtendRowChecked(logical_row) => {
                let (anchor, checking) =
                    *self.checked_row_anchor.get_or_insert((logical_row, true));
                let prev_focus = self.checked_row_last_extend.unwrap_or(anchor);
                let range = |a: u64, b: u64| {
                    if a <= b { (a, b) } else { (b, a) }
                };
                let (new_lo, new_hi) = range(anchor, logical_row);
                let (old_lo, old_hi) = range(anchor, prev_focus);
                for r in new_lo..=new_hi {
                    let physical = self.model.logical_to_physical(r);
                    if checking {
                        self.checked_rows.insert(physical);
                    } else {
                        self.checked_rows.remove(&physical);
                    }
                }
                // Rows the previous extend touched but the new range no
                // longer covers "give back" to the opposite of this
                // gesture's direction — moving the shift+click focus
                // inward un-does what moving it outward had just done.
                for r in old_lo..=old_hi {
                    if r >= new_lo && r <= new_hi {
                        continue;
                    }
                    let physical = self.model.logical_to_physical(r);
                    if checking {
                        self.checked_rows.remove(&physical);
                    } else {
                        self.checked_rows.insert(physical);
                    }
                }
                self.checked_row_last_extend = Some(logical_row);
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
                // Invalidate any in-progress shift+click gesture: its
                // anchor's direction/range no longer corresponds to what
                // this bulk toggle just did, so a following
                // ExtendRowChecked must start a fresh gesture instead of
                // replaying a stale one.
                self.checked_row_anchor = None;
                self.checked_row_last_extend = None;
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
    fn extend_row_checked_checks_range_from_anchor() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(1));
        s.apply(GridCommand::ExtendRowChecked(3));
        assert_eq!(s.checked_rows, [1, 2, 3].into_iter().collect());
    }

    #[test]
    fn extend_row_checked_handles_reversed_range() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(3));
        s.apply(GridCommand::ExtendRowChecked(1));
        assert_eq!(s.checked_rows, [1, 2, 3].into_iter().collect());
    }

    #[test]
    fn extend_row_checked_without_prior_anchor_checks_single_row() {
        let mut s = make_state();
        s.apply(GridCommand::ExtendRowChecked(2));
        assert_eq!(s.checked_rows, [2].into_iter().collect());
    }

    #[test]
    fn extend_row_checked_shrinks_range_when_focus_moves_inward() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(0));
        s.apply(GridCommand::ExtendRowChecked(4));
        assert_eq!(s.checked_rows, (0..=4).collect());

        // Still holding shift, moving the focus inward to row 3 must
        // give back row 4 — the checked set tracks the live
        // [anchor, focus] range like a drag-selection, not just grow.
        s.apply(GridCommand::ExtendRowChecked(3));
        assert_eq!(s.checked_rows, (0..=3).collect());
    }

    #[test]
    fn extend_row_checked_shrink_works_for_an_unchecking_gesture_too() {
        let mut s = make_state();
        s.checked_rows.extend(0..5);
        // Un-checking row 0 starts an "uncheck" gesture.
        s.apply(GridCommand::ToggleRowChecked(0));
        s.apply(GridCommand::ExtendRowChecked(4));
        assert!(s.checked_rows.is_empty());

        // Shrinking the gesture back to row 3 gives row 4 back its
        // pre-gesture (checked) state.
        s.apply(GridCommand::ExtendRowChecked(3));
        assert_eq!(s.checked_rows, [4].into_iter().collect());
    }

    #[test]
    fn extend_row_checked_keeps_anchor_across_consecutive_extends() {
        let mut s = make_state();
        // No plain ToggleRowChecked at all — the very first checkbox
        // interaction of the gesture is a shift+click.
        s.apply(GridCommand::ExtendRowChecked(2));
        assert_eq!(s.checked_rows, [2].into_iter().collect());

        // A second shift+click, still with no intervening plain click,
        // must extend from the *same* anchor (row 2) rather than
        // re-deriving a fresh one from row 5 and dropping row 2.
        s.apply(GridCommand::ExtendRowChecked(5));
        assert_eq!(s.checked_rows, (2..=5).collect());
    }

    #[test]
    fn toggle_all_filtered_checked_resets_gesture_anchor() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(2));
        s.apply(GridCommand::ExtendRowChecked(4));
        assert_eq!(s.checked_rows, (2..=4).collect());

        // A bulk header toggle checks every row — this must invalidate
        // the in-progress gesture so it can't be replayed afterwards.
        s.apply(GridCommand::ToggleAllFilteredChecked);
        assert_eq!(s.checked_rows, (0..5).collect());

        // Shift+click with no new plain click first: must start a
        // fresh gesture (anchor = row 1, checking), not resume the
        // stale anchor=2 range and undo part of the bulk check.
        s.apply(GridCommand::ExtendRowChecked(1));
        assert_eq!(s.checked_rows, (0..5).collect());
    }

    #[test]
    fn set_column_filter_resets_gesture_anchor() {
        let mut s = make_state();
        s.apply(GridCommand::ToggleRowChecked(0));
        s.apply(GridCommand::ExtendRowChecked(2));
        assert_eq!(s.checked_rows, (0..=2).collect());

        // Filtering reshuffles logical→physical mapping — the anchor
        // must not survive to be reinterpreted against it.
        s.apply(GridCommand::SetColumnFilter {
            col_key: "a".into(),
            text: "1".into(),
        });
        assert_eq!(s.checked_row_anchor, None);
        assert_eq!(s.checked_row_last_extend, None);

        // A following shift+click with no new plain click starts a
        // fresh single-row gesture instead of reusing the pre-filter
        // anchor.
        s.checked_rows.clear();
        s.apply(GridCommand::ExtendRowChecked(3));
        assert_eq!(s.checked_rows, [3].into_iter().collect());
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
