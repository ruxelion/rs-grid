use std::collections::HashMap;

use crate::{
    column::{ColumnDef, ColumnOffsets},
    datasource::{CellStatus, DataSource, VecDataSource},
    row::RowRecord,
    sort::SortDir,
};

/// Whether sort/filter are performed client-side or
/// delegated to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataSourceMode {
    /// All data is in memory. Sort/filter/search done locally.
    #[default]
    ClientSide,
    /// Data comes from a remote server. Sort/filter are
    /// delegated — client-side `apply_sort`/`apply_filter`
    /// become no-ops.
    ServerSide,
}

/// The data model: columns, a virtual data source, and sizing constants.
#[derive(Debug, Clone)]
pub struct GridModel {
    pub columns: Vec<ColumnDef>,
    pub data: Box<dyn DataSource>,
    /// Height of every data row in logical pixels.
    pub row_height: f64,
    /// Height of the sticky header row in logical pixels.
    pub header_height: f64,
    /// Precomputed column offsets (recomputed when columns change).
    pub column_offsets: ColumnOffsets,
    /// Edited cell values that override the underlying datasource (works for
    /// any source, including read-only `FnDataSource`).
    pub patches: HashMap<(u64, String), String>,
    /// Width of the sticky row-number gutter on the left in logical pixels (0 = hidden).
    pub row_number_width: f64,
    /// Logical→physical row index mapping built by `apply_sort`.
    /// Empty = natural (unsorted) order.
    pub sort_order: Vec<u64>,
    /// Number of leading columns that remain fixed during horizontal scroll.
    /// 0 = no pinned columns (default).
    pub pinned_count: usize,
    /// Per-column text filters (col_key → search text, case-insensitive
    /// contains match). Empty map = no filter active.
    pub filters: HashMap<String, String>,
    /// Physical row indices that pass all active filters, stored in
    /// sort order.  Empty = no filter active (all rows visible).
    pub filtered_indices: Vec<u64>,
    /// Whether data operations run client-side or are delegated
    /// to a server.
    pub mode: DataSourceMode,
    /// Height of the horizontal scrollbar in logical pixels.
    /// Used to reserve space at the bottom so the last row
    /// is not obscured.
    pub scrollbar_size: f64,
}

impl GridModel {
    /// Create a model backed by an in-memory Vec (backwards-compatible API).
    pub fn new(
        columns: Vec<ColumnDef>,
        rows: Vec<RowRecord>,
        row_height: f64,
        header_height: f64,
    ) -> Self {
        Self::with_data_source(
            columns,
            Box::new(VecDataSource::new(rows)),
            row_height,
            header_height,
        )
    }

    /// Create a model backed by any `DataSource` (virtual / lazy sources).
    pub fn with_data_source(
        columns: Vec<ColumnDef>,
        data: Box<dyn DataSource>,
        row_height: f64,
        header_height: f64,
    ) -> Self {
        let column_offsets = ColumnOffsets::compute(&columns);
        let row_number_width =
            Self::compute_row_number_width(data.row_count());
        Self {
            columns,
            data,
            row_height,
            header_height,
            column_offsets,
            patches: HashMap::new(),
            row_number_width,
            sort_order: Vec::new(),
            pinned_count: 0,
            filters: HashMap::new(),
            filtered_indices: Vec::new(),
            mode: DataSourceMode::ClientSide,
            scrollbar_size: 14.0,
        }
    }

    /// Compute gutter width based on the number of digits
    /// in the largest row number.
    /// Uses ~9px per digit + 24px padding (12px each side).
    pub fn compute_row_number_width(row_count: u64) -> f64 {
        let digits = if row_count == 0 {
            1
        } else {
            (row_count as f64).log10().floor() as u32 + 1
        };
        let char_width = 9.0;
        let padding = 24.0;
        (digits as f64 * char_width + padding).max(40.0)
    }

