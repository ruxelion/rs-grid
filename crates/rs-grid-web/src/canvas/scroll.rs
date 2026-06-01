use rs_grid_core::{commands::GridCommand, scrollbar::ScrollbarGeom};
use wasm_bindgen::{prelude::Closure, JsCast};

use super::GridCanvas;

/// Discriminates the two track-scroll axes.
#[derive(Copy, Clone)]
enum ScrollAxis {
    Vertical { click_y: f64, going_down: bool },
    Horizontal { click_x: f64, going_right: bool },
}

impl GridCanvas {
    // ── arrow-button auto-scroll ─────────────────────────────

    /// Start a repeating scroll (arrows): immediate first
    /// scroll on mousedown, then ~350 ms pause, then repeat
    /// every 60 ms.
    pub(super) fn start_scroll_repeat(&self, dy: f64) {
        self.start_arrow_repeat(0.0, dy);
    }

    pub(super) fn pan_cursor() -> &'static str {
        "all-scroll"
    }

    pub(super) fn stop_pan(&self) {
        self.stop_scroll_repeat();
        *self.0.drag.borrow_mut() = None;
        let _ = self.0.canvas.style().set_property("cursor", "");
    }

    pub(super) fn stop_scroll_repeat(&self) {
        let win = web_sys::window().expect("no window");
        if let Some(id) = self.0.scroll_timeout.borrow_mut().take() {
            win.clear_timeout_with_handle(id);
        }
        if let Some(id) = self.0.scroll_interval.borrow_mut().take() {
            win.clear_interval_with_handle(id);
        }
        // Drop timer closures (breaks Rc cycle).
        self.0.scroll_closures.borrow_mut().clear();
    }

    /// Animate scroll toward `click_y` (AG Grid style):
    /// 1. Mini easing at ~100 ms interval (slow start)
    /// 2. At 350 ms: switch to full 60 fps easing
    pub(super) fn start_track_scroll_repeat(
        &self,
        click_y: f64,
        going_down: bool,
    ) {
        self.start_track_repeat(ScrollAxis::Vertical {
            click_y,
            going_down,
        });
    }

    /// Single easing step toward `click_y`.
    fn do_track_scroll_step(&self, click_y: f64, going_down: bool) {
        let Some(sb) = self.scrollbar() else {
            self.stop_scroll_repeat();
            return;
        };

        let thumb_center = sb.thumb_y + sb.thumb_h * 0.5;
        let still_going = if going_down {
            thumb_center < click_y
        } else {
            thumb_center > click_y
        };
        if !still_going {
            self.stop_scroll_repeat();
            return;
        }

        let (total_h, viewport_h, header_h) = {
            let s = self.0.state.borrow();
            (
                s.model.total_height(),
                s.viewport.height,
                s.model.header_height,
            )
        };
        let target =
            sb.track_click_scroll(click_y, total_h, viewport_h, header_h);
        let current = self.0.state.borrow().viewport.scroll_y;
        let remaining = target - current;

        if remaining.abs() < 1.0 {
            self.stop_scroll_repeat();
            return;
        }

        let step = if remaining > 0.0 {
            (remaining * 0.10).max(1.0)
        } else {
            (remaining * 0.10).min(-1.0)
        };
        self.dispatch(GridCommand::ScrollBy { dx: 0.0, dy: step });
    }

    // ── horizontal track scroll ──────────────────────────────

    /// Arrow-button auto-repeat for horizontal axis.
    pub(super) fn start_scroll_repeat_x(&self, dx: f64) {
        self.start_arrow_repeat(dx, 0.0);
    }

    /// Single easing step toward `click_x`.
    fn do_htrack_scroll_step(&self, click_x: f64, going_right: bool) {
        let Some(sb) = self.hscrollbar() else {
            self.stop_scroll_repeat();
            return;
        };

        let thumb_center = sb.thumb_x + sb.thumb_w * 0.5;
        let still_going = if going_right {
            thumb_center < click_x
        } else {
            thumb_center > click_x
        };
        if !still_going {
            self.stop_scroll_repeat();
            return;
        }

        let (total_w, viewport_w, gutter_w, vsb_w) = {
            let s = self.0.state.borrow();
            let sb_w = self.0.builder.borrow().theme.scrollbar_width;
            let vsb_w = if ScrollbarGeom::compute(
                s.viewport.scroll_y,
                s.viewport.width,
                s.viewport.height,
                s.model.header_height,
                s.model.total_height(),
                sb_w,
            )
            .is_some()
            {
                sb_w
            } else {
                0.0
            };
            (
                s.model.total_width(),
                s.viewport.width,
                s.model.row_number_width,
                vsb_w,
            )
        };
        let target = sb
            .track_click_scroll(click_x, total_w, viewport_w, gutter_w, vsb_w);
        let current = self.0.state.borrow().viewport.scroll_x;
        let remaining = target - current;

        if remaining.abs() < 1.0 {
            self.stop_scroll_repeat();
            return;
        }

        let step = if remaining > 0.0 {
            (remaining * 0.10).max(1.0)
        } else {
            (remaining * 0.10).min(-1.0)
        };
        self.dispatch(GridCommand::ScrollBy { dx: step, dy: 0.0 });
    }

    /// AG Grid-style three-phase track scroll for
    /// horizontal.
    pub(super) fn start_htrack_scroll_repeat(
        &self,
        click_x: f64,
        going_right: bool,
    ) {
        self.start_track_repeat(ScrollAxis::Horizontal {
            click_x,
            going_right,
        });
    }

    // ── private helpers ──────────────────────────────────────

    /// Shared timeout→interval logic for arrow-button
    /// auto-repeat: fires the scroll command immediately, then
    /// pauses 350 ms, then repeats every 60 ms.
    fn start_arrow_repeat(&self, dx: f64, dy: f64) {
        self.stop_scroll_repeat();
        let gc = self.clone();
        let win = web_sys::window().expect("no window");
        let timeout_cb = Closure::<dyn FnMut()>::new(move || {
            let gc2 = gc.clone();
            let interval_cb = Closure::<dyn FnMut()>::new(move || {
                gc2.dispatch(GridCommand::ScrollBy { dx, dy });
            });
            let win2 = web_sys::window().expect("no window");
            let id = win2
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    interval_cb.as_ref().unchecked_ref(),
                    60,
                )
                .expect("setInterval");
            *gc.0.scroll_interval.borrow_mut() = Some(id);
            gc.0.scroll_closures
                .borrow_mut()
                .push(Box::new(interval_cb));
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
        self.0
            .scroll_closures
            .borrow_mut()
            .push(Box::new(timeout_cb));
    }

    /// Shared two-phase easing for track-click auto-scroll:
    /// slow interval (100 ms) for the first 350 ms, then
    /// fast interval (16 ms) until the thumb reaches the target.
    fn start_track_repeat(&self, axis: ScrollAxis) {
        self.stop_scroll_repeat();
        let win = web_sys::window().expect("no window");

        // Phase 1: slow mini-easing (few steps, 100 ms).
        let gc1 = self.clone();
        let slow_cb = Closure::<dyn FnMut()>::new(move || match axis {
            ScrollAxis::Vertical {
                click_y,
                going_down,
            } => gc1.do_track_scroll_step(click_y, going_down),
            ScrollAxis::Horizontal {
                click_x,
                going_right,
            } => gc1.do_htrack_scroll_step(click_x, going_right),
        });
        let slow_id = win
            .set_interval_with_callback_and_timeout_and_arguments_0(
                slow_cb.as_ref().unchecked_ref(),
                100,
            )
            .expect("setInterval");
        *self.0.scroll_interval.borrow_mut() = Some(slow_id);
        self.0.scroll_closures.borrow_mut().push(Box::new(slow_cb));

        // Phase 2: after 350 ms, switch to full 60 fps.
        let gc2 = self.clone();
        let switch_cb = Closure::<dyn FnMut()>::new(move || {
            if let Some(id) = gc2.0.scroll_interval.borrow_mut().take() {
                web_sys::window()
                    .expect("no window")
                    .clear_interval_with_handle(id);
            }
            let gc3 = gc2.clone();
            let fast_cb = Closure::<dyn FnMut()>::new(move || match axis {
                ScrollAxis::Vertical {
                    click_y,
                    going_down,
                } => gc3.do_track_scroll_step(click_y, going_down),
                ScrollAxis::Horizontal {
                    click_x,
                    going_right,
                } => gc3.do_htrack_scroll_step(click_x, going_right),
            });
            let win2 = web_sys::window().expect("no window");
            let fast_id = win2
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    fast_cb.as_ref().unchecked_ref(),
                    16,
                )
                .expect("setInterval");
            *gc2.0.scroll_interval.borrow_mut() = Some(fast_id);
            gc2.0.scroll_closures.borrow_mut().push(Box::new(fast_cb));
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                switch_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
        self.0
            .scroll_closures
            .borrow_mut()
            .push(Box::new(switch_cb));
    }
}

// ── cell-drag edge auto-scroll ───────────────────────────────────────────────

/// Distance from the viewport edge that activates auto-scroll (px).
const DRAG_EDGE_ZONE: f64 = 50.0;
/// Maximum scroll step per 16 ms tick at full acceleration (px).
const DRAG_MAX_STEP: f64 = 40.0;
/// Interval between scroll ticks (ms).
const DRAG_INTERVAL_MS: i32 = 16;
/// Ticks to reach full speed (~2 s at 16 ms/tick).
const DRAG_ACCEL_TICKS: f64 = 120.0;

/// Returns the raw depth factor (0.0 = zone entry, 1.0 = absolute edge)
/// for one axis, or `None` if outside the edge zone.
fn edge_depth(pos: f64, lo: f64, hi: f64) -> Option<f64> {
    if pos < lo + DRAG_EDGE_ZONE {
        Some(((lo + DRAG_EDGE_ZONE - pos) / DRAG_EDGE_ZONE).clamp(0.0, 1.0))
    } else if pos > hi - DRAG_EDGE_ZONE {
        Some(((pos - (hi - DRAG_EDGE_ZONE)) / DRAG_EDGE_ZONE).clamp(0.0, 1.0))
    } else {
        None
    }
}

/// Signed scroll velocity combining:
/// - **position**: cubic depth³ (faster near the edge),
/// - **time**:     linear ramp over DRAG_ACCEL_TICKS (faster the longer
///                 the cursor stays in the zone — AG Grid behaviour).
fn edge_velocity(pos: f64, lo: f64, hi: f64, ticks: u32) -> f64 {
    // Start at 30% and climb to 100% over ~2 s.
    let time_factor =
        0.3 + 0.7 * (ticks as f64 / DRAG_ACCEL_TICKS).clamp(0.0, 1.0);

    if pos < lo + DRAG_EDGE_ZONE {
        let d = ((lo + DRAG_EDGE_ZONE - pos) / DRAG_EDGE_ZONE).clamp(0.0, 1.0);
        -DRAG_MAX_STEP * d * d * time_factor
    } else if pos > hi - DRAG_EDGE_ZONE {
        let d =
            ((pos - (hi - DRAG_EDGE_ZONE)) / DRAG_EDGE_ZONE).clamp(0.0, 1.0);
        DRAG_MAX_STEP * d * d * time_factor
    } else {
        0.0
    }
}

impl GridCanvas {
    /// Called on every mousemove while `ActiveDrag::Cell` is active.
    /// Starts, updates, or stops the edge auto-scroll interval.
    pub(super) fn update_cell_drag_scroll(&self, vx: f64, vy: f64) {
        let (vp_w, vp_h, header_h) = {
            let s = self.0.state.borrow();
            (s.viewport.width, s.viewport.height, s.model.header_height)
        };
        let in_zone = edge_depth(vy, header_h, vp_h).is_some()
            || edge_depth(vx, 0.0, vp_w).is_some();

        if !in_zone {
            // Cursor left the edge zone: stop scroll and reset acceleration.
            if self.0.drag_scroll_interval.borrow().is_some() {
                self.stop_cell_drag_scroll();
                self.0.drag_scroll_ticks.set(0);
            }
            return;
        }
        // Already running — interval picks up the new last_pos automatically.
        if self.0.drag_scroll_interval.borrow().is_some() {
            return;
        }
        // Entering the zone fresh: reset ticks so acceleration restarts.
        self.0.drag_scroll_ticks.set(0);
        self.start_cell_drag_scroll();
    }

    fn start_cell_drag_scroll(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut()>::new(move || {
            gc.step_cell_drag_scroll();
        });
        let win = web_sys::window().expect("no window");
        let id = win
            .set_interval_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                DRAG_INTERVAL_MS,
            )
            .expect("setInterval drag scroll");
        *self.0.drag_scroll_interval.borrow_mut() = Some(id);
        self.0.drag_scroll_closures.borrow_mut().push(Box::new(cb));
    }

    pub(super) fn stop_cell_drag_scroll(&self) {
        if let Some(id) = self.0.drag_scroll_interval.borrow_mut().take() {
            if let Some(win) = web_sys::window() {
                win.clear_interval_with_handle(id);
            }
        }
        self.0.drag_scroll_closures.borrow_mut().clear();
    }

    fn step_cell_drag_scroll(&self) {
        use super::ActiveDrag;
        if !matches!(*self.0.drag.borrow(), Some(ActiveDrag::Cell)) {
            self.stop_cell_drag_scroll();
            return;
        }
        let (vx, vy) = self.0.drag_last_pos.get();
        let (vp_w, vp_h, header_h) = {
            let s = self.0.state.borrow();
            (s.viewport.width, s.viewport.height, s.model.header_height)
        };
        // Increment tick counter for time-based acceleration.
        let ticks = self.0.drag_scroll_ticks.get().saturating_add(1);
        self.0.drag_scroll_ticks.set(ticks);

        let dy = edge_velocity(vy, header_h, vp_h, ticks);
        let dx = edge_velocity(vx, 0.0, vp_w, ticks);

        if dx == 0.0 && dy == 0.0 {
            self.stop_cell_drag_scroll();
            self.0.drag_scroll_ticks.set(0);
            return;
        }
        self.dispatch(GridCommand::ScrollBy { dx, dy });
        // Re-hit-test after the scroll so the selection extends to the newly
        // visible row/column.  Extract the result before the `if let` so the
        // immutable borrow is released before dispatch tries a mutable one.
        let maybe_coord = self.0.state.borrow().hit_test(vx, vy);
        if let Some(coord) = maybe_coord {
            self.dispatch(GridCommand::ExtendSelection(coord));
        }
    }
}
