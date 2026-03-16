use crate::{
    column::{ColumnDef, ColumnOffsets},
    row::RowRecord,
};

/// The data model: columns, rows, and sizing constants.
#[derive(Debug, Clone)]
pub struct GridModel {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<RowRecord>,
    /// Height of every data row in logical pixels.
    pub row_height: f64,
    /// Height of the sticky header row in logical pixels.
    pub header_height: f64,
    /// Precomputed column offsets (recomputed when columns change).
    pub column_offsets: ColumnOffsets,
}

impl GridModel {
    pub fn new(
        columns: Vec<ColumnDef>,
        rows: Vec<RowRecord>,
        row_height: f64,
        header_height: f64,
    ) -> Self {
        let column_offsets = ColumnOffsets::compute(&columns);
        Self {
            columns,
            rows,
            row_height,
            header_height,
            column_offsets,
        }
    }

    /// Total scrollable height (header + all rows).
    pub fn total_height(&self) -> f64 {
        self.header_height + self.rows.len() as f64 * self.row_height
    }

    /// Total scrollable width.
    pub fn total_width(&self) -> f64 {
        self.column_offsets.total_width
    }

    /// Y position of the top edge of a data row (in content space, before scroll offset).
    pub fn row_top(&self, row_index: usize) -> f64 {
        self.header_height + row_index as f64 * self.row_height
    }

    /// Rebuild column offsets after columns are mutated.
    pub fn rebuild_offsets(&mut self) {
        self.column_offsets = ColumnOffsets::compute(&self.columns);
    }
}
