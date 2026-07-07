//! Tri-state for the row-selection checkbox column's header checkbox.

/// State of the header checkbox in a row-selection checkbox column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckboxTriState {
    /// Every row in scope (filtered rows, or all rows when unfiltered)
    /// is checked.
    Checked,
    /// No row in scope is checked.
    Unchecked,
    /// Some, but not all, rows in scope are checked.
    Indeterminate,
}
