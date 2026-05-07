const MIN_THUMB_H: f64 = 24.0;
const MIN_THUMB_W: f64 = 24.0;

/// Pre-computed geometry of the vertical scrollbar for one frame.
///
/// All coordinates are in viewport space (CSS / logical pixels).
#[derive(Debug, Clone)]
pub struct ScrollbarGeom {
    /// Left edge of the track (and buttons).
    pub track_x: f64,
    /// Track width.
    pub track_w: f64,

    // ── arrow buttons ─────────────────────────────────────────────────────────
    /// Top edge of the up-arrow button.
    pub up_btn_y: f64,
    /// Top edge of the down-arrow button.
    pub down_btn_y: f64,
    /// Height of each arrow button (= `track_w` → square).
    pub arrow_h: f64,

    // ── track (between the two buttons) ──────────────────────────────────────
    /// Top edge of the scrollable track area.
    pub track_y: f64,
    /// Height of the scrollable track area.
    pub track_h: f64,

    // ── thumb ─────────────────────────────────────────────────────────────────
    /// Top edge of the thumb.
    pub thumb_y: f64,
    /// Height of the thumb.
    pub thumb_h: f64,
}

impl ScrollbarGeom {
    /// Returns `None` when all content fits without scrolling.
    pub fn compute(
        scroll_y: f64,
        viewport_w: f64,
        viewport_h: f64,
        header_h: f64,
        total_h: f64,
        track_w: f64,
    ) -> Option<Self> {
        let full_h = (viewport_h - header_h).max(0.0);
        if total_h <= viewport_h {
            return None;
        }

        let arrow_h = track_w; // square buttons
        let track_x = viewport_w - track_w;
        let up_btn_y = header_h;
        let down_btn_y = viewport_h - arrow_h;

        let track_y = header_h + arrow_h;
        let track_h = (full_h - 2.0 * arrow_h).max(0.0);

        let thumb_ratio = (full_h / total_h).clamp(0.0, 1.0);
        let thumb_h = (thumb_ratio * track_h).max(MIN_THUMB_H).min(track_h);
        let max_scroll = (total_h - full_h).max(1.0);
        let thumb_travel = (track_h - thumb_h).max(0.0);
        let thumb_offset =
            (scroll_y / max_scroll * thumb_travel).clamp(0.0, thumb_travel);

        Some(Self {
            track_x,
            track_w,
            up_btn_y,
            down_btn_y,
            arrow_h,
            track_y,
            track_h,
            thumb_y: track_y + thumb_offset,
            thumb_h,
        })
    }

    /// `true` if (x, y) is on the up-arrow button.
    pub fn hit_up_arrow(&self, x: f64, y: f64) -> bool {
        x >= self.track_x
            && x <= self.track_x + self.track_w
            && y >= self.up_btn_y
            && y <= self.up_btn_y + self.arrow_h
    }

    /// `true` if (x, y) is on the down-arrow button.
    pub fn hit_down_arrow(&self, x: f64, y: f64) -> bool {
        x >= self.track_x
            && x <= self.track_x + self.track_w
            && y >= self.down_btn_y
            && y <= self.down_btn_y + self.arrow_h
    }

    /// `true` if (x, y) lands on the thumb.
    pub fn hit_thumb(&self, x: f64, y: f64) -> bool {
        x >= self.track_x
            && x <= self.track_x + self.track_w
            && y >= self.thumb_y
            && y <= self.thumb_y + self.thumb_h
    }

    /// `true` if (x, y) lands on the track (excluding buttons).
    pub fn hit_track(&self, x: f64, y: f64) -> bool {
        x >= self.track_x
            && x <= self.track_x + self.track_w
            && y >= self.track_y
            && y <= self.track_y + self.track_h
    }

    /// Scroll delta produced by dragging the thumb `dy` pixels.
    pub fn drag_to_scroll(
        &self,
        dy: f64,
        total_h: f64,
        viewport_h: f64,
        header_h: f64,
    ) -> f64 {
        let full_h = (viewport_h - header_h).max(0.0);
        let max_scroll = (total_h - full_h).max(1.0);
        let thumb_travel = (self.track_h - self.thumb_h).max(1.0);
        dy / thumb_travel * max_scroll
    }

