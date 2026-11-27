use crate::{model::GridModel, selection::CellCoord};

/// Convert a pointer position in *viewport* space (logical pixels, top-left
/// origin) into a `CellCoord`, accounting for the current scroll offset and
/// row-number gutter.
///
/// Returns `None` when:
/// - The pointer is over the row-number gutter.
/// - The pointer is over the column header.
/// - The pointer is below the last row.
/// - The pointer is to the right of the last column.
pub fn hit_test(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
    scroll_y: f64,
) -> Option<CellCoord> {
    // Gutter zone (row numbers only — the checkbox column, when shown,
    // scrolls with the data instead of living in this fixed band).
    let rnw = model.effective_row_number_width();
    if vx < rnw {
        return None;
    }

    // Convert viewport coords to content (absolute) coords.
    // Pinned columns are not affected by scroll_x; scrollable columns are.
    let vx_data = vx - rnw;
    let pinned_width = model.pinned_width();
    let abs_x = if vx_data < pinned_width {
        vx_data // pinned zone: no scroll
    } else {
        // Scrollable zone: the checkbox column (if shown) occupies the
        // first `ccw` px of this zone and scrolls away like any other
        // column — a hit here is the checkbox, not a data cell.
        let raw = vx_data + scroll_x;
        let ccw = model.effective_checkbox_column_width();
        if ccw > 0.0 && raw < pinned_width + ccw {
            return None;
        }
        raw - ccw
    };
    // Header (+ filter row, when shown) is sticky — data starts below it.
    if vy < model.data_top() {
        return None;
    }

    let row = logical_row_at_vy(vy, model, scroll_y)?;

    // Column index using precomputed offsets.
    let col = model.column_offsets.hit_column(abs_x, &model.columns)?;

    Some(CellCoord { row, col })
}

/// Returns the column index when the pointer is over a column header.
///
/// Returns `None` when the pointer is outside the header zone, in the
/// row-number gutter, or over the (scrolling) checkbox column.
pub fn hit_test_col_header(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
) -> Option<usize> {
    let rnw = model.effective_row_number_width();
    // Must be in header row and to the right of the row-number gutter.
    if vy >= model.effective_header_height() || vx < rnw {
        return None;
    }
    let vx_data = vx - rnw;
    let pinned_width = model.pinned_width();
    let abs_x = if vx_data < pinned_width {
        vx_data
    } else {
        let raw = vx_data + scroll_x;
        let ccw = model.effective_checkbox_column_width();
        if ccw > 0.0 && raw < pinned_width + ccw {
            return None;
        }
        raw - ccw
    };
    model.column_offsets.hit_column(abs_x, &model.columns)
}

/// Returns the column index when the pointer is over the floating
/// filter row — same column resolution as `hit_test_col_header`, but
/// checked against the filter row's own vertical band (directly below
/// the header) instead of the header's.
///
/// Returns `None` when the filter row is hidden, the pointer is outside
/// its band, in the row-number gutter, or over the (scrolling) checkbox
/// column.
pub fn hit_test_filter_row_cell(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
) -> Option<usize> {
    let fh = model.effective_filter_row_height();
    if fh <= 0.0 {
        return None;
    }
    let hh = model.effective_header_height();
    if vy < hh || vy >= hh + fh {
        return None;
    }
    let rnw = model.effective_row_number_width();
    if vx < rnw {
        return None;
    }
    let vx_data = vx - rnw;
    let pinned_width = model.pinned_width();
    let abs_x = if vx_data < pinned_width {
        vx_data
    } else {
        let raw = vx_data + scroll_x;
        let ccw = model.effective_checkbox_column_width();
        if ccw > 0.0 && raw < pinned_width + ccw {
            return None;
        }
        raw - ccw
    };
    model.column_offsets.hit_column(abs_x, &model.columns)
}

/// Returns the row index when the pointer is over the sticky row-number gutter.
///
/// Returns `None` when the pointer is outside the gutter, in the header area,
/// or below the last row.
pub fn hit_test_row_header(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_y: f64,
) -> Option<u64> {
    let rnw = model.effective_row_number_width();
    if rnw <= 0.0 || vx >= rnw {
        return None;
    }
    if vy < model.data_top() {
        return None;
    }
    logical_row_at_vy(vy, model, scroll_y)
}

