mod clipboard;
mod context_menu;
mod dom_helpers;
mod edit;
mod events;
mod scroll;

use std::{cell::RefCell, rc::Rc};

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;
use rs_grid_scene::{builder::SceneBuilder, Theme};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlInputElement, MouseEvent, ResizeObserver,
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
    /// Optional callback fired after every command that mutates cell data.
    on_change: RefCell<Option<Box<dyn Fn()>>>,
    /// DOM `<input>` element used for inline cell editing.
    edit_input: RefCell<Option<HtmlInputElement>>,
    /// Text waiting to be placed on the clipboard by the next
    /// `copy` event (set by context-menu copy/cut before
    /// triggering `execCommand("copy")`).
    pending_clipboard: RefCell<Option<String>>,
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
            on_change: RefCell::new(None),
            edit_input: RefCell::new(None),
            pending_clipboard: RefCell::new(None),
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
        let is_mutation = matches!(
            cmd,
            GridCommand::PasteAt { .. } | GridCommand::CommitEdit { .. }
        );
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
}
