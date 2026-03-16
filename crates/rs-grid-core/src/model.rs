use crate::{
    column::{ColumnDef, ColumnOffsets},
    datasource::{DataSource, VecDataSource},
    row::RowRecord,
};

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
        Self { columns, data, row_height, header_height, column_offsets }
    }

    /// Total scrollable height (header + all rows).
    pub fn total_height(&self) -> f64 {
        self.header_height + self.data.row_count() as f64 * self.row_height
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
