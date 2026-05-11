mod animation;
mod clipboard;
mod context_menu;
pub mod context_menu_config;
mod dispatch;
mod dom_helpers;
mod edit;
mod events;
pub mod fetcher;
mod hittest;
mod keyboard;
mod scroll;
mod search;

use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

use rs_grid_core::{
    commands::GridCommand, page_cache::PageCacheDataSource, state::GridState,
};
use rs_grid_render_canvas::renderer::CanvasRenderer;

use fetcher::FetchConfig;
use rs_grid_scene::{
    builder::SceneBuilder, class_map::ClassResolver, frame::SceneFrame, Theme,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, HtmlInputElement, ResizeObserver,
};

use crate::locale::Locale;

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
    /// The last rendered frame, retained so button zones can be
    /// hit-tested on mousedown without recomputing geometry.
    last_frame: RefCell<Option<SceneFrame>>,
    /// Optional callback fired after every command that mutates cell data.
    ///
    /// Stored as `Rc` so the dispatch path can clone it out of the
    /// `RefCell` borrow before invoking — that lets callbacks safely
    /// dispatch further commands without re-entrant borrow panics.
    on_change: RefCell<Option<Rc<dyn Fn()>>>,
    /// Optional callback fired after every command that mutates column
    /// layout (resize, move, auto-fit, pin count).
    on_columns_changed: RefCell<Option<Rc<dyn Fn()>>>,
    /// Optional callback fired after every command that mutates the
    /// selection rectangle (single click, shift-extend, row/col select,
    /// clear, arrow-key move).
    on_selection_changed: RefCell<Option<Rc<dyn Fn()>>>,
    /// Optional callback fired when a validator rejects a cell edit.
    /// Arguments: (row, col_key, error_message).
    #[allow(clippy::type_complexity)]
    on_validation_error: RefCell<Option<Rc<dyn Fn(u64, &str, &str)>>>,
    /// Optional callback fired when a cell button is clicked.
    /// Arguments: (row, col_key, button_id).
    #[allow(clippy::type_complexity)]
    on_cell_button_click: RefCell<Option<Rc<dyn Fn(u64, &str, &str)>>>,
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
    /// Locale strings for UI chrome (context menu, search bar).
    locale: RefCell<Locale>,
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
        start_flex: Option<f64>,
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
    /// # Parameters
    ///
    /// - `canvas`: The target `<canvas>` DOM element. Its CSS
    ///   dimensions must already be set (e.g. via `style`).
    ///   `mount()` reads `clientWidth`/`clientHeight` to
    ///   compute the physical size and sets `canvas.width`
    ///   and `canvas.height` accordingly (CSS size × DPR).
    ///
    /// - `state`: The initial grid state. **This value is
    ///   mutated before it is stored**: three commands are
    ///   applied synchronously —
    ///   [`GridCommand::Resize`] (viewport from CSS
    ///   dimensions), [`GridCommand::SetHeaderHeight`] and
    ///   [`GridCommand::SetRowHeight`] (both from `theme`).
    ///
    /// - `theme`: Visual configuration (colours, row height,
    ///   header height, font). Use `theme_from_css_vars()`
    ///   to read values from CSS custom properties.
    ///
    /// - `locale`: UI string translations. Use
    ///   [`Locale::default()`] for English.
    ///
    /// # Side-effects
    ///
    /// - Sets `canvas.style.background-color` from
    ///   `theme.bg` to prevent transparent flashes.
    /// - Sets `tabindex="0"` and `outline: none` so the
    ///   canvas can receive keyboard events.
    /// - Attaches canvas-level listeners: `wheel`,
    ///   `mousedown`, `mouseleave`, `dblclick`,
    ///   `contextmenu`, `keydown`, `copy`, `cut`, `paste`.
    /// - Attaches document-level listeners: `mousemove`,
    ///   `mouseup`.
    /// - Installs a `ResizeObserver` that keeps the
    ///   physical canvas size in sync with CSS layout.
    /// - Renders the first frame synchronously before
    ///   returning.
    ///
    /// # Cleanup
    ///
    /// Call [`GridCanvas::detach()`] when unmounting to
    /// remove all event listeners and the
    /// `ResizeObserver`. Failing to call `detach()` will
    /// leak listeners for the lifetime of the page.
    ///
    /// # Panics
    ///
    /// Panics if there is no `window` object or if the
    /// `"2d"` canvas context cannot be obtained.
    pub fn mount(
        canvas: HtmlCanvasElement,
        mut state: GridState,
        theme: Theme,
        locale: Locale,
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
            last_frame: RefCell::new(None),
            on_change: RefCell::new(None),
            on_columns_changed: RefCell::new(None),
            on_selection_changed: RefCell::new(None),
            on_validation_error: RefCell::new(None),
            on_cell_button_click: RefCell::new(None),
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
            locale: RefCell::new(locale),
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
        // Retain the frame so button zones can be hit-tested
        // on the next mousedown.
        *self.0.last_frame.borrow_mut() = Some(frame);
        if flash.is_some() || drag_anim || momentum {
            self.render();
        }
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

    /// Update the locale in-place without remounting the grid.
    ///
    /// The new strings take effect the next time the context
    /// menu or search bar is opened.
    pub fn set_locale(&self, locale: Locale) {
        *self.0.locale.borrow_mut() = locale;
    }

    /// Set the CSS class resolver used for `CellFormat::Styled`.
    ///
    /// Call this once after mounting — typically from the `on_mount`
    /// callback — to wire up a framework-specific resolver such as
    /// the DaisyUI resolver from `example-common`.
    ///
    /// ```ignore
    /// on_mount: Box::new(|gc| {
    ///     gc.set_class_resolver(Rc::new(example_common::class_map::resolve_classes));
    /// })
    /// ```
    pub fn set_class_resolver(&self, resolver: Rc<ClassResolver>) {
        self.0.builder.borrow_mut().set_class_resolver(resolver);
        self.render();
    }

    // ── column layout readers ────────────────────────────────────────────────

    /// Snapshot of `(col_key, width)` for every column in their current
    /// display order. Use this from `on_columns_changed` callbacks to persist
    /// user-resized layouts.
    pub fn column_widths(&self) -> Vec<(String, f64)> {
        self.0
            .state
            .borrow()
            .model
            .columns
            .iter()
            .map(|c| (c.key.clone(), c.width))
            .collect()
    }

    /// Snapshot of every column key in current display order. Reflects
    /// `MoveColumn` re-orderings made by the user.
    pub fn column_order(&self) -> Vec<String> {
        self.0
            .state
            .borrow()
            .model
            .columns
            .iter()
            .map(|c| c.key.clone())
            .collect()
    }

    /// Current count of pinned (frozen) columns.
    pub fn pinned_count(&self) -> usize {
        self.0.state.borrow().model.pinned_count
    }

    /// Physical row indices currently inside the selection rectangle.
    ///
    /// Returns an empty `Vec` when nothing is selected. The selection model
    /// is anchor + focus (a single rectangle), so the result is always a
    /// contiguous range — non-contiguous multi-select is not supported yet.
    ///
    /// Use together with [`GridCanvas::set_on_selection_changed`] to drive
    /// row-level toolbars (bulk delete, bulk approve, …).
    pub fn selected_row_indices(&self) -> Vec<u64> {
        let sel = &self.0.state.borrow().selection;
        match sel.range() {
            Some((tl, br)) => (tl.row..=br.row).collect(),
            None => Vec::new(),
        }
    }

    // ── public API for new v1 features ────────────────────────────────────────

    /// Set the number of pinned (frozen) columns.
    pub fn set_pinned_count(&self, count: usize) {
        self.dispatch(GridCommand::SetPinnedColumnCount { count });
    }

    /// Enable or disable inline cell editing grid-wide.
    pub fn set_editable(&self, editable: bool) {
        self.dispatch(GridCommand::SetEditable(editable));
    }

    /// Enable or disable cell/row/column selection grid-wide.
    pub fn set_selectable(&self, selectable: bool) {
        self.dispatch(GridCommand::SetSelectable(selectable));
    }

    /// Enable or disable header drag-to-reorder of columns.
    /// Programmatic `MoveColumn` commands are unaffected.
    pub fn set_column_reorderable(&self, reorderable: bool) {
        self.dispatch(GridCommand::SetColumnReorderable(reorderable));
    }

    /// Show or hide the column header row.
    pub fn set_show_header(&self, show: bool) {
        self.dispatch(GridCommand::SetShowHeader(show));
    }

    /// Show or hide the row-number gutter.
    pub fn set_show_row_numbers(&self, show: bool) {
        self.dispatch(GridCommand::SetShowRowNumbers(show));
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
