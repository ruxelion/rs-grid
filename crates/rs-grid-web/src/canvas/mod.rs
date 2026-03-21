mod clipboard;
mod context_menu;
pub mod context_menu_config;
mod dom_helpers;
mod edit;
mod events;
pub mod fetcher;
mod scroll;
mod search;

use std::{cell::RefCell, rc::Rc};

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    page_cache::PageCacheDataSource,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;

use fetcher::FetchConfig;
use rs_grid_scene::{
    builder::{ColumnDragHint, SceneBuilder},
    Theme,
};
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
    /// DOM `<input>` element for the search bar (Ctrl+F).
    search_input: RefCell<Option<HtmlInputElement>>,
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
            search_input: RefCell::new(None),
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
        gc.render();
        gc
    }

    /// Render the current state immediately.
    pub fn render(&self) {
        let state = self.0.state.borrow();
        let hint = self.column_drag_hint();
        let frame = self.0.builder.borrow().build(&state, hint.as_ref());
        self.0.renderer.render(&frame);
    }

    /// Apply a command, redraw, and return the output.
    fn dispatch_with_output(&self, cmd: GridCommand) -> CommandOutput {
        let is_mutation = matches!(
            cmd,
            GridCommand::PasteAt { .. } | GridCommand::CommitEdit { .. }
        );
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

    /// Build a `ColumnDragHint` from the current drag state,
    /// or `None` if no column drag is active.
    fn column_drag_hint(&self) -> Option<ColumnDragHint> {
        let drag = self.0.drag.borrow();
        match *drag {
            Some(ActiveDrag::ColumnDrag {
                col_idx,
                current_vx,
            }) => {
                drop(drag);
                let insert = self.insertion_index(current_vx);
                Some(ColumnDragHint {
                    source_col: col_idx,
                    insert_before: insert,
                    cursor_vx: current_vx,
                })
            }
            _ => None,
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
