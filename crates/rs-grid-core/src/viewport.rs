use crate::column::ColumnOffsets;

/// Current scroll position and canvas dimensions.
#[derive(Debug, Clone)]
pub struct ViewportState {
    /// Horizontal scroll offset in logical pixels.
    pub scroll_x: f64,
    /// Vertical scroll offset in logical pixels.
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
    /// Create a viewport with the given dimensions and
    /// default scroll/overscan values.
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
        row_count: u64,
        row_height: f64,
        header_height: f64,
    ) -> (u64, u64) {
        if row_count == 0 || row_height <= 0.0 {
            return (0, 0);
        }
        // how far into the data area the viewport is scrolled
        let content_y = (self.scroll_y - header_height).max(0.0);
        let first = (content_y / row_height) as u64;
        let first = first.saturating_sub(self.overscan as u64);

        let visible_height = (self.height - header_height).max(0.0);
        let last_raw =
            ((content_y + visible_height) / row_height).ceil() as u64;
        let last = last_raw.saturating_add(self.overscan as u64).min(row_count);

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
        // first = index of the first column whose right edge enters
        // the viewport; last = index of the first column that starts
        // past the viewport's right edge (exclusive upper bound).
        let mut first = 0usize;
        let mut last = col_count;

        for (i, (&offset, &w)) in
            offsets.offsets.iter().zip(col_widths.iter()).enumerate()
        {
            // Column ends before viewport starts → still fully hidden.
            if offset + w <= x_start {
                first = i + 1;
            }
            // Column starts past viewport end → everything after is hidden too.
            if offset >= x_end && last == col_count {
                last = i;
                break;
            }
        }

        (first.min(col_count), last.min(col_count))
    }

    /// Visible column range for the scrollable (non-pinned) area.
    ///
    /// Columns `0..pinned_count` are always visible and excluded
    /// from this range.  `pinned_width` is the sum of their widths,
    /// `row_number_width` is the gutter width; both are subtracted
    /// from the viewport to obtain the scrollable band.
    pub fn visible_scrollable_columns(
        &self,
        offsets: &ColumnOffsets,
        col_widths: &[f64],
        pinned_count: usize,
        pinned_width: f64,
        row_number_width: f64,
    ) -> (usize, usize) {
        let avail = (self.width - row_number_width - pinned_width).max(0.0);
        let x_start = pinned_width + self.scroll_x;
        let x_end = x_start + avail;

        let col_count = offsets.offsets.len();
        let mut first = pinned_count;
        let mut last = col_count;

        for (i, (&offset, &w)) in offsets.offsets[pinned_count..col_count]
            .iter()
            .zip(&col_widths[pinned_count..col_count])
            .enumerate()
        {
            // enumerate() yields 0-based indices into the slice;
            // add pinned_count to recover the absolute column index.
            let i = i + pinned_count;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{ColumnDef, ColumnOffsets};

    fn vp(scroll_y: f64) -> ViewportState {
        ViewportState {
            scroll_y,
            ..ViewportState::new(800.0, 600.0)
        }
    }

    // ── visible_rows ──────────────────────────────────────────────────────────

    #[test]
    fn visible_rows_empty_grid() {
        let vp = vp(0.0);
        assert_eq!(vp.visible_rows(0, 30.0, 40.0), (0, 0));
    }

    #[test]
    fn visible_rows_at_top() {
        // height=600, header=40, row_height=30, overscan=3
        // visible_height = 560, last_raw = ceil(560/30) = 19
        // last = min(19+3, 100) = 22
        let vp = vp(0.0);
        let (first, last) = vp.visible_rows(100, 30.0, 40.0);
        assert_eq!(first, 0); // saturating_sub overscan from 0 → 0
        assert_eq!(last, 22);
    }

    #[test]
    fn visible_rows_scrolled() {
        // scroll_y=300, content_y=(300-40).max(0)=260
        // first = (260/30) as u64 = 8, -overscan = 5
        // last_raw = ceil((260+560)/30) = ceil(27.33) = 28
        // last = min(28+3, 100) = 31
        let vp = vp(300.0);
        let (first, last) = vp.visible_rows(100, 30.0, 40.0);
        assert_eq!(first, 5);
        assert_eq!(last, 31);
    }

    #[test]
    fn visible_rows_clamped_to_row_count() {
        // scroll near the bottom — last must not exceed row_count
        let vp = ViewportState {
            scroll_y: 10_000.0,
            ..ViewportState::new(800.0, 600.0)
        };
        let (_, last) = vp.visible_rows(10, 30.0, 40.0);
        assert_eq!(last, 10);
    }

    // ── visible_columns ───────────────────────────────────────────────────────

    fn make_offsets() -> (ColumnOffsets, Vec<f64>) {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let widths = cols.iter().map(|c| c.width).collect();
        (ColumnOffsets::compute(&cols), widths)
    }

    #[test]
    fn visible_columns_all_visible() {
        let vp = ViewportState::new(800.0, 600.0); // width=800 > total 450
        let (offsets, widths) = make_offsets();
        assert_eq!(vp.visible_columns(&offsets, &widths), (0, 3));
    }

    #[test]
    fn visible_columns_scrolled_past_first() {
        let vp = ViewportState {
            scroll_x: 110.0, // past first col (0..100)
            ..ViewportState::new(400.0, 600.0)
        };
        let (offsets, widths) = make_offsets();
        let (first, _) = vp.visible_columns(&offsets, &widths);
        assert_eq!(first, 1);
    }
}
