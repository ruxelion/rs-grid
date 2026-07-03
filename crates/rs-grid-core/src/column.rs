use std::{fmt, rc::Rc};

use crate::{
    format::CellFormat,
    model::GridModel,
    validation::{ValidationRule, validate_rules},
};

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

// ── cell editability predicate ──────────────────────────

/// Per-cell editability predicate signature.
///
/// Receives the row index and full read access to the [`GridModel`]
/// so the closure can implement cross-column logic (e.g. lock a cell
/// when another column's value is `"locked"`), not just this
/// column's own value.
pub type EditablePredicateFn = dyn Fn(u64, &GridModel) -> bool;

/// Dynamic per-cell editability override.
///
/// Checked *after* the static [`ColumnDef::editable`] flag — if the
/// column is statically non-editable, the predicate is never called
/// (mirrors the `rules` → `validator` layering of
/// [`ColumnDef::validate_value`]).
///
/// Wrap your closure with [`EditablePredicate::new`]:
/// ```ignore
/// EditablePredicate::new(|row, model| {
///     model.get_cell(row, "status").as_deref() != Some("locked")
/// })
/// ```
///
/// # Thread safety
///
/// `EditablePredicate` is `!Send + !Sync` (it wraps an `Rc`), matching
/// [`CellValidator`] — the grid targets single-threaded WASM / browser
/// environments.
pub struct EditablePredicate(pub Rc<EditablePredicateFn>);

impl EditablePredicate {
    /// Create a new predicate from a closure.
    pub fn new(f: impl Fn(u64, &GridModel) -> bool + 'static) -> Self {
        Self(Rc::new(f))
    }

    /// Evaluate the predicate for `row` against `model`.
    pub fn is_editable(&self, row: u64, model: &GridModel) -> bool {
        (self.0)(row, model)
    }
}

impl Clone for EditablePredicate {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl fmt::Debug for EditablePredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EditablePredicate(..)")
    }
}

// ── cell decoration ─────────────────────────────────────

/// Per-cell decoration callback signature. Mirrors
/// [`EditablePredicateFn`] exactly — row index + full `GridModel` access
/// for cross-column logic.
pub type CellDecoratorFn = dyn Fn(u64, &GridModel) -> Option<CellDecoration>;

/// Persistent, at-rest visual annotation for a single cell. Purely
/// cosmetic — never affects whether a value can be written (contrast
/// with `rules`/`validator`). RGBA fields follow the `[u8; 4]`
/// convention used by
/// [`FormattedCell::color`](crate::format::FormattedCell::color),
/// not `rs-grid-scene`'s `Color`, so `rs-grid-core` stays
/// renderer-agnostic.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CellDecoration {
    /// RGBA border color. `None` = no border drawn. The stroke width
    /// itself is themed (`Theme::decoration_border_width` in
    /// `rs-grid-scene`) since it's uniform across every decorated cell
    /// regardless of color.
    pub border_color: Option<[u8; 4]>,
    /// RGBA background tint, blended over the cell's normal background.
    /// `None` = no tint drawn.
    pub background_tint: Option<[u8; 4]>,
    /// Reserved for a future native hover tooltip — **not rendered
    /// anywhere yet**. Safe to leave `None`.
    pub message: Option<String>,
}

impl CellDecoration {
    /// Set the border color. Returns `self` for chaining.
    ///
    /// `#[non_exhaustive]` blocks struct-literal construction from
    /// outside this crate, so these builders (mirroring `ColumnDef`'s
    /// own `with_*`/`.*_when` chaining style) are the public way to
    /// build a `CellDecoration`, starting from `CellDecoration::default()`.
    pub fn with_border_color(mut self, color: [u8; 4]) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set the background tint. Returns `self` for chaining.
    pub fn with_background_tint(mut self, color: [u8; 4]) -> Self {
        self.background_tint = Some(color);
        self
    }

