use std::{cell::RefCell, rc::Rc};

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    selection::CopyError,
    state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;
use rs_grid_scene::{builder::SceneBuilder, Theme};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, ResizeObserver,
    WheelEvent,
};

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
    /// Handle returned by `setTimeout` for the initial auto-scroll delay.
    scroll_timeout: RefCell<Option<i32>>,
    /// Handle returned by `setInterval` for arrow-button auto-scroll (cleared on mouseup).
    scroll_interval: RefCell<Option<i32>>,
    /// Current mouse position during middle-click autoscroll (client coords).
    pan_mouse: RefCell<(f64, f64)>,
    _resize_closure: RefCell<Option<Closure<dyn FnMut(js_sys::Array)>>>,
    _resize_observer: RefCell<Option<ResizeObserver>>,
    /// Stored references to document-level listeners so they can be removed on detach.
    doc_listeners: RefCell<Vec<(String, js_sys::Function)>>,
    /// Optional callback fired after every command that mutates cell data.
    on_change: RefCell<Option<Box<dyn Fn()>>>,
}

enum ActiveDrag {
    Thumb(ThumbDrag),
    HThumb(HThumbDrag),
    Cell,
    Row,
    Col,
    Pan { origin_x: f64, origin_y: f64 },
    ColumnResize {
        col_idx: usize,
        start_client_x: f64,
        start_width: f64,
    },
}

struct ThumbDrag {
    start_client_y: f64,
    start_scroll_y: f64,
}

struct HThumbDrag {
    start_client_x: f64,
    start_scroll_x: f64,
}

// ── impl ──────────────────────────────────────────────────────────────────────

impl GridCanvas {
    /// Mount a grid onto an existing `<canvas>` element.
    ///
    /// - Sets the canvas physical size = CSS size × device-pixel-ratio.
    /// - Registers `wheel`, `mousedown`, `mousemove` (document), `mouseup` (document).
    pub fn mount(
        canvas: HtmlCanvasElement,
        mut state: GridState,
        theme: Theme,
    ) -> Self {
        let win = web_sys::window().expect("no window");
        let dpr = win.device_pixel_ratio();

        let css_w = canvas.client_width() as f64;
        let css_h = canvas.client_height() as f64;
        let phys_w = (css_w * dpr) as u32;
        let phys_h = (css_h * dpr) as u32;
        canvas.set_width(phys_w);
        canvas.set_height(phys_h);

        // Pin CSS background to the theme bg so the canvas never flashes
        // transparent while the context is being reset.
        let _ = canvas
            .style()
            .set_property("background-color", &theme.bg.to_css());

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
            builder: RefCell::new(SceneBuilder::with_theme(dpr, theme)),
            renderer: CanvasRenderer::new(ctx),
            canvas,
            drag: RefCell::new(None),
            scroll_timeout: RefCell::new(None),
            scroll_interval: RefCell::new(None),
            pan_mouse: RefCell::new((0.0, 0.0)),
            _resize_closure: RefCell::new(None),
            _resize_observer: RefCell::new(None),
            doc_listeners: RefCell::new(Vec::new()),
            on_change: RefCell::new(None),
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
        let is_mutation = matches!(cmd, GridCommand::PasteAt { .. });
        let out = self.0.state.borrow_mut().apply(cmd);
        self.render();
        if is_mutation {
            if let Some(cb) = self.0.on_change.borrow().as_ref() {
                cb();
            }
        }
        out
    }

    /// Apply a command then redraw.
    pub fn dispatch(&self, cmd: GridCommand) {
        self.dispatch_with_output(cmd);
    }

    /// Register a callback fired after every cell-data mutation (paste).
    pub fn set_on_change(&self, cb: impl Fn() + 'static) {
        *self.0.on_change.borrow_mut() = Some(Box::new(cb));
    }

