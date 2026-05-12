use std::rc::Rc;

use rs_grid_core::commands::{CommandOutput, GridCommand};
use wasm_bindgen::{JsCast, JsValue};

use super::{dom_helpers::document, FlashState, GridCanvas};

impl GridCanvas {
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
    pub(super) fn dispatch_with_output(
        &self,
        cmd: GridCommand,
    ) -> CommandOutput {
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
                self.0.state.borrow_mut().apply(GridCommand::CancelEdit);
                self.render();
                let cb = self.0.on_validation_error.borrow().clone();
                if let Some(cb) = cb {
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
        // Commands that mutate column layout (width, order, pin count) —
        // fire the on_columns_changed callback so JS callers can persist
        // the user's per-grid layout preferences.
        let is_column_change = matches!(
            cmd,
            GridCommand::CommitColumnResize { .. }
                | GridCommand::MoveColumn { .. }
                | GridCommand::AutoFitColumn { .. }
                | GridCommand::AutoFitAllColumns { .. }
                | GridCommand::SetPinnedColumnCount { .. }
        );
        // Commands that mutate the selection rectangle — fire the
        // on_selection_changed callback so JS callers can react to
        // row/range selection (e.g. show a bulk-action toolbar).
        let is_selection_change = matches!(
            cmd,
            GridCommand::SelectCell(_)
                | GridCommand::ExtendSelection(_)
                | GridCommand::ClearSelection
                | GridCommand::MoveSelection { .. }
                | GridCommand::SelectRow(_)
                | GridCommand::ExtendRowSelection(_)
                | GridCommand::SelectCol(_)
                | GridCommand::ExtendColSelection(_)
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
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "rs-grid: sort skipped — {row_count} rows exceeds \
                 the {limit}-row client-side limit. Use a \
                 server-side data source for large datasets."
            )));
        }
        self.render();
        // Clone the `Rc` out of each `RefCell` before invoking — that
        // releases the borrow, so a callback that re-dispatches a command
        // of the same kind won't re-borrow the cell and panic.
        if is_mutation {
            let cb = self.0.on_change.borrow().clone();
            if let Some(cb) = cb {
                cb();
            }
        }
        if is_column_change {
            let cb = self.0.on_columns_changed.borrow().clone();
            if let Some(cb) = cb {
                cb();
            }
        }
        if is_selection_change {
            let cb = self.0.on_selection_changed.borrow().clone();
            if let Some(cb) = cb {
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

    /// Register a callback fired after every command that mutates cell data
    /// (edits, paste). Use it to persist patches or push to a backend.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe — the callback is held by `Rc` and cloned out of its cell
    /// before invocation, so the dispatch path has no live borrow when
    /// user code runs.
    pub fn set_on_change(&self, cb: impl Fn() + 'static) {
        *self.0.on_change.borrow_mut() = Some(Rc::new(cb));
    }

    /// Register a callback fired after every command that mutates column
    /// **layout**: `CommitColumnResize`, `MoveColumn`, `AutoFitColumn`,
    /// `AutoFitAllColumns`, `SetPinnedColumnCount`. Combine with
    /// [`GridCanvas::column_widths`], [`GridCanvas::column_order`] and
    /// [`GridCanvas::pinned_count`] to persist per-user grid layouts.
    ///
    /// # Scope
    ///
    /// Layout = widths, order, pin count. **Sort and filter state are NOT
    /// covered** by this callback — use `set_on_change` (or a custom
    /// solution) if you need to persist them too.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_columns_changed(&self, cb: impl Fn() + 'static) {
        *self.0.on_columns_changed.borrow_mut() = Some(Rc::new(cb));
    }

    /// Register a callback fired after every command that mutates the
    /// selection rectangle: `SelectCell`, `ExtendSelection`,
    /// `ClearSelection`, `MoveSelection`, `SelectRow`, `ExtendRowSelection`,
    /// `SelectCol`, `ExtendColSelection`. Use it together with
    /// [`GridCanvas::selected_row_indices`] to drive row-level toolbars.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_selection_changed(&self, cb: impl Fn() + 'static) {
        *self.0.on_selection_changed.borrow_mut() = Some(Rc::new(cb));
    }

    /// Register a callback fired when a per-column validator rejects an
    /// edit. Arguments: `(row, col_key, error_message)`.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_validation_error(
        &self,
        cb: impl Fn(u64, &str, &str) + 'static,
    ) {
        *self.0.on_validation_error.borrow_mut() = Some(Rc::new(cb));
    }

    /// Register a callback fired when a cell button is clicked.
    /// Arguments: `(row, col_key, button_id)`.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_cell_button_click(
        &self,
        cb: impl Fn(u64, &str, &str) + 'static,
    ) {
        *self.0.on_cell_button_click.borrow_mut() = Some(Rc::new(cb));
    }

    /// Register a callback invoked on document `"click"` events that
    /// originate **outside** this grid's canvas element.
    ///
    /// Typical use: dispatch `ClearSelection` to dismiss row highlights
    /// when the user clicks filter chips, action toolbars, or any other
    /// UI adjacent to the grid.
    ///
    /// The listener is registered on `document` and removed
    /// automatically when [`GridCanvas::detach()`] is called (e.g. on
    /// component unmount), so no manual cleanup is needed.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching a `GridCommand` from inside this callback is safe
    /// (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_outside_click(&self, cb: impl Fn() + 'static) {
        use wasm_bindgen::prelude::Closure;
        let canvas_node: web_sys::Node = self.0.canvas.clone().unchecked_into();
        let cb = Rc::new(cb);
        let closure = Closure::<dyn Fn(JsValue)>::new(move |ev: JsValue| {
            let target =
                js_sys::Reflect::get(&ev, &JsValue::from_str("target"))
                    .ok()
                    .filter(|v| !v.is_null() && !v.is_undefined());
            let on_canvas = target
                .and_then(|t| t.dyn_into::<web_sys::Node>().ok())
                .map(|n| canvas_node.is_same_node(Some(&n)))
                .unwrap_or(false);
            if !on_canvas {
                cb();
            }
        });
        let f: js_sys::Function =
            closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
        let _ = document().add_event_listener_with_callback("click", &f);
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("click".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(closure));
    }

    /// Serialize the current patch layer as versioned TSV text.
    ///
    /// Format:
    /// ```text
    /// rs-grid-patches/v1
    /// physical_row\tcol_key\tvalue
    /// ...
    /// ```
    ///
    /// The first line is a version header. Tab, newline, and backslash
    /// characters inside keys/values are escaped as `\t`, `\n`, `\\`.
    /// Pass the result to [`import_patches`] to restore the state.
    pub fn export_patches(&self) -> String {
        let state = self.0.state.borrow();
        let mut out = String::from("rs-grid-patches/v1\n");
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

    /// Deserialize TSV text produced by [`export_patches`] and apply
    /// it, replacing any existing patches. Triggers a redraw.
    ///
    /// Accepts both the current versioned format (`rs-grid-patches/v1`
    /// header) and legacy data without a header, so previously saved
    /// patches remain loadable after an upgrade.
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
        let mut lines = data.lines().peekable();
        // Skip version header if present; accept legacy headerless
        // format for backwards compatibility.
        if lines
            .peek()
            .map(|l| l.starts_with("rs-grid-patches/"))
            .unwrap_or(false)
        {
            lines.next();
        }
        let mut state = self.0.state.borrow_mut();
        state.model.patches.clear();
        for line in lines {
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
}
