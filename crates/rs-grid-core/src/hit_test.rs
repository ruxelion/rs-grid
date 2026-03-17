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
    // Data columns are shifted right by rnw, so subtract rnw before adding scroll.
    let abs_x = (vx - rnw) + scroll_x;
    let abs_y = vy + scroll_y;

    // Header zone — not a data cell.
    if abs_y < model.header_height {
        return None;
    }

    // Row index.
    let row_y = abs_y - model.header_height;
    let row = (row_y / model.row_height) as u64;
    if row >= model.data.row_count() {
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
    let abs_x = (vx - rnw) + scroll_x;
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

    let abs_y = vy + scroll_y;
    if abs_y < model.header_height {
        return None;
    }

    let row_y = abs_y - model.header_height;
    let row = (row_y / model.row_height) as u64;
    if row >= model.data.row_count() {
        return None;
    }

    Some(row)
}
