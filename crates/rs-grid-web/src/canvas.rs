use std::{cell::RefCell, rc::Rc};

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    scrollbar::ScrollbarGeom,
    selection::CopyError,
    state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;
use rs_grid_scene::builder::SceneBuilder;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, ResizeObserver, WheelEvent};

// ── public handle ─────────────────────────────────────────────────────────────

/// A mounted grid that owns its event listeners and render pipeline.
///
/// Cheaply cloneable (inner `Rc`).
#[derive(Clone)]
pub struct GridCanvas(Rc<Inner>);

// ── internal state ────────────────────────────────────────────────────────────

struct Inner {
    state: RefCell<GridState>,
    builder: RefCell<SceneBuilder>,
    renderer: CanvasRenderer,
    canvas: HtmlCanvasElement,
    /// Active drag interaction, if any.
    drag: RefCell<Option<ActiveDrag>>,
    _resize_closure: RefCell<Option<Closure<dyn FnMut(js_sys::Array)>>>,
    _resize_observer: RefCell<Option<ResizeObserver>>,
}

enum ActiveDrag {
    Thumb(ThumbDrag),
    Cell,
}

struct ThumbDrag {
    /// `clientY` of the mousedown that started the drag.
    start_client_y: f64,
    /// `scroll_y` at the moment the drag started.
    start_scroll_y: f64,
}

// ── impl ──────────────────────────────────────────────────────────────────────

impl GridCanvas {
    /// Mount a grid onto an existing `<canvas>` element.
    ///
    /// - Sets the canvas physical size = CSS size × device-pixel-ratio.
    /// - Registers `wheel`, `mousedown`, `mousemove` (document), `mouseup` (document).
    pub fn mount(canvas: HtmlCanvasElement, mut state: GridState) -> Self {
        let win = web_sys::window().expect("no window");
        let dpr = win.device_pixel_ratio();

        let css_w = canvas.client_width() as f64;
        let css_h = canvas.client_height() as f64;
        canvas.set_width((css_w * dpr) as u32);
        canvas.set_height((css_h * dpr) as u32);

        state.apply(GridCommand::Resize {
            width: css_w,
            height: css_h,
        });

        let ctx = canvas
            .get_context("2d")
            .expect("getContext")
            .expect("2d context")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("cast");

        let inner = Rc::new(Inner {
            state: RefCell::new(state),
            builder: RefCell::new(SceneBuilder::new(dpr)),
            renderer: CanvasRenderer::new(ctx),
            canvas,
            drag: RefCell::new(None),
            _resize_closure: RefCell::new(None),
            _resize_observer: RefCell::new(None),
        });

        let gc = GridCanvas(inner);
        gc.attach_listeners();
        gc.attach_resize_observer();
        gc.render();
        gc
    }

    /// Render the current state immediately.
    pub fn render(&self) {
        let state = self.0.state.borrow();
        let frame = self.0.builder.borrow().build(&state);
        self.0.renderer.render(&frame);
    }

    /// Apply a command, redraw, and return the output.
    fn dispatch_with_output(&self, cmd: GridCommand) -> CommandOutput {
        let out = self.0.state.borrow_mut().apply(cmd);
        self.render();
        out
    }