    /// Translate a display row index to its physical (datasource) index.
    ///
    /// When a filter is active `filtered_indices` already holds
    /// physical rows in sort order, so we index directly.
    /// Otherwise we fall back to `sort_order`.
    pub fn logical_to_physical(&self, logical: u64) -> u64 {
        if !self.filtered_indices.is_empty() {
            return self
                .filtered_indices
                .get(logical as usize)
                .copied()
                .unwrap_or(logical);
        }
        if self.sort_order.is_empty() {
            logical
        } else {
            self.sort_order
                .get(logical as usize)
                .copied()
                .unwrap_or(logical)
        }
    }

    /// Number of rows currently visible (respects active filters).
    pub fn display_row_count(&self) -> u64 {
        if self.filtered_indices.is_empty() {
            self.data.row_count()
        } else {
            self.filtered_indices.len() as u64
        }
    }

    /// Rebuild `filtered_indices` from active `filters`.
    ///
    /// Iterates rows in sort order and keeps those that match
    /// every filter (case-insensitive contains).  No-op for
    /// datasets larger than 1 000 000 rows.
    pub fn apply_filter(&mut self) {
        if self.mode == DataSourceMode::ServerSide {
            return;
        }
        if self.filters.is_empty() {
            self.filtered_indices.clear();
            return;
        }
        const MAX: u64 = 1_000_000;
        let n = self.data.row_count();
        if n > MAX {
            self.filtered_indices.clear();
            return;
        }
        let count = n as usize;
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let physical = if self.sort_order.is_empty() {
                i as u64
            } else {
                self.sort_order[i]
            };
            let passes = self.filters.iter().all(|(col_key, text)| {
                let cell =
                    self.data.get_cell(physical, col_key).unwrap_or_default();
                cell.to_ascii_lowercase()
                    .contains(&text.to_ascii_lowercase())
            });
            if passes {
                result.push(physical);
            }
        }
        self.filtered_indices = result;
    }

    /// Read a cell value, checking local patches before the datasource.
    /// Applies the sort mapping so callers always use logical row indices.
    pub fn get_cell(&self, logical_row: u64, col_key: &str) -> Option<String> {
        let physical = self.logical_to_physical(logical_row);
        if let Some(v) = self.patches.get(&(physical, col_key.to_owned())) {
            return Some(v.clone());
        }
        self.data.get_cell(physical, col_key)
    }

    /// Return the loading status of a cell, checking patches first.
    pub fn cell_status(&self, logical_row: u64, col_key: &str) -> CellStatus {
        let physical = self.logical_to_physical(logical_row);
        if let Some(v) = self.patches.get(&(physical, col_key.to_owned())) {
            return CellStatus::Ready(v.clone());
        }
        self.data.cell_status(physical, col_key)
    }

    /// Write a cell value into the patch layer (works for any datasource).
    /// Applies the sort mapping so callers always use logical row indices.
    pub fn set_cell(
        &mut self,
        logical_row: u64,
        col_key: impl Into<String>,
        value: String,
    ) {
        let physical = self.logical_to_physical(logical_row);
        self.patches.insert((physical, col_key.into()), value);
    }

    /// Build `sort_order` by sorting row indices by cell values for `col_key`.
    /// Tries numeric comparison first, falls back to lexicographic.
    /// No-op for datasources with more than 1 000 000 rows.
    pub fn apply_sort(&mut self, col_key: &str, dir: &SortDir) {
        if self.mode == DataSourceMode::ServerSide {
            return;
        }
        const MAX_SORT_ROWS: u64 = 1_000_000;
        let n = self.data.row_count();
        if n > MAX_SORT_ROWS {
            return;
        }
        let mut indices: Vec<u64> = (0..n).collect();
        indices.sort_by(|&a, &b| {
            let va = self.data.get_cell(a, col_key).unwrap_or_default();
            let vb = self.data.get_cell(b, col_key).unwrap_or_default();
            let cmp = match (va.parse::<f64>(), vb.parse::<f64>()) {
                (Ok(fa), Ok(fb)) => {
                    fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
                }
                _ => va.cmp(&vb),
            };
            if *dir == SortDir::Desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        self.sort_order = indices;
    }

    /// Total width of the pinned (frozen) columns in logical pixels.
    pub fn pinned_width(&self) -> f64 {
        if self.pinned_count == 0 {
            return 0.0;
        }
        let n = self.pinned_count.min(self.columns.len());
        if n == self.columns.len() {
            self.column_offsets.total_width
        } else {
            self.column_offsets.offsets[n]
        }
    }

    /// Total scrollable height (header + visible rows).
    pub fn total_height(&self) -> f64 {
        self.header_height + self.display_row_count() as f64 * self.row_height
    }

    /// Total scrollable width.
    pub fn total_width(&self) -> f64 {
        self.column_offsets.total_width
    }

    /// Y position of the top edge of a data row (in content space, before scroll offset).
    pub fn row_top(&self, row_index: u64) -> f64 {
        self.header_height + row_index as f64 * self.row_height
    }

    /// Rebuild column offsets after columns are mutated.
    pub fn rebuild_offsets(&mut self) {
        self.column_offsets = ColumnOffsets::compute(&self.columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{column::ColumnDef, row::RowRecord};

    fn make_model() -> GridModel {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("a", "hello");
                r.set("b", "world");
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("a", "foo");
                r.set("b", "bar");
                r
            },
        ];
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    #[test]
    fn get_cell_from_datasource() {
        let m = make_model();
        assert_eq!(m.get_cell(0, "a"), Some("hello".into()));
        assert_eq!(m.get_cell(1, "b"), Some("bar".into()));
    }

    #[test]
    fn get_cell_missing_key() {
        let m = make_model();
        assert_eq!(m.get_cell(0, "z"), None);
    }

    #[test]
    fn get_cell_out_of_range() {
        let m = make_model();
        assert_eq!(m.get_cell(99, "a"), None);
    }

    #[test]
    fn set_cell_patch_overrides_datasource() {
        let mut m = make_model();
        m.set_cell(0, "a", "patched".into());
        assert_eq!(m.get_cell(0, "a"), Some("patched".into()));
        // other row unchanged
        assert_eq!(m.get_cell(1, "a"), Some("foo".into()));
    }

    #[test]
    fn total_height() {
        let m = make_model();
        // header=40 + 2 rows × 30 = 100
        assert_eq!(m.total_height(), 100.0);
    }

    #[test]
    fn total_width() {
        let m = make_model();
        // 100 + 150 = 250
        assert_eq!(m.total_width(), 250.0);
    }

    #[test]
    fn row_top() {
        let m = make_model();
        assert_eq!(m.row_top(0), 40.0); // header_height
        assert_eq!(m.row_top(1), 70.0); // 40 + 30
        assert_eq!(m.row_top(3), 130.0); // 40 + 3*30
    }

    #[test]
    fn rebuild_offsets_after_column_change() {
        let mut m = make_model();
        m.columns[0].width = 200.0;
        m.rebuild_offsets();
        assert_eq!(m.column_offsets.offsets[1], 200.0);
        assert_eq!(m.total_width(), 350.0);
    }

    #[test]
    fn pinned_width_default_zero() {
        let m = make_model();
        assert_eq!(m.pinned_count, 0);
        assert_eq!(m.pinned_width(), 0.0);
    }

    #[test]
    fn pinned_width_one_column() {
        let mut m = make_model();
        m.pinned_count = 1;
        // First column width = 100
        assert_eq!(m.pinned_width(), 100.0);
    }

    #[test]
    fn pinned_width_all_columns() {
        let mut m = make_model();
        m.pinned_count = 2; // all columns
        assert_eq!(m.pinned_width(), 250.0); // 100 + 150
    }

    #[test]
    fn pinned_width_clamped_to_col_count() {
        let mut m = make_model();
        m.pinned_count = 99;
        assert_eq!(m.pinned_width(), 250.0);
    }
}