    /// Serialize the current patch layer as TSV text (one line per edited
    /// cell: `physical_row\tcol_key\tvalue`). Tab/newline/backslash in
    /// keys/values are escaped as `\t`, `\n`, `\\`.
    pub fn export_patches(&self) -> String {
        let state = self.0.state.borrow();
        let mut out = String::new();
        for ((row, col), val) in &state.model.patches {
            let ec = col
                .replace('\\', "\\\\")
                .replace('\t', "\\t")
                .replace('\n', "\\n");
            let ev = val
                .replace('\\', "\\\\")
                .replace('\t', "\\t")
                .replace('\n', "\\n");
            out.push_str(&format!("{row}\t{ec}\t{ev}\n"));
        }
        out
    }

    /// Deserialize TSV text produced by `export_patches` and apply it,
    /// replacing any existing patches. Triggers a redraw.
    pub fn import_patches(&self, data: &str) {
        let unescape = |s: &str| {
            s.replace("\\\\", "\x00")
                .replace("\\t", "\t")
                .replace("\\n", "\n")
                .replace('\x00', "\\")
        };
        let mut state = self.0.state.borrow_mut();
        state.model.patches.clear();
        for line in data.lines() {
            let mut parts = line.splitn(3, '\t');
            let (Some(r), Some(c), Some(v)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Ok(row) = r.parse::<u64>() else { continue };
            state.model.patches.insert((row, unescape(c)), unescape(v));
        }
        drop(state);
        self.render();
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn scrollbar(&self) -> Option<ScrollbarGeom> {
        let s = self.0.state.borrow();
        let track_w = self.0.builder.borrow().theme.scrollbar_width;
        ScrollbarGeom::compute(
            s.viewport.scroll_y,
            s.viewport.width,
            s.viewport.height,
            s.model.header_height,
            s.model.total_height(),
            track_w,
        )
    }

    fn hscrollbar(&self) -> Option<HScrollbarGeom> {
        let s = self.0.state.borrow();
        let track_h = self.0.builder.borrow().theme.scrollbar_width;
        let vsb_w = if ScrollbarGeom::compute(
            s.viewport.scroll_y, s.viewport.width, s.viewport.height,
            s.model.header_height, s.model.total_height(), track_h,
        ).is_some() { track_h } else { 0.0 };
        HScrollbarGeom::compute(
            s.viewport.scroll_x,
            s.viewport.width,
            s.viewport.height,
            s.model.row_number_width,
            s.model.total_width(),
            vsb_w,
            track_h,
        )
    }

    fn canvas_xy(&self, evt: &MouseEvent) -> (f64, f64) {
        let rect = self.0.canvas.get_bounding_client_rect();
        (
            evt.client_x() as f64 - rect.left(),
            evt.client_y() as f64 - rect.top(),
        )
    }

    /// Returns `Some(col_idx)` when `(vx, vy)` is within `HIT_ZONE` px of a
    /// column separator in the header, enabling the resize cursor / drag.
    fn hit_col_resize_separator(&self, vx: f64, vy: f64) -> Option<usize> {
        const HIT_ZONE: f64 = 4.0;
        let state = self.0.state.borrow();
        let model = &state.model;
        if vy >= model.header_height { return None; }
        if vx < model.row_number_width { return None; }
        let scroll_x = state.viewport.scroll_x;
        let rnw = model.row_number_width;
        for (i, col) in model.columns.iter().enumerate() {
            let sep_vx =
                model.column_offsets.offsets[i] + col.width - scroll_x + rnw;
            if (vx - sep_vx).abs() <= HIT_ZONE {
                return Some(i);
            }
        }
        None
    }

    fn set_cursor(&self, cursor: &str) {
        let _ = self.0.canvas.style().set_property("cursor", cursor);
    }

    /// Returns the data row index under viewport point `(vx, vy)`, or `None`
    /// if the point is in the header, gutter, or below the last row.
    fn row_at(&self, vx: f64, vy: f64) -> Option<u64> {
        let state = self.0.state.borrow();
        let model = &state.model;
        if vy < model.header_height { return None; }
        if vx < 0.0 || vx > state.viewport.width { return None; }
        let abs_y = vy - model.header_height + state.viewport.scroll_y;
        let row = (abs_y / model.row_height) as u64;
        if row < model.data.row_count() { Some(row) } else { None }
    }

    fn handle_copy(&self) {
        match self.dispatch_with_output(GridCommand::CopySelection) {
            CommandOutput::CopyText(text) => self.write_clipboard(text),
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Copy annulé : {actual} lignes sélectionnées (max {max})"
                );
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                    &msg,
                ));
            }
            _ => {}
        }
    }

    fn handle_cut(&self) {
        match self.dispatch_with_output(GridCommand::CutSelection) {
            CommandOutput::CopyText(text) => self.write_clipboard(text),
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Cut annulé : {actual} lignes sélectionnées (max {max})"
                );
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                    &msg,
                ));
            }
            _ => {}
        }
    }

    fn handle_copy_headers(&self) {
        let header_row = {
            let state = self.0.state.borrow();
            let (tl, br) = match state.selection.range() {
                Some(r) => r,
                None => return,
            };
            let cols = &state.model.columns;
            (tl.col..=br.col)
                .map(|ci| cols[ci].label.clone())
                .collect::<Vec<_>>()
                .join("\t")
        };
        match self.dispatch_with_output(GridCommand::CopySelection) {
            CommandOutput::CopyText(data) => {
                self.write_clipboard(format!("{header_row}\n{data}"));
            }
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Copy annulé : {actual} lignes sélectionnées (max {max})"
                );
                web_sys::console::warn_1(
                    &wasm_bindgen::JsValue::from_str(&msg),
                );
            }
            _ => {}
        }
    }

    fn write_clipboard(&self, text: String) {
        let window = web_sys::window().expect("no window");
        let clipboard = window.navigator().clipboard();
        let promise = clipboard.write_text(&text);
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = wasm_bindgen_futures::JsFuture::from(promise).await
            {
                web_sys::console::warn_1(&e);
            }
        });
    }

    // ── context menu ─────────────────────────────────────────────────────────

    fn attach_contextmenu(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            evt.prevent_default();

            // Select cell under right-click if nothing is selected yet
            let (cx, cy) = gc.canvas_xy(&evt);
            let has_sel = gc.0.state.borrow().selection.has_selection();
            if !has_sel {
                // Row gutter takes priority.
                let row = gc.0.state.borrow().hit_test_row_header(cx, cy);
                if let Some(row) = row {
                    gc.dispatch(GridCommand::SelectRow(row));
                } else {
                    let coord = gc.0.state.borrow().hit_test(cx, cy);
                    if let Some(coord) = coord {
                        gc.dispatch(GridCommand::SelectCell(coord));
                    }
                }
            }

            gc.show_context_menu(evt.client_x(), evt.client_y());
        });
        self.0
            .canvas
            .add_event_listener_with_callback(
                "contextmenu",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
        cb.forget();
    }

    fn show_context_menu(&self, x: i32, y: i32) {
        let doc = document();
        remove_ctx_menu();

        let body = doc.body().expect("no body");

        // ── backdrop (transparent overlay, closes menu on click) ──────────────
        let backdrop = make_el(&doc, "div");
        backdrop.set_id("rs-grid-ctx-backdrop");
        set_styles(
            &backdrop,
            &[("position", "fixed"), ("inset", "0"), ("z-index", "9998")],
        );
        {
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
            });
            backdrop
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }

        // ── menu container ────────────────────────────────────────────────────
        let menu = make_el(&doc, "div");
        menu.set_id("rs-grid-ctx-menu");
        set_styles(
            &menu,
            &[
                ("position", "fixed"),
                ("left", &format!("{}px", x)),
                ("top", &format!("{}px", y)),
                ("z-index", "9999"),
                ("background", "#ffffff"),
                ("border", "1px solid #d1d5db"),
                ("border-radius", "6px"),
                ("box-shadow", "0 4px 16px rgba(0,0,0,0.12)"),
                ("padding", "4px 0"),
                ("font", "13px/1.4 system-ui,sans-serif"),
                ("min-width", "160px"),
                ("user-select", "none"),
            ],
        );

        let has_selection = self.0.state.borrow().selection.has_selection();

        // ── Cut ───────────────────────────────────────────────────────────────
        let cut_item = make_menu_item(
            &doc,
            ICON_CUT,
            "Couper",
            "Ctrl+X",
            has_selection,
        );
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_cut();
            });
            cut_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&cut_item).unwrap();

        // ── Copy ──────────────────────────────────────────────────────────────
        let copy_item = make_menu_item(
            &doc,
            ICON_COPY,
            "Copier",
            "Ctrl+C",
            has_selection,
        );
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_copy();
            });
            copy_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&copy_item).unwrap();

        // ── Copy with Headers ─────────────────────────────────────────────────
        let copy_hdrs_item = make_menu_item(
            &doc,
            ICON_COPY,
            "Copier avec en-têtes",
            "",
            has_selection,
        );
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_copy_headers();
            });
            copy_hdrs_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&copy_hdrs_item).unwrap();

        // ── separator ─────────────────────────────────────────────────────────
        menu.append_child(&make_menu_separator(&doc)).unwrap();

        // ── Paste ─────────────────────────────────────────────────────────────
        let paste_item =
            make_menu_item(&doc, ICON_PASTE, "Coller", "Ctrl+V", has_selection);
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                if !gc.0.state.borrow().selection.has_selection() {
                    return;
                }
                let win = web_sys::window().expect("no window");
                let promise = win.navigator().clipboard().read_text();
                let gc2 = gc.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match wasm_bindgen_futures::JsFuture::from(promise).await {
                        Ok(val) => {
                            if let Some(text) = val.as_string() {
                                gc2.dispatch(GridCommand::PasteAt { text });
                            }
                        }
                        Err(e) => {
                            web_sys::console::warn_1(&e);
                        }
                    }
                });
            });
            paste_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&paste_item).unwrap();

        body.append_child(&backdrop).unwrap();
        body.append_child(&menu).unwrap();
    }

    // ── arrow-button auto-scroll ─────────────────────────────────────────────

    /// Start a repeating scroll (arrows): immediate first scroll on mousedown,
    /// then ~350 ms pause, then repeat every 60 ms.
    fn start_scroll_repeat(&self, dy: f64) {
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
            interval_cb.forget();
            *gc.0.scroll_interval.borrow_mut() = Some(id);
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        timeout_cb.forget();
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
    }

    fn pan_cursor() -> &'static str {
        // SVG autoscroll cursor: circle + 4 arrows, hotspot at centre (16 16).
        concat!(
            "url(\"data:image/svg+xml,",
            "%3Csvg xmlns='http://www.w3.org/2000/svg' width='32' height='32'%3E",
            "%3Ccircle cx='16' cy='16' r='5' fill='none' stroke='%23555' stroke-width='1.5'/%3E",
            "%3Ccircle cx='16' cy='16' r='2' fill='%23555'/%3E",
            // up
            "%3Cpolygon points='16,3 12.5,10 19.5,10' fill='%23555'/%3E",
            // down
            "%3Cpolygon points='16,29 12.5,22 19.5,22' fill='%23555'/%3E",
            // left
            "%3Cpolygon points='3,16 10,12.5 10,19.5' fill='%23555'/%3E",
            // right
            "%3Cpolygon points='29,16 22,12.5 22,19.5' fill='%23555'/%3E",
            "%3C/svg%3E\") 16 16, all-scroll",
        )
    }

    fn stop_pan(&self) {
        self.stop_scroll_repeat();
        *self.0.drag.borrow_mut() = None;
        let _ = self.0.canvas.style().set_property("cursor", "");
    }

    fn stop_scroll_repeat(&self) {
        let win = web_sys::window().expect("no window");
        if let Some(id) = self.0.scroll_timeout.borrow_mut().take() {
            win.clear_timeout_with_handle(id);
        }
        if let Some(id) = self.0.scroll_interval.borrow_mut().take() {
            win.clear_interval_with_handle(id);
        }
    }

    /// Animate scroll toward `click_y` (AG Grid style):
    /// 1. Mini easing at ~100 ms interval (slow start)
    /// 2. At 350 ms: switch to full 60 fps easing
    fn start_track_scroll_repeat(&self, click_y: f64, going_down: bool) {
        self.stop_scroll_repeat();
        let win = web_sys::window().expect("no window");

        // Phase 1: slow mini-easing (few steps, 100 ms apart).
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
        slow_cb.forget();
        *self.0.scroll_interval.borrow_mut() = Some(slow_id);

        // Phase 2: after 350 ms, switch to full 60 fps easing.
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
            fast_cb.forget();
            *gc2.0.scroll_interval.borrow_mut() = Some(fast_id);
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                switch_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        switch_cb.forget();
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
    }

    /// Single easing step toward `click_y`; stops interval when done.
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
            (s.model.total_height(), s.viewport.height, s.model.header_height)
        };
        let target = sb.track_click_scroll(click_y, total_h, viewport_h, header_h);
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

    // ── horizontal track scroll (mirrors vertical) ────────────────────────────

    /// Arrow-button auto-repeat for horizontal (mirrors `start_scroll_repeat`).
    fn start_scroll_repeat_x(&self, dx: f64) {
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
            interval_cb.forget();
            *gc.0.scroll_interval.borrow_mut() = Some(id);
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        timeout_cb.forget();
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
    }

    /// Single easing step toward `click_x`; stops interval when done.
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
                s.viewport.scroll_y, s.viewport.width, s.viewport.height,
                s.model.header_height, s.model.total_height(), sb_w,
            ).is_some() { sb_w } else { 0.0 };
            (s.model.total_width(), s.viewport.width, s.model.row_number_width, vsb_w)
        };
        let target = sb.track_click_scroll(click_x, total_w, viewport_w, gutter_w, vsb_w);
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

    /// AG Grid-style three-phase track scroll for horizontal axis.
    fn start_htrack_scroll_repeat(&self, click_x: f64, going_right: bool) {
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
        slow_cb.forget();
        *self.0.scroll_interval.borrow_mut() = Some(slow_id);

        // Phase 2: after 350 ms, switch to full 60 fps easing.
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
            fast_cb.forget();
            *gc2.0.scroll_interval.borrow_mut() = Some(fast_id);
        });
        let tid = win
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                switch_cb.as_ref().unchecked_ref(),
                350,
            )
            .expect("setTimeout");
        switch_cb.forget();
        *self.0.scroll_timeout.borrow_mut() = Some(tid);
    }

    // ── event wiring ─────────────────────────────────────────────────────────

    fn attach_resize_observer(&self) {
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

                // Only reset the canvas when physical dimensions actually
                // change.  set_width/set_height wipe all pixels to
                // transparent, which can produce a visible flash if the
                // browser paints between the clear and the subsequent
                // render() call.
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

    fn attach_listeners(&self) {
        self.attach_wheel();
        self.attach_mousedown();
        self.attach_mouseleave();
        self.attach_contextmenu();
        self.attach_mousemove(); // on document — works even if cursor leaves canvas
        self.attach_mouseup(); // on document
        self.attach_keydown(); // on document
        self.attach_paste(); // on document
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
            .add_event_listener_with_callback(
                "wheel",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
        cb.forget();
    }

    fn attach_mouseleave(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            if gc.0.state.borrow().hovered_row.is_some() {
                gc.dispatch(GridCommand::SetHoveredRow(None));
            }
        });
        self.0
            .canvas
            .add_event_listener_with_callback(
                "mouseleave",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
        cb.forget();
    }

    fn attach_mousedown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            // Right-click is handled by contextmenu event
            if evt.button() == 2 {
                return;
            }

            // Middle-click → browser-style autoscroll
            if evt.button() == 1 {
                evt.prevent_default();
                // If already panning, stop on second middle-click.
                if matches!(*gc.0.drag.borrow(), Some(ActiveDrag::Pan { .. })) {
                    gc.stop_pan();
                    return;
                }
                let cx = evt.client_x() as f64;
                let cy = evt.client_y() as f64;
                *gc.0.pan_mouse.borrow_mut() = (cx, cy);
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::Pan { origin_x: cx, origin_y: cy });
                let _ = gc.0.canvas.style().set_property("cursor", GridCanvas::pan_cursor());
                // Interval: scroll proportional to distance from origin.
                let gc2 = gc.clone();
                let cb2 = Closure::<dyn FnMut()>::new(move || {
                    let (ox, oy) = match *gc2.0.drag.borrow() {
                        Some(ActiveDrag::Pan { origin_x, origin_y }) => (origin_x, origin_y),
                        _ => return,
                    };
                    let (mx, my) = *gc2.0.pan_mouse.borrow();
                    const DEADZONE: f64 = 8.0;
                    const SPEED: f64 = 0.04;
                    let raw_dx = mx - ox;
                    let raw_dy = my - oy;
                    let dx = if raw_dx.abs() > DEADZONE { (raw_dx.abs() - DEADZONE) * raw_dx.signum() * SPEED * 16.0 } else { 0.0 };
                    let dy = if raw_dy.abs() > DEADZONE { (raw_dy.abs() - DEADZONE) * raw_dy.signum() * SPEED * 16.0 } else { 0.0 };
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
                cb2.forget();
                *gc.0.scroll_interval.borrow_mut() = Some(id);
                return;
            }

            let (x, y) = gc.canvas_xy(&evt);

            // Close any open context menu
            remove_ctx_menu();

            // ── scrollbar interaction ─────────────────────────────────────────
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
                    // Start thumb drag
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

            // ── horizontal scrollbar interaction ──────────────────────────────
            if let Some(hsb) = gc.hscrollbar() {
                if hsb.hit_left_arrow(x, y) {
                    let col_w = gc.0.state.borrow().model.columns
                        .first().map_or(40.0, |c| c.width);
                    gc.dispatch(GridCommand::ScrollBy { dx: -col_w, dy: 0.0 });
                    gc.start_scroll_repeat_x(-col_w);
                    return;
                }
                if hsb.hit_right_arrow(x, y) {
                    let col_w = gc.0.state.borrow().model.columns
                        .first().map_or(40.0, |c| c.width);
                    gc.dispatch(GridCommand::ScrollBy { dx: col_w, dy: 0.0 });
                    gc.start_scroll_repeat_x(col_w);
                    return;
                }
                if hsb.hit_thumb(x, y) {
                    *gc.0.drag.borrow_mut() = Some(ActiveDrag::HThumb(HThumbDrag {
                        start_client_x: evt.client_x() as f64,
                        start_scroll_x: gc.0.state.borrow().viewport.scroll_x,
                    }));
                    return;
                }
                if hsb.hit_track(x, y) {
                    let going_right = x > hsb.thumb_x + hsb.thumb_w * 0.5;
                    gc.start_htrack_scroll_repeat(x, going_right);
                    return;
                }
            }


            // ── row header selection ──────────────────────────────────────────
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

            // ── column resize separator (takes priority over col select) ─────
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

            // ── column header: sort on click, extend-select on shift+click ───
            let col = gc.0.state.borrow().hit_test_col_header(x, y);
            if let Some(col) = col {
                if evt.shift_key() {
                    gc.dispatch(GridCommand::ExtendColSelection(col));
                } else {
                    let key = gc
                        .0
                        .state
                        .borrow()
                        .model
                        .columns[col]
                        .key
                        .clone();
                    gc.dispatch(GridCommand::ToggleSort { col_key: key });
                }
                *gc.0.drag.borrow_mut() = Some(ActiveDrag::Col);
                return;
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
            .add_event_listener_with_callback(
                "mousedown",
                cb.as_ref().unchecked_ref(),
            )
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

                    gc.dispatch(GridCommand::ScrollTo {
                        x: 0.0,
                        y: start_scroll + scroll_delta,
                    });
                }
                Some(ActiveDrag::HThumb(ref ds)) => {
                    let dx = evt.client_x() as f64 - ds.start_client_x;
                    let start_scroll = ds.start_scroll_x;
                    drop(drag);
                    let scroll_delta = {
                        let s = gc.0.state.borrow();
                        let track_h = gc.0.builder.borrow().theme.scrollbar_width;
                        let vsb_w = if ScrollbarGeom::compute(
                            s.viewport.scroll_y, s.viewport.width, s.viewport.height,
                            s.model.header_height, s.model.total_height(), track_h,
                        ).is_some() { track_h } else { 0.0 };
                        if let Some(hsb) = HScrollbarGeom::compute(
                            s.viewport.scroll_x, s.viewport.width, s.viewport.height,
                            s.model.row_number_width, s.model.total_width(), vsb_w, track_h,
                        ) {
                            hsb.drag_to_scroll(dx, s.model.total_width(),
                                s.viewport.width, s.model.row_number_width, vsb_w)
                        } else { return; }
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
                Some(ActiveDrag::Pan { .. }) => {
                    drop(drag);
                    *gc.0.pan_mouse.borrow_mut() = (
                        evt.client_x() as f64,
                        evt.client_y() as f64,
                    );
                }
                Some(ActiveDrag::ColumnResize {
                    col_idx,
                    start_client_x,
                    start_width,
                }) => {
                    let (ci, scx, sw) =
                        (col_idx, start_client_x, start_width);
                    drop(drag);
                    let dx = evt.client_x() as f64 - scx;
                    gc.dispatch(GridCommand::ResizeColumn {
                        col_idx: ci,
                        new_width: sw + dx,
                    });
                }
                None => {
                    drop(drag);
                    let (x, y) = gc.canvas_xy(&evt);
                    // Cursor: col-resize near header separators, else default
                    if gc.hit_col_resize_separator(x, y).is_some() {
                        gc.set_cursor("col-resize");
                    } else {
                        gc.set_cursor("default");
                    }
                    // Hover row highlight
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
            .unwrap();
        self.0.doc_listeners.borrow_mut().push(("mousemove".to_string(), f));
        cb.forget();
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
            // Restore cursor after a column-resize drag.
            if matches!(
                *gc.0.drag.borrow(),
                Some(ActiveDrag::ColumnResize { .. })
            ) {
                gc.set_cursor("default");
            }
            *gc.0.drag.borrow_mut() = None;
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("mouseup", &f)
            .unwrap();
        self.0.doc_listeners.borrow_mut().push(("mouseup".to_string(), f));
        cb.forget();
    }

    fn attach_keydown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
            let key = evt.key();
            let shift = evt.shift_key();
            let ctrl = evt.ctrl_key() || evt.meta_key();
            match key.as_str() {
                "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" => {
                    if !gc.0.state.borrow().selection.has_selection() {
                        return;
                    }
                    evt.prevent_default();
                    let (dr, dc) = match key.as_str() {
                        "ArrowUp" => (-1_i64, 0_i64),
                        "ArrowDown" => (1, 0),
                        "ArrowLeft" => (0, -1),
                        "ArrowRight" => (0, 1),
                        _ => unreachable!(),
                    };
                    gc.dispatch(GridCommand::MoveSelection {
                        delta_row: dr,
                        delta_col: dc,
                        extend: shift,
                    });
                }
                "c" if ctrl => {
                    if !gc.0.state.borrow().selection.has_selection() {
                        return;
                    }
                    evt.prevent_default();
                    gc.handle_copy();
                }
                "Escape" => {
                    remove_ctx_menu();
                    gc.dispatch(GridCommand::ClearSelection);
                }
                _ => {}
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("keydown", &f)
            .unwrap();
        self.0.doc_listeners.borrow_mut().push(("keydown".to_string(), f));
        cb.forget();
    }

    fn attach_paste(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(
            move |evt: web_sys::ClipboardEvent| {
                if !gc.0.state.borrow().selection.has_selection() {
                    return;
                }
                if let Some(dt) = evt.clipboard_data() {
                    if let Ok(text) = dt.get_data("text/plain") {
                        evt.prevent_default();
                        gc.dispatch(GridCommand::PasteAt { text });
                    }
                }
            },
        );
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("paste", &f)
            .unwrap();
        self.0.doc_listeners.borrow_mut().push(("paste".to_string(), f));
        cb.forget();
    }

    /// Remove all document-level event listeners and disconnect the ResizeObserver.
    ///
    /// Call this when the grid is unmounted to prevent listener accumulation.
    pub fn detach(&self) {
        let doc = document();
        for (event, f) in self.0.doc_listeners.borrow().iter() {
            let _ = doc.remove_event_listener_with_callback(event, f);
        }
        self.0.doc_listeners.borrow_mut().clear();
        if let Some(ro) = self.0._resize_observer.borrow_mut().take() {
            ro.disconnect();
        }
    }

    /// Update the theme in-place without remounting the grid.
    pub fn set_theme(&self, theme: Theme) {
        let _ = self.0.canvas
            .style()
            .set_property("background-color", &theme.bg.to_css());
        self.0.builder.borrow_mut().theme = theme;
        self.render();
    }
}

// ── DOM helpers ───────────────────────────────────────────────────────────────

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

fn make_el(doc: &web_sys::Document, tag: &str) -> HtmlElement {
    doc.create_element(tag)
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap()
}

fn set_styles(el: &HtmlElement, styles: &[(&str, &str)]) {
    let s = el.style();
    for (prop, val) in styles {
        s.set_property(prop, val).unwrap();
    }
}

// ── context-menu icons (Feather Icons) ───────────────────────────────────────

const ICON_CUT: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/>"#,
    r#"<line x1="20" y1="4" x2="8.12" y2="15.88"/>"#,
    r#"<line x1="14.47" y1="14.48" x2="20" y2="20"/>"#,
    r#"<line x1="8.12" y1="8.12" x2="12" y2="12"/></svg>"#
);
const ICON_COPY: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>"#,
    r#"<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>"#,
    r#"</svg>"#
);
const ICON_PASTE: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6"#,
    r#" a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>"#,
    r#"<rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>"#
);