/// Returns `true` when viewport-x `vx` falls within the checkbox
/// column's current (scroll-dependent) band — it is the first unpinned
/// column, so its screen position moves with `scroll_x` exactly like a
/// real column.
///
/// `None`/`false` results from the callers below when the checkbox
/// column is hidden, `vx` is in the row-number gutter or the pinned
/// band, or the checkbox has scrolled out of view.
fn checkbox_band_hit(vx: f64, model: &GridModel, scroll_x: f64) -> bool {
    let ccw = model.effective_checkbox_column_width();
    if ccw <= 0.0 {
        return false;
    }
    let rnw = model.effective_row_number_width();
    if vx < rnw {
        return false;
    }
    let vx_data = vx - rnw;
    let pinned_width = model.pinned_width();
    if vx_data < pinned_width {
        return false; // pinned real column, not the checkbox
    }
    let raw = vx_data + scroll_x;
    raw >= pinned_width && raw < pinned_width + ccw
}

/// Returns the row index when the pointer is over the (scrolling)
/// checkbox column.
///
/// Returns `None` when the checkbox column is hidden, scrolled out of
/// view, in the header area, or below the last row.
pub fn hit_test_checkbox_row(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
    scroll_y: f64,
) -> Option<u64> {
    if !checkbox_band_hit(vx, model, scroll_x) {
        return None;
    }
    if vy < model.data_top() {
        return None;
    }
    logical_row_at_vy(vy, model, scroll_y)
}

/// Returns `true` when the pointer is over the header's checkbox
/// (select-all) cell.
///
/// Returns `false` when the checkbox column is hidden, scrolled out of
/// view, or the pointer is outside the header row.
pub fn hit_test_checkbox_header(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
) -> bool {
    vy < model.effective_header_height()
        && checkbox_band_hit(vx, model, scroll_x)
}

