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
        let thumb_offset = (scroll_y / max_scroll * thumb_travel).clamp(0.0, thumb_travel);

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
    pub fn drag_to_scroll(&self, dy: f64, total_h: f64, viewport_h: f64, header_h: f64) -> f64 {
        let full_h = (viewport_h - header_h).max(0.0);
        let max_scroll = (total_h - full_h).max(1.0);
        let thumb_travel = (self.track_h - self.thumb_h).max(1.0);
        dy / thumb_travel * max_scroll
    }

    /// Absolute scroll-y for a click on the track at viewport-y `click_y`
    /// (centers the thumb under the cursor).
    pub fn track_click_scroll(&self, click_y: f64, total_h: f64, viewport_h: f64, header_h: f64) -> f64 {
        let full_h = (viewport_h - header_h).max(0.0);
        let max_scroll = (total_h - full_h).max(1.0);
        let thumb_travel = (self.track_h - self.thumb_h).max(1.0);
        let rel = (click_y - self.track_y - self.thumb_h / 2.0).clamp(0.0, thumb_travel);
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
        let thumb_offset = (scroll_x / max_scroll * thumb_travel).clamp(0.0, thumb_travel);

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
    pub fn drag_to_scroll(&self, dx: f64, total_w: f64, viewport_w: f64, gutter_w: f64, vsb_w: f64) -> f64 {
        let available_w = (viewport_w - gutter_w - vsb_w).max(0.0);
        let max_scroll = (total_w - available_w).max(1.0);
        let thumb_travel = (self.track_w - self.thumb_w).max(1.0);
        dx / thumb_travel * max_scroll
    }

    /// Absolute scroll-x for a click on the track at viewport-x `click_x`.
    pub fn track_click_scroll(&self, click_x: f64, total_w: f64, viewport_w: f64, gutter_w: f64, vsb_w: f64) -> f64 {
        let available_w = (viewport_w - gutter_w - vsb_w).max(0.0);
        let max_scroll = (total_w - available_w).max(1.0);
        let thumb_travel = (self.track_w - self.thumb_w).max(1.0);
        let rel = (click_x - self.track_x - self.thumb_w / 2.0).clamp(0.0, thumb_travel);
        rel / thumb_travel * max_scroll
    }
}