fn make_menu_separator(doc: &web_sys::Document) -> HtmlElement {
    let sep = make_el(doc, "div");
    set_styles(
        &sep,
        &[("border-top", "1px solid #e5e7eb"), ("margin", "4px 0")],
    );
    sep
}

fn make_menu_item(
    doc: &web_sys::Document,
    icon: &str,
    label: &str,
    shortcut: &str,
    enabled: bool,
) -> HtmlElement {
    let item = make_el(doc, "div");
    let row = make_el(doc, "div");
    set_styles(
        &row,
        &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", "8px"),
            ("padding", "6px 12px"),
        ],
    );
    let icon_el = make_el(doc, "span");
    set_styles(
        &icon_el,
        &[
            ("width", "16px"),
            ("height", "16px"),
            ("display", "flex"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("flex-shrink", "0"),
            ("opacity", "0.6"),
        ],
    );
    icon_el.set_inner_html(icon);
    let label_el = make_el(doc, "span");
    set_styles(&label_el, &[("flex", "1")]);
    label_el.set_text_content(Some(label));
    let sc_el = make_el(doc, "span");
    set_styles(
        &sc_el,
        &[("color", "#9ca3af"), ("font-size", "11px"),
          ("white-space", "nowrap")],
    );
    sc_el.set_text_content(Some(shortcut));
    row.append_child(&icon_el).unwrap();
    row.append_child(&label_el).unwrap();
    row.append_child(&sc_el).unwrap();
    item.append_child(&row).unwrap();

    let (color, cursor) = if enabled {
        ("#111827", "pointer")
    } else {
        ("#9ca3af", "default")
    };
    set_styles(&item, &[("color", color), ("cursor", cursor)]);
    if enabled {
        let item_over = item.clone();
        let cb_over = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_over
                .style()
                .set_property("background", "#f3f4f6")
                .unwrap();
        });
        item.add_event_listener_with_callback(
            "mouseover",
            cb_over.as_ref().unchecked_ref(),
        )
        .unwrap();
        cb_over.forget();

        let item_out = item.clone();
        let cb_out = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_out.style().set_property("background", "").unwrap();
        });
        item.add_event_listener_with_callback(
            "mouseout",
            cb_out.as_ref().unchecked_ref(),
        )
        .unwrap();
        cb_out.forget();
    }
    item
}

fn remove_ctx_menu() {
    let doc = document();
    if let Some(el) = doc.get_element_by_id("rs-grid-ctx-backdrop") {
        el.remove();
    }
    if let Some(el) = doc.get_element_by_id("rs-grid-ctx-menu") {
        el.remove();
    }
}
