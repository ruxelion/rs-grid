use std::borrow::Cow;

use crate::row::RowRecord;

/// Distinguishes "data not yet fetched" from "data fetched but absent".
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CellStatus {
    /// Cell value is available.
    Ready(String),
    /// The page containing this row has not been fetched yet.
    Loading,
    /// The page is fetched but this cell has no value.
    Absent,
}

/// Trait abstraction over row data backends (in-memory,
/// lazy, or server-side).
pub trait DataSource: std::fmt::Debug {
    /// Total number of rows in the data set.
    fn row_count(&self) -> u64;
    /// Read a single cell value by physical row and column key.
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String>;
    /// Borrow a cell value without cloning, when the backing
    /// store allows it.
    ///
    /// Returns `Cow::Borrowed` for in-memory sources
    /// (`VecDataSource`) — zero allocation per call.
    /// The default delegates to [`get_cell`](Self::get_cell)
    /// and wraps the result in `Cow::Owned`.
    fn get_cell_ref(&self, row: u64, col_key: &str) -> Option<Cow<'_, str>> {
        self.get_cell(row, col_key).map(Cow::Owned)
    }
    /// Attempt to clone into a boxed trait object.
    ///
    /// Returns `Some` for in-memory and page-cache sources.
    /// Returns `None` for closure-based sources (`FnDataSource`)
    /// that cannot be cloned — wrap the `GridModel` in `Rc` if
    /// sharing is needed.
    fn clone_box(&self) -> Option<Box<dyn DataSource>>;
    /// Write a cell value. Default is a no-op for read-only sources.
    fn set_cell(&mut self, _row: u64, _col_key: &str, _value: String) {}
    /// Return the loading status of a cell. The default maps
    /// `None` to `Absent` (legacy behaviour for in-memory
    /// sources).
    fn cell_status(&self, row: u64, col_key: &str) -> CellStatus {
        match self.get_cell(row, col_key) {
            Some(v) => CellStatus::Ready(v),
            None => CellStatus::Absent,
        }
    }
}

// ── VecDataSource ─────────────────────────────────────────────────────────────

/// In-memory data source backed by a `Vec<RowRecord>`.
#[derive(Debug, Clone)]
pub struct VecDataSource {
    /// Row records stored in insertion order.
    pub rows: Vec<RowRecord>,
}

impl VecDataSource {
    /// Create a data source from the given rows.
    pub fn new(rows: Vec<RowRecord>) -> Self {
        Self { rows }
    }
}

impl DataSource for VecDataSource {
    fn row_count(&self) -> u64 {
        self.rows.len() as u64
    }
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String> {
        let row = usize::try_from(row).ok()?;
        self.rows.get(row)?.get(col_key).map(str::to_owned)
    }
    fn get_cell_ref(&self, row: u64, col_key: &str) -> Option<Cow<'_, str>> {
        let row = usize::try_from(row).ok()?;
        self.rows.get(row)?.get(col_key).map(Cow::Borrowed)
    }
    fn clone_box(&self) -> Option<Box<dyn DataSource>> {
        Some(Box::new(self.clone()))
    }
    fn set_cell(&mut self, row: u64, col_key: &str, value: String) {
        if let Ok(idx) = usize::try_from(row) {
            if let Some(record) = self.rows.get_mut(idx) {
                record.set(col_key, value);
            }
        }
    }
}

// ── FnDataSource ──────────────────────────────────────────────────────────────

/// Closure-based virtual data source for computed/lazy data.
pub struct FnDataSource<F: Fn(u64, &str) -> Option<String>> {
    count: u64,
    f: F,
}

impl<F: Fn(u64, &str) -> Option<String>> FnDataSource<F> {
    /// Create a virtual data source with `count` rows and
    /// a closure that generates cell values on demand.
    pub fn new(count: u64, f: F) -> Self {
        Self { count, f }
    }
}

impl<F: Fn(u64, &str) -> Option<String>> DataSource for FnDataSource<F> {
    fn row_count(&self) -> u64 {
        self.count
    }
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String> {
        (self.f)(row, col_key)
    }
    /// Returns `None` — closure-based sources cannot be cloned.
    fn clone_box(&self) -> Option<Box<dyn DataSource>> {
        None
    }
}

