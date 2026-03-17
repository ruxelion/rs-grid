/// Sort direction for a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

/// Active sort: which column and in which direction.
#[derive(Debug, Clone, PartialEq)]
pub struct SortState {
    pub col_key: String,
    pub dir:     SortDir,
}