/// Shared row-index math for `hit_test`/`hit_test_row_header`/
/// `hit_test_checkbox_row` — converts a viewport y-coordinate (already
/// known to be below the header) into a logical row index, preserving
/// f64 precision at extreme row counts.
///
/// Returns `None` when the resulting row is at or past
/// `model.display_row_count()`.
fn logical_row_at_vy(vy: f64, model: &GridModel, scroll_y: f64) -> Option<u64> {
    let hh = model.data_top();
    let rh = model.row_height;
    // row = floor((vy - hh + scroll_y) / rh)
    // When scroll_y >= hh, decompose to keep numbers small (avoid
    // subtracting two large f64s at extreme scroll offsets).
    let row = if scroll_y >= hh {
        let sy_content = scroll_y - hh;
        let first_row = (sy_content / rh) as u64;
        let frac = sy_content % rh; // sub-row offset within first_row
        // vy + frac is the inverse of the scene builder's
        // row_vy(ri) = -frac + (ri - first_row) * rh.
        let offset = ((vy + frac) / rh) as u64;
        first_row + offset
    } else {
        ((vy + scroll_y - hh) / rh) as u64
    };
    if row >= model.display_row_count() {
        return None;
    }
    Some(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{column::ColumnDef, model::GridModel, row::RowRecord};

    /// 2 columns (100 + 150 px), 5 rows, row_height=30, header=40,
    /// row_number_width=50 (default).
    fn make_model() -> GridModel {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows = (0..5).map(RowRecord::new).collect();
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    // ── hit_test (data cells)
    // ─────────────────────────────────────────────────

    #[test]
    fn hit_first_cell() {
        let m = make_model();
        // vx=60 (past gutter=50) → abs_x=10 → col 0
        // vy=50 (past header=40) → abs_y=50 → row_y=10 → row 0
        let c = hit_test(60.0, 50.0, &m, 0.0, 0.0).unwrap();
        assert_eq!(c.row, 0);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn hit_second_column() {
        let m = make_model();
        // abs_x = (160 - 50) + 0 = 110 → col 1 (offset 100)
        let c = hit_test(160.0, 50.0, &m, 0.0, 0.0).unwrap();
        assert_eq!(c.col, 1);
    }

    #[test]
    fn hit_second_row() {
        let m = make_model();
        // vy=85, abs_y=85, row_y=85-40=45, row=45/30=1
        let c = hit_test(60.0, 85.0, &m, 0.0, 0.0).unwrap();
        assert_eq!(c.row, 1);
    }

    #[test]
    fn hit_in_gutter_returns_none() {
        let m = make_model();
        assert!(hit_test(30.0, 50.0, &m, 0.0, 0.0).is_none());
    }

    #[test]
    fn hit_in_header_returns_none() {
        let m = make_model();
        // vy=20 → abs_y=20 < header_height=40
        assert!(hit_test(60.0, 20.0, &m, 0.0, 0.0).is_none());
    }

    #[test]
    fn hit_below_last_row_returns_none() {
        let m = make_model();
        // 5 rows × 30 = 150 px of data; vy = 40 + 150 + 10 = 200
        assert!(hit_test(60.0, 200.0, &m, 0.0, 0.0).is_none());
    }

    #[test]
    fn hit_right_of_last_column_returns_none() {
        let m = make_model();
        // total col width = 250; abs_x = (350 - 50) + 0 = 300 → past last col
        assert!(hit_test(350.0, 50.0, &m, 0.0, 0.0).is_none());
    }

    #[test]
    fn hit_with_scroll() {
        let m = make_model();
        // scroll_y=30 → row 0 is scrolled off; vy=50 now hits row 1
        let c = hit_test(60.0, 50.0, &m, 0.0, 30.0).unwrap();
        assert_eq!(c.row, 1);
    }

    #[test]
    fn hit_test_data_top_shifts_by_filter_row_when_shown() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // Data now starts at 40 (header) + 36 (filter row) = 76 —
        // a point that used to hit row 0 (vy=50) now falls inside the
        // filter row band and hits nothing.
        assert!(hit_test(60.0, 50.0, &m, 0.0, 0.0).is_none());
        // Just past the new boundary hits row 0.
        let c = hit_test(60.0, 80.0, &m, 0.0, 0.0).unwrap();
        assert_eq!(c.row, 0);
    }

    #[test]
    fn hit_test_col_header_band_unaffected_by_filter_row() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // The header row's own band is still exactly [0, 40) —
        // hit_test_col_header must not grow just because a filter row
        // was added below it.
        assert_eq!(hit_test_col_header(60.0, 20.0, &m, 0.0), Some(0));
        assert_eq!(hit_test_col_header(60.0, 50.0, &m, 0.0), None);
    }

    // ── hit_test_col_header
    // ───────────────────────────────────────────────────

    #[test]
    fn col_header_hit() {
        let m = make_model();
        // vy=20 < header=40, vx=60 → col 0
        assert_eq!(hit_test_col_header(60.0, 20.0, &m, 0.0), Some(0));
    }

    #[test]
    fn col_header_below_header_returns_none() {
        let m = make_model();
        assert_eq!(hit_test_col_header(60.0, 50.0, &m, 0.0), None);
    }

    #[test]
    fn col_header_in_gutter_returns_none() {
        let m = make_model();
        assert_eq!(hit_test_col_header(30.0, 20.0, &m, 0.0), None);
    }

    // ── hit_test_filter_row_cell
    // ─────────────────────────────────────────

    #[test]
    fn filter_row_cell_hidden_by_default_returns_none() {
        let m = make_model();
        assert!(!m.show_filter_row);
        // Even at a plausible vy, the row doesn't exist yet.
        assert_eq!(hit_test_filter_row_cell(60.0, 50.0, &m, 0.0), None);
    }

    #[test]
    fn filter_row_cell_hit() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // Band is [40, 76) — vy=50, vx=60 → col 0.
        assert_eq!(hit_test_filter_row_cell(60.0, 50.0, &m, 0.0), Some(0));
    }

    #[test]
    fn filter_row_cell_below_band_returns_none() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // vy=80 is past the band (data starts at 76).
        assert_eq!(hit_test_filter_row_cell(60.0, 80.0, &m, 0.0), None);
    }

    #[test]
    fn filter_row_cell_above_band_returns_none() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        // vy=20 is still inside the header, above the filter row.
        assert_eq!(hit_test_filter_row_cell(60.0, 20.0, &m, 0.0), None);
    }

    #[test]
    fn filter_row_cell_in_gutter_returns_none() {
        let mut m = make_model();
        m.show_filter_row = true;
        m.filter_row_height = 36.0;
        assert_eq!(hit_test_filter_row_cell(30.0, 50.0, &m, 0.0), None);
    }

    // ── hit_test_row_header
    // ───────────────────────────────────────────────────

    #[test]
    fn row_header_hit_first() {
        let m = make_model();
        // vx=20 < rnw=50, vy=50 → abs_y=50 → row_y=10 → row 0
        assert_eq!(hit_test_row_header(20.0, 50.0, &m, 0.0), Some(0));
    }

    #[test]
    fn row_header_outside_gutter_returns_none() {
        let m = make_model();
        assert_eq!(hit_test_row_header(60.0, 50.0, &m, 0.0), None);
    }

    #[test]
    fn row_header_in_header_zone_returns_none() {
        let m = make_model();
        // vy=20 → abs_y=20 < header=40
        assert_eq!(hit_test_row_header(20.0, 20.0, &m, 0.0), None);
    }

    #[test]
    fn row_header_below_last_row_returns_none() {
        let m = make_model();
        // 5 rows; vy=200
        assert_eq!(hit_test_row_header(20.0, 200.0, &m, 0.0), None);
    }

    // ── hit_test_checkbox_row / hit_test_checkbox_header
    // ─────────────────────────

    fn make_checkbox_model() -> GridModel {
        let mut m = make_model();
        m.show_checkbox_column = true;
        m
    }

    // The checkbox column scrolls with the data (it's the first slot of
    // the scrollable/unpinned region, not a fixed gutter) — its viewport
    // band is `[rnw + pinned_width - scroll_x, .. + ccw)`. With no pinned
    // columns and scroll_x=0 that's `[rnw, rnw+ccw)`, same as the old
    // fixed-gutter geometry; the scroll-dependence is covered by
    // `checkbox_row_scrolls_with_data` below.

    #[test]
    fn checkbox_row_hit_first() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        // Just right of the row-number gutter, still left of data columns.
        assert_eq!(
            hit_test_checkbox_row(rnw + 5.0, 50.0, &m, 0.0, 0.0),
            Some(0)
        );
    }

    #[test]
    fn checkbox_row_disabled_returns_none() {
        let m = make_model(); // show_checkbox_column defaults to false
        let rnw = m.row_number_width;
        assert_eq!(hit_test_checkbox_row(rnw + 5.0, 50.0, &m, 0.0, 0.0), None);
    }

    #[test]
    fn checkbox_row_outside_band_returns_none() {
        let m = make_checkbox_model();
        // Inside the row-number gutter, not the checkbox band.
        assert_eq!(hit_test_checkbox_row(20.0, 50.0, &m, 0.0, 0.0), None);
        // Past the checkbox band, into data columns.
        let rnw = m.row_number_width;
        let ccw = GridModel::CHECKBOX_COLUMN_WIDTH;
        assert_eq!(
            hit_test_checkbox_row(rnw + ccw + 5.0, 50.0, &m, 0.0, 0.0),
            None
        );
    }

    #[test]
    fn checkbox_row_scrolls_with_data() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        let ccw = GridModel::CHECKBOX_COLUMN_WIDTH;
        // Scrolled past the checkbox column entirely → hidden.
        assert_eq!(hit_test_checkbox_row(rnw + 5.0, 50.0, &m, ccw, 0.0), None);
        // Partially scrolled: a click at the checkbox's new (shifted-left)
        // on-screen position still hits it, just like a real unpinned
        // column would.
        assert_eq!(
            hit_test_checkbox_row(rnw + 1.0, 50.0, &m, 10.0, 0.0),
            Some(0)
        );
    }

    #[test]
    fn checkbox_header_hit() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        assert!(hit_test_checkbox_header(rnw + 5.0, 20.0, &m, 0.0));
    }

    #[test]
    fn checkbox_header_below_header_returns_none() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        assert!(!hit_test_checkbox_header(rnw + 5.0, 50.0, &m, 0.0));
    }

    #[test]
    fn data_hit_test_skips_checkbox_column() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        let ccw = GridModel::CHECKBOX_COLUMN_WIDTH;
        // Still inside the checkbox band → not a data cell.
        assert!(hit_test(rnw + ccw - 1.0, 50.0, &m, 0.0, 0.0).is_none());
        // Just past it → col 0.
        let c = hit_test(rnw + ccw + 10.0, 50.0, &m, 0.0, 0.0).unwrap();
        assert_eq!(c.col, 0);
    }

    #[test]
    fn data_hit_test_reveals_first_column_once_checkbox_scrolled_away() {
        let m = make_checkbox_model();
        let rnw = m.row_number_width;
        let ccw = GridModel::CHECKBOX_COLUMN_WIDTH;
        // scroll_x == ccw scrolls the checkbox fully out of view — a
        // click right at the gutter edge should now land on column 0.
        let c = hit_test(rnw + 5.0, 50.0, &m, ccw, 0.0).unwrap();
        assert_eq!(c.col, 0);
    }

    // ── pinned column hit tests ──────────────────────────────────────────────

    fn make_pinned_model() -> GridModel {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let rows = (0..10).map(RowRecord::new).collect();
        let mut m = GridModel::new(cols, rows, 30.0, 40.0);
        m.pinned_count = 1; // pin column "a" (100px)
        m
    }

    #[test]
    fn hit_pinned_col_ignores_scroll_x() {
        let m = make_pinned_model();
        let rnw = m.row_number_width;
        // Click in pinned zone (vx < rnw + pinned_width=100),
        // with scroll_x=500 — pinned column is unaffected.
        let c = hit_test(rnw + 10.0, 50.0, &m, 500.0, 0.0).unwrap();
        assert_eq!(c.col, 0, "should hit pinned col 0");
    }

    #[test]
    fn hit_col_header_pinned_ignores_scroll_x() {
        let m = make_pinned_model();
        let rnw = m.row_number_width;
        // Click in header zone on pinned col with scroll_x=500
        let col = hit_test_col_header(rnw + 10.0, 20.0, &m, 500.0);
        assert_eq!(col, Some(0));
    }

    // ── scroll_y >= hh decomposition path ────────────────

    #[test]
    fn hit_with_large_scroll_y() {
        let m = make_model();
        // scroll_y=100 > header_height=40 → triggers the precision-preserving
        // path sy_content = 100-40=60, first_row = 60/30=2
        // frac = 60%30=0
        // vy=50 → offset = (50+0)/30 = 1 → row = 2+1 = 3
        let c = hit_test(60.0, 50.0, &m, 0.0, 100.0).unwrap();
        assert_eq!(c.row, 3);
    }

    #[test]
    fn row_header_with_large_scroll_y() {
        let m = make_model();
        // scroll_y=100 > header_height=40 → precision path
        let row = hit_test_row_header(20.0, 50.0, &m, 100.0);
        assert_eq!(row, Some(3));
    }
}

