/// Distinct values for a column, computed by
/// [`crate::model::GridModel::unique_values`] — backs the AG-Grid-style
/// checklist half of the filter popup (`rs-grid-web`'s
/// `show_column_filter_popup`).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum UniqueValues {
    /// Sorted, deduplicated values — the distinct count was within the
    /// requested cap.
    Values(Vec<String>),
    /// The distinct-value count exceeded `cap` before the scan finished.
    /// Callers should show a message instead of rendering a checkbox per
    /// value.
    TooMany {
        /// The cap that was exceeded.
        cap: usize,
    },
}

/// Comparison operator for a per-column filter condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum FilterOp {
    /// Case-insensitive substring match.
    #[default]
    Contains,
    /// Negation of [`FilterOp::Contains`].
    NotContains,
    /// Case-insensitive prefix match.
    StartsWith,
    /// Case-insensitive suffix match.
    EndsWith,
    /// Numeric equality when the column format is numeric-like
    /// (`CellFormat::is_numeric_like`), case-insensitive string
    /// equality otherwise.
    Equals,
    /// Negation of [`FilterOp::Equals`].
    NotEquals,
    /// Cell value is empty (after trimming whitespace).
    Blank,
    /// Negation of [`FilterOp::Blank`].
    NotBlank,
    /// Numeric greater-than. Non-numeric cell values never match.
    GreaterThan,
    /// Numeric greater-than-or-equal. Non-numeric cell values
    /// never match.
    GreaterThanOrEqual,
    /// Numeric less-than. Non-numeric cell values never match.
    LessThan,
    /// Numeric less-than-or-equal. Non-numeric cell values never
    /// match.
    LessThanOrEqual,
}

impl FilterOp {
    /// Whether this operator needs a comparison value to be
    /// meaningful. `Blank`/`NotBlank` ignore
    /// `FilterCondition::value` entirely.
    fn needs_value(self) -> bool {
        !matches!(self, FilterOp::Blank | FilterOp::NotBlank)
    }
}

/// An active per-column filter: an operator plus its comparison
/// value. `Default` is an empty `Contains` condition (`is_empty()`
/// is `true`), the same "no filter" starting point every filter-row
/// cell shows before a value is typed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FilterCondition {
    /// The comparison operator.
    pub op: FilterOp,
    /// The comparison value. Ignored by `Blank`/`NotBlank`.
    pub value: String,
}

impl FilterCondition {
    /// Build a `Contains` condition — the common case and the
    /// default operator for the plain-text filter API.
    pub fn contains(value: impl Into<String>) -> Self {
        Self {
            op: FilterOp::Contains,
            value: value.into(),
        }
    }

    /// A condition with no effective filtering to apply: an
    /// operator that needs a value but has none. `Blank`/
    /// `NotBlank` are never "empty" — they carry meaning without
    /// a value.
    pub fn is_empty(&self) -> bool {
        self.op.needs_value() && self.value.is_empty()
    }

    /// Whether `cell` passes this condition. `numeric` selects
    /// numeric comparison for `Equals`/`NotEquals` — pass the
    /// column's `CellFormat::is_numeric_like()`. The purely
    /// numeric operators (`GreaterThan` and friends) always
    /// compare numerically regardless of `numeric`.
    pub fn matches(&self, cell: &str, numeric: bool) -> bool {
        match self.op {
            FilterOp::Contains => cell
                .to_ascii_lowercase()
                .contains(&self.value.to_ascii_lowercase()),
            FilterOp::NotContains => !cell
                .to_ascii_lowercase()
                .contains(&self.value.to_ascii_lowercase()),
            FilterOp::StartsWith => cell
                .to_ascii_lowercase()
                .starts_with(&self.value.to_ascii_lowercase()),
            FilterOp::EndsWith => cell
                .to_ascii_lowercase()
                .ends_with(&self.value.to_ascii_lowercase()),
            FilterOp::Equals => {
                if numeric {
                    numeric_cmp(cell, &self.value)
                        .is_some_and(|o| o == std::cmp::Ordering::Equal)
                } else {
                    cell.eq_ignore_ascii_case(&self.value)
                }
            }
            FilterOp::NotEquals => {
                if numeric {
                    numeric_cmp(cell, &self.value)
                        .is_some_and(|o| o != std::cmp::Ordering::Equal)
                } else {
                    !cell.eq_ignore_ascii_case(&self.value)
                }
            }
            FilterOp::Blank => cell.trim().is_empty(),
            FilterOp::NotBlank => !cell.trim().is_empty(),
            FilterOp::GreaterThan => numeric_cmp(cell, &self.value)
                .is_some_and(|o| o == std::cmp::Ordering::Greater),
            FilterOp::GreaterThanOrEqual => numeric_cmp(cell, &self.value)
                .is_some_and(|o| o != std::cmp::Ordering::Less),
            FilterOp::LessThan => numeric_cmp(cell, &self.value)
                .is_some_and(|o| o == std::cmp::Ordering::Less),
            FilterOp::LessThanOrEqual => numeric_cmp(cell, &self.value)
                .is_some_and(|o| o != std::cmp::Ordering::Greater),
        }
    }
}

