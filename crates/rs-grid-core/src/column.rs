use std::{fmt, rc::Rc};

use crate::format::CellFormat;

// ── cell validator ──────────────────────────────────────

/// Validation callback type alias.
///
/// Wrapped in [`Rc`] for the same reason as [`CellFormat::Custom`]:
/// the grid is single-threaded. [`Clone`] on [`CellValidator`] is a
/// cheap `Rc::clone`.
pub type ValidateFn = dyn Fn(&str) -> Result<(), String>;

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
///
/// # Thread safety
///
/// `CellValidator` is `!Send + !Sync` (it wraps an `Rc`). This is
/// intentional — the grid targets single-threaded WASM / browser
/// environments where atomic reference counting would be unnecessary
/// overhead.
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

/// Default column width used by [`ColumnDef::simple`].
pub const DEFAULT_COL_WIDTH: f64 = 150.0;

/// Absolute minimum column width in logical pixels.
///
/// Applied as a floor even when [`ColumnDef::min_width`] is `None`.
pub const MIN_COL_WIDTH: f64 = 20.0;

/// Definition of a single grid column.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Unique key used to look up cell values in a row.
    pub key: String,
    /// Display label shown in the column header.
    pub label: String,
    /// Width in logical (CSS) pixels.
    pub width: f64,
    /// Optional minimum width in logical pixels.
    /// Enforced during resize and auto-fit.
    /// Falls back to [`MIN_COL_WIDTH`] when `None`.
    pub min_width: Option<f64>,
    /// Optional maximum width in logical pixels.
    /// Enforced during resize and auto-fit.
    pub max_width: Option<f64>,
    /// Optional flex factor for proportional sizing.
    ///
    /// When set, the column shares remaining viewport space
    /// proportionally with other flex columns. The `width`
    /// field is overwritten by the flex computation on each
    /// viewport resize. Cleared when the user manually
    /// resizes or auto-fits the column.
    ///
    /// `None` = fixed-width column (default).
    pub flex: Option<f64>,
    /// Optional display format for cell values.
    pub format: Option<CellFormat>,
    /// Optional editor override for inline editing.
    pub editor: Option<CellEditor>,
    /// Optional validator called before committing an edit.
    /// Returns `Ok(())` to accept or `Err(message)` to reject.
    pub validator: Option<CellValidator>,
    /// Render cell text with bold weight (`font-weight: 600`).
    pub bold: bool,
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
            min_width: None,
            max_width: None,
            flex: None,
            format: None,
            editor: None,
            validator: None,
            bold: false,
        }
    }

    /// Create a column with the default width
    /// ([`DEFAULT_COL_WIDTH`] = 150 px).
    pub fn simple(
        key: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self::new(key, label, DEFAULT_COL_WIDTH)
    }

    /// Render cell text in bold weight. Returns `self` for chaining.
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set the flex factor. Returns `self` for chaining.
    pub fn with_flex(mut self, flex: f64) -> Self {
        self.flex = Some(flex);
        self
    }

    /// Set the minimum width in logical pixels. Returns `self`
    /// for chaining.
    pub fn with_min_width(mut self, min: f64) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Set the maximum width in logical pixels. Returns `self`
    /// for chaining.
    pub fn with_max_width(mut self, max: f64) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Set the display format. Returns `self` for chaining.
    pub fn with_format(mut self, format: CellFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the editor override. Returns `self` for chaining.
    pub fn with_editor(mut self, editor: CellEditor) -> Self {
        self.editor = Some(editor);
        self
    }

    /// Set the validator. Returns `self` for chaining.
    pub fn with_validator(mut self, validator: CellValidator) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Clamp `w` to this column's [`min_width`]..=[`max_width`]
    /// range, with [`MIN_COL_WIDTH`] as the absolute floor.
    pub fn clamp_width(&self, w: f64) -> f64 {
        let floor = self
            .min_width
            .unwrap_or(MIN_COL_WIDTH)
            .max(MIN_COL_WIDTH);
        let w = w.max(floor);
        match self.max_width {
            Some(max) => w.min(max.max(floor)),
            None => w,
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

    // ── simple constructor ────────────────────────────

    #[test]
    fn simple_uses_default_width() {
        let col = ColumnDef::simple("a", "A");
        assert_eq!(col.width, DEFAULT_COL_WIDTH);
        assert!(col.min_width.is_none());
        assert!(col.max_width.is_none());
    }

    // ── clamp_width ───────────────────────────────────

    #[test]
    fn clamp_width_no_constraints() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert_eq!(col.clamp_width(200.0), 200.0);
        // Absolute floor at MIN_COL_WIDTH
        assert_eq!(col.clamp_width(5.0), MIN_COL_WIDTH);
    }

    #[test]
    fn clamp_width_with_min() {
        let mut col = ColumnDef::new("a", "A", 100.0);
        col.min_width = Some(50.0);
        assert_eq!(col.clamp_width(30.0), 50.0);
        assert_eq!(col.clamp_width(80.0), 80.0);
    }

    #[test]
    fn clamp_width_with_max() {
        let mut col = ColumnDef::new("a", "A", 100.0);
        col.max_width = Some(200.0);
        assert_eq!(col.clamp_width(300.0), 200.0);
        assert_eq!(col.clamp_width(150.0), 150.0);
    }

    #[test]
    fn clamp_width_min_and_max() {
        let mut col = ColumnDef::new("a", "A", 100.0);
        col.min_width = Some(60.0);
        col.max_width = Some(200.0);
        assert_eq!(col.clamp_width(30.0), 60.0);
        assert_eq!(col.clamp_width(150.0), 150.0);
        assert_eq!(col.clamp_width(300.0), 200.0);
    }

    #[test]
    fn clamp_width_min_below_absolute_floor() {
        // min_width < MIN_COL_WIDTH → absolute floor wins
        let mut col = ColumnDef::new("a", "A", 100.0);
        col.min_width = Some(5.0);
        assert_eq!(col.clamp_width(10.0), MIN_COL_WIDTH);
    }

    #[test]
    fn clamp_width_max_below_min() {
        // max_width < min_width → min wins (max is raised)
        let mut col = ColumnDef::new("a", "A", 100.0);
        col.min_width = Some(100.0);
        col.max_width = Some(50.0);
        assert_eq!(col.clamp_width(30.0), 100.0);
        assert_eq!(col.clamp_width(200.0), 100.0);
    }

    // ── flex ──────────────────────────────────────────

    #[test]
    fn flex_default_none() {
        assert!(ColumnDef::new("a", "A", 100.0).flex.is_none());
        assert!(ColumnDef::simple("a", "A").flex.is_none());
    }

    #[test]
    fn with_flex_builder() {
        let col = ColumnDef::simple("a", "A").with_flex(2.0);
        assert_eq!(col.flex, Some(2.0));
        assert_eq!(col.width, DEFAULT_COL_WIDTH);
    }

    #[test]
    fn builder_chain() {
        let col = ColumnDef::new("a", "A", 100.0)
            .with_min_width(50.0)
            .with_max_width(300.0)
            .with_flex(1.0);
        assert_eq!(col.min_width, Some(50.0));
        assert_eq!(col.max_width, Some(300.0));
        assert_eq!(col.flex, Some(1.0));
    }

    // ── CellValidator ─────────────────────────────────────

    #[test]
    fn cell_validator_accepts_valid_input() {
        let v = CellValidator::new(|s| {
            s.parse::<u32>()
                .map(|_| ())
                .map_err(|_| "not a number".into())
        });
        assert!(v.validate("42").is_ok());
    }

    #[test]
    fn cell_validator_rejects_invalid_input() {
        let v = CellValidator::new(|s| {
            s.parse::<u32>()
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert!(v.validate("abc").is_err());
    }

    #[test]
    fn cell_validator_clone_shares_closure() {
        let v = CellValidator::new(|s| {
            if s.is_empty() {
                Err("empty".into())
            } else {
                Ok(())
            }
        });
        let v2 = v.clone();
        assert!(v2.validate("x").is_ok());
        assert!(v2.validate("").is_err());
    }

    #[test]
    fn cell_validator_debug_format() {
        let v = CellValidator::new(|_| Ok(()));
        let s = format!("{v:?}");
        assert!(s.contains("CellValidator"));
    }

    // ── with_editor ───────────────────────────────────────

    #[test]
    fn with_editor_text() {
        let col =
            ColumnDef::new("a", "A", 100.0).with_editor(CellEditor::Text);
        assert!(matches!(col.editor, Some(CellEditor::Text)));
    }

    #[test]
    fn with_editor_select() {
        let opts = vec![
            SelectOption {
                value: "y".into(),
                label: "Yes".into(),
                icon: None,
            },
            SelectOption {
                value: "n".into(),
                label: "No".into(),
                icon: Some("icon.png".into()),
            },
        ];
        let col = ColumnDef::new("a", "A", 100.0)
            .with_editor(CellEditor::Select { options: opts });
        assert!(matches!(
            col.editor,
            Some(CellEditor::Select { .. })
        ));
    }

    // ── with_validator ────────────────────────────────────

    #[test]
    fn with_validator_sets_field() {
        let col = ColumnDef::new("a", "A", 100.0)
            .with_validator(CellValidator::new(|_| Ok(())));
        assert!(col.validator.is_some());
        let v = col.validator.unwrap();
        assert!(v.validate("anything").is_ok());
    }
}
