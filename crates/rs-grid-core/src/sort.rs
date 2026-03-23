/// Sort direction for a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDir {
    /// Ascending (A → Z, 0 → 9).
    Asc,
    /// Descending (Z → A, 9 → 0).
    Desc,
}

/// Active sort: which column and in which direction.
#[derive(Debug, Clone, PartialEq)]
pub struct SortState {
    /// Column key being sorted.
    pub col_key: String,
    /// Current sort direction.
    pub dir: SortDir,
}