// Manual Debug impl because closures do not implement Debug.
impl<F: Fn(u64, &str) -> Option<String>> std::fmt::Debug for FnDataSource<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnDataSource")
            .field("count", &self.count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::RowRecord;

    fn make_rows() -> Vec<RowRecord> {
        let mut r0 = RowRecord::new(0);
        r0.set("a", "hello");
        r0.set("b", "world");
        let mut r1 = RowRecord::new(1);
        r1.set("a", "foo");
        r1.set("b", "bar");
        vec![r0, r1]
    }

    // ── VecDataSource ────────────────────────────────

    #[test]
    fn vec_row_count() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(ds.row_count(), 2);
    }

    #[test]
    fn vec_get_cell() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(ds.get_cell(0, "a"), Some("hello".into()));
        assert_eq!(ds.get_cell(1, "b"), Some("bar".into()));
    }

    #[test]
    fn vec_get_cell_missing_key() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(ds.get_cell(0, "z"), None);
    }

    #[test]
    fn vec_get_cell_out_of_range() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(ds.get_cell(99, "a"), None);
    }

    #[test]
    fn vec_get_cell_ref_borrows() {
        let ds = VecDataSource::new(make_rows());
        let val = ds.get_cell_ref(0, "a");
        assert_eq!(val.as_deref(), Some("hello"));
        // Should be Borrowed, not Owned
        assert!(matches!(val, Some(std::borrow::Cow::Borrowed(_))));
    }

    #[test]
    fn vec_set_cell() {
        let mut ds = VecDataSource::new(make_rows());
        ds.set_cell(0, "a", "updated".into());
        assert_eq!(ds.get_cell(0, "a"), Some("updated".into()));
        // Other row unchanged
        assert_eq!(ds.get_cell(1, "a"), Some("foo".into()));
    }

    #[test]
    fn vec_set_cell_out_of_range_noop() {
        let mut ds = VecDataSource::new(make_rows());
        ds.set_cell(99, "a", "nope".into());
        assert_eq!(ds.row_count(), 2);
    }

    #[test]
    fn vec_clone_box() {
        let ds = VecDataSource::new(make_rows());
        let cloned = ds.clone_box();
        assert!(cloned.is_some());
        let cloned = cloned.unwrap();
        assert_eq!(cloned.row_count(), 2);
        assert_eq!(cloned.get_cell(0, "a"), Some("hello".into()));
    }

    #[test]
    fn vec_cell_status_ready() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(
            ds.cell_status(0, "a"),
            CellStatus::Ready("hello".into())
        );
    }

    #[test]
    fn vec_cell_status_absent() {
        let ds = VecDataSource::new(make_rows());
        assert_eq!(ds.cell_status(0, "missing"), CellStatus::Absent);
    }

    #[test]
    fn vec_empty_data_source() {
        let ds = VecDataSource::new(vec![]);
        assert_eq!(ds.row_count(), 0);
        assert_eq!(ds.get_cell(0, "a"), None);
    }

    // ── FnDataSource ─────────────────────────────────

    #[test]
    fn fn_row_count() {
        let ds = FnDataSource::new(1000, |_r, _c| None);
        assert_eq!(ds.row_count(), 1000);
    }

    #[test]
    fn fn_get_cell() {
        let ds = FnDataSource::new(10, |row, col| {
            Some(format!("{col}_{row}"))
        });
        assert_eq!(
            ds.get_cell(3, "name"),
            Some("name_3".into())
        );
    }

    #[test]
    fn fn_get_cell_returns_none() {
        let ds = FnDataSource::new(10, |_r, _c| None);
        assert_eq!(ds.get_cell(0, "a"), None);
    }

    #[test]
    fn fn_clone_box_returns_none() {
        let ds = FnDataSource::new(10, |_r, _c| None);
        assert!(ds.clone_box().is_none());
    }

    #[test]
    fn fn_cell_status_ready() {
        let ds =
            FnDataSource::new(10, |_r, _c| Some("val".into()));
        assert_eq!(
            ds.cell_status(0, "x"),
            CellStatus::Ready("val".into())
        );
    }

    #[test]
    fn fn_cell_status_absent() {
        let ds = FnDataSource::new(10, |_r, _c| None);
        assert_eq!(ds.cell_status(0, "x"), CellStatus::Absent);
    }

    #[test]
    fn fn_debug_format() {
        let ds = FnDataSource::new(42, |_r, _c| None);
        let dbg = format!("{ds:?}");
        assert!(dbg.contains("FnDataSource"));
        assert!(dbg.contains("42"));
    }

    // ── CellStatus ───────────────────────────────────

    #[test]
    fn cell_status_eq() {
        assert_eq!(
            CellStatus::Ready("x".into()),
            CellStatus::Ready("x".into())
        );
        assert_eq!(CellStatus::Loading, CellStatus::Loading);
        assert_eq!(CellStatus::Absent, CellStatus::Absent);
        assert_ne!(CellStatus::Loading, CellStatus::Absent);
    }
}