    /// Set the reserved tooltip message. Returns `self` for chaining.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Dynamic per-cell decoration override.
///
/// Evaluated every frame for every visible cell — same cost profile as
/// [`EditablePredicate`] and `CellFormat::Custom`. Not gated by any
/// static flag: a column either has a decorator or it doesn't.
///
/// Wrap your closure with [`CellDecorator::new`]:
/// ```ignore
/// CellDecorator::new(|row, model| {
///     let mismatched = model.get_cell(row, "doc1_file").as_deref().unwrap_or("").is_empty()
///         != model.get_cell(row, "doc1_label").as_deref().unwrap_or("").is_empty();
///     mismatched.then(|| {
///         CellDecoration::default().with_border_color([239, 68, 68, 255])
///     })
/// })
/// ```
///
/// # Thread safety
///
/// `CellDecorator` is `!Send + !Sync` (it wraps an `Rc`), matching
/// [`EditablePredicate`] and [`CellValidator`].
pub struct CellDecorator(pub Rc<CellDecoratorFn>);

impl CellDecorator {
    /// Create a new decorator from a closure.
    pub fn new(
        f: impl Fn(u64, &GridModel) -> Option<CellDecoration> + 'static,
    ) -> Self {
        Self(Rc::new(f))
    }

    /// Evaluate the decorator for `row` against `model`.
    pub fn decorate(
        &self,
        row: u64,
        model: &GridModel,
    ) -> Option<CellDecoration> {
        (self.0)(row, model)
    }
}

impl Clone for CellDecorator {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl fmt::Debug for CellDecorator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CellDecorator(..)")
    }
}

// ── cell button ────────────────────────────────────────────

/// Visual style variant for a cell button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ButtonStyle {
    /// Solid primary-colour fill.
    #[default]
    Primary,
    /// Muted secondary-colour fill.
    Secondary,
    /// Destructive red fill.
    Danger,
    /// Transparent background, border only.
    Ghost,
}

/// Definition of a single button rendered inside a cell.
///
/// Buttons are column-level: the same `ButtonDef` applies to
/// every row in the column.  The click callback receives the
/// row index, column key, and this button's `id`.
#[derive(Debug, Clone)]
pub struct ButtonDef {
    /// Stable identifier passed to the click callback.
    /// Must be unique within a column.
    pub id: String,
    /// Label rendered on the button face.
    pub label: String,
    /// Visual style variant.
    pub style: ButtonStyle,
}

