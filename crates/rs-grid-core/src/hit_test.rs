use crate::{model::GridModel, selection::CellCoord};

/// Convert a pointer position in *viewport* space (logical pixels, top-left origin)
/// into a `CellCoord`, accounting for the current scroll offset.
///
/// Returns `None` when:
/// - The pointer is over the header.
/// - The pointer is below the last row.
/// - The pointer is to the right of the last column.
pub fn hit_test(
    vx: f64,
    vy: f64,
    model: &GridModel,
    scroll_x: f64,
    scroll_y: f64,
) -> Option<CellCoord> {
    // Convert viewport coords to content (absolute) coords.
    let abs_x = vx + scroll_x;
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
