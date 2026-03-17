const MIN_THUMB_H: f64 = 24.0;

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