impl ButtonDef {
    /// Create a new button definition.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        style: ButtonStyle,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            style,
        }
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
    ///
    /// Kept alongside [`Self::rules`] for backward compatibility;
    /// prefer `rules` for new code.
    pub validator: Option<CellValidator>,
    /// Declarative validation rules checked (in order) before
    /// committing an edit. Evaluated before the legacy `validator`.
    pub rules: Vec<ValidationRule>,
    /// Render cell text with bold weight (`font-weight: 600`).
    pub bold: bool,
    /// Allow inline editing for this column (`true` by default).
    /// When `false`, double-clicking the column does nothing.
    pub editable: bool,
    /// Optional dynamic per-cell editability override, checked after
    /// `editable` (only when `editable` is `true`). `None` = every
    /// cell in the column follows the static `editable` flag
    /// unconditionally. See [`ColumnDef::is_cell_editable`].
    pub editable_predicate: Option<EditablePredicate>,
    /// Optional dynamic per-cell decoration (border/tint/message),
    /// evaluated every frame. Purely cosmetic. `None` = no decoration.
    /// See [`ColumnDef::cell_decoration`].
    pub decorator: Option<CellDecorator>,
    /// Clickable buttons rendered at the right side of every
    /// cell in this column.
    ///
    /// Buttons are drawn right-to-left (first entry is the
    /// rightmost).  The click callback receives the row
    /// index, column key, and the button's `id`.
    pub cell_buttons: Vec<ButtonDef>,
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
            rules: Vec::new(),
            bold: false,
            editable: true,
            editable_predicate: None,
            decorator: None,
            cell_buttons: Vec::new(),
        }
    }

    /// Create a column with the default width
    /// ([`DEFAULT_COL_WIDTH`] = 150 px).
    pub fn simple(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(key, label, DEFAULT_COL_WIDTH)
    }

    /// Render cell text in bold weight. Returns `self` for chaining.
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Disable inline editing for this column. Returns `self` for chaining.
    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }

    /// Set a dynamic per-cell editability predicate. Returns `self`
    /// for chaining. Checked only when `editable` (the static flag)
    /// is `true` — see [`ColumnDef::editable_predicate`].
    pub fn editable_when(
        mut self,
        f: impl Fn(u64, &GridModel) -> bool + 'static,
    ) -> Self {
        self.editable_predicate = Some(EditablePredicate::new(f));
        self
    }

    /// Set a dynamic per-cell decoration callback. Returns `self` for
    /// chaining. Purely visual — see [`ColumnDef::decorator`].
    pub fn decorated_when(
        mut self,
        f: impl Fn(u64, &GridModel) -> Option<CellDecoration> + 'static,
    ) -> Self {
        self.decorator = Some(CellDecorator::new(f));
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

    /// Set the declarative validation rules. Returns `self` for
    /// chaining.
    pub fn with_rules(mut self, rules: Vec<ValidationRule>) -> Self {
        self.rules = rules;
        self
    }

    /// Require a non-empty value. Returns `self` for chaining.
    pub fn required(mut self) -> Self {
        self.rules.push(ValidationRule::required());
        self
    }

    /// Require at least `min` characters. Returns `self` for
    /// chaining.
    pub fn with_min_length(mut self, min: usize) -> Self {
        self.rules.push(ValidationRule::min_length(min));
        self
    }

    /// Require at most `max` characters. Returns `self` for
    /// chaining.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.rules.push(ValidationRule::max_length(max));
        self
    }

    /// Require a numeric value within `min..=max`. Returns `self`
    /// for chaining.
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.rules.push(ValidationRule::range(min, max));
        self
    }

    /// Require the value to match one entry of an allowed-value
    /// list. Returns `self` for chaining.
    pub fn with_allowed_values(mut self, values: Vec<String>) -> Self {
        self.rules.push(ValidationRule::one_of(values));
        self
    }

    /// Validate `value` against [`Self::rules`] (in order), then
    /// against the legacy [`Self::validator`] if all rules passed.
    /// Returns the first failure's message, if any.
    pub fn validate_value(&self, value: &str) -> Result<(), String> {
        validate_rules(&self.rules, value)?;
        if let Some(v) = &self.validator {
            v.validate(value)?;
        }
        Ok(())
    }

    /// Resolve whether `row` is editable in this column — the single
    /// source of truth combining all three editability layers, in
    /// order (each short-circuits the next): `model.editable` (grid-wide
    /// toggle), then the static per-column `editable` flag, then the
    /// dynamic `editable_predicate` (not even called if either static
    /// layer is `false`); `true` if no predicate is set and both static
    /// layers pass.
    pub fn is_cell_editable(&self, row: u64, model: &GridModel) -> bool {
        model.editable
            && self.editable
            && self
                .editable_predicate
                .as_ref()
                .is_none_or(|p| p.is_editable(row, model))
    }

    /// Resolve this cell's decoration, if any. No static gate (unlike
    /// `is_cell_editable`'s 3-layer stack) — purely cosmetic.
    pub fn cell_decoration(
        &self,
        row: u64,
        model: &GridModel,
    ) -> Option<CellDecoration> {
        self.decorator.as_ref().and_then(|d| d.decorate(row, model))
    }

    /// Set the cell buttons. Returns `self` for chaining.
    pub fn with_cell_buttons(mut self, buttons: Vec<ButtonDef>) -> Self {
        self.cell_buttons = buttons;
        self
    }

    /// Clamp `w` to this column's [`min_width`]..=[`max_width`]
    /// range, with [`MIN_COL_WIDTH`] as the absolute floor.
    pub fn clamp_width(&self, w: f64) -> f64 {
        let floor = self.min_width.unwrap_or(MIN_COL_WIDTH).max(MIN_COL_WIDTH);
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
            s.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())
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

    // ── EditablePredicate / editable_when ──────────────────

    fn model_with_status(row0_status: &str) -> GridModel {
        use crate::row::RowRecord;
        let cols = vec![
            ColumnDef::new("status", "Status", 100.0),
            ColumnDef::new("notes", "Notes", 100.0),
        ];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("status", row0_status);
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("status", "open");
                r
            },
        ];
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    #[test]
    fn editable_predicate_default_none() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert!(col.editable_predicate.is_none());
    }

    #[test]
    fn editable_when_sets_field() {
        let col = ColumnDef::new("a", "A", 100.0).editable_when(|_, _| false);
        assert!(col.editable_predicate.is_some());
    }

    #[test]
    fn is_cell_editable_true_when_no_predicate() {
        let col = ColumnDef::new("a", "A", 100.0);
        let model = model_with_status("open");
        assert!(col.is_cell_editable(0, &model));
        assert!(col.is_cell_editable(1, &model));
    }

    #[test]
    fn is_cell_editable_false_when_static_false_predicate_not_called() {
        let col = ColumnDef::new("a", "A", 100.0)
            .read_only()
            .editable_when(|_, _| panic!("predicate must not be called"));
        let model = model_with_status("open");
        assert!(!col.is_cell_editable(0, &model));
    }

    #[test]
    fn is_cell_editable_delegates_to_predicate() {
        let col = ColumnDef::new("a", "A", 100.0)
            .editable_when(|row, _| row % 2 == 0);
        let model = model_with_status("open");
        assert!(col.is_cell_editable(0, &model));
        assert!(!col.is_cell_editable(1, &model));
    }

    #[test]
    fn is_cell_editable_predicate_reads_other_column() {
        let col = ColumnDef::new("notes", "Notes", 100.0).editable_when(
            |row, model| {
                model.get_cell(row, "status").as_deref() != Some("locked")
            },
        );
        let model = model_with_status("locked");
        assert!(!col.is_cell_editable(0, &model));
        assert!(col.is_cell_editable(1, &model));
    }

    #[test]
    fn is_cell_editable_false_when_grid_wide_editable_false_predicate_not_called()
     {
        let col = ColumnDef::new("a", "A", 100.0)
            .editable_when(|_, _| panic!("predicate must not be called"));
        let mut model = model_with_status("open");
        model.editable = false;
        assert!(!col.is_cell_editable(0, &model));
    }

    #[test]
    fn editable_predicate_debug_format() {
        let p = EditablePredicate::new(|_, _| true);
        let s = format!("{p:?}");
        assert!(s.contains("EditablePredicate"));
    }

    #[test]
    fn editable_predicate_clone_shares_closure() {
        let p = EditablePredicate::new(|row, _| row == 0);
        let p2 = p.clone();
        let model = model_with_status("open");
        assert!(p2.is_editable(0, &model));
        assert!(!p2.is_editable(1, &model));
    }

    // ── decorator ────────────────────────────────────────

    #[test]
    fn decorator_default_none() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert!(col.decorator.is_none());
    }

    #[test]
    fn decorated_when_sets_field() {
        let col = ColumnDef::new("a", "A", 100.0).decorated_when(|_, _| None);
        assert!(col.decorator.is_some());
    }

    #[test]
    fn cell_decoration_none_when_no_decorator() {
        let col = ColumnDef::new("a", "A", 100.0);
        let model = model_with_status("open");
        assert_eq!(col.cell_decoration(0, &model), None);
    }

    #[test]
    fn cell_decoration_delegates_to_closure() {
        let col = ColumnDef::new("a", "A", 100.0).decorated_when(|row, _| {
            (row == 0).then(|| CellDecoration {
                border_color: Some([239, 68, 68, 255]),
                ..Default::default()
            })
        });
        let model = model_with_status("open");
        assert!(col.cell_decoration(0, &model).is_some());
        assert!(col.cell_decoration(1, &model).is_none());
    }

    #[test]
    fn cell_decoration_predicate_reads_other_column() {
        let col = ColumnDef::new("notes", "Notes", 100.0).decorated_when(
            |row, model| {
                let mismatched =
                    model.get_cell(row, "status").as_deref() == Some("locked");
                mismatched.then(CellDecoration::default)
            },
        );
        let model = model_with_status("locked");
        assert!(col.cell_decoration(0, &model).is_some());
        assert!(col.cell_decoration(1, &model).is_none());
    }

    #[test]
    fn decorator_debug_format() {
        let d = CellDecorator::new(|_, _| None);
        let s = format!("{d:?}");
        assert!(s.contains("CellDecorator"));
    }

    #[test]
    fn decorator_clone_shares_closure() {
        let d = CellDecorator::new(|row, _| {
            (row == 0).then(CellDecoration::default)
        });
        let d2 = d.clone();
        let model = model_with_status("open");
        assert!(d2.decorate(0, &model).is_some());
        assert!(d2.decorate(1, &model).is_none());
    }

    #[test]
    fn cell_decoration_default_all_none() {
        let deco = CellDecoration::default();
        assert!(deco.border_color.is_none());
        assert!(deco.background_tint.is_none());
        assert!(deco.message.is_none());
    }

    // ── with_editor ───────────────────────────────────────

    #[test]
    fn with_editor_text() {
        let col = ColumnDef::new("a", "A", 100.0).with_editor(CellEditor::Text);
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
        assert!(matches!(col.editor, Some(CellEditor::Select { .. })));
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

    // ── validation rules ──────────────────────────────────

    #[test]
    fn required_pushes_rule() {
        let col = ColumnDef::new("a", "A", 100.0).required();
        assert_eq!(col.rules.len(), 1);
        assert!(col.validate_value("").is_err());
        assert!(col.validate_value("x").is_ok());
    }

    #[test]
    fn with_min_length_pushes_rule() {
        let col = ColumnDef::new("a", "A", 100.0).with_min_length(3);
        assert!(col.validate_value("ab").is_err());
        assert!(col.validate_value("abc").is_ok());
    }

    #[test]
    fn with_max_length_pushes_rule() {
        let col = ColumnDef::new("a", "A", 100.0).with_max_length(3);
        assert!(col.validate_value("abcd").is_err());
        assert!(col.validate_value("abc").is_ok());
    }

    #[test]
    fn with_range_pushes_rule() {
        let col = ColumnDef::new("a", "A", 100.0).with_range(0.0, 10.0);
        assert!(col.validate_value("5").is_ok());
        assert!(col.validate_value("50").is_err());
    }

    #[test]
    fn with_allowed_values_pushes_rule() {
        let col = ColumnDef::new("a", "A", 100.0)
            .with_allowed_values(vec!["A".into(), "B".into()]);
        assert!(col.validate_value("A").is_ok());
        assert!(col.validate_value("C").is_err());
    }

    #[test]
    fn with_rules_sets_field() {
        let col = ColumnDef::new("a", "A", 100.0)
            .with_rules(vec![ValidationRule::required()]);
        assert_eq!(col.rules.len(), 1);
    }

    #[test]
    fn rules_default_empty() {
        let col = ColumnDef::new("a", "A", 100.0);
        assert!(col.rules.is_empty());
        assert!(col.validate_value("anything").is_ok());
    }

    #[test]
    fn validate_value_runs_rules_before_legacy_validator() {
        let col = ColumnDef::new("a", "A", 100.0)
            .required()
            .with_validator(CellValidator::new(|_| Err("legacy".into())));
        // Required fails first — legacy validator is never reached.
        assert_eq!(
            col.validate_value("").unwrap_err(),
            "This field is required."
        );
        // Once rules pass, the legacy validator still runs.
        assert_eq!(col.validate_value("x").unwrap_err(), "legacy");
    }

    #[test]
    fn with_bold_sets_flag() {
        let col = ColumnDef::new("a", "A", 100.0).with_bold();
        assert!(col.bold);
    }

    #[test]
    fn with_cell_buttons_sets_field() {
        let col = ColumnDef::new("a", "A", 100.0).with_cell_buttons(vec![
            ButtonDef::new("del", "Delete", ButtonStyle::Danger),
        ]);
        assert_eq!(col.cell_buttons.len(), 1);
        assert_eq!(col.cell_buttons[0].id, "del");
    }
}