/// Parse both sides as `f64` and compare. `None` if either side
/// fails to parse — a non-numeric cell never matches a numeric
/// operator, it never panics.
fn numeric_cmp(cell: &str, value: &str) -> Option<std::cmp::Ordering> {
    let a: f64 = cell.trim().parse().ok()?;
    let b: f64 = value.trim().parse().ok()?;
    a.partial_cmp(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_ctor_defaults_to_contains_op() {
        let c = FilterCondition::contains("abc");
        assert_eq!(c.op, FilterOp::Contains);
        assert_eq!(c.value, "abc");
    }

    #[test]
    fn is_empty_true_for_empty_value_needing_op() {
        assert!(FilterCondition::contains("").is_empty());
    }

    #[test]
    fn is_empty_false_for_blank_op_with_no_value() {
        let c = FilterCondition {
            op: FilterOp::Blank,
            value: String::new(),
        };
        assert!(!c.is_empty());
    }

    #[test]
    fn contains_is_case_insensitive() {
        let c = FilterCondition::contains("ALI");
        assert!(c.matches("Alice", false));
        assert!(!c.matches("Bob", false));
    }

    #[test]
    fn not_contains_negates() {
        let c = FilterCondition {
            op: FilterOp::NotContains,
            value: "ali".into(),
        };
        assert!(!c.matches("Alice", false));
        assert!(c.matches("Bob", false));
    }

    #[test]
    fn starts_with_and_ends_with() {
        let starts = FilterCondition {
            op: FilterOp::StartsWith,
            value: "al".into(),
        };
        assert!(starts.matches("Alice", false));
        assert!(!starts.matches("Malice", false));

        let ends = FilterCondition {
            op: FilterOp::EndsWith,
            value: "ce".into(),
        };
        assert!(ends.matches("Alice", false));
        assert!(!ends.matches("Bob", false));
    }

    #[test]
    fn equals_string_mode_is_case_insensitive_full_match() {
        let c = FilterCondition {
            op: FilterOp::Equals,
            value: "Alice".into(),
        };
        assert!(c.matches("alice", false));
        assert!(!c.matches("Alicia", false));
    }

    #[test]
    fn equals_numeric_mode_compares_as_numbers() {
        let c = FilterCondition {
            op: FilterOp::Equals,
            value: "10".into(),
        };
        assert!(c.matches("10.0", true));
        assert!(!c.matches("10.5", true));
    }

    #[test]
    fn not_equals_numeric_mode() {
        let c = FilterCondition {
            op: FilterOp::NotEquals,
            value: "10".into(),
        };
        assert!(c.matches("11", true));
        assert!(!c.matches("10", true));
    }

    #[test]
    fn blank_and_not_blank() {
        let blank = FilterCondition {
            op: FilterOp::Blank,
            value: String::new(),
        };
        assert!(blank.matches("", false));
        assert!(blank.matches("   ", false));
        assert!(!blank.matches("x", false));

        let not_blank = FilterCondition {
            op: FilterOp::NotBlank,
            value: String::new(),
        };
        assert!(!not_blank.matches("", false));
        assert!(not_blank.matches("x", false));
    }

    #[test]
    fn greater_than_and_less_than() {
        let gt = FilterCondition {
            op: FilterOp::GreaterThan,
            value: "5".into(),
        };
        assert!(gt.matches("6", false));
        assert!(!gt.matches("5", false));
        assert!(!gt.matches("4", false));

        let lt = FilterCondition {
            op: FilterOp::LessThan,
            value: "5".into(),
        };
        assert!(lt.matches("4", false));
        assert!(!lt.matches("5", false));
    }

    #[test]
    fn greater_than_or_equal_and_less_than_or_equal() {
        let gte = FilterCondition {
            op: FilterOp::GreaterThanOrEqual,
            value: "5".into(),
        };
        assert!(gte.matches("5", false));
        assert!(gte.matches("6", false));
        assert!(!gte.matches("4", false));

        let lte = FilterCondition {
            op: FilterOp::LessThanOrEqual,
            value: "5".into(),
        };
        assert!(lte.matches("5", false));
        assert!(!lte.matches("6", false));
    }

    #[test]
    fn numeric_op_on_non_numeric_cell_never_matches_never_panics() {
        let gt = FilterCondition {
            op: FilterOp::GreaterThan,
            value: "5".into(),
        };
        assert!(!gt.matches("not-a-number", false));

        let eq = FilterCondition {
            op: FilterOp::Equals,
            value: "5".into(),
        };
        assert!(!eq.matches("not-a-number", true));
    }
}