    /// Absolute scroll-y for a click on the track at viewport-y `click_y`
    /// (centers the thumb under the cursor).
    pub fn track_click_scroll(
        &self,
        click_y: f64,
        total_h: f64,
        viewport_h: f64,
        header_h: f64,
    ) -> f64 {
        let full_h = (viewport_h - header_h).max(0.0);
        let max_scroll = (total_h - full_h).max(1.0);
        let thumb_travel = (self.track_h - self.thumb_h).max(1.0);
        let rel = (click_y - self.track_y - self.thumb_h / 2.0)
            .clamp(0.0, thumb_travel);
        rel / thumb_travel * max_scroll
    }
}

// ── horizontal scrollbar ──────────────────────────────────────────────────────

/// Pre-computed geometry of the horizontal scrollbar for one frame.
#[derive(Debug, Clone)]
pub struct HScrollbarGeom {
    /// Top edge of the track (and buttons).
    pub track_y: f64,
    /// Track height (= scrollbar_width from theme).
    pub track_h: f64,

    // ── arrow buttons ─────────────────────────────────────────────────────────
    /// Left edge of the left-arrow button.
    pub left_btn_x: f64,
    /// Left edge of the right-arrow button.
    pub right_btn_x: f64,
    /// Width of each arrow button (= `track_h` → square).
    pub arrow_w: f64,

    // ── track (between the two buttons) ──────────────────────────────────────
    /// Left edge of the scrollable track area.
    pub track_x: f64,
    /// Width of the scrollable track area.
    pub track_w: f64,

    // ── thumb ─────────────────────────────────────────────────────────────────
    /// Left edge of the thumb.
    pub thumb_x: f64,
    /// Width of the thumb.
    pub thumb_w: f64,
}

impl HScrollbarGeom {
    /// Returns `None` when all content fits without horizontal scrolling.
    ///
    /// `gutter_w` — width of the row-number gutter (left inset).
    /// `vsb_w`    — width of the vertical scrollbar (right inset), 0 if absent.
    pub fn compute(
        scroll_x: f64,
        viewport_w: f64,
        viewport_h: f64,
        gutter_w: f64,
        total_w: f64,
        vsb_w: f64,
        track_h: f64,
    ) -> Option<Self> {
        let available_w = (viewport_w - gutter_w - vsb_w).max(0.0);
        if total_w <= available_w {
            return None;
        }

        let arrow_w = track_h; // square buttons
        let track_y = viewport_h - track_h;
        let left_btn_x = gutter_w;
        let right_btn_x = viewport_w - vsb_w - arrow_w;

        let track_x = gutter_w + arrow_w;
        let track_w = (available_w - 2.0 * arrow_w).max(0.0);

        let thumb_ratio = (available_w / total_w).clamp(0.0, 1.0);
        let thumb_w = (thumb_ratio * track_w).max(MIN_THUMB_W).min(track_w);
        let max_scroll = (total_w - available_w).max(1.0);
        let thumb_travel = (track_w - thumb_w).max(0.0);
        let thumb_offset =
            (scroll_x / max_scroll * thumb_travel).clamp(0.0, thumb_travel);

        Some(Self {
            track_y,
            track_h,
            left_btn_x,
            right_btn_x,
            arrow_w,
            track_x,
            track_w,
            thumb_x: track_x + thumb_offset,
            thumb_w,
        })
    }

    /// `true` if (x, y) is on the left-arrow button.
    pub fn hit_left_arrow(&self, x: f64, y: f64) -> bool {
        y >= self.track_y
            && y <= self.track_y + self.track_h
            && x >= self.left_btn_x
            && x <= self.left_btn_x + self.arrow_w
    }

    /// `true` if (x, y) is on the right-arrow button.
    pub fn hit_right_arrow(&self, x: f64, y: f64) -> bool {
        y >= self.track_y
            && y <= self.track_y + self.track_h
            && x >= self.right_btn_x
            && x <= self.right_btn_x + self.arrow_w
    }

    /// `true` if (x, y) lands on the thumb.
    pub fn hit_thumb(&self, x: f64, y: f64) -> bool {
        y >= self.track_y
            && y <= self.track_y + self.track_h
            && x >= self.thumb_x
            && x <= self.thumb_x + self.thumb_w
    }

    /// `true` if (x, y) lands on the track (excluding buttons).
    pub fn hit_track(&self, x: f64, y: f64) -> bool {
        y >= self.track_y
            && y <= self.track_y + self.track_h
            && x >= self.track_x
            && x <= self.track_x + self.track_w
    }