// ── O(log n) invariant ───────────────────────────────────────────────────────
//
// Executable form of the AGENTS.md invariant "hit-testing stays O(log n) via
// precomputed offsets". Hit-test cost must be independent of the row count
// (it is O(log n_cols), not O(n_rows)). This catches a regression to an
// O(n_rows) scan, which the prose rule alone cannot enforce.
#[cfg(test)]
mod complexity_invariant {
    use std::{hint::black_box, time::Instant};

    use super::*;
    use crate::{
        column::ColumnDef, datasource::FnDataSource, model::GridModel,
    };

    fn model_with_rows(n_cols: usize, n_rows: u64) -> GridModel {
        let cols = (0..n_cols)
            .map(|i| ColumnDef::new(format!("c{i}"), format!("C{i}"), 100.0))
            .collect();
        let data = Box::new(FnDataSource::new(n_rows, |_, _| None));
        GridModel::with_data_source(cols, data, 30.0, 40.0)
    }

    /// Minimum wall-clock (ns) of `iters` hit-tests at a mid-scroll position,
    /// taken over `repeats` runs. Using the minimum filters out scheduler
    /// noise — noise can only ever *add* time, so the floor is the most
    /// stable signal available without a hardware op counter.
    fn min_hit_test_ns(n_rows: u64, iters: u32, repeats: u32) -> u128 {
        let model = model_with_rows(1_000, n_rows);
        let rnw = model.effective_row_number_width();
        let vx = rnw + 400.0;
        let vy = model.effective_header_height() + 15.0;
        let scroll_x = model.total_width() / 2.0;
        let scroll_y = (n_rows / 2) as f64 * 30.0;
        let mut best = u128::MAX;
        for _ in 0..repeats {
            let start = Instant::now();
            for _ in 0..iters {
                black_box(hit_test(
                    black_box(vx),
                    black_box(vy),
                    &model,
                    black_box(scroll_x),
                    black_box(scroll_y),
                ));
            }
            best = best.min(start.elapsed().as_nanos());
        }
        best
    }

    /// 1 K vs 1 quadrillion rows: under O(n_rows) the gap would be ~10^12; a
    /// 20× ceiling cleanly separates O(log n) from any linear regression while
    /// tolerating CI timing noise.
    #[test]
    fn hit_test_cost_is_independent_of_row_count() {
        const ITERS: u32 = 200_000;
        const REPEATS: u32 = 5;
        let small = min_hit_test_ns(1_000, ITERS, REPEATS);
        let huge = min_hit_test_ns(1_000_000_000_000_000, ITERS, REPEATS);
        let ceiling = small.saturating_mul(20).max(small + 1_000);
        assert!(
            huge < ceiling,
            "hit-test scaled with row count: 1k={small}ns, 1Q={huge}ns \
             (ceiling {ceiling}ns) — likely an O(n_rows) regression",
        );
    }
}
