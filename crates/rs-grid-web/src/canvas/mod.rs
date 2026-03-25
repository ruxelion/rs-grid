mod clipboard;
mod context_menu;
pub mod context_menu_config;
mod dom_helpers;
mod edit;
mod events;
pub mod fetcher;
mod keyboard;
mod scroll;
mod search;

use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    page_cache::PageCacheDataSource,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;

use fetcher::FetchConfig;
use rs_grid_scene::{
    builder::{ColumnDragHint, FlashHint, SceneBuilder},
    Theme,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, HtmlInputElement, MouseEvent,
    ResizeObserver,
};

use dom_helpers::document;

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
    #[allow(clippy::type_complexity)]
    _resize_closure: RefCell<Option<Closure<dyn FnMut(js_sys::Array)>>>,
    _resize_observer: RefCell<Option<ResizeObserver>>,
    /// Stored references to document-level listeners so they can be removed on detach.
    doc_listeners: RefCell<Vec<(String, js_sys::Function)>>,
    /// Stored references to canvas-level listeners for removal on detach.
    canvas_listeners: RefCell<Vec<(String, js_sys::Function)>>,
    /// Closures backing the event listeners — stored here to
    /// prevent `forget()`-style leaks and to break the
    /// `Rc<Inner>` cycle on `detach()`.
    closures: RefCell<Vec<Box<dyn Any>>>,
    /// Closures backing scroll timers (setTimeout /
    /// setInterval) — cleared on `stop_scroll_repeat`.
    scroll_closures: RefCell<Vec<Box<dyn Any>>>,
    /// Whether a render is already scheduled via rAF.
    raf_scheduled: Cell<bool>,
    /// Scroll momentum velocity in logical px/frame (mouse wheel only).
    /// Decays each frame via exponential smoothing until negligible.
    scroll_vx: Cell<f64>,
    scroll_vy: Cell<f64>,
    /// Column index whose header menu icon is currently hovered, if any.
    hovered_menu_col: RefCell<Option<usize>>,
    /// Current animated column offsets during a drag
    /// (`col_idx → cumulative left offset`).
    /// Initialised from real positions when drag starts and
    /// lerped toward the preview target each frame.
    drag_col_offsets: RefCell<Vec<f64>>,
    /// Active flash-cells animation, if any.
    flash: RefCell<Option<FlashState>>,
    /// Optional callback fired after every command that mutates cell data.
    on_change: RefCell<Option<Box<dyn Fn()>>>,
    /// Optional callback fired when a validator rejects a cell edit.
    /// Arguments: (row, col_key, error_message).
    #[allow(clippy::type_complexity)]
    on_validation_error: RefCell<Option<Box<dyn Fn(u64, &str, &str)>>>,
    /// DOM element used for inline cell editing (`<input>` or `<select>`).
    edit_input: RefCell<Option<HtmlElement>>,
    /// Closures on the edit `<input>` (keydown, blur).
    edit_closures: RefCell<Vec<Box<dyn Any>>>,
    /// Event listener refs on the edit element, for explicit removal.
    edit_listener_refs: RefCell<Vec<(String, js_sys::Function)>>,
    /// DOM `<input>` element for the search bar (Ctrl+F).
    search_input: RefCell<Option<HtmlInputElement>>,
    /// Closures on the search `<input>` (input, keydown).
    search_closures: RefCell<Vec<Box<dyn Any>>>,
    /// Event listener refs on the search input, for explicit removal.
    search_listener_refs: RefCell<Vec<(String, js_sys::Function)>>,
    /// Text waiting to be placed on the clipboard by the next
    /// `copy` event (set by context-menu copy/cut before
    /// triggering `execCommand("copy")`).
    pending_clipboard: RefCell<Option<String>>,
    /// Context menu configuration (items + overrides).
    ctx_menu_config: RefCell<context_menu_config::ContextMenuConfig>,
    /// Shared page cache for async data sources (None for client-side).
    page_cache: RefCell<Option<PageCacheDataSource>>,
    /// Async fetch configuration (None = no async fetching).
    fetch_config: RefCell<Option<FetchConfig>>,
}