    /// Scroll delta produced by dragging the thumb `dx` pixels.
    pub fn drag_to_scroll(
        &self,
        dx: f64,
        total_w: f64,
        viewport_w: f64,
        gutter_w: f64,
        vsb_w: f64,
    ) -> f64 {
        let available_w = (viewport_w - gutter_w - vsb_w).max(0.0);
        let max_scroll = (total_w - available_w).max(1.0);
        let thumb_travel = (self.track_w - self.thumb_w).max(1.0);
        dx / thumb_travel * max_scroll
    }

    /// Absolute scroll-x for a click on the track at viewport-x `click_x`.
    pub fn track_click_scroll(
        &self,
        click_x: f64,
        total_w: f64,
        viewport_w: f64,
        gutter_w: f64,
        vsb_w: f64,
    ) -> f64 {
        let available_w = (viewport_w - gutter_w - vsb_w).max(0.0);
        let max_scroll = (total_w - available_w).max(1.0);
        let thumb_travel = (self.track_w - self.thumb_w).max(1.0);
        let rel = (click_x - self.track_x - self.thumb_w / 2.0)
            .clamp(0.0, thumb_travel);
        rel / thumb_travel * max_scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ScrollbarGeom ─────────────────────────────────────────────────────────

    /// viewport 800×600, header=40, total_h=3000, track_w=16
    fn make_vscroll(scroll_y: f64) -> ScrollbarGeom {
        ScrollbarGeom::compute(scroll_y, 800.0, 600.0, 40.0, 3000.0, 16.0)
            .unwrap()
    }

    #[test]
    fn vscroll_none_when_content_fits() {
        // total_h=500 <= viewport_h=600 → None
        assert!(ScrollbarGeom::compute(0.0, 800.0, 600.0, 40.0, 500.0, 16.0)
            .is_none());
    }

    #[test]
    fn vscroll_geometry_at_top() {
        let g = make_vscroll(0.0);
        assert_eq!(g.track_x, 784.0); // 800 - 16
        assert_eq!(g.up_btn_y, 40.0); // header_h
        assert_eq!(g.down_btn_y, 584.0); // 600 - 16
        assert_eq!(g.track_y, 56.0); // header + arrow_h
        assert_eq!(g.thumb_y, g.track_y); // scroll=0 → thumb at top
    }

    #[test]
    fn vscroll_thumb_moves_with_scroll() {
        let g0 = make_vscroll(0.0);
        let g1 = make_vscroll(1000.0);
        assert!(g1.thumb_y > g0.thumb_y);
    }

    #[test]
    fn vscroll_hit_up_arrow() {
        let g = make_vscroll(0.0);
        assert!(g.hit_up_arrow(790.0, 50.0)); // inside
        assert!(!g.hit_up_arrow(790.0, 100.0)); // below arrow
        assert!(!g.hit_up_arrow(700.0, 50.0)); // wrong x
    }

    #[test]
    fn vscroll_hit_down_arrow() {
        let g = make_vscroll(0.0);
        assert!(g.hit_down_arrow(790.0, 590.0)); // inside
        assert!(!g.hit_down_arrow(790.0, 50.0)); // wrong y
    }

    #[test]
    fn vscroll_hit_thumb() {
        let g = make_vscroll(0.0);
        let mid = g.thumb_y + g.thumb_h / 2.0;
        assert!(g.hit_thumb(790.0, mid));
        assert!(!g.hit_thumb(700.0, mid)); // wrong x
    }

    #[test]
    fn vscroll_hit_track() {
        let g = make_vscroll(0.0);
        // Between up and down arrows, on the track
        let track_mid = g.track_y + g.track_h / 2.0;
        assert!(g.hit_track(790.0, track_mid));
        assert!(!g.hit_track(700.0, track_mid));
    }

    #[test]
    fn vscroll_drag_to_scroll_positive() {
        let g = make_vscroll(0.0);
        let delta = g.drag_to_scroll(10.0, 3000.0, 600.0, 40.0);
        assert!(delta > 0.0);
    }

    // ── HScrollbarGeom ────────────────────────────────────────────────────────

    /// viewport 800×600, gutter=50, total_w=2000, vsb_w=16, track_h=16
    fn make_hscroll(scroll_x: f64) -> HScrollbarGeom {
        HScrollbarGeom::compute(
            scroll_x, 800.0, 600.0, 50.0, 2000.0, 16.0, 16.0,
        )
        .unwrap()
    }

    #[test]
    fn hscroll_none_when_content_fits() {
        // total_w=500, available_w=800-50-16=734 → None
        assert!(HScrollbarGeom::compute(
            0.0, 800.0, 600.0, 50.0, 500.0, 16.0, 16.0
        )
        .is_none());
    }

    #[test]
    fn hscroll_geometry() {
        let g = make_hscroll(0.0);
        assert_eq!(g.track_y, 584.0); // 600 - 16
        assert_eq!(g.left_btn_x, 50.0); // gutter_w
        assert_eq!(g.right_btn_x, 784.0 - 16.0); // 800 - vsb_w - arrow_w
        assert_eq!(g.thumb_x, g.track_x); // scroll=0 → thumb at left
    }

    #[test]
    fn hscroll_thumb_moves_with_scroll() {
        let g0 = make_hscroll(0.0);
        let g1 = make_hscroll(500.0);
        assert!(g1.thumb_x > g0.thumb_x);
    }

    #[test]
    fn hscroll_hit_left_arrow() {
        let g = make_hscroll(0.0);
        assert!(g.hit_left_arrow(55.0, 590.0));
        assert!(!g.hit_left_arrow(10.0, 590.0)); // left of gutter
    }

    #[test]
    fn hscroll_drag_to_scroll_positive() {
        let g = make_hscroll(0.0);
        let delta = g.drag_to_scroll(10.0, 2000.0, 800.0, 50.0, 16.0);
        assert!(delta > 0.0);
    }

    #[test]
    fn vscroll_track_click_scroll() {
        let g = make_vscroll(0.0);
        // Click at top of track → scroll ≈ 0
        let s0 = g.track_click_scroll(g.track_y, 3000.0, 600.0, 40.0);
        assert!(s0 < 50.0, "click at track top → near zero scroll");
        // Click at bottom of track → scroll near max
        let max_scroll = 3000.0 - (600.0 - 40.0);
        let s1 =
            g.track_click_scroll(g.track_y + g.track_h, 3000.0, 600.0, 40.0);
        assert!(
            (s1 - max_scroll).abs() < 50.0,
            "click at track bottom → near max scroll"
        );
    }

    #[test]
    fn hscroll_hit_right_arrow() {
        let g = make_hscroll(0.0);
        let mid_y = g.track_y + g.track_h / 2.0;
        assert!(g.hit_right_arrow(g.right_btn_x + 5.0, mid_y));
        assert!(!g.hit_right_arrow(100.0, mid_y)); // wrong x
    }

    #[test]
    fn min_thumb_size_enforced() {
        // Very large content → thumb should still be >= MIN_THUMB_H
        let g =
            ScrollbarGeom::compute(0.0, 800.0, 600.0, 40.0, 100_000.0, 16.0)
                .unwrap();
        assert!(
            g.thumb_h >= MIN_THUMB_H,
            "thumb_h={} < MIN_THUMB_H={}",
            g.thumb_h,
            MIN_THUMB_H,
        );
    }

    // ── HScrollbarGeom — hit_thumb / hit_track / track_click ────────────────

    #[test]
    fn hscroll_hit_thumb() {
        let g = make_hscroll(0.0);
        let mid_y = g.track_y + g.track_h / 2.0;
        let mid_x = g.thumb_x + g.thumb_w / 2.0;
        assert!(g.hit_thumb(mid_x, mid_y));
        // Wrong y → outside track band
        assert!(!g.hit_thumb(mid_x, g.track_y - 5.0));
        // Wrong x → outside thumb
        assert!(!g.hit_thumb(g.track_x - 1.0, mid_y));
    }

    #[test]
    fn hscroll_hit_track() {
        let g = make_hscroll(0.0);
        let mid_y = g.track_y + g.track_h / 2.0;
        let mid_x = g.track_x + g.track_w / 2.0;
        assert!(g.hit_track(mid_x, mid_y));
        // Outside band
        assert!(!g.hit_track(mid_x, g.track_y - 1.0));
        // Left of track
        assert!(!g.hit_track(g.track_x - 1.0, mid_y));
    }

    #[test]
    fn hscroll_track_click_scroll() {
        let g = make_hscroll(0.0);
        // Click at left edge of track → near 0 scroll
        let s0 = g.track_click_scroll(g.track_x, 2000.0, 800.0, 50.0, 16.0);
        assert!(s0 < 50.0, "click at track left → near zero scroll");
        // Click at right edge → near max scroll
        let available_w = 800.0 - 50.0 - 16.0;
        let max_scroll = 2000.0 - available_w;
        let s1 = g.track_click_scroll(
            g.track_x + g.track_w,
            2000.0,
            800.0,
            50.0,
            16.0,
        );
        assert!(
            (s1 - max_scroll).abs() < 100.0,
            "click at track right → near max scroll"
        );
    }
}
