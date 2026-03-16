use crate::column::ColumnOffsets;

/// Current scroll position and canvas dimensions.
#[derive(Debug, Clone)]
pub struct ViewportState {
    pub scroll_x: f64,
    pub scroll_y: f64,
    /// Canvas width in logical pixels.
    pub width: f64,
    /// Canvas height in logical pixels.
    pub height: f64,
    /// Extra rows rendered above and below the visible area to prevent flicker.
    pub overscan: usize,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            width: 800.0,
            height: 600.0,
            overscan: 3,
        }
    }
}

impl ViewportState {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Inclusive start / exclusive end of the visible row range (with overscan).
    pub fn visible_rows(
        &self,
        row_count: usize,
        row_height: f64,
        header_height: f64,
    ) -> (usize, usize) {
        if row_count == 0 || row_height <= 0.0 {
            return (0, 0);
        }
        // how far into the data area the viewport is scrolled
        let content_y = (self.scroll_y - header_height).max(0.0);
        let first = (content_y / row_height) as usize;
        let first = first.saturating_sub(self.overscan);

        let visible_height = (self.height - header_height).max(0.0);
        let last_raw = ((content_y + visible_height) / row_height).ceil() as usize;
        let last = last_raw.saturating_add(self.overscan).min(row_count);

        (first, last)
    }

    /// Inclusive start / exclusive end of the visible column range.
    pub fn visible_columns(
        &self,
        offsets: &ColumnOffsets,
        col_widths: &[f64],
    ) -> (usize, usize) {
        let x_start = self.scroll_x;
        let x_end = self.scroll_x + self.width;

        let col_count = offsets.offsets.len();
        let mut first = 0usize;
        let mut last = col_count;

        for (i, (&offset, &w)) in offsets.offsets.iter().zip(col_widths.iter()).enumerate() {
            if offset + w <= x_start {
                first = i + 1;
            }
            if offset >= x_end && last == col_count {
                last = i;
                break;
            }
        }

        (first.min(col_count), last.min(col_count))
    }
}