    /// Apply a command then redraw.
    pub fn dispatch(&self, cmd: GridCommand) {
        self.dispatch_with_output(cmd);
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn scrollbar(&self) -> Option<ScrollbarGeom> {
        let s = self.0.state.borrow();
        ScrollbarGeom::compute(
            s.viewport.scroll_y,
            s.viewport.width,
            s.viewport.height,
            s.model.header_height,
            s.model.total_height(),
        )
    }

    fn canvas_xy(&self, evt: &MouseEvent) -> (f64, f64) {
        let rect = self.0.canvas.get_bounding_client_rect();
        (
            evt.client_x() as f64 - rect.left(),
            evt.client_y() as f64 - rect.top(),
        )
    }

    fn handle_copy(&self) {
        match self.dispatch_with_output(GridCommand::CopySelection) {
            CommandOutput::CopyText(text) => self.write_clipboard(text),
            CommandOutput::CopyError(CopyError::TooManyRows { actual, max }) => {
                let msg = format!("Copy annulé : {actual} lignes sélectionnées (max {max})");
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&msg));
            }
            _ => {}
        }
    }

    fn write_clipboard(&self, text: String) {
        let window = web_sys::window().expect("no window");
        let clipboard = window.navigator().clipboard();
        let promise = clipboard.write_text(&text);
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = wasm_bindgen_futures::JsFuture::from(promise).await {
                web_sys::console::warn_1(&e);
            }
        });
    }

    // ── event wiring ─────────────────────────────────────────────────────────

    fn attach_resize_observer(&self) {
        let gc = self.clone();

        let cb = Closure::<dyn FnMut(js_sys::Array)>::new(move |_entries: js_sys::Array| {
            let canvas = &gc.0.canvas;
            let win = web_sys::window().expect("no window");
            let dpr = win.device_pixel_ratio();

            let css_w = canvas.client_width() as f64;
            let css_h = canvas.client_height() as f64;

            if css_w <= 0.0 || css_h <= 0.0 {
                return;
            }

            canvas.set_width((css_w * dpr) as u32);
            canvas.set_height((css_h * dpr) as u32);

            gc.0.builder.borrow_mut().dpr = dpr;

            gc.dispatch(GridCommand::Resize { width: css_w, height: css_h });
        });

        let observer =
            ResizeObserver::new(cb.as_ref().unchecked_ref()).expect("ResizeObserver::new");
        observer.observe(&self.0.canvas);

        *self.0._resize_closure.borrow_mut() = Some(cb);
        *self.0._resize_observer.borrow_mut() = Some(observer);
    }

    fn attach_listeners(&self) {
        self.attach_wheel();
        self.attach_mousedown();
        self.attach_mousemove(); // on document — works even if cursor leaves canvas
        self.attach_mouseup();   // on document
        self.attach_keydown();   // on document
        self.attach_paste();     // on document
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
        self.0
            .canvas
            .add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    fn attach_mousedown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            let (x, y) = gc.canvas_xy(&evt);

            // ── scrollbar interaction ─────────────────────────────────────────
            if let Some(sb) = gc.scrollbar() {
                if sb.hit_thumb(x, y) {
                    // Start thumb drag
                    *gc.0.drag.borrow_mut() = Some(ActiveDrag::Thumb(ThumbDrag {
                        start_client_y: evt.client_y() as f64,
                        start_scroll_y: gc.0.state.borrow().viewport.scroll_y,
                    }));
                    return;
                }
                if sb.hit_track(x, y) {
                    // Click on track: jump scroll so thumb centres under cursor
                    let (total_h, vp_h, hdr_h) = {
                        let s = gc.0.state.borrow();
                        (s.model.total_height(), s.viewport.height, s.model.header_height)
                    };
                    let new_y = sb.track_click_scroll(y, total_h, vp_h, hdr_h);
                    gc.dispatch(GridCommand::ScrollTo { x: 0.0, y: new_y });
                    return;
                }
            }

            // ── cell selection ────────────────────────────────────────────────
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
        self.0
            .canvas
            .add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
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
                        if let Some(sb) = ScrollbarGeom::compute(
                            s.viewport.scroll_y,
                            s.viewport.width,
                            s.viewport.height,
                            s.model.header_height,
                            s.model.total_height(),
                        ) {
                            sb.drag_to_scroll(dy, s.model.total_height(), s.viewport.height, s.model.header_height)
                        } else {
                            return;
                        }
                    };

                    gc.dispatch(GridCommand::ScrollTo {
                        x: 0.0,
                        y: start_scroll + scroll_delta,
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
                None => {}
            }
        });
        document()
            .add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    fn attach_mouseup(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            *gc.0.drag.borrow_mut() = None;
        });
        document()
            .add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    fn attach_keydown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
            let key = evt.key();
            let shift = evt.shift_key();
            let ctrl  = evt.ctrl_key() || evt.meta_key();
            match key.as_str() {
                "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => {
                    if !gc.0.state.borrow().selection.has_selection() { return; }
                    evt.prevent_default();
                    let (dr, dc) = match key.as_str() {
                        "ArrowUp"    => (-1_i64,  0_i64),
                        "ArrowDown"  => ( 1,  0),
                        "ArrowLeft"  => ( 0, -1),
                        "ArrowRight" => ( 0,  1),
                        _ => unreachable!(),
                    };
                    gc.dispatch(GridCommand::MoveSelection { delta_row: dr, delta_col: dc, extend: shift });
                }
                "c" if ctrl => {
                    if !gc.0.state.borrow().selection.has_selection() { return; }
                    evt.prevent_default();
                    gc.handle_copy();
                }
                "Escape" => { gc.dispatch(GridCommand::ClearSelection); }
                _ => {}
            }
        });
        document()
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }

    fn attach_paste(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: web_sys::ClipboardEvent| {
            if !gc.0.state.borrow().selection.has_selection() { return; }
            if let Some(dt) = evt.clipboard_data() {
                if let Ok(text) = dt.get_data("text/plain") {
                    evt.prevent_default();
                    gc.dispatch(GridCommand::PasteAt { text });
                }
            }
        });
        document()
            .add_event_listener_with_callback("paste", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    }
}

fn document() -> web_sys::Document {
    web_sys::window().expect("no window").document().expect("no document")
}
