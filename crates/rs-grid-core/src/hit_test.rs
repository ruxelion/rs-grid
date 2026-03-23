use crate::{model::GridModel, selection::CellCoord};

/// Convert a pointer position in *viewport* space (logical pixels, top-left origin)
/// into a `CellCoord`, accounting for the current scroll offset and row-number gutter.
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
    // Gutter zone — not a data cell.
    let rnw = model.row_number_width;
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
        vx_data + scroll_x // scrollable zone: add scroll
    };
    // Header is sticky — always at vy 0..hh.
    if vy < model.header_height {
        return None;
    }

    // Row index — avoid large absolute coordinates to
    // preserve f64 precision at extreme row counts.
    // row = floor((vy - hh + scroll_y) / rh)
    // When scroll_y >= hh, decompose to keep numbers small.
    let hh = model.header_height;
    let rh = model.row_height;
    let row = if scroll_y >= hh {
        let sy_content = scroll_y - hh;
        let first_row = (sy_content / rh) as u64;
        // Use fmod to avoid subtracting two large f64s.
        let frac = sy_content % rh;
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

    // Column index using precomputed offsets.
    let col = model.column_offsets.hit_column(abs_x, &model.columns)?;

    Some(CellCoord { row, col })
}

/// Returns the column index when the pointer is over a column header.
///
/// Returns `None` when the pointer is outside the header zone or in the
/// row-number corner (x < row_number_width).
pub fn hit_test_col_header(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
) -> Option<usize> {
    let rnw = model.row_number_width;
    // Must be in header row and to the right of the row-number gutter corner.
    if vy >= model.header_height || vx < rnw {
        return None;
    }
    let vx_data = vx - rnw;
    let pinned_width = model.pinned_width();
    let abs_x = if vx_data < pinned_width {
        vx_data
    } else {
        vx_data + scroll_x
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
    let rnw = model.row_number_width;
    if rnw <= 0.0 || vx >= rnw {
        return None;
    }

    let hh = model.header_height;
    let rh = model.row_height;
    if vy < hh {
        return None;
    }

    // Same precision-preserving decomposition as in hit_test():
    // decompose scroll_y to avoid subtracting two large f64s.
    let row = if scroll_y >= hh {
        let sy_content = scroll_y - hh;
        let first_row = (sy_content / rh) as u64;
        let frac = sy_content % rh; // sub-row offset within first_row
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
        let rows = (0..5).map(|i| RowRecord::new(i)).collect();
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    // ── hit_test (data cells) ─────────────────────────────────────────────────

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

    // ── hit_test_col_header ───────────────────────────────────────────────────

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

    // ── hit_test_row_header ───────────────────────────────────────────────────

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
}