enum ActiveDrag {
    Thumb(ThumbDrag),
    HThumb(HThumbDrag),
    Cell,
    Row,
    /// Shift-drag extend column selection.
    Col,
    /// Single click on column header — deferred sort on mouseup,
    /// upgrades to `ColumnDrag` if mouse moves > 5 px.
    ColClick {
        col_idx: usize,
        start_client_x: f64,
    },
    /// Column reorder drag (activated from `ColClick`).
    ColumnDrag {
        col_idx: usize,
        /// Current viewport-relative X of the cursor.
        current_vx: f64,
        /// Current viewport-relative Y of the cursor.
        current_vy: f64,
    },
    Pan {
        origin_x: f64,
        origin_y: f64,
    },
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

/// Tracks an active flash-cells animation.
struct FlashState {
    /// `performance.now()` timestamp when the flash was triggered (ms).
    start_ms: f64,
    /// Total fade duration in milliseconds.
    duration_ms: f64,
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

        // Make the canvas focusable so document-level keydown /
        // clipboard handlers can check whether *this* grid owns
        // focus before intercepting the event.
        canvas.set_attribute("tabindex", "0").expect("set tabindex");
        // Remove the default focus outline — the grid draws its
        // own selection indicators.
        let _ = canvas.style().set_property("outline", "none");

        state.apply(GridCommand::Resize {
            width: css_w,
            height: css_h,
        });
        state.apply(GridCommand::SetHeaderHeight(theme.header_height));
        state.apply(GridCommand::SetRowHeight(theme.row_height));

