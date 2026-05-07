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
    ///
    /// Uses binary search on sorted offsets for O(log n).
    pub fn visible_columns(
        &self,
        offsets: &ColumnOffsets,
        col_widths: &[f64],
    ) -> (usize, usize) {
        let col_count = offsets.offsets.len();
        if col_count == 0 {
            return (0, 0);
        }
        let x_start = self.scroll_x;
        let x_end = self.scroll_x + self.width;

        // First visible: the last column whose offset <= x_start,
        // then scan back to find one whose right edge > x_start.
        let mut first = offsets.offsets.partition_point(|&o| o <= x_start);
        // partition_point gives the first offset > x_start; go back one
        // since that column may still overlap the viewport.
        first = first.saturating_sub(1);
        // Advance past columns fully to the left of the viewport.
        while first < col_count
            && offsets.offsets[first] + col_widths[first] <= x_start
        {
            first += 1;
        }

        // Last visible: first column whose offset >= x_end.
        let last = offsets.offsets[first..col_count]
            .partition_point(|&o| o < x_end)
            + first;

        (first.min(col_count), last.min(col_count))
    }

    /// Visible column range for the scrollable (non-pinned) area.
    ///
    /// Columns `0..pinned_count` are always visible and excluded
    /// from this range.  `pinned_width` is the sum of their widths,
    /// `row_number_width` is the gutter width; both are subtracted
    /// from the viewport to obtain the scrollable band.
    ///
    /// Uses binary search on sorted offsets for O(log n).
    pub fn visible_scrollable_columns(
        &self,
        offsets: &ColumnOffsets,
        col_widths: &[f64],
        pinned_count: usize,
        pinned_width: f64,
        row_number_width: f64,
    ) -> (usize, usize) {
        let col_count = offsets.offsets.len();
        if pinned_count >= col_count {
            return (col_count, col_count);
        }
        let avail = (self.width - row_number_width - pinned_width).max(0.0);
        let x_start = pinned_width + self.scroll_x;
        let x_end = x_start + avail;

        let slice = &offsets.offsets[pinned_count..col_count];

        // First visible in the scrollable slice.
        let mut rel_first = slice.partition_point(|&o| o <= x_start);
        rel_first = rel_first.saturating_sub(1);
        while rel_first < slice.len()
            && slice[rel_first] + col_widths[pinned_count + rel_first]
                <= x_start
        {
            rel_first += 1;
        }

        // Last visible.
        let rel_last =
            slice[rel_first..].partition_point(|&o| o < x_end) + rel_first;

        let first = (pinned_count + rel_first).min(col_count);
        let last = (pinned_count + rel_last).min(col_count);
        (first, last)
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

    // ── visible_scrollable_columns ───────────────────────────────────────────

    #[test]
    fn scrollable_columns_no_pinned() {
        let vp = ViewportState::new(800.0, 600.0);
        let (offsets, widths) = make_offsets();
        let (first, last) =
            vp.visible_scrollable_columns(&offsets, &widths, 0, 0.0, 40.0);
        assert_eq!(first, 0);
        assert_eq!(last, 3);
    }

    #[test]
    fn scrollable_columns_with_pinned() {
        // 5 columns of 100px each = 500px total.
        let cols: Vec<ColumnDef> = (0..5)
            .map(|i| ColumnDef::new(&format!("c{i}"), &format!("C{i}"), 100.0))
            .collect();
        let widths: Vec<f64> = cols.iter().map(|c| c.width).collect();
        let offsets = ColumnOffsets::compute(&cols);
        // viewport=350, gutter=40, pinned=1 col (100px).
        // avail = 350 - 40 - 100 = 210, shows c1..c3
        let vp = ViewportState::new(350.0, 600.0);
        let (first, last) =
            vp.visible_scrollable_columns(&offsets, &widths, 1, 100.0, 40.0);
        assert!(first >= 1, "pinned col excluded, first={first}");
        assert!(last <= 5);
    }

    #[test]
    fn scrollable_columns_pinned_fills_viewport() {
        // pinned_width > viewport → avail = 0, no scrollable columns
        let cols = vec![
            ColumnDef::new("a", "A", 200.0),
            ColumnDef::new("b", "B", 200.0),
            ColumnDef::new("c", "C", 100.0),
        ];
        let widths: Vec<f64> = cols.iter().map(|c| c.width).collect();
        let offsets = ColumnOffsets::compute(&cols);
        // viewport=300, gutter=40, pinned=2 cols (400px).
        // avail = (300 - 40 - 400).max(0) = 0
        let vp = ViewportState::new(300.0, 600.0);
        let (first, last) =
            vp.visible_scrollable_columns(&offsets, &widths, 2, 400.0, 40.0);
        assert_eq!(first, last, "no room for scrollable columns");
    }

    #[test]
    fn visible_columns_empty_returns_zero_zero() {
        let vp = ViewportState::new(800.0, 600.0);
        let offsets = ColumnOffsets::compute(&[]);
        let widths: Vec<f64> = vec![];
        assert_eq!(vp.visible_columns(&offsets, &widths), (0, 0));
    }

    #[test]
    fn visible_scrollable_all_pinned() {
        // pinned_count >= col_count → returns (col_count, col_count)
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        let widths: Vec<f64> = cols.iter().map(|c| c.width).collect();
        let offsets = ColumnOffsets::compute(&cols);
        let vp = ViewportState::new(800.0, 600.0);
        let (first, last) =
            vp.visible_scrollable_columns(&offsets, &widths, 5, 200.0, 40.0);
        assert_eq!(first, 2);
        assert_eq!(last, 2);
    }

    #[test]
    fn visible_columns_scrolled_past_first_col() {
        // Scroll right so first column is fully off-screen
        // → the `first += 1` loop should advance
        let cols: Vec<ColumnDef> = (0..5)
            .map(|i| ColumnDef::new(&format!("c{i}"), &format!("C{i}"), 100.0))
            .collect();
        let widths: Vec<f64> = cols.iter().map(|c| c.width).collect();
        let offsets = ColumnOffsets::compute(&cols);
        // scroll_x=150 → col 0 (0..100) fully off, col 1 (100..200)
        // partially visible
        let vp = ViewportState {
            scroll_x: 150.0,
            ..ViewportState::new(200.0, 600.0)
        };
        let (first, _last) = vp.visible_columns(&offsets, &widths);
        assert_eq!(first, 1, "col 0 should be skipped");
    }

    #[test]
    fn visible_scrollable_columns_scrolled_past_first() {
        let cols: Vec<ColumnDef> = (0..10)
            .map(|i| ColumnDef::new(&format!("c{i}"), &format!("C{i}"), 100.0))
            .collect();
        let widths: Vec<f64> = cols.iter().map(|c| c.width).collect();
        let offsets = ColumnOffsets::compute(&cols);
        // pinned=1 (100px), scroll_x=250 → scrollable cols 1..9
        // col 1 at offset 100..200 is fully before 100+250=350
        // col 2 at offset 200..300 is partially before 350
        // col 3 at offset 300..400 visible
        let vp = ViewportState {
            scroll_x: 250.0,
            ..ViewportState::new(400.0, 600.0)
        };
        let (first, _last) =
            vp.visible_scrollable_columns(&offsets, &widths, 1, 100.0, 40.0);
        assert!(
            first >= 2,
            "scrollable cols before scroll should be skipped, first={first}"
        );
    }
}
