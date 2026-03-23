use rs_grid_core::{commands::GridCommand, scrollbar::ScrollbarGeom};
use wasm_bindgen::{prelude::Closure, JsCast};

use super::GridCanvas;

impl GridCanvas {
    // ── arrow-button auto-scroll ─────────────────────────────

    /// Start a repeating scroll (arrows): immediate first
    /// scroll on mousedown, then ~350 ms pause, then repeat
    /// every 60 ms.
    pub(super) fn start_scroll_repeat(&self, dy: f64) {
        self.stop_scroll_repeat();
        let gc = self.clone();
        let win = web_sys::window().expect("no window");
        let timeout_cb = Closure::<dyn FnMut()>::new(move || {
            let gc2 = gc.clone();
            let interval_cb = Closure::<dyn FnMut()>::new(move || {
                gc2.dispatch(GridCommand::ScrollBy { dx: 0.0, dy });
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

    pub(super) fn pan_cursor() -> &'static str {
        concat!(
            "url(\"data:image/svg+xml,",
            "%3Csvg xmlns='http://www.w3.org/2000/svg' ",
            "width='32' height='32'%3E",
            "%3Ccircle cx='16' cy='16' r='5' fill='none' ",
            "stroke='%23555' stroke-width='1.5'/%3E",
            "%3Ccircle cx='16' cy='16' r='2' ",
            "fill='%23555'/%3E",
            "%3Cpolygon points='16,3 12.5,10 19.5,10' ",
            "fill='%23555'/%3E",
            "%3Cpolygon points='16,29 12.5,22 19.5,22' ",
            "fill='%23555'/%3E",
            "%3Cpolygon points='3,16 10,12.5 10,19.5' ",
            "fill='%23555'/%3E",
            "%3Cpolygon points='29,16 22,12.5 22,19.5' ",
            "fill='%23555'/%3E",
            "%3C/svg%3E\") 16 16, all-scroll",
        )
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
        self.stop_scroll_repeat();
        let win = web_sys::window().expect("no window");

        // Phase 1: slow mini-easing (few steps, 100 ms).
        let gc1 = self.clone();
        let slow_cb = Closure::<dyn FnMut()>::new(move || {
            gc1.do_track_scroll_step(click_y, going_down);
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
            let fast_cb = Closure::<dyn FnMut()>::new(move || {
                gc3.do_track_scroll_step(click_y, going_down);
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
        self.stop_scroll_repeat();
        let gc = self.clone();
        let win = web_sys::window().expect("no window");
        let timeout_cb = Closure::<dyn FnMut()>::new(move || {
            let gc2 = gc.clone();
            let interval_cb = Closure::<dyn FnMut()>::new(move || {
                gc2.dispatch(GridCommand::ScrollBy { dx, dy: 0.0 });
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
        self.stop_scroll_repeat();
        let win = web_sys::window().expect("no window");

        // Phase 1: slow mini-easing at 100 ms.
        let gc1 = self.clone();
        let slow_cb = Closure::<dyn FnMut()>::new(move || {
            gc1.do_htrack_scroll_step(click_x, going_right);
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
            let fast_cb = Closure::<dyn FnMut()>::new(move || {
                gc3.do_htrack_scroll_step(click_x, going_right);
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
