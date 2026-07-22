use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    column::{ColumnDef, ColumnOffsets},
    datasource::{CellStatus, DataSource, VecDataSource},
    filter::{FilterCondition, UniqueValues},
    format::CellFormat,
    row::RowRecord,
    sort::SortDir,
    validation::InvalidEditMode,
};

/// Whether sort/filter are performed client-side or
/// delegated to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DataSourceMode {
    /// All data is in memory. Sort/filter/search done locally.
    #[default]
    ClientSide,
    /// Data comes from a remote server. Sort/filter are
    /// delegated — client-side `apply_sort`/`apply_filter`
    /// become no-ops.
    ServerSide,
}

// ── Sort key cache ──────────────────────────────────

/// Cached pre-extracted sort keys for a single column.
/// Avoids re-running the O(n) `get_cell` pass when
/// toggling between Asc and Desc on the same column.
#[derive(Default)]
enum SortKeyCache {
    /// No cache (initial state or invalidated).
    #[default]
    None,
    /// All values parsed as f64 — compact 8 B/row.
    Numeric { col_key: String, keys: Vec<f64> },
    /// Mixed numeric + string values.
    Mixed {
        col_key: String,
        keys: Vec<MixedSortKey>,
    },
}

impl std::fmt::Debug for SortKeyCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "SortKeyCache::None"),
            Self::Numeric { col_key, keys } => write!(
                f,
                "SortKeyCache::Numeric({}, {} keys)",
                col_key,
                keys.len()
            ),
            Self::Mixed { col_key, keys } => write!(
                f,
                "SortKeyCache::Mixed({}, {} keys)",
                col_key,
                keys.len()
            ),
        }
    }
}

/// A single sort key for mixed (numeric + string) columns.
enum MixedSortKey {
    Num(f64),
    Str(String),
    Empty,
}

// ── Radix sort (numeric fast path) ──────────────────

/// Maps an `f64` bit pattern to a `u64` that preserves total
/// order across the entire domain, including negative values and
/// NaN (which sorts last, consistent with `f64::total_cmp`).
#[inline]
fn f64_to_sort_key(f: f64) -> u64 {
    let bits = f.to_bits();
    // Positive floats and +0: flip sign bit so they sort after negatives.
    // Negative floats and -0: flip all bits so larger-magnitude negatives
    // map to smaller keys.
    if bits >> 63 == 0 {
        bits | (1u64 << 63)
    } else {
        !bits
    }
}

/// LSD radix sort on `indices` ordered by pre-extracted `f64`
/// keys.  O(8 × n) time, O(n) auxiliary space — significantly
/// faster than comparison-based sort for n ≳ 50 000.
fn radix_sort_f64_indices(
    indices: &mut Vec<u64>,
    keys: &[f64],
    ascending: bool,
) {
    let n = indices.len();
    if n <= 1 {
        return;
    }
    // Convert all keys once so the inner loop only touches u64s.
    let sort_keys: Vec<u64> =
        keys.iter().map(|&f| f64_to_sort_key(f)).collect();
    let mut aux: Vec<u64> = vec![0; n];
    for pass in 0..8u32 {
        let shift = pass * 8;
        let mut counts = [0usize; 256];
        for &idx in indices.iter() {
            let byte = ((sort_keys[idx as usize] >> shift) & 0xFF) as usize;
            counts[byte] += 1;
        }
        // Exclusive prefix sum → bucket start positions.
        let mut offsets = [0usize; 256];
        for i in 1..256 {
            offsets[i] = offsets[i - 1] + counts[i - 1];
        }
        for &idx in indices.iter() {
            let byte = ((sort_keys[idx as usize] >> shift) & 0xFF) as usize;
            aux[offsets[byte]] = idx;
            offsets[byte] += 1;
        }
        std::mem::swap(indices, &mut aux);
    }
    if !ascending {
        indices.reverse();
    }
}

/// The data model: columns, a virtual data source, and sizing constants.
#[derive(Debug)]
pub struct GridModel {
    /// Ordered column definitions.
    pub columns: Vec<ColumnDef>,
    /// Backing row data provider.
    pub data: Box<dyn DataSource>,
    /// Height of every data row in logical pixels.
    pub row_height: f64,
    /// Height of the sticky header row in logical pixels.
    pub header_height: f64,
    /// Height of the floating filter row in logical pixels, when shown
    /// (see `show_filter_row`). The stored value is preserved when the
    /// row is hidden, so it can be restored — same pattern as
    /// `header_height`/`show_header`.
    pub filter_row_height: f64,
    /// Whether the floating filter row (a quick per-column "contains"
    /// input, one row directly under the column headers) is rendered.
    /// Opt-in, like `show_checkbox_column` — `false` by default. AND-
    /// combines with (does not replace) the header funnel icon + popup:
    /// both write to the same `filters` map via
    /// `GridCommand::SetColumnFilter`.
    pub show_filter_row: bool,
    /// Precomputed column offsets (recomputed when columns change).
    pub column_offsets: ColumnOffsets,
    /// Edited cell values that override the underlying datasource (works for
    /// any source, including read-only `FnDataSource`).
    pub patches: HashMap<(u64, String), String>,
    /// Width of the sticky row-number gutter on the left in logical pixels (0
    /// = hidden).
    pub row_number_width: f64,
    /// Whether `row_number_width` auto-follows the data source's row
    /// count. Starts `true` (set from `data.row_count()` at construction
    /// and recomputed on every `GridCommand::SetTotalRowCount`, e.g. once
    /// a `PageCacheDataSource` learns its real total from the server —
    /// see `docs/row-count-limits.md` for why a fixed width doesn't scale
    /// here the way it does for AG Grid's row-number column: rs-grid's
    /// supported range spans ~1 to ~15 digits, not a handful). Set to
    /// `false` by `GridCommand::SetRowNumberWidth` — an explicit manual
    /// width is a deliberate override that a later total-count update
    /// must not clobber.
    pub row_number_width_auto: bool,
    /// Whether the row-selection checkbox column is rendered. Unlike the
    /// row-number gutter, it is **not** a fixed/pinned band — it is the
    /// first unpinned (scrollable) column, so it scrolls away with the
    /// data.
    pub show_checkbox_column: bool,
    /// Width of the row-selection checkbox column in logical pixels.
    /// Defaults to [`GridModel::CHECKBOX_COLUMN_WIDTH`]. The checkbox
    /// itself is always centered within this width, so widening it grows
    /// the visual margin around the box symmetrically.
    pub checkbox_column_width: f64,
    /// Logical→physical row index mapping built by `apply_sort`.
    /// Empty = natural (unsorted) order.
    pub sort_order: Vec<u64>,
    /// Cached sort keys from the last `apply_sort` call.
    /// Re-used when toggling direction on the same column
    /// to skip the expensive O(n) extraction pass.
    sort_key_cache: SortKeyCache,
    /// Number of leading columns that remain fixed during horizontal scroll.
    /// 0 = no pinned columns (default).
    pub pinned_count: usize,
    /// Per-column filter conditions (col_key → operator + value).
    /// Empty map = no filter active.
    pub filters: HashMap<String, FilterCondition>,
    /// Per-column value-set restrictions (col_key → allowed values),
    /// the AG-Grid-style checklist filter. AND-combined with `filters`
    /// and with other columns' entries — see
    /// [`Self::apply_filter`]. A present entry restricts that column to
    /// exactly those values; an absent entry means no restriction. An
    /// empty set is a valid, deliberately restrictive state ("matches no
    /// rows"), distinct from an absent entry.
    pub value_filters: HashMap<String, HashSet<String>>,
    /// Physical row indices that pass all active filters, stored in
    /// sort order. Only meaningful when [`Self::is_filter_applied`] is
    /// `true` — an empty `Vec` is ambiguous on its own between "no
    /// filter active" and "a filter is active and genuinely matches zero
    /// rows," which is why every reader goes through that accessor
    /// rather than checking `filtered_indices.is_empty()` directly.
    pub filtered_indices: Vec<u64>,
    /// Backs [`Self::is_filter_applied`] — set by [`Self::apply_filter`]
    /// only on the branch that actually performs a full row scan and
    /// assigns a definitive (possibly empty) `filtered_indices`. `false`
    /// when no filter is active, filtering is delegated to the server,
    /// or the row count exceeds the client-side cap and filtering was
    /// skipped as a safety measure — in all three cases
    /// `filtered_indices` must not be trusted even though it's also
    /// empty in those cases.
    filter_computed: bool,
    /// Whether data operations run client-side or are delegated
    /// to a server.
    pub mode: DataSourceMode,
    /// Height of the horizontal scrollbar in logical pixels.
    /// Used to reserve space at the bottom so the last row
    /// is not obscured.
    pub scrollbar_size: f64,
    /// Whether the column header row is rendered.
    /// `header_height` is preserved so it can be restored.
    pub show_header: bool,
    /// Whether the row-number gutter is rendered.
    /// `row_number_width` is preserved so it can be restored.
    pub show_row_numbers: bool,
    /// Allow inline cell editing grid-wide (true by default).
    /// When false, no cell can be edited regardless of
    /// per-column editable flags.
    pub editable: bool,
    /// Allow cell/row/column selection (true by default).
    /// When false, all selection commands are ignored and
    /// any existing selection is cleared.
    pub selectable: bool,
    /// Allow header drag-to-reorder of columns (true by default).
    /// When false, the column drag interaction is suppressed.
    /// Programmatic `GridCommand::MoveColumn` still works.
    pub column_reorderable: bool,
    /// Policy applied when a `CommitEdit` fails validation.
    /// `Revert` (default) reverts the cell to its previous value;
    /// `Block` keeps the edit session active until the value is
    /// corrected.
    pub invalid_edit_mode: InvalidEditMode,
}

