/// Sort direction for a column.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_dir_eq() {
        assert_eq!(SortDir::Asc, SortDir::Asc);
        assert_eq!(SortDir::Desc, SortDir::Desc);
        assert_ne!(SortDir::Asc, SortDir::Desc);
    }

    #[test]
    fn sort_dir_clone() {
        let d = SortDir::Asc;
        let d2 = d.clone();
        assert_eq!(d, d2);
    }

    #[test]
    fn sort_state_construction() {
        let s = SortState {
            col_key: "price".to_string(),
            dir: SortDir::Desc,
        };
        assert_eq!(s.col_key, "price");
        assert_eq!(s.dir, SortDir::Desc);
    }

    #[test]
    fn sort_state_eq() {
        let a = SortState {
            col_key: "x".into(),
            dir: SortDir::Asc,
        };
        let b = SortState {
            col_key: "x".into(),
            dir: SortDir::Asc,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn sort_state_ne_different_dir() {
        let a = SortState {
            col_key: "x".into(),
            dir: SortDir::Asc,
        };
        let b = SortState {
            col_key: "x".into(),
            dir: SortDir::Desc,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn sort_state_ne_different_col() {
        let a = SortState {
            col_key: "x".into(),
            dir: SortDir::Asc,
        };
        let b = SortState {
            col_key: "y".into(),
            dir: SortDir::Asc,
        };
        assert_ne!(a, b);
    }
}
