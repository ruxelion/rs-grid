use rs_grid_core::{
    commands::GridCommand,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{MouseEvent, ResizeObserver, WheelEvent};

use super::context_menu::remove_ctx_menu;
use super::dom_helpers::document;
use super::{ActiveDrag, GridCanvas, HThumbDrag, ThumbDrag};

impl GridCanvas {
    pub(super) fn attach_resize_observer(&self) {
        let gc = self.clone();

        let cb = Closure::<dyn FnMut(js_sys::Array)>::new(
            move |_entries: js_sys::Array| {
                let canvas = &gc.0.canvas;
                let win = web_sys::window().expect("no window");
                let dpr = win.device_pixel_ratio();

                let css_w = canvas.client_width() as f64;
                let css_h = canvas.client_height() as f64;

                if css_w <= 0.0 || css_h <= 0.0 {
                    return;
                }

                let new_w = (css_w * dpr) as u32;
                let new_h = (css_h * dpr) as u32;

                // Only reset the canvas when physical dimensions
                // actually change.  set_width/set_height wipe all
                // pixels to transparent, which can produce a
                // visible flash if the browser paints between the
                // clear and the subsequent render() call.
                if canvas.width() != new_w || canvas.height() != new_h {
                    canvas.set_width(new_w);
                    canvas.set_height(new_h);
                }

                gc.0.builder.borrow_mut().dpr = dpr;

                gc.dispatch(GridCommand::Resize {
                    width: css_w,
                    height: css_h,
                });
            },
        );

        let observer = ResizeObserver::new(cb.as_ref().unchecked_ref())
            .expect("ResizeObserver::new");
        observer.observe(&self.0.canvas);

        *self.0._resize_closure.borrow_mut() = Some(cb);
        *self.0._resize_observer.borrow_mut() = Some(observer);
    }

    pub(super) fn attach_listeners(&self) {
        self.attach_wheel();
        self.attach_mousedown();
        self.attach_mouseleave();
        self.attach_dblclick();
        self.attach_contextmenu();
        self.attach_mousemove();
        self.attach_mouseup();
        self.attach_keydown();
        self.attach_copy();
        self.attach_cut();
        self.attach_paste();
    }

    fn attach_wheel(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: WheelEvent| {
            evt.prevent_default();
            gc.dispatch(GridCommand::ScrollBy {
                dx: evt.delta_x(),
                dy: evt.delta_y(),
            });
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        // Browsers default wheel to passive — we must
        // opt out so preventDefault() actually works.
        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        self.0
            .canvas
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel", &f, &opts,
            )
            .expect("add wheel listener");
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("wheel".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn attach_mouseleave(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            if gc.0.state.borrow().hovered_row.is_some() {
                gc.dispatch(GridCommand::SetHoveredRow(None));
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        self.0
            .canvas
            .add_event_listener_with_callback("mouseleave", &f)
            .expect("add mouseleave listener");
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("mouseleave".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn attach_dblclick(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            let (x, y) = gc.canvas_xy(&evt);
            // Double-click on column separator → auto-fit width
            if let Some(col_idx) = gc.hit_col_resize_separator(x, y) {
                let theme = &gc.0.builder.borrow().theme;
                let char_width = theme.font_size * 0.6;
                let header_char_width = if theme.header_font_bold {
                    theme.header_font_size * 0.65
                } else {
                    theme.header_font_size * 0.6
                };
                let cell_padding = theme.cell_padding;
                gc.dispatch(GridCommand::AutoFitColumn {
                    col_idx,
                    char_width,
                    header_char_width,
                    cell_padding,
                });
                return;
            }
            let coord = gc.0.state.borrow().hit_test(x, y);
            if let Some(coord) = coord {
                let col_key =
                    gc.0.state.borrow().model.columns[coord.col].key.clone();
                gc.dispatch(GridCommand::StartEdit {
                    row: coord.row,
                    col_key,
                });
                gc.show_edit_input();
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        self.0
            .canvas
            .add_event_listener_with_callback("dblclick", &f)
            .expect("add dblclick listener");
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("dblclick".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn attach_mousedown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            // Claim focus so keydown / clipboard handlers know
            // this grid is the active target.
            let _ = gc.0.canvas.focus();

            // Right-click is handled by contextmenu event
            if evt.button() == 2 {
                return;
            }

            // Middle-click → browser-style autoscroll
            if evt.button() == 1 {
                evt.prevent_default();
                if matches!(*gc.0.drag.borrow(), Some(ActiveDrag::Pan { .. })) {
                    gc.stop_pan();
                    return;
                }
                let cx = evt.client_x() as f64;
                let cy = evt.client_y() as f64;
                *gc.0.pan_mouse.borrow_mut() = (cx, cy);
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::Pan {
                    origin_x: cx,
                    origin_y: cy,
                });
                let _ =
                    gc.0.canvas
                        .style()
                        .set_property("cursor", GridCanvas::pan_cursor());
                let gc2 = gc.clone();
                let cb2 = Closure::<dyn FnMut()>::new(move || {
                    let (ox, oy) = match *gc2.0.drag.borrow() {
                        Some(ActiveDrag::Pan { origin_x, origin_y }) => {
                            (origin_x, origin_y)
                        }
                        _ => return,
                    };
                    let (mx, my) = *gc2.0.pan_mouse.borrow();
                    const DEADZONE: f64 = 8.0;
                    const SPEED: f64 = 0.04;
                    let raw_dx = mx - ox;
                    let raw_dy = my - oy;
                    let dx = if raw_dx.abs() > DEADZONE {
                        (raw_dx.abs() - DEADZONE)
                            * raw_dx.signum()
                            * SPEED
                            * 16.0
                    } else {
                        0.0
                    };
                    let dy = if raw_dy.abs() > DEADZONE {
                        (raw_dy.abs() - DEADZONE)
                            * raw_dy.signum()
                            * SPEED
                            * 16.0
                    } else {
                        0.0
                    };
                    if dx != 0.0 || dy != 0.0 {
                        gc2.dispatch(GridCommand::ScrollBy { dx, dy });
                    }
                });
                let win2 = web_sys::window().expect("no window");
                let id = win2
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        cb2.as_ref().unchecked_ref(),
                        16,
                    )
                    .expect("setInterval");
                *gc.0.scroll_interval.borrow_mut() = Some(id);
                gc.0.scroll_closures.borrow_mut().push(Box::new(cb2));
                return;
            }

            let (x, y) = gc.canvas_xy(&evt);

            // Close any open context menu
            remove_ctx_menu();

            // ── scrollbar interaction ─────────────────────
            if let Some(sb) = gc.scrollbar() {
                if sb.hit_up_arrow(x, y) {
                    let row_h = gc.0.state.borrow().model.row_height;
                    gc.dispatch(GridCommand::ScrollBy {
                        dx: 0.0,
                        dy: -row_h,
                    });
                    gc.start_scroll_repeat(-row_h);
                    return;
                }
                if sb.hit_down_arrow(x, y) {
                    let row_h = gc.0.state.borrow().model.row_height;
                    gc.dispatch(GridCommand::ScrollBy { dx: 0.0, dy: row_h });
                    gc.start_scroll_repeat(row_h);
                    return;
                }
                if sb.hit_thumb(x, y) {
                    *gc.0.drag.borrow_mut() =
                        Some(ActiveDrag::Thumb(ThumbDrag {
                            start_client_y: evt.client_y() as f64,
                            start_scroll_y: gc
                                .0
                                .state
                                .borrow()
                                .viewport
                                .scroll_y,
                        }));
                    return;
                }
                if sb.hit_track(x, y) {
                    let going_down = y > sb.thumb_y + sb.thumb_h * 0.5;
                    gc.start_track_scroll_repeat(y, going_down);
                    return;
                }
            }

            // ── horizontal scrollbar interaction ──────────
            if let Some(hsb) = gc.hscrollbar() {
                if hsb.hit_left_arrow(x, y) {
                    let col_w =
                        gc.0.state
                            .borrow()
                            .model
                            .columns
                            .first()
                            .map_or(40.0, |c| c.width);
                    gc.dispatch(GridCommand::ScrollBy {
                        dx: -col_w,
                        dy: 0.0,
                    });
                    gc.start_scroll_repeat_x(-col_w);
                    return;
                }
                if hsb.hit_right_arrow(x, y) {
                    let col_w =
                        gc.0.state
                            .borrow()
                            .model
                            .columns
                            .first()
                            .map_or(40.0, |c| c.width);
                    gc.dispatch(GridCommand::ScrollBy { dx: col_w, dy: 0.0 });
                    gc.start_scroll_repeat_x(col_w);
                    return;
                }
                if hsb.hit_thumb(x, y) {
                    *gc.0.drag.borrow_mut() =
                        Some(ActiveDrag::HThumb(HThumbDrag {
                            start_client_x: evt.client_x() as f64,
                            start_scroll_x: gc
                                .0
                                .state
                                .borrow()
                                .viewport
                                .scroll_x,
                        }));
                    return;
                }
                if hsb.hit_track(x, y) {
                    let going_right = x > hsb.thumb_x + hsb.thumb_w * 0.5;
                    gc.start_htrack_scroll_repeat(x, going_right);
                    return;
                }
            }

            // ── row header selection ──────────────────────
            let row = gc.0.state.borrow().hit_test_row_header(x, y);
            if let Some(row) = row {
                if evt.shift_key() {
                    gc.dispatch(GridCommand::ExtendRowSelection(row));
                } else {
                    gc.dispatch(GridCommand::SelectRow(row));
                }
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::Row);
                return;
            }

            // ── column resize separator ───────────────────
            if let Some(col_idx) = gc.hit_col_resize_separator(x, y) {
                let start_width =
                    gc.0.state.borrow().model.columns[col_idx].width;
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::ColumnResize {
                    col_idx,
                    start_client_x: evt.client_x() as f64,
                    start_width,
                });
                gc.set_cursor("col-resize");
                return;
            }

            // ── column header ─────────────────────────────
            let col = gc.0.state.borrow().hit_test_col_header(x, y);
            if let Some(col) = col {
                if evt.shift_key() {
                    gc.dispatch(GridCommand::ExtendColSelection(col));
                    *gc.0.drag.borrow_mut() = Some(ActiveDrag::Col);
                } else {
                    *gc.0.drag.borrow_mut() = Some(ActiveDrag::ColClick {
                        col_idx: col,
                        start_client_x: evt.client_x() as f64,
                    });
                }
                return;
            }

            // ── cell selection ────────────────────────────
            let coord = gc.0.state.borrow().hit_test(x, y);
            if let Some(coord) = coord {
                if evt.shift_key() {
                    gc.dispatch(GridCommand::ExtendSelection(coord));
                } else {
                    gc.dispatch(GridCommand::SelectCell(coord));
                }
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::Cell);
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        self.0
            .canvas
            .add_event_listener_with_callback("mousedown", &f)
            .expect("add mousedown listener");
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("mousedown".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn attach_mousemove(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            let drag = gc.0.drag.borrow();
            match *drag {
                Some(ActiveDrag::Thumb(ref ds)) => {
                    let dy = evt.client_y() as f64 - ds.start_client_y;
                    let start_scroll = ds.start_scroll_y;
                    drop(drag);

                    let scroll_delta = {
                        let s = gc.0.state.borrow();
                        let track_w =
                            gc.0.builder.borrow().theme.scrollbar_width;
                        if let Some(sb) = ScrollbarGeom::compute(
                            s.viewport.scroll_y,
                            s.viewport.width,
                            s.viewport.height,
                            s.model.header_height,
                            s.model.total_height(),
                            track_w,
                        ) {
                            sb.drag_to_scroll(
                                dy,
                                s.model.total_height(),
                                s.viewport.height,
                                s.model.header_height,
                            )
                        } else {
                            return;
                        }
                    };

                    let current_x = gc.0.state.borrow().viewport.scroll_x;
                    gc.dispatch(GridCommand::ScrollTo {
                        x: current_x,
                        y: start_scroll + scroll_delta,
                    });
                }
                Some(ActiveDrag::HThumb(ref ds)) => {
                    let dx = evt.client_x() as f64 - ds.start_client_x;
                    let start_scroll = ds.start_scroll_x;
                    drop(drag);
                    let scroll_delta = {
                        let s = gc.0.state.borrow();
                        let track_h =
                            gc.0.builder.borrow().theme.scrollbar_width;
                        let vsb_w = if ScrollbarGeom::compute(
                            s.viewport.scroll_y,
                            s.viewport.width,
                            s.viewport.height,
                            s.model.header_height,
                            s.model.total_height(),
                            track_h,
                        )
                        .is_some()
                        {
                            track_h
                        } else {
                            0.0
                        };
                        if let Some(hsb) = HScrollbarGeom::compute(
                            s.viewport.scroll_x,
                            s.viewport.width,
                            s.viewport.height,
                            s.model.row_number_width,
                            s.model.total_width(),
                            vsb_w,
                            track_h,
                        ) {
                            hsb.drag_to_scroll(
                                dx,
                                s.model.total_width(),
                                s.viewport.width,
                                s.model.row_number_width,
                                vsb_w,
                            )
                        } else {
                            return;
                        }
                    };
                    let current_y = gc.0.state.borrow().viewport.scroll_y;
                    gc.dispatch(GridCommand::ScrollTo {
                        x: start_scroll + scroll_delta,
                        y: current_y,
                    });
                }
                Some(ActiveDrag::Cell) => {
                    drop(drag);
                    let (x, y) = gc.canvas_xy(&evt);
                    let coord = gc.0.state.borrow().hit_test(x, y);
                    if let Some(coord) = coord {
                        gc.dispatch(GridCommand::ExtendSelection(coord));
                    }
                }
                Some(ActiveDrag::Row) => {
                    drop(drag);
                    let (x, y) = gc.canvas_xy(&evt);
                    let row = gc.0.state.borrow().hit_test_row_header(x, y);
                    if let Some(row) = row {
                        gc.dispatch(GridCommand::ExtendRowSelection(row));
                    }
                }
                Some(ActiveDrag::Col) => {
                    drop(drag);
                    let (x, y) = gc.canvas_xy(&evt);
                    let col = gc.0.state.borrow().hit_test_col_header(x, y);
                    if let Some(col) = col {
                        gc.dispatch(GridCommand::ExtendColSelection(col));
                    }
                }
                Some(ActiveDrag::ColClick {
                    col_idx,
                    start_client_x,
                }) => {
                    let (ci, scx) = (col_idx, start_client_x);
                    drop(drag);
                    let dx = evt.client_x() as f64 - scx;
                    if dx.abs() > 5.0 {
                        let (vx, vy) = gc.canvas_xy(&evt);
                        // Seed animation from real column positions
                        // so the lerp starts from the actual layout.
                        {
                            let state = gc.0.state.borrow();
                            let mut cum = 0.0_f64;
                            let init: Vec<f64> = state
                                .model
                                .columns
                                .iter()
                                .map(|c| {
                                    let off = cum;
                                    cum += c.width;
                                    off
                                })
                                .collect();
                            drop(state);
                            *gc.0.drag_col_offsets.borrow_mut() =
                                init;
                        }
                        *gc.0.drag.borrow_mut() =
                            Some(ActiveDrag::ColumnDrag {
                                col_idx: ci,
                                current_vx: vx,
                                current_vy: vy,
                            });
                        gc.set_cursor("grabbing");
                        gc.render();
                    }
                }
                Some(ActiveDrag::ColumnDrag { col_idx, .. }) => {
                    let ci = col_idx;
                    drop(drag);
                    let (vx, vy) = gc.canvas_xy(&evt);
                    *gc.0.drag.borrow_mut() = Some(ActiveDrag::ColumnDrag {
                        col_idx: ci,
                        current_vx: vx,
                        current_vy: vy,
                    });
                    gc.set_cursor("grabbing");
                    gc.render();
                }
                Some(ActiveDrag::Pan { .. }) => {
                    drop(drag);
                    *gc.0.pan_mouse.borrow_mut() =
                        (evt.client_x() as f64, evt.client_y() as f64);
                }
                Some(ActiveDrag::ColumnResize {
                    col_idx,
                    start_client_x,
                    start_width,
                }) => {
                    let (ci, scx, sw) = (col_idx, start_client_x, start_width);
                    drop(drag);
                    let dx = evt.client_x() as f64 - scx;
                    gc.dispatch(GridCommand::ResizeColumn {
                        col_idx: ci,
                        new_width: sw + dx,
                    });
                }
                None => {
                    drop(drag);
                    // Skip hover while an edit overlay is open
                    // (e.g. the custom dropdown).
                    if gc.0.edit_input.borrow().is_some() {
                        return;
                    }
                    let (x, y) = gc.canvas_xy(&evt);
                    if gc.hit_col_resize_separator(x, y).is_some() {
                        gc.set_cursor("col-resize");
                    } else {
                        gc.set_cursor("default");
                    }
                    let new_row = gc.row_at(x, y);
                    if gc.0.state.borrow().hovered_row != new_row {
                        gc.dispatch(GridCommand::SetHoveredRow(new_row));
                    }
                }
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("mousemove", &f)
            .expect("add mousemove listener");
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("mousemove".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn attach_mouseup(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            // Middle-button release stops pan.
            if evt.button() == 1 {
                if matches!(*gc.0.drag.borrow(), Some(ActiveDrag::Pan { .. })) {
                    gc.stop_pan();
                }
                return;
            }
            gc.stop_scroll_repeat();

            let finished_drag = gc.0.drag.borrow_mut().take();
            match finished_drag {
                Some(ActiveDrag::ColClick { col_idx, .. }) => {
                    let key =
                        gc.0.state
                            .borrow()
                            .model
                            .columns
                            .get(col_idx)
                            .map(|c| c.key.clone());
                    if let Some(key) = key {
                        gc.dispatch(GridCommand::ToggleSort { col_key: key });
                    }
                }
                Some(ActiveDrag::ColumnDrag {
                    col_idx,
                    current_vx,
                    ..
                }) => {
                    gc.0.drag_col_offsets.borrow_mut().clear();
                    gc.set_cursor("default");
                    let insert = gc.insertion_index(current_vx);
                    let to = if insert > col_idx { insert - 1 } else { insert };
                    if to != col_idx {
                        gc.dispatch(GridCommand::MoveColumn {
                            from_idx: col_idx,
                            to_idx: to,
                        });
                    } else {
                        // No move — just redraw to clear
                        // the drag preview.
                        gc.render();
                    }
                }
                Some(ActiveDrag::ColumnResize { .. }) => {
                    gc.set_cursor("default");
                }
                _ => {}
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("mouseup", &f)
            .expect("add mouseup listener");
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("mouseup".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }
}