impl GridModel {
    /// Maximum number of rows for which client-side sort is performed.
    /// `apply_sort` is a no-op above this threshold and returns `false`.
    pub const MAX_CLIENT_SORT_ROWS: u64 = 1_000_000;

    /// Default width of the row-selection checkbox column in logical
    /// pixels. Override via `checkbox_column_width` /
    /// `GridCommand::SetCheckboxColumnWidth`.
    pub const CHECKBOX_COLUMN_WIDTH: f64 = 42.0;

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
        let row_number_width = Self::compute_row_number_width(data.row_count());
        Self {
            columns,
            data,
            row_height,
            header_height,
            filter_row_height: 36.0,
            show_filter_row: false,
            column_offsets,
            patches: HashMap::new(),
            row_number_width,
            row_number_width_auto: true,
            show_checkbox_column: false,
            checkbox_column_width: Self::CHECKBOX_COLUMN_WIDTH,
            sort_order: Vec::new(),
            sort_key_cache: SortKeyCache::None,
            pinned_count: 0,
            filters: HashMap::new(),
            value_filters: HashMap::new(),
            filtered_indices: Vec::new(),
            filter_computed: false,
            mode: DataSourceMode::ClientSide,
            scrollbar_size: 14.0,
            show_header: true,
            show_row_numbers: true,
            editable: true,
            selectable: true,
            column_reorderable: true,
            invalid_edit_mode: InvalidEditMode::default(),
        }
    }

    /// Effective header height for layout and rendering.
    ///
    /// Returns `header_height` when `show_header` is `true`, `0.0`
    /// otherwise. The stored value is preserved so it can be
    /// restored when the header is shown again.
    pub fn effective_header_height(&self) -> f64 {
        if self.show_header {
            self.header_height
        } else {
            0.0
        }
    }

    /// Effective filter-row height for layout and rendering.
    ///
    /// Returns `filter_row_height` when `show_filter_row` is `true`,
    /// `0.0` otherwise — same pattern as `effective_header_height()`.
    pub fn effective_filter_row_height(&self) -> f64 {
        if self.show_filter_row {
            self.filter_row_height
        } else {
            0.0
        }
    }

    /// Viewport-y where the *data* band starts — below both the header
    /// and, when shown, the floating filter row. Distinct from
    /// `effective_header_height()` alone, which measures only the
    /// header's own band (still the right boundary for header-specific
    /// hit-tests that don't grow when the filter row is added).
    pub fn data_top(&self) -> f64 {
        self.effective_header_height() + self.effective_filter_row_height()
    }

    /// Effective row-number gutter width for layout and rendering.
    ///
    /// Returns `row_number_width` when `show_row_numbers` is
    /// `true`, `0.0` otherwise.
    pub fn effective_row_number_width(&self) -> f64 {
        if self.show_row_numbers {
            self.row_number_width
        } else {
            0.0
        }
    }

    /// Effective checkbox-column width for layout and rendering.
    ///
    /// Returns `checkbox_column_width` when `show_checkbox_column` is
    /// `true`, `0.0` otherwise. Unlike the row-number gutter, the
    /// checkbox column is **not** a fixed/pinned band — it is the first
    /// unpinned (scrollable) column, so this width is added to
    /// [`Self::total_width`] and to unpinned columns' screen position,
    /// never to a fixed gutter offset. See [`Self::total_width`].
    pub fn effective_checkbox_column_width(&self) -> f64 {
        if self.show_checkbox_column {
            self.checkbox_column_width
        } else {
            0.0
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
    /// Map a logical (display) row index to its physical (datasource) index.
    ///
    /// Accounts for active sort and filter. If `logical` is out of range
    /// (e.g. the dataset was shrunk while a command was in flight), the
    /// index is returned unchanged rather than panicking.
    pub fn logical_to_physical(&self, logical: u64) -> u64 {
        if self.is_filter_applied() {
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
        if self.is_filter_applied() {
            self.filtered_indices.len() as u64
        } else {
            self.data.row_count()
        }
    }

    /// Whether `filtered_indices` should be trusted as-is, including
    /// when empty (meaning literally zero rows match every active
    /// filter) — as opposed to `filtered_indices` being empty because no
    /// filter is active, filtering is delegated to the server, or the
    /// row count exceeds the client-side cap and filtering was skipped
    /// as a safety measure. The two situations produce the same empty
    /// `Vec`; this accessor is the only way to tell them apart. See
    /// [`Self::apply_filter`].
    pub fn is_filter_applied(&self) -> bool {
        self.filter_computed
    }

    /// Rebuild `filtered_indices` from active `filters` and
    /// `value_filters`.
    ///
    /// Iterates rows in sort order and keeps those that match every
    /// active condition and fall within every active value-set
    /// restriction (the two AND-combine).  No-op for datasets larger
    /// than 1 000 000 rows.
    pub fn apply_filter(&mut self) {
        if self.mode == DataSourceMode::ServerSide {
            self.filter_computed = false;
            return;
        }
        if self.filters.is_empty() && self.value_filters.is_empty() {
            self.filtered_indices.clear();
            self.filter_computed = false;
            return;
        }
        // Same threshold as MAX_CLIENT_SORT_ROWS.
        const MAX: u64 = 1_000_000;
        let n = self.data.row_count();
        if n > MAX {
            self.filtered_indices.clear();
            self.filter_computed = false;
            return;
        }
        // Resolve each active filter's column numeric-ness once,
        // not once per row — otherwise this degrades from
        // O(n_rows * n_filters) to O(n_rows * n_filters * n_columns).
        let active: Vec<(&str, &FilterCondition, bool)> = self
            .filters
            .iter()
            .map(|(col_key, condition)| {
                let numeric = self
                    .columns
                    .iter()
                    .find(|c| &c.key == col_key)
                    .and_then(|c| c.format.as_ref())
                    .is_some_and(CellFormat::is_numeric_like);
                (col_key.as_str(), condition, numeric)
            })
            .collect();
        let value_active: Vec<(&str, &HashSet<String>)> = self
            .value_filters
            .iter()
            .map(|(col_key, allowed)| (col_key.as_str(), allowed))
            .collect();
        let count = n as usize;
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let physical = if self.sort_order.is_empty() {
                i as u64
            } else {
                self.sort_order[i]
            };
            let passes = active.iter().all(|(col_key, condition, numeric)| {
                let cell = self
                    .data
                    .get_cell_ref(physical, col_key)
                    .unwrap_or_default();
                condition.matches(&cell, *numeric)
            }) && value_active.iter().all(|(col_key, allowed)| {
                let cell = self
                    .data
                    .get_cell_ref(physical, col_key)
                    .unwrap_or_default();
                allowed.contains(cell.as_ref())
            });
            if passes {
                result.push(physical);
            }
        }
        self.filtered_indices = result;
        self.filter_computed = true;
    }

    /// Distinct values for `col_key` across up to
    /// [`Self::MAX_CLIENT_SORT_ROWS`] rows, sorted — backs the
    /// AG-Grid-style checklist filter popup.
    ///
    /// Bails out to [`UniqueValues::TooMany`] as soon as the running
    /// distinct count exceeds `cap`, bounding memory to `cap + 1`
    /// entries regardless of row count — a fully-unique column (e.g. an
    /// email column) returns fast instead of scanning to completion.
    pub fn unique_values(&self, col_key: &str, cap: usize) -> UniqueValues {
        let mut set = BTreeSet::new();
        let n = self.data.row_count().min(Self::MAX_CLIENT_SORT_ROWS);
        for i in 0..n {
            let cell = self.data.get_cell_ref(i, col_key).unwrap_or_default();
            set.insert(cell.into_owned());
            if set.len() > cap {
                return UniqueValues::TooMany { cap };
            }
        }
        UniqueValues::Values(set.into_iter().collect())
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

    /// Build `sort_order` by sorting row indices by cell values
    /// for `col_key`.
    ///
    /// Pre-extracts all sort keys in a single O(n) pass, then
    /// sorts the pre-parsed keys. **When toggling direction on
    /// the same column, the cached keys are re-used — skipping
    /// the expensive extraction entirely.**
    ///
    /// - **Numeric columns** use a compact `Vec<f64>` (8 B/row) with
    ///   `f64::total_cmp` for cache-friendly, branchless comparison.
    /// - **Mixed / string columns** fall back to a `Vec<MixedSortKey>` with
    ///   lexicographic comparison.
    ///
    /// Returns `true` when the sort was applied, `false` when it was
    /// skipped (server-side mode or row count exceeds
    /// [`Self::MAX_CLIENT_SORT_ROWS`]).
    pub fn apply_sort(&mut self, col_key: &str, dir: &SortDir) -> bool {
        if self.mode == DataSourceMode::ServerSide {
            return false;
        }
        let n = self.data.row_count();
        if n > Self::MAX_CLIENT_SORT_ROWS {
            return false;
        }

        // ── Cache hit? Re-sort without extraction ────────
        let cache_hit = match &self.sort_key_cache {
            SortKeyCache::Numeric { col_key: k, keys }
                if k == col_key && keys.len() == n as usize =>
            {
                true
            }
            SortKeyCache::Mixed { col_key: k, keys }
                if k == col_key && keys.len() == n as usize =>
            {
                true
            }
            _ => false,
        };

        if cache_hit {
            let mut indices: Vec<u64> = (0..n).collect();
            let asc = *dir == SortDir::Asc;
            match &self.sort_key_cache {
                SortKeyCache::Numeric { keys, .. } => {
                    radix_sort_f64_indices(&mut indices, keys, asc);
                }
                SortKeyCache::Mixed { keys, .. } => {
                    indices.sort_unstable_by(|&a, &b| {
                        let cmp =
                            cmp_mixed(&keys[a as usize], &keys[b as usize]);
                        if asc { cmp } else { cmp.reverse() }
                    });
                }
                SortKeyCache::None => {
                    // Defensive: the cache_hit guard above only sets
                    // true for Numeric/Mixed, so reaching None here
                    // means the guard logic drifted. Fail loud in
                    // debug, skip the sort in release.
                    debug_assert!(
                        false,
                        "SortKeyCache::None reached cached sort path",
                    );
                    return false;
                }
            }
            self.sort_order = indices;
            return true;
        }

        // ── Cache miss — extract keys ────────────────────

        // For ImageText columns the raw value is
        // "{data_uri} {label}" — sort on the label only.
        let is_image_text = self
            .columns
            .iter()
            .find(|c| c.key == col_key)
            .and_then(|c| c.format.as_ref())
            .is_some_and(|f| f.is_image_text());

        // Phase 1: try all-numeric fast path.
        // Compact Vec<f64> (8 B/row) — 8 MB at 1M rows.
        let len = n as usize;
        let mut fkeys: Vec<f64> = Vec::with_capacity(len);
        let mut all_numeric = true;

        for i in 0..n {
            match self.data.get_cell_ref(i, col_key) {
                Some(s) => {
                    let src = if is_image_text {
                        s.find(' ').map(|p| &s[p + 1..]).unwrap_or(&s)
                    } else {
                        &s
                    };
                    if let Ok(f) = src.parse::<f64>() {
                        fkeys.push(f);
                    } else {
                        all_numeric = false;
                        break;
                    }
                }
                None => {
                    fkeys.push(f64::NAN);
                }
            }
        }

        if all_numeric {
            let mut indices: Vec<u64> = (0..n).collect();
            radix_sort_f64_indices(&mut indices, &fkeys, *dir == SortDir::Asc);
            self.sort_order = indices;
            self.sort_key_cache = SortKeyCache::Numeric {
                col_key: col_key.to_owned(),
                keys: fkeys,
            };
            return true;
        }

        // Phase 2: mixed / string fallback.
        drop(fkeys);

        let mut keys: Vec<MixedSortKey> = Vec::with_capacity(len);
        for i in 0..n {
            match self.data.get_cell_ref(i, col_key) {
                Some(s) => {
                    let val = if is_image_text {
                        s.find(' ').map(|p| &s[p + 1..]).unwrap_or(&s)
                    } else {
                        &s
                    };
                    if let Ok(f) = val.parse::<f64>() {
                        keys.push(MixedSortKey::Num(f));
                    } else {
                        keys.push(MixedSortKey::Str(val.to_owned()));
                    }
                }
                None => {
                    keys.push(MixedSortKey::Empty);
                }
            }
        }

        let mut indices: Vec<u64> = (0..n).collect();
        let rev = *dir == SortDir::Desc;
        indices.sort_unstable_by(|&a, &b| {
            let cmp = cmp_mixed(&keys[a as usize], &keys[b as usize]);
            if rev { cmp.reverse() } else { cmp }
        });

        self.sort_order = indices;
        self.sort_key_cache = SortKeyCache::Mixed {
            col_key: col_key.to_owned(),
            keys,
        };
        true
    }

    /// Invalidate the sort key cache (e.g. after a cell
    /// edit that may affect sort order).
    pub fn invalidate_sort_cache(&mut self) {
        self.sort_key_cache = SortKeyCache::None;
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

    /// Total scrollable height (header + filter row + visible rows).
    pub fn total_height(&self) -> f64 {
        self.effective_header_height()
            + self.effective_filter_row_height()
            + self.display_row_count() as f64 * self.row_height
    }

    /// Total scrollable width — real columns plus the checkbox column
    /// (when shown), which occupies the first slot of the scrollable
    /// (unpinned) region and scrolls away like any other column.
    pub fn total_width(&self) -> f64 {
        self.column_offsets.total_width + self.effective_checkbox_column_width()
    }

    /// Y position of the top edge of a data row (in content space, before
    /// scroll offset).
    pub fn row_top(&self, row_index: u64) -> f64 {
        self.effective_header_height()
            + self.effective_filter_row_height()
            + row_index as f64 * self.row_height
    }

    /// On-screen X position of column `col_idx`'s left edge,
    /// honoring pinned columns (fixed, no scroll offset) vs.
    /// scrolling columns (shifted by `scroll_x`). `None` if
    /// `col_idx` is out of range.
    ///
    /// Does not include the row-number gutter or checkbox-column
    /// width — callers that need viewport-absolute coordinates add
    /// `effective_row_number_width() + effective_checkbox_column_width()`
    /// themselves, mirroring every existing hit-test call site.
    pub fn column_screen_x(
        &self,
        col_idx: usize,
        scroll_x: f64,
    ) -> Option<f64> {
        let off = *self.column_offsets.offsets.get(col_idx)?;
        Some(if col_idx < self.pinned_count {
            off
        } else {
            off - scroll_x
        })
    }

    /// Rebuild column offsets after columns are mutated.
    pub fn rebuild_offsets(&mut self) {
        self.column_offsets = ColumnOffsets::compute(&self.columns);
    }

    /// Recalculate widths of flex columns to fill the viewport.
    ///
    /// Available space =
    ///   `viewport_width − row_number_width − scrollbar_size − fixed_sum`.
    /// Each flex column gets `available × (flex / total_flex)`,
    /// clamped by `min_width` / `max_width`.
    ///
    /// When available space is zero or negative, flex columns
    /// collapse to their minimum width.
    ///
    /// Call [`rebuild_offsets`](Self::rebuild_offsets) after this.
    pub fn recalculate_flex_widths(&mut self, viewport_width: f64) {
        let mut fixed_sum = 0.0_f64;
        let mut total_flex = 0.0_f64;
        let mut flex_indices: Vec<usize> = Vec::new();

        for (i, col) in self.columns.iter().enumerate() {
            match col.flex {
                Some(f) if f > 0.0 => {
                    total_flex += f;
                    flex_indices.push(i);
                }
                _ => {
                    fixed_sum += col.width;
                }
            }
        }

        if flex_indices.is_empty() {
            return;
        }

        let available = viewport_width
            - self.row_number_width
            - self.effective_checkbox_column_width()
            - self.scrollbar_size
            - fixed_sum;

        // Iterative distribution: when a column hits its
        // min or max, lock it in and redistribute the rest.
        let mut remaining = available;
        let mut remaining_flex = total_flex;
        let mut settled = vec![false; self.columns.len()];

        for _ in 0..flex_indices.len() {
            let mut changed = false;
            for &i in &flex_indices {
                if settled[i] {
                    continue;
                }
                let f = self.columns[i].flex.unwrap_or(0.0);
                let raw = if remaining_flex > 0.0 {
                    remaining * (f / remaining_flex)
                } else {
                    0.0
                };
                let clamped = self.columns[i].clamp_width(raw);
                if (clamped - raw).abs() > f64::EPSILON {
                    settled[i] = true;
                    self.columns[i].width = clamped;
                    remaining -= clamped;
                    remaining_flex -= f;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // Assign remaining unsettled flex columns.
        for &i in &flex_indices {
            if settled[i] {
                continue;
            }
            let f = self.columns[i].flex.unwrap_or(0.0);
            let raw = if remaining_flex > 0.0 {
                remaining * (f / remaining_flex)
            } else {
                0.0
            };
            self.columns[i].width = self.columns[i].clamp_width(raw);
        }
    }
}

// ── builder ────────────────────────────────────────────

/// Builder for constructing a [`GridModel`] with ergonomic
/// defaults.
///
/// ```ignore
/// let model = GridModelBuilder::new(columns, data)
///     .row_height(40.0)
///     .header_height(50.0)
///     .pinned_count(2)
///     .build();
/// ```
pub struct GridModelBuilder {
    columns: Vec<ColumnDef>,
    data: Box<dyn DataSource>,
    row_height: f64,
    header_height: f64,
    filter_row_height: f64,
    pinned_count: usize,
    mode: DataSourceMode,
    scrollbar_size: f64,
    show_header: bool,
    show_row_numbers: bool,
    show_checkbox_column: bool,
    show_filter_row: bool,
    editable: bool,
    selectable: bool,
    column_reorderable: bool,
    invalid_edit_mode: InvalidEditMode,
}

impl GridModelBuilder {
    /// Create a new builder with required parameters.
    ///
    /// Defaults: `row_height = 30.0`, `header_height = 40.0`,
    /// `pinned_count = 0`, `mode = ClientSide`,
    /// `scrollbar_size = 14.0`.
    pub fn new(columns: Vec<ColumnDef>, data: Box<dyn DataSource>) -> Self {
        Self {
            columns,
            data,
            row_height: 30.0,
            header_height: 40.0,
            filter_row_height: 36.0,
            pinned_count: 0,
            mode: DataSourceMode::ClientSide,
            scrollbar_size: 14.0,
            show_header: true,
            show_row_numbers: true,
            show_checkbox_column: false,
            show_filter_row: false,
            editable: true,
            selectable: true,
            column_reorderable: true,
            invalid_edit_mode: InvalidEditMode::default(),
        }
    }

    /// Set the data row height in logical pixels.
    pub fn row_height(mut self, h: f64) -> Self {
        self.row_height = h;
        self
    }

    /// Set the header row height in logical pixels.
    pub fn header_height(mut self, h: f64) -> Self {
        self.header_height = h;
        self
    }

    /// Set the floating filter row height in logical pixels.
    pub fn filter_row_height(mut self, h: f64) -> Self {
        self.filter_row_height = h;
        self
    }

    /// Set the number of leading columns to pin (freeze).
    /// Clamped to `columns.len()` at build time.
    pub fn pinned_count(mut self, n: usize) -> Self {
        self.pinned_count = n;
        self
    }

    /// Set the data source mode (client-side or server-side).
    pub fn mode(mut self, m: DataSourceMode) -> Self {
        self.mode = m;
        self
    }

    /// Set the scrollbar reserved size in logical pixels.
    pub fn scrollbar_size(mut self, s: f64) -> Self {
        self.scrollbar_size = s;
        self
    }

    /// Show or hide the column header row.
    pub fn show_header(mut self, v: bool) -> Self {
        self.show_header = v;
        self
    }

    /// Show or hide the row-number gutter.
    pub fn show_row_numbers(mut self, v: bool) -> Self {
        self.show_row_numbers = v;
        self
    }

    /// Show or hide the row-selection checkbox column.
    pub fn row_selection_checkboxes(mut self, v: bool) -> Self {
        self.show_checkbox_column = v;
        self
    }

    /// Show or hide the floating filter row.
    pub fn show_filter_row(mut self, v: bool) -> Self {
        self.show_filter_row = v;
        self
    }

    /// Allow or disallow inline cell editing grid-wide.
    ///
    /// When `false`, `StartEdit` commands are no-ops regardless of
    /// per-column [`ColumnDef::editable`] flags. Defaults to `true`.
    pub fn editable(mut self, v: bool) -> Self {
        self.editable = v;
        self
    }

    /// Allow or disallow cell/row/column selection.
    pub fn selectable(mut self, v: bool) -> Self {
        self.selectable = v;
        self
    }

    /// Allow or disallow header drag-to-reorder of columns.
    pub fn column_reorderable(mut self, v: bool) -> Self {
        self.column_reorderable = v;
        self
    }

    /// Set the policy applied when a `CommitEdit` fails validation.
    pub fn invalid_edit_mode(mut self, v: InvalidEditMode) -> Self {
        self.invalid_edit_mode = v;
        self
    }

    /// Build the [`GridModel`].
    pub fn build(self) -> GridModel {
        let pinned = self.pinned_count.min(self.columns.len());
        let mut model = GridModel::with_data_source(
            self.columns,
            self.data,
            self.row_height,
            self.header_height,
        );
        model.pinned_count = pinned;
        model.mode = self.mode;
        model.scrollbar_size = self.scrollbar_size;
        model.show_header = self.show_header;
        model.show_row_numbers = self.show_row_numbers;
        model.show_checkbox_column = self.show_checkbox_column;
        model.filter_row_height = self.filter_row_height;
        model.show_filter_row = self.show_filter_row;
        model.editable = self.editable;
        model.selectable = self.selectable;
        model.column_reorderable = self.column_reorderable;
        model.invalid_edit_mode = self.invalid_edit_mode;
        model
    }
}

/// Compare two mixed sort keys: numbers < strings < empty.
fn cmp_mixed(a: &MixedSortKey, b: &MixedSortKey) -> std::cmp::Ordering {
    match (a, b) {
        (MixedSortKey::Num(fa), MixedSortKey::Num(fb)) => fa.total_cmp(fb),
        (MixedSortKey::Str(sa), MixedSortKey::Str(sb)) => sa.cmp(sb),
        (MixedSortKey::Num(_), _) => std::cmp::Ordering::Less,
        (_, MixedSortKey::Num(_)) => std::cmp::Ordering::Greater,
        (MixedSortKey::Empty, MixedSortKey::Empty) => std::cmp::Ordering::Equal,
        (MixedSortKey::Empty, _) => std::cmp::Ordering::Greater,
        (_, MixedSortKey::Empty) => std::cmp::Ordering::Less,
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
    fn get_cell_follows_sort_mapping() {
        // make_numeric_model(5): physical rows hold "5","4","3","2","1".
        let mut m = make_numeric_model(5);
        m.apply_sort("v", &SortDir::Asc);
        // After ascending sort, logical row 0 is the smallest value.
        assert_eq!(m.get_cell(0, "v"), Some("1".into()));
        assert_eq!(m.get_cell(4, "v"), Some("5".into()));
    }

    #[test]
    fn get_cell_follows_filter_mapping() {
        // Only the row whose value contains "1" ("1" itself) survives the
        // filter; it becomes logical row 0.
        let mut m = make_numeric_model(5);
        m.filters.insert("v".into(), FilterCondition::contains("1"));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 1);
        assert_eq!(m.get_cell(0, "v"), Some("1".into()));
    }

    #[test]
    fn get_cell_out_of_range_under_filter_clamps_not_none() {
        // Documents a sharp edge relied on by `cell_at_logical`'s docs: under
        // an active filter a logical row past the filtered count is *clamped*
        // to the raw index by `logical_to_physical` rather than mapped to
        // `None`. With one surviving row (logical 0), logical row 1 falls
        // through to physical row 1 ("4") — a filtered-OUT, undisplayed row.
        let mut m = make_numeric_model(5);
        m.filters.insert("v".into(), FilterCondition::contains("1"));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 1);
        assert_eq!(m.get_cell(1, "v"), Some("4".into()));
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
    fn effective_filter_row_height_false_by_default() {
        let m = make_model();
        assert!(!m.show_filter_row);
        assert_eq!(m.effective_filter_row_height(), 0.0);
    }

    #[test]
    fn effective_filter_row_height_when_shown() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        assert_eq!(m.effective_filter_row_height(), 36.0);
    }

    #[test]
    fn row_top_and_total_height_include_filter_row_when_shown() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // header=40 + filter_row=36 + 0 rows
        assert_eq!(m.row_top(0), 76.0);
        assert_eq!(m.row_top(1), 106.0); // 76 + 30
        // header=40 + filter_row=36 + 2 rows × 30 = 136
        assert_eq!(m.total_height(), 136.0);
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

    // ── Sort tests ───────────────────────────────────────

    fn make_numeric_model(n: usize) -> GridModel {
        let cols = vec![ColumnDef::new("v", "Value", 100.0)];
        let rows: Vec<RowRecord> = (0..n)
            .map(|i| {
                let mut r = RowRecord::new(i as u64);
                r.set("v", (n - i).to_string());
                r
            })
            .collect();
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    #[test]
    fn sort_numeric_asc() {
        let mut m = make_numeric_model(1000);
        m.apply_sort("v", &SortDir::Asc);
        assert_eq!(m.sort_order.len(), 1000);
        // Physical row (n-1) has value "1", should be first
        assert_eq!(m.sort_order[0], 999);
        // Physical row 0 has value "1000", should be last
        assert_eq!(m.sort_order[999], 0);
        // Verify full ordering
        for w in m.sort_order.windows(2) {
            let va: f64 = m.data.get_cell(w[0], "v").unwrap().parse().unwrap();
            let vb: f64 = m.data.get_cell(w[1], "v").unwrap().parse().unwrap();
            assert!(va <= vb);
        }
    }

    #[test]
    fn sort_numeric_desc() {
        let mut m = make_numeric_model(1000);
        m.apply_sort("v", &SortDir::Desc);
        // Physical row 0 has value "1000", should be first
        assert_eq!(m.sort_order[0], 0);
        assert_eq!(m.sort_order[999], 999);
    }

    #[test]
    fn sort_string_lexicographic() {
        let cols = vec![ColumnDef::new("s", "S", 100.0)];
        let values = ["banana", "apple", "cherry", "date"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("s", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("s", &SortDir::Asc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "s").unwrap())
            .collect();
        assert_eq!(sorted, vec!["apple", "banana", "cherry", "date"]);
    }

    #[test]
    fn sort_mixed_numeric_and_string() {
        let cols = vec![ColumnDef::new("m", "M", 100.0)];
        let values = ["banana", "3", "1", "apple", "2"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("m", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("m", &SortDir::Asc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "m").unwrap())
            .collect();
        // Numerics sort first (1, 2, 3), then strings
        assert_eq!(sorted, vec!["1", "2", "3", "apple", "banana"]);
    }

    #[test]
    fn sort_with_empty_cells() {
        let cols = vec![ColumnDef::new("e", "E", 100.0)];
        let mut rows = Vec::new();
        // Row 0: has value
        let mut r = RowRecord::new(0);
        r.set("e", "beta");
        rows.push(r);
        // Row 1: missing column "e"
        rows.push(RowRecord::new(1));
        // Row 2: has value
        let mut r = RowRecord::new(2);
        r.set("e", "alpha");
        rows.push(r);

        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("e", &SortDir::Asc);
        let sorted: Vec<Option<String>> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "e"))
            .collect();
        // Non-empty first (alpha, beta), empty last
        assert_eq!(
            sorted,
            vec![Some("alpha".into()), Some("beta".into()), None]
        );
    }

    #[test]
    fn sort_clear_restores_natural_order() {
        let mut m = make_numeric_model(100);
        m.apply_sort("v", &SortDir::Asc);
        assert!(!m.sort_order.is_empty());
        m.sort_order.clear();
        assert_eq!(m.logical_to_physical(0), 0);
        assert_eq!(m.logical_to_physical(99), 99);
    }

    #[test]
    fn sort_cache_reused_on_direction_toggle() {
        let mut m = make_numeric_model(1000);

        // First sort — populates cache
        m.apply_sort("v", &SortDir::Asc);
        let asc_first = m.sort_order[0];
        let asc_last = m.sort_order[999];
        assert!(matches!(m.sort_key_cache, SortKeyCache::Numeric { .. }));

        // Toggle to Desc — should hit cache
        m.apply_sort("v", &SortDir::Desc);
        assert_eq!(m.sort_order[0], asc_last);
        assert_eq!(m.sort_order[999], asc_first);

        // Cache still present
        assert!(matches!(m.sort_key_cache, SortKeyCache::Numeric { .. }));
    }

    #[test]
    fn sort_cache_invalidated_on_clear() {
        let mut m = make_numeric_model(100);
        m.apply_sort("v", &SortDir::Asc);
        assert!(!matches!(m.sort_key_cache, SortKeyCache::None));
        m.invalidate_sort_cache();
        assert!(matches!(m.sort_key_cache, SortKeyCache::None));
    }

    #[test]
    fn sort_cache_replaced_on_different_column() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        let rows: Vec<RowRecord> = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", (100 - i).to_string());
                r.set("b", (i + 1).to_string());
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);

        m.apply_sort("a", &SortDir::Asc);
        assert!(
            matches!(
                &m.sort_key_cache,
                SortKeyCache::Numeric { col_key, .. }
                    if col_key == "a"
            ),
            "expected Numeric cache for col a"
        );

        // Sort on different column replaces cache
        m.apply_sort("b", &SortDir::Asc);
        assert!(
            matches!(
                &m.sort_key_cache,
                SortKeyCache::Numeric { col_key, .. }
                    if col_key == "b"
            ),
            "expected Numeric cache for col b"
        );
    }

    // ── GridModelBuilder ──────────────────────────────

    #[test]
    fn builder_defaults() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = vec![RowRecord::new(0)];
        let m = GridModelBuilder::new(cols, Box::new(VecDataSource::new(rows)))
            .build();
        assert_eq!(m.row_height, 30.0);
        assert_eq!(m.header_height, 40.0);
        assert_eq!(m.pinned_count, 0);
        assert_eq!(m.mode, DataSourceMode::ClientSide);
        assert_eq!(m.scrollbar_size, 14.0);
    }

    #[test]
    fn builder_overrides() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        let rows = vec![RowRecord::new(0)];
        let m = GridModelBuilder::new(cols, Box::new(VecDataSource::new(rows)))
            .row_height(50.0)
            .header_height(60.0)
            .pinned_count(1)
            .mode(DataSourceMode::ServerSide)
            .scrollbar_size(20.0)
            .build();
        assert_eq!(m.row_height, 50.0);
        assert_eq!(m.header_height, 60.0);
        assert_eq!(m.pinned_count, 1);
        assert_eq!(m.mode, DataSourceMode::ServerSide);
        assert_eq!(m.scrollbar_size, 20.0);
    }

    // ── column_screen_x ──────────────────────────────

    #[test]
    fn column_screen_x_pinned_ignores_scroll() {
        let mut m = make_model();
        m.pinned_count = 1;
        let off = m.column_offsets.offsets[0];
        assert_eq!(m.column_screen_x(0, 500.0), Some(off));
    }

    #[test]
    fn column_screen_x_unpinned_shifts_by_scroll() {
        let m = make_model();
        let off = m.column_offsets.offsets[1];
        assert_eq!(m.column_screen_x(1, 40.0), Some(off - 40.0));
    }

    #[test]
    fn column_screen_x_out_of_range_is_none() {
        let m = make_model();
        assert_eq!(m.column_screen_x(999, 0.0), None);
    }

    #[test]
    fn builder_pinned_count_clamped() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = vec![RowRecord::new(0)];
        let m = GridModelBuilder::new(cols, Box::new(VecDataSource::new(rows)))
            .pinned_count(999)
            .build();
        // Clamped to number of columns
        assert_eq!(m.pinned_count, 1);
    }

    #[test]
    fn builder_show_header_false() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .show_header(false)
                .build();
        assert!(!m.show_header);
    }

    #[test]
    fn builder_show_filter_row() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .show_filter_row(true)
                .filter_row_height(30.0)
                .build();
        assert!(m.show_filter_row);
        assert_eq!(m.effective_filter_row_height(), 30.0);
    }

    #[test]
    fn builder_show_row_numbers_false() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .show_row_numbers(false)
                .build();
        assert!(!m.show_row_numbers);
    }

    #[test]
    fn builder_row_selection_checkboxes() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .row_selection_checkboxes(true)
                .build();
        assert!(m.show_checkbox_column);
        assert_eq!(
            m.effective_checkbox_column_width(),
            GridModel::CHECKBOX_COLUMN_WIDTH
        );
        // Not a fixed gutter — it's folded into total_width() as the
        // first slot of the scrollable region.
        assert_eq!(m.total_width(), 100.0 + GridModel::CHECKBOX_COLUMN_WIDTH);
    }

    #[test]
    fn checkbox_column_width_zero_when_disabled() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .build();
        assert!(!m.show_checkbox_column);
        assert_eq!(m.effective_checkbox_column_width(), 0.0);
        assert_eq!(m.total_width(), 100.0);
    }

    #[test]
    fn builder_selectable_false() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .selectable(false)
                .build();
        assert!(!m.selectable);
    }

    #[test]
    fn builder_column_reorderable_false() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .column_reorderable(false)
                .build();
        assert!(!m.column_reorderable);
    }

    #[test]
    fn builder_editable_false() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![])))
                .editable(false)
                .build();
        assert!(!m.editable);
    }

    // ── recalculate_flex_widths ─────────────────────

    fn flex_model(cols: Vec<ColumnDef>) -> GridModel {
        let rows = vec![RowRecord::new(0)];
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    #[test]
    fn flex_basic_distribution() {
        // 1 fixed (100) + 2 flex:1
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::simple("b", "B").with_flex(1.0),
            ColumnDef::simple("c", "C").with_flex(1.0),
        ];
        let mut m = flex_model(cols);
        // viewport=600, rnw=m.row_number_width, sb=14
        let vp = 600.0;
        let avail = vp - m.row_number_width - m.scrollbar_size - 100.0;
        m.recalculate_flex_widths(vp);
        let half = avail / 2.0;
        assert!(
            (m.columns[1].width - half).abs() < 0.01,
            "col1={}, expected {half}",
            m.columns[1].width
        );
        assert!(
            (m.columns[2].width - half).abs() < 0.01,
            "col2={}, expected {half}",
            m.columns[2].width
        );
        // Fixed column unchanged
        assert_eq!(m.columns[0].width, 100.0);
    }

    #[test]
    fn flex_weighted_distribution() {
        // flex:1 and flex:2 → 1/3 and 2/3
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::simple("b", "B").with_flex(1.0),
            ColumnDef::simple("c", "C").with_flex(2.0),
        ];
        let mut m = flex_model(cols);
        let vp = 600.0;
        let avail = vp - m.row_number_width - m.scrollbar_size - 100.0;
        m.recalculate_flex_widths(vp);
        let third = avail / 3.0;
        assert!(
            (m.columns[1].width - third).abs() < 0.01,
            "col1={}, expected {third}",
            m.columns[1].width
        );
        assert!(
            (m.columns[2].width - third * 2.0).abs() < 0.01,
            "col2={}, expected {}",
            m.columns[2].width,
            third * 2.0
        );
    }

    #[test]
    fn flex_with_max_redistributes() {
        // flex:1 with max=80 + flex:1 unconstrained
        let mut col_b = ColumnDef::simple("b", "B").with_flex(1.0);
        col_b.max_width = Some(80.0);
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            col_b,
            ColumnDef::simple("c", "C").with_flex(1.0),
        ];
        let mut m = flex_model(cols);
        let vp = 600.0;
        let avail = vp - m.row_number_width - m.scrollbar_size - 100.0;
        m.recalculate_flex_widths(vp);
        // col_b clamped to 80, remainder goes to col_c
        assert_eq!(m.columns[1].width, 80.0);
        let expected_c = avail - 80.0;
        assert!(
            (m.columns[2].width - expected_c).abs() < 0.01,
            "col2={}, expected {expected_c}",
            m.columns[2].width
        );
    }

    #[test]
    fn flex_negative_available_collapses_to_min() {
        // Fixed columns exceed viewport → flex gets min
        let cols = vec![
            ColumnDef::new("a", "A", 500.0),
            ColumnDef::new("b", "B", 500.0),
            ColumnDef::simple("c", "C").with_flex(1.0),
        ];
        let mut m = flex_model(cols);
        m.recalculate_flex_widths(200.0); // way too small
        assert_eq!(
            m.columns[2].width,
            crate::column::MIN_COL_WIDTH,
            "flex col should collapse to min"
        );
    }

    #[test]
    fn flex_all_columns() {
        // All flex, no fixed columns
        let cols = vec![
            ColumnDef::simple("a", "A").with_flex(1.0),
            ColumnDef::simple("b", "B").with_flex(1.0),
        ];
        let mut m = flex_model(cols);
        let vp = 400.0;
        let avail = vp - m.row_number_width - m.scrollbar_size;
        m.recalculate_flex_widths(vp);
        let half = avail / 2.0;
        assert!(
            (m.columns[0].width - half).abs() < 0.01,
            "col0={}, expected {half}",
            m.columns[0].width
        );
        assert!(
            (m.columns[1].width - half).abs() < 0.01,
            "col1={}, expected {half}",
            m.columns[1].width
        );
    }

    #[test]
    fn flex_no_flex_columns_is_noop() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 200.0),
        ];
        let mut m = flex_model(cols);
        m.recalculate_flex_widths(800.0);
        assert_eq!(m.columns[0].width, 100.0);
        assert_eq!(m.columns[1].width, 200.0);
    }

    // ── SortKeyCache Debug ───────────────────────────────

    #[test]
    fn sort_key_cache_debug_none() {
        let m = make_model();
        let dbg = format!("{:?}", m);
        assert!(dbg.contains("SortKeyCache::None"));
    }

    #[test]
    fn sort_key_cache_debug_numeric() {
        let mut m = make_numeric_model(10);
        m.apply_sort("v", &SortDir::Asc);
        let dbg = format!("{:?}", m);
        assert!(dbg.contains("SortKeyCache::Numeric"));
    }

    #[test]
    fn sort_key_cache_debug_mixed() {
        let cols = vec![ColumnDef::new("s", "S", 100.0)];
        let values = ["banana", "123"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("s", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("s", &SortDir::Asc);
        let dbg = format!("{:?}", m);
        assert!(dbg.contains("SortKeyCache::Mixed"));
    }

    // ── f64_to_sort_key: negative float path ─────────────

    #[test]
    fn sort_negative_numbers() {
        let cols = vec![ColumnDef::new("v", "V", 100.0)];
        let values = ["-5", "3", "-1", "0", "10"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("v", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("v", &SortDir::Asc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "v").unwrap())
            .collect();
        assert_eq!(sorted, vec!["-5", "-1", "0", "3", "10"]);
    }

    // ── radix_sort_f64_indices n<=1 early return ─────────

    #[test]
    fn sort_single_row() {
        let cols = vec![ColumnDef::new("v", "V", 100.0)];
        let mut r = RowRecord::new(0);
        r.set("v", "42");
        let mut m = GridModel::new(cols, vec![r], 30.0, 40.0);
        assert!(m.apply_sort("v", &SortDir::Asc));
        assert_eq!(m.sort_order, vec![0]);
    }

    // ── logical_to_physical with sort order (no filter) ──

    #[test]
    fn logical_to_physical_with_sort() {
        let mut m = make_numeric_model(5);
        m.apply_sort("v", &SortDir::Asc);
        // After sorting, logical row 0 maps to physical row 4
        // (value "1" is at physical row 4)
        let physical = m.logical_to_physical(0);
        assert_eq!(physical, 4);
    }

    // ── server-side mode: apply_sort returns false ────────

    #[test]
    fn apply_sort_server_side_is_noop() {
        let cols = vec![ColumnDef::new("v", "V", 100.0)];
        let rows = vec![RowRecord::new(0)];
        let mut m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(rows)))
                .mode(DataSourceMode::ServerSide)
                .build();
        assert!(!m.apply_sort("v", &SortDir::Asc));
        assert!(m.sort_order.is_empty());
    }

    // ── apply_filter: server-side, empty, large dataset ──

    #[test]
    fn apply_filter_server_side_is_noop() {
        let cols = vec![ColumnDef::new("v", "V", 100.0)];
        let mut r = RowRecord::new(0);
        r.set("v", "hello");
        let mut m =
            GridModelBuilder::new(cols, Box::new(VecDataSource::new(vec![r])))
                .mode(DataSourceMode::ServerSide)
                .build();
        m.filters
            .insert("v".into(), FilterCondition::contains("hello"));
        m.apply_filter();
        assert!(
            m.filtered_indices.is_empty(),
            "server-side mode skips filter"
        );
        assert!(!m.is_filter_applied());
    }

    // ── is_filter_applied: distinguishing "no filter" from
    // "filter matches zero rows" — both leave filtered_indices empty ──

    #[test]
    fn is_filter_applied_false_with_no_filters() {
        let m = make_model();
        assert!(!m.is_filter_applied());
        assert_eq!(m.display_row_count(), 2);
    }

    #[test]
    fn is_filter_applied_true_when_condition_filter_matches_nothing() {
        let mut m = make_model();
        m.filters
            .insert("a".into(), FilterCondition::contains("zzz-no-match"));
        m.apply_filter();
        assert!(m.filtered_indices.is_empty());
        assert!(m.is_filter_applied());
        assert_eq!(
            m.display_row_count(),
            0,
            "a filter matching zero rows must report 0 rows, not the \
             unfiltered total"
        );
    }

    #[test]
    fn is_filter_applied_resets_false_after_clearing_a_zero_match_filter() {
        // Regression test: a direct `filtered_indices.clear()` (bypassing
        // apply_filter) would leave `filter_computed` stale `true` from
        // the zero-match filter below, making display_row_count() keep
        // reporting 0 instead of springing back to the full count.
        let mut m = make_model();
        m.filters
            .insert("a".into(), FilterCondition::contains("zzz-no-match"));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 0);
        m.filters.clear();
        m.apply_filter();
        assert!(!m.is_filter_applied());
        assert_eq!(m.display_row_count(), 2);
    }

    #[test]
    fn apply_filter_empty_filters_clears() {
        let mut m = make_model();
        // First add a filter to populate filtered_indices
        m.filters
            .insert("a".into(), FilterCondition::contains("hello"));
        m.apply_filter();
        assert!(!m.filtered_indices.is_empty());
        // Clear the filters map and reapply
        m.filters.clear();
        m.apply_filter();
        assert!(m.filtered_indices.is_empty());
    }

    // ── apply_filter: value_filters (checklist filter) ───

    #[test]
    fn apply_filter_value_filter_alone_restricts_rows() {
        // make_model rows: ("hello","world"), ("foo","bar").
        let mut m = make_model();
        m.value_filters
            .insert("a".into(), HashSet::from(["hello".to_string()]));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 1);
        assert_eq!(m.get_cell(0, "a"), Some("hello".into()));
    }

    #[test]
    fn apply_filter_value_filter_empty_set_matches_nothing() {
        let mut m = make_model();
        m.value_filters.insert("a".into(), HashSet::new());
        m.apply_filter();
        assert_eq!(m.display_row_count(), 0);
    }

    #[test]
    fn apply_filter_condition_and_value_filter_and_combine() {
        let mut m = make_model();
        // Condition alone would match both rows ("o" is in both).
        m.filters.insert("a".into(), FilterCondition::contains("o"));
        // Value filter alone would match only the "foo" row.
        m.value_filters
            .insert("a".into(), HashSet::from(["foo".to_string()]));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 1);
        assert_eq!(m.get_cell(0, "a"), Some("foo".into()));
    }

    #[test]
    fn apply_filter_value_filters_on_different_columns_and_combine() {
        let mut m = make_model();
        m.value_filters
            .insert("a".into(), HashSet::from(["hello".to_string()]));
        m.value_filters
            .insert("b".into(), HashSet::from(["bar".to_string()]));
        m.apply_filter();
        // Row 0 has a="hello" (passes column a) but b="world" (fails
        // column b) — no row satisfies both.
        assert_eq!(m.display_row_count(), 0);
    }

    #[test]
    fn apply_filter_clearing_value_filter_restores_all_rows() {
        let mut m = make_model();
        m.value_filters
            .insert("a".into(), HashSet::from(["hello".to_string()]));
        m.apply_filter();
        assert_eq!(m.display_row_count(), 1);
        m.value_filters.remove("a");
        m.apply_filter();
        assert_eq!(m.display_row_count(), 2);
    }

    // ── unique_values ─────────────────────────────────────

    #[test]
    fn unique_values_returns_sorted_distinct_values() {
        let m = make_model();
        assert_eq!(
            m.unique_values("a", 10),
            UniqueValues::Values(vec!["foo".to_string(), "hello".to_string()])
        );
    }

    #[test]
    fn unique_values_dedups() {
        let cols = vec![ColumnDef::new("v", "V", 100.0)];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("v", "x");
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("v", "x");
                r
            },
            {
                let mut r = RowRecord::new(2);
                r.set("v", "y");
                r
            },
        ];
        let m = GridModel::new(cols, rows, 30.0, 40.0);
        assert_eq!(
            m.unique_values("v", 10),
            UniqueValues::Values(vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn unique_values_too_many_when_cap_exceeded() {
        let m = make_model();
        assert_eq!(m.unique_values("a", 1), UniqueValues::TooMany { cap: 1 });
    }

    #[test]
    fn unique_values_missing_column_yields_empty_string() {
        // A missing cell resolves to the default (empty string), same as
        // apply_filter's own `unwrap_or_default()` handling.
        let m = make_model();
        assert_eq!(
            m.unique_values("does-not-exist", 10),
            UniqueValues::Values(vec!["".to_string()])
        );
    }

    // ── cell_status with patches ─────────────────────────

    #[test]
    fn cell_status_returns_patched_value() {
        let mut m = make_model();
        m.set_cell(0, "a", "patched".into());
        assert_eq!(m.cell_status(0, "a"), CellStatus::Ready("patched".into()));
    }

    // ── ImageText sort extraction ────────────────────────

    #[test]
    fn sort_image_text_column_numeric() {
        use crate::format::CellFormat;
        let cols = vec![ColumnDef {
            key: "c".into(),
            label: "C".into(),
            width: 100.0,
            min_width: None,
            max_width: None,
            flex: None,
            format: Some(CellFormat::ImageText {
                base_url: String::new(),
                suffix: String::new(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 4.0,
            }),
            editor: None,
            validator: None,
            rules: Vec::new(),
            bold: false,
            editable: true,
            editable_predicate: None,
            decorator: None,
            cell_buttons: Vec::new(),
            cell_buttons_visible: None,
        }];
        let values = ["us 3", "fr 1", "de 2"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("c", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("c", &SortDir::Asc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "c").unwrap())
            .collect();
        assert_eq!(sorted, vec!["fr 1", "de 2", "us 3"]);
    }

    #[test]
    fn sort_image_text_column_mixed() {
        use crate::format::CellFormat;
        let cols = vec![ColumnDef {
            key: "c".into(),
            label: "C".into(),
            width: 100.0,
            min_width: None,
            max_width: None,
            flex: None,
            format: Some(CellFormat::ImageText {
                base_url: String::new(),
                suffix: String::new(),
                image_size: 20.0,
                border_radius: 0.0,
                gap: 4.0,
            }),
            editor: None,
            validator: None,
            rules: Vec::new(),
            bold: false,
            editable: true,
            editable_predicate: None,
            decorator: None,
            cell_buttons: Vec::new(),
            cell_buttons_visible: None,
        }];
        // Mix of numeric and string labels → mixed sort path
        let values = ["us France", "fr 1", "de Allemagne"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("c", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("c", &SortDir::Asc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "c").unwrap())
            .collect();
        // Numeric first (1), then strings (Allemagne, France)
        assert_eq!(sorted, vec!["fr 1", "de Allemagne", "us France"]);
    }

    // ── MixedSortKey::Empty == Empty ─────────────────────

    #[test]
    fn sort_multiple_empty_cells_mixed() {
        let cols = vec![ColumnDef::new("s", "S", 100.0)];
        // 2 rows with value + 2 rows empty → force Mixed path
        let mut rows = Vec::new();
        let mut r = RowRecord::new(0);
        r.set("s", "beta");
        rows.push(r);
        rows.push(RowRecord::new(1)); // empty
        rows.push(RowRecord::new(2)); // empty
        let mut r = RowRecord::new(3);
        r.set("s", "alpha");
        rows.push(r);
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("s", &SortDir::Asc);
        let sorted: Vec<Option<String>> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "s"))
            .collect();
        // alpha, beta, empty, empty
        assert_eq!(
            sorted,
            vec![Some("alpha".into()), Some("beta".into()), None, None,]
        );
    }

    // ── sort cache hit with Mixed ────────────────────────

    #[test]
    fn sort_cache_reused_for_mixed() {
        let cols = vec![ColumnDef::new("s", "S", 100.0)];
        let values = ["banana", "1", "apple"];
        let rows: Vec<RowRecord> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let mut r = RowRecord::new(i as u64);
                r.set("s", *v);
                r
            })
            .collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.apply_sort("s", &SortDir::Asc);
        assert!(matches!(m.sort_key_cache, SortKeyCache::Mixed { .. }));
        // Toggle direction — should hit cache
        m.apply_sort("s", &SortDir::Desc);
        let sorted: Vec<String> = m
            .sort_order
            .iter()
            .map(|&i| m.data.get_cell(i, "s").unwrap())
            .collect();
        assert_eq!(sorted, vec!["banana", "apple", "1"]);
    }
}
