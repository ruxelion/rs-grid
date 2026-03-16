/// Definition of a single grid column.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Unique key used to look up cell values in a row.
    pub key: String,
    /// Display label shown in the column header.
    pub label: String,
    /// Width in logical (CSS) pixels.
    pub width: f64,
}

impl ColumnDef {
    pub fn new(key: impl Into<String>, label: impl Into<String>, width: f64) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            width,
        }
    }
}

/// Precomputed left-edge offsets for every column, plus total content width.
#[derive(Debug, Clone, Default)]
pub struct ColumnOffsets {
    /// `offsets[i]` is the x position of the left edge of column `i`.
    pub offsets: Vec<f64>,
    pub total_width: f64,
}

impl ColumnOffsets {
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
    pub fn hit_column(&self, x: f64, columns: &[ColumnDef]) -> Option<usize> {
        for (i, &offset) in self.offsets.iter().enumerate() {
            let right = offset + columns[i].width;
            if x >= offset && x < right {
                return Some(i);
            }
        }
        None
    }
}
