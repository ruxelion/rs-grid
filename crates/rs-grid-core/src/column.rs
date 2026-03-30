use std::{fmt, rc::Rc};

use crate::format::CellFormat;

// ── cell validator ──────────────────────────────────────

/// Per-column validation callback.
///
/// Called before a cell edit is committed. Returns `Ok(())` to
/// accept the new value or `Err(message)` to reject it.
///
/// Wrap your closure with [`CellValidator::new`]:
/// ```ignore
/// CellValidator::new(|v| {
///     v.parse::<u32>().map(|_| ()).map_err(|_| "not a number".into())
/// })
/// ```
/// Validation callback type alias.
pub type ValidateFn = dyn Fn(&str) -> Result<(), String>;

pub struct CellValidator(pub Rc<ValidateFn>);

impl CellValidator {
    /// Create a new validator from a closure.
    pub fn new(f: impl Fn(&str) -> Result<(), String> + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// Run the validator against `value`.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        (self.0)(value)
    }
}

impl Clone for CellValidator {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl fmt::Debug for CellValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CellValidator(..)")
    }
}

// ── cell editor ────────────────────────────────────────────

/// A single option for the [`CellEditor::Select`] dropdown.
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// Value stored in the cell on commit.
    pub value: String,
    /// Display label shown in the dropdown.
    pub label: String,
    /// Optional icon URL (e.g. data URI) shown left of
    /// the label.
    pub icon: Option<String>,
}

/// Per-column editor override.
///
/// When a cell enters edit mode, the renderer reads this
/// to decide which DOM widget to create.
/// `None` on [`ColumnDef`] = default text `<input>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum CellEditor {
    /// Plain `<input type="text">`.
    Text,
    /// Dropdown with fixed options.
    Select {
        /// Ordered list of choices shown in the dropdown.
        options: Vec<SelectOption>,
    },
}

// ── column definition ───────────────────────────────────

/// Definition of a single grid column.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Unique key used to look up cell values in a row.
    pub key: String,
    /// Display label shown in the column header.
    pub label: String,
    /// Width in logical (CSS) pixels.
    pub width: f64,
    /// Optional display format for cell values.
    pub format: Option<CellFormat>,
    /// Optional editor override for inline editing.
    pub editor: Option<CellEditor>,
    /// Optional validator called before committing an edit.
    /// Returns `Ok(())` to accept or `Err(message)` to reject.
    pub validator: Option<CellValidator>,
}

impl ColumnDef {
    /// Create a column with the given key, label, and width
    /// (no format override).
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        width: f64,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            width,
            format: None,
            editor: None,
            validator: None,
        }
    }
}

/// Precomputed left-edge offsets for every column, plus total content width.
#[derive(Debug, Clone, Default)]
pub struct ColumnOffsets {
    /// `offsets[i]` is the x position of the left edge of column `i`.
    pub offsets: Vec<f64>,
    /// Sum of all column widths (total content width).
    pub total_width: f64,
}

impl ColumnOffsets {
    /// Build offsets from a slice of column definitions.
    pub fn compute(columns: &[ColumnDef]) -> Self {
        let mut offsets = Vec::with_capacity(columns.len());
        let mut x = 0.0_f64;
        for col in columns {
            offsets.push(x);
            x += col.width;
        }
        Self {
            offsets,
            total_width: x,
        }
    }

    /// Return the column index whose bounds contain `x`, or `None`.
    ///
    /// Uses binary search on the sorted offsets for O(log n).
    pub fn hit_column(&self, x: f64, columns: &[ColumnDef]) -> Option<usize> {
        if x < 0.0 || self.offsets.is_empty() {
            return None;
        }
        // partition_point returns the first index where offset > x.
        let idx = self.offsets.partition_point(|&o| o <= x);
        let col = idx.checked_sub(1)?;
        if col < columns.len() && x < self.offsets[col] + columns[col].width {
            Some(col)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 50.0),
        ]
    }

    #[test]
    fn compute_offsets() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.offsets, vec![0.0, 100.0, 250.0]);
        assert_eq!(o.total_width, 300.0);
    }

    #[test]
    fn compute_empty() {
        let o = ColumnOffsets::compute(&[]);
        assert!(o.offsets.is_empty());
        assert_eq!(o.total_width, 0.0);
    }

    #[test]
    fn hit_column_first() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(0.0, &cols), Some(0));
        assert_eq!(o.hit_column(99.9, &cols), Some(0));
    }

    #[test]
    fn hit_column_second() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(100.0, &cols), Some(1));
        assert_eq!(o.hit_column(249.9, &cols), Some(1));
    }

    #[test]
    fn hit_column_last() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(250.0, &cols), Some(2));
        assert_eq!(o.hit_column(299.9, &cols), Some(2));
    }

    #[test]
    fn hit_column_out_of_range() {
        let cols = cols();
        let o = ColumnOffsets::compute(&cols);
        assert_eq!(o.hit_column(300.0, &cols), None);
        assert_eq!(o.hit_column(-1.0, &cols), None);
    }

    #[test]
    fn columndef_format_default_none() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert!(col.format.is_none());
    }
}