        // Opaque canvas (alpha: false) enables sub-pixel text
        // rendering (ClearType on Windows, LCD on macOS).
        let ctx_opts = js_sys::Object::new();
        js_sys::Reflect::set(&ctx_opts, &"alpha".into(), &false.into())
            .expect("set alpha");
        let ctx = canvas
            .get_context_with_context_options("2d", &ctx_opts)
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
            canvas_listeners: RefCell::new(Vec::new()),
            closures: RefCell::new(Vec::new()),
            scroll_closures: RefCell::new(Vec::new()),
            raf_scheduled: Cell::new(false),
            scroll_vx: Cell::new(0.0),
            scroll_vy: Cell::new(0.0),
            hovered_menu_col: RefCell::new(None),
            drag_col_offsets: RefCell::new(Vec::new()),
            flash: RefCell::new(None),
            on_change: RefCell::new(None),
            on_validation_error: RefCell::new(None),
            edit_input: RefCell::new(None),
            edit_closures: RefCell::new(Vec::new()),
            edit_listener_refs: RefCell::new(Vec::new()),
            search_input: RefCell::new(None),
            search_closures: RefCell::new(Vec::new()),
            search_listener_refs: RefCell::new(Vec::new()),
            pending_clipboard: RefCell::new(None),
            ctx_menu_config: RefCell::new(
                context_menu_config::ContextMenuConfig::default(),
            ),
            page_cache: RefCell::new(None),
            fetch_config: RefCell::new(None),
        });

        let gc = GridCanvas(inner);
        gc.attach_listeners();
        gc.attach_resize_observer();
        gc.render_immediate();
        gc
    }

    /// Schedule a render on the next animation frame.
    ///
    /// Multiple calls within the same frame are coalesced —
    /// only one scene build + canvas draw happens per frame.
    pub fn render(&self) {
        if self.0.raf_scheduled.get() {
            return;
        }
        self.0.raf_scheduled.set(true);
        let gc = self.clone();
        // `once_into_js` transfers ownership to JS; the
        // captured values are dropped after the callback
        // fires — no leak.
        let cb = Closure::once_into_js(move || {
            gc.0.raf_scheduled.set(false);
            gc.render_immediate();
        });
        let win = web_sys::window().expect("no window");
        let _ = win.request_animation_frame(cb.unchecked_ref());
    }

    /// Render the current state synchronously (used by
    /// `mount` and the rAF callback).
    fn render_immediate(&self) {
        // Advance column-drag animation one step; returns true
        // while columns are still moving toward their targets.
        let drag_anim = self.step_drag_animation();
        // Apply one frame of scroll momentum (dispatches ScrollBy
        // internally); returns true while still decelerating.
        let momentum = self.step_scroll_momentum();
        let state = self.0.state.borrow();
        let hint = self.column_drag_hint();
        let flash = self.compute_flash_hint();
        let hovered_menu = *self.0.hovered_menu_col.borrow();
        let frame = self.0.builder.borrow().build(
            &state,
            hint.as_ref(),
            flash.as_ref(),
            hovered_menu,
        );
        drop(state);
        self.0.renderer.render(&frame);
        if flash.is_some() || drag_anim || momentum {
            self.render();
        }
    }

    /// Apply one frame of scroll momentum and decay the velocity.
    ///
    /// Returns `true` while the velocity is still significant
    /// (caller should schedule another rAF frame).
    fn step_scroll_momentum(&self) -> bool {
        let vx = self.0.scroll_vx.get();
        let vy = self.0.scroll_vy.get();
        if vx.abs() < 0.5 && vy.abs() < 0.5 {
            self.0.scroll_vx.set(0.0);
            self.0.scroll_vy.set(0.0);
            return false;
        }
        // ~300 ms deceleration at 60 fps.
        const FRICTION: f64 = 0.88;
        self.0.scroll_vx.set(vx * FRICTION);
        self.0.scroll_vy.set(vy * FRICTION);
        // dispatch triggers page fetches for async data sources.
        self.dispatch(GridCommand::ScrollBy { dx: vx, dy: vy });
        true
    }

    /// Lerp `drag_col_offsets` one step toward the preview target.
    ///
    /// Returns `true` while columns have not yet reached their
    /// target positions (caller should schedule another rAF).
    fn step_drag_animation(&self) -> bool {
        // Only active during a ColumnDrag.
        let (src, vx) = {
            let drag = self.0.drag.borrow();
            match *drag {
                Some(ActiveDrag::ColumnDrag {
                    col_idx,
                    current_vx,
                    ..
                }) => (col_idx, current_vx),
                _ => return false,
            }
        };

        let ins = self.insertion_index(vx);

        // Compute target offsets for the prospective drop position.
        let target: Vec<f64> = {
            let state = self.0.state.borrow();
            let cols = &state.model.columns;
            if src >= cols.len() {
                return false;
            }
            let dst = if ins > src {
                ins.saturating_sub(1)
            } else {
                ins
            };
            let mut order: Vec<usize> =
                (0..cols.len()).filter(|&i| i != src).collect();
            let at = dst.min(order.len());
            order.insert(at, src);
            let mut offs = vec![0.0_f64; cols.len()];
            let mut cum = 0.0_f64;
            for &ci in &order {
                offs[ci] = cum;
                cum += cols[ci].width;
            }
            offs
        };

        let mut anim = self.0.drag_col_offsets.borrow_mut();
        if anim.len() != target.len() {
            // Offsets not yet initialised — use target as starting
            // point (events.rs should have initialised from real
            // positions, but guard against races).
            *anim = target;
            return false;
        }

        // Exponential smoothing — alpha from the theme CSS var
        // `--rs-grid-drag-anim-alpha` (default 0.30 ≈ 200 ms).
        let alpha = self.0.builder.borrow().theme.drag_anim_alpha;
        let mut settled = true;
        for (a, &t) in anim.iter_mut().zip(target.iter()) {
            let diff = t - *a;
            if diff.abs() < 0.5 {
                *a = t;
            } else {
                *a += diff * alpha;
                settled = false;
            }
        }
        !settled
    }

    /// Compute the current flash alpha factor, clearing the flash
    /// state when expired. Returns `None` when no flash is active.
    fn compute_flash_hint(&self) -> Option<FlashHint> {
        let mut flash = self.0.flash.borrow_mut();
        let f = flash.as_ref()?;
        let now = web_sys::window()
            .expect("no window")
            .performance()
            .expect("no performance")
            .now();
        let elapsed = now - f.start_ms;
        if elapsed >= f.duration_ms {
            *flash = None;
            return None;
        }
        let alpha_factor = 1.0 - elapsed / f.duration_ms;
        Some(FlashHint { alpha_factor })
    }

    /// Trigger a brief golden-yellow flash on the currently selected cells.
    ///
    /// No-op if there is no active selection. Multiple calls restart
    /// the animation from full intensity.
    pub fn flash_selection(&self) {
        if !self.0.state.borrow().selection.has_selection() {
            return;
        }
        let now = web_sys::window()
            .expect("no window")
            .performance()
            .expect("no performance")
            .now();
        *self.0.flash.borrow_mut() = Some(FlashState {
            start_ms: now,
            duration_ms: 400.0,
        });
        self.render();
    }

    /// Apply a command, redraw, and return the output.
    fn dispatch_with_output(&self, cmd: GridCommand) -> CommandOutput {
        // Run per-column validator before committing a cell edit.
        if let GridCommand::CommitEdit {
            row,
            ref col_key,
            ref value,
        } = cmd
        {
            let validation_result = {
                let state = self.0.state.borrow();
                state
                    .model
                    .columns
                    .iter()
                    .find(|c| c.key == *col_key)
                    .and_then(|c| c.validator.as_ref())
                    .map(|v| v.validate(value))
            };
            if let Some(Err(msg)) = validation_result {
                self.0
                    .state
                    .borrow_mut()
                    .apply(GridCommand::CancelEdit);
                self.render();
                if let Some(cb) =
                    self.0.on_validation_error.borrow().as_ref()
                {
                    cb(row, col_key, &msg);
                }
                return CommandOutput::None;
            }
        }

        // Commands that write cell data — fire the on_change callback
        // so JS callers can react (e.g. mark the document as dirty).
        let is_mutation = matches!(
            cmd,
            GridCommand::PasteAt { .. } | GridCommand::CommitEdit { .. }
        );
        // Commands that may expose new rows — trigger a page fetch in
        // server-side pagination mode (PageCacheDataSource).
        let triggers_fetch = matches!(
            cmd,
            GridCommand::ScrollTo { .. }
                | GridCommand::ScrollBy { .. }
                | GridCommand::Resize { .. }
                | GridCommand::NotifyPageLoaded
                | GridCommand::ToggleSort { .. }
                | GridCommand::SetColumnFilter { .. }
                | GridCommand::ClearAllFilters
        );
        // In server-side mode, sort/filter changes
        // invalidate the entire page cache.
        let invalidates_cache = matches!(
            cmd,
            GridCommand::ToggleSort { .. }
                | GridCommand::SetColumnFilter { .. }
                | GridCommand::ClearAllFilters
        );
        if invalidates_cache {
            if let Some(cache) = self.0.page_cache.borrow().as_ref() {
                cache.clear();
            }
        }
        let out = self.0.state.borrow_mut().apply(cmd);
        if let CommandOutput::SortWarning { row_count, limit } = &out {
            web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                &format!(
                    "rs-grid: sort skipped — {row_count} rows exceeds \
                     the {limit}-row client-side limit. Use a \
                     server-side data source for large datasets."
                ),
            ));
        }
        self.render();
        if is_mutation {
            if let Some(cb) = self.0.on_change.borrow().as_ref() {
                cb();
            }
        }
        if triggers_fetch {
            self.maybe_fetch_pages();
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

    /// Register a callback fired when a per-column validator rejects an
    /// edit. Arguments: `(row, col_key, error_message)`.
    pub fn set_on_validation_error(
        &self,
        cb: impl Fn(u64, &str, &str) + 'static,
    ) {
        *self.0.on_validation_error.borrow_mut() = Some(Box::new(cb));
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
        // Unescape in two passes: first stash literal `\\` as the
        // NUL sentinel so `\\t` is not mistaken for a tab, then
        // restore it at the end.
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

    /// Returns `true` when this grid should handle keyboard /
    /// clipboard events.  The grid "has focus" when the active
    /// element is its canvas, its inline edit `<input>`, or its
    /// search `<input>`.
    fn has_focus(&self) -> bool {
        use wasm_bindgen::JsCast;
        let Some(active) = document().active_element() else {
            return false;
        };
        // Compare via the underlying Element pointer.
        let active_el: &web_sys::Element = &active;
        if let Some(canvas_el) = self.0.canvas.dyn_ref::<web_sys::Element>() {
            if active_el == canvas_el {
                return true;
            }
        }
        if let Some(ref el) = *self.0.edit_input.borrow() {
            if let Some(input_el) = el.dyn_ref::<web_sys::Element>() {
                if active_el == input_el {
                    return true;
                }
            }
        }
        if let Some(ref el) = *self.0.search_input.borrow() {
            if let Some(input_el) = el.dyn_ref::<web_sys::Element>() {
                if active_el == input_el {
                    return true;
                }
            }
        }
        false
    }

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
        if vy >= model.header_height {
            return None;
        }
        if vx < model.row_number_width {
            return None;
        }
        let scroll_x = state.viewport.scroll_x;
        let rnw = model.row_number_width;
        let pinned = model.pinned_count;
        for (i, col) in model.columns.iter().enumerate() {
            let off = model.column_offsets.offsets[i] + col.width;
            let sep_vx = if i < pinned {
                off + rnw
            } else {
                off - scroll_x + rnw
            };
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
        if vy < model.header_height {
            return None;
        }
        if vx < 0.0 || vx > state.viewport.width {
            return None;
        }
        let abs_y = vy - model.header_height + state.viewport.scroll_y;
        let row = (abs_y / model.row_height) as u64;
        if row < model.display_row_count() {
            Some(row)
        } else {
            None
        }
    }

    /// Compute which column gap the cursor is closest to.
    /// Returns the index to insert *before* (0..=columns.len()).
    fn insertion_index(&self, vx: f64) -> usize {
        let state = self.0.state.borrow();
        let model = &state.model;
        let sx = state.viewport.scroll_x;
        let rnw = model.row_number_width;
        let pinned = model.pinned_count;
        let len = model.columns.len();

        let edge_vx = |i: usize| -> f64 {
            if i < len {
                let off = model.column_offsets.offsets[i];
                if i < pinned {
                    off + rnw
                } else {
                    off - sx + rnw
                }
            } else {
                let last = len - 1;
                let off = model.column_offsets.offsets[last]
                    + model.columns[last].width;
                if last < pinned {
                    off + rnw
                } else {
                    off - sx + rnw
                }
            }
        };

        let mut best_idx = 0;
        let mut best_dist = f64::MAX;
        for i in 0..=len {
            let d = (vx - edge_vx(i)).abs();
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Returns `Some(col_idx)` when `(vx, vy)` falls inside the
    /// three-dot menu icon zone at the right edge of a column header.
    fn hit_header_menu_icon(&self, vx: f64, vy: f64) -> Option<usize> {
        let col_idx =
            self.0.state.borrow().hit_test_col_header(vx, vy)?;
        let theme = self.0.builder.borrow();
        let mr = theme.theme.header_menu_icon_margin_r;
        let bw = theme.theme.header_menu_icon_btn_w;
        drop(theme);
        let state = self.0.state.borrow();
        let model = &state.model;
        let off = model.column_offsets.offsets[col_idx];
        let sx = state.viewport.scroll_x;
        let rnw = model.row_number_width;
        let col_left_vx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - sx + rnw
        };
        let col_right_vx = col_left_vx + model.columns[col_idx].width;
        if vx >= col_right_vx - mr - bw && vx < col_right_vx - mr {
            Some(col_idx)
        } else {
            None
        }
    }

    /// Build a `ColumnDragHint` from the current drag state,
    /// or `None` if no column drag is active.
    fn column_drag_hint(&self) -> Option<ColumnDragHint> {
        let drag = self.0.drag.borrow();
        match *drag {
            Some(ActiveDrag::ColumnDrag {
                col_idx,
                current_vx,
                current_vy,
            }) => {
                drop(drag);
                let insert = self.insertion_index(current_vx);
                let animated_offsets =
                    self.0.drag_col_offsets.borrow().clone();
                Some(ColumnDragHint {
                    source_col: col_idx,
                    insert_before: insert,
                    cursor_vx: current_vx,
                    cursor_vy: current_vy,
                    animated_offsets,
                })
            }
            _ => None,
        }
    }

    /// Remove all event listeners, disconnect the
    /// ResizeObserver, and drop stored closures.
    ///
    /// Call this when the grid is unmounted to prevent
    /// listener accumulation and break the `Rc` cycle.
    pub fn detach(&self) {
        // 1. Remove document-level listeners.
        let doc = document();
        for (event, f) in self.0.doc_listeners.borrow().iter() {
            let _ = doc.remove_event_listener_with_callback(event, f);
        }
        self.0.doc_listeners.borrow_mut().clear();

        // 2. Remove canvas-level listeners.
        for (event, f) in self.0.canvas_listeners.borrow().iter() {
            let _ = self.0.canvas.remove_event_listener_with_callback(event, f);
        }
        self.0.canvas_listeners.borrow_mut().clear();

        // 3. Disconnect ResizeObserver + drop its closure.
        if let Some(ro) = self.0._resize_observer.borrow_mut().take() {
            ro.disconnect();
        }
        self.0._resize_closure.borrow_mut().take();

        // 4. Stop any running scroll timers.
        self.stop_scroll_repeat();

        // 5. Remove ephemeral overlays (edit / search).
        self.remove_edit_input();
        self.remove_search_input();

        // 6. Drop all stored closures (invalidates their JS
        //    functions and breaks Rc<Inner> cycles).
        self.0.closures.borrow_mut().clear();
        self.0.scroll_closures.borrow_mut().clear();
    }

    /// Update the theme in-place without remounting the grid.
    pub fn set_theme(&self, theme: Theme) {
        let _ = self
            .0
            .canvas
            .style()
            .set_property("background-color", &theme.bg.to_css());
        self.0.builder.borrow_mut().theme = theme;
        self.render();
    }

    // ── public API for new v1 features ────────────────────────────────────────

    /// Set the number of pinned (frozen) columns.
    pub fn set_pinned_count(&self, count: usize) {
        self.dispatch(GridCommand::SetPinnedColumnCount { count });
    }

    /// Set a text filter on a column (case-insensitive contains).
    /// Pass an empty string to clear the filter for that column.
    pub fn set_filter(&self, col_key: &str, text: &str) {
        self.dispatch(GridCommand::SetColumnFilter {
            col_key: col_key.to_string(),
            text: text.to_string(),
        });
    }

    /// Remove all column filters.
    pub fn clear_filters(&self) {
        self.dispatch(GridCommand::ClearAllFilters);
    }

    /// Enable async page-based data fetching.
    ///
    /// Call this after `mount()` when the `GridModel` uses a
    /// `PageCacheDataSource`. The fetch coordinator will
    /// start loading pages immediately.
    pub fn enable_async_fetch(
        &self,
        cache: PageCacheDataSource,
        config: FetchConfig,
    ) {
        *self.0.page_cache.borrow_mut() = Some(cache);
        *self.0.fetch_config.borrow_mut() = Some(config);
        self.maybe_fetch_pages();
    }

    /// Replace the context menu configuration.
    pub fn set_context_menu(
        &self,
        config: context_menu_config::ContextMenuConfig,
    ) {
        *self.0.ctx_menu_config.borrow_mut() = config;
    }
}
