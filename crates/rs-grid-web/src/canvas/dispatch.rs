use std::rc::Rc;

use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    selection::CellCoord,
};
use wasm_bindgen::{JsCast, JsValue};

use super::{FlashState, GridCanvas, dom_helpers::document};

/// Commands that may expose new rows — trigger a page fetch in
/// server-side pagination mode (`PageCacheDataSource`).
///
/// Every sort/filter variant must be listed here: skipping one leaves
/// that command's in-memory state updated (and, for sort, silently
/// reflected in `GridState`) without ever re-fetching the server-backed
/// page, which is indistinguishable from the command doing nothing.
fn triggers_fetch(cmd: &GridCommand) -> bool {
    matches!(
        cmd,
        GridCommand::ScrollTo { .. }
            | GridCommand::ScrollBy { .. }
            | GridCommand::Resize { .. }
            | GridCommand::NotifyPageLoaded
            | GridCommand::ToggleSort { .. }
            | GridCommand::SetSort { .. }
            | GridCommand::ClearSort
            | GridCommand::SetColumnFilter { .. }
            | GridCommand::ClearAllFilters
    )
}

/// In server-side mode, sort/filter changes invalidate the entire page
/// cache. Must stay in sync with [`triggers_fetch`] for every sort/filter
/// variant — a command that invalidates the cache but doesn't also
/// trigger a fetch would leave the cache empty until the next unrelated
/// fetch.
fn invalidates_cache(cmd: &GridCommand) -> bool {
    matches!(
        cmd,
        GridCommand::ToggleSort { .. }
            | GridCommand::SetSort { .. }
            | GridCommand::ClearSort
            | GridCommand::SetColumnFilter { .. }
            | GridCommand::ClearAllFilters
    )
}

impl GridCanvas {
    /// Trigger a brief golden-yellow flash on exactly `cells`.
    ///
    /// No-op for an empty slice. Multiple calls restart the animation
    /// from full intensity, replacing the previous cell set. Callers
    /// should pass the cells actually written by a mutation (e.g.
    /// `CommandOutput::PasteApplied`'s or `CommandOutput::CellsCleared`'s
    /// `cells`), not a selection rectangle, which may extend past cells
    /// that were skipped.
    pub fn flash_cells(&self, cells: &[CellCoord]) {
        if cells.is_empty() {
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
            cells: cells.iter().map(|c| (c.row, c.col)).collect(),
        });
        self.render();
    }

    /// Apply a command, redraw, and return the output.
    pub(super) fn dispatch_with_output(
        &self,
        cmd: GridCommand,
    ) -> CommandOutput {
        // Commands that write cell data — fire the on_change callback
        // so JS callers can react (e.g. mark the document as dirty).
        let is_mutation = matches!(
            cmd,
            GridCommand::PasteAt { .. }
                | GridCommand::CommitEdit { .. }
                | GridCommand::ClearCells
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
        // Commands that mutate `checked_rows` — fire the
        // on_checked_rows_changed callback so JS callers can react to row
        // checkbox toggles (e.g. show a bulk-action toolbar).
        let is_checked_rows_change = matches!(
            cmd,
            GridCommand::ToggleRowChecked(_)
                | GridCommand::ToggleAllFilteredChecked
        );
        // Commands that may expose new rows — trigger a page fetch in
        // server-side pagination mode (PageCacheDataSource).
        let triggers_fetch = triggers_fetch(&cmd);
        // Commands that change the active edit session's validation
        // state — fire on_validation_state_changed with the fresh
        // value so a consumer can drive a live, custom validation UI.
        let is_edit_state_change = matches!(
            cmd,
            GridCommand::StartEdit { .. }
                | GridCommand::ValidateEdit { .. }
                | GridCommand::CommitEdit { .. }
                | GridCommand::CancelEdit
        );
        // In server-side mode, sort/filter changes
        // invalidate the entire page cache.
        let invalidates_cache = invalidates_cache(&cmd);
        if invalidates_cache
            && let Some(cache) = self.0.page_cache.borrow().as_ref()
        {
            cache.clear();
        }
        let out = self.0.state.borrow_mut().apply(cmd);
        if let CommandOutput::SortWarning { row_count, limit } = &out {
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "rs-grid: sort skipped — {row_count} rows exceeds \
                 the {limit}-row client-side limit. Use a \
                 server-side data source for large datasets."
            )));
        }
        let validation_error = if let CommandOutput::ValidationError {
            row,
            col_key,
            message,
        } = &out
        {
            Some((*row, col_key.clone(), message.clone()))
        } else {
            None
        };
        self.render();
        // Clone the `Rc` out of each `RefCell` before invoking — that
        // releases the borrow, so a callback that re-dispatches a command
        // of the same kind won't re-borrow the cell and panic.
        // A rejected CommitEdit (ValidationError) never wrote to the
        // model, so it must not fire on_change.
        if is_mutation && validation_error.is_none() {
            let cb = self.0.on_change.borrow().clone();
            if let Some(cb) = cb {
                cb();
            }
        }
        if let Some((row, col_key, message)) = &validation_error {
            let cb = self.0.on_validation_error.borrow().clone();
            if let Some(cb) = cb {
                cb(*row, col_key, message);
            }
        }
        if is_edit_state_change {
            let cb = self.0.on_validation_state_changed.borrow().clone();
            if let Some(cb) = cb {
                cb(self.validation_error());
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
        if is_checked_rows_change {
            let cb = self.0.on_checked_rows_changed.borrow().clone();
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

    /// Register a callback fired after every command that mutates
    /// `checked_rows`: `ToggleRowChecked`, `ToggleAllFilteredChecked`. Use
    /// it together with [`GridCanvas::checked_row_indices`] to drive
    /// row-level bulk-action toolbars.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    pub fn set_on_checked_rows_changed(&self, cb: impl Fn() + 'static) {
        *self.0.on_checked_rows_changed.borrow_mut() = Some(Rc::new(cb));
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

    /// Register a callback fired after `StartEdit`, `ValidateEdit`,
    /// `CommitEdit`, or `CancelEdit` with the fresh
    /// [`GridCanvas::validation_error`] value.
    ///
    /// Unlike [`GridCanvas::set_on_validation_error`] (fired only when a
    /// commit is rejected), this fires on *every* keystroke while editing —
    /// use it to drive a custom validation UI (tooltip, banner, icon) built
    /// with your own framework/CSS, rs-grid does not impose one. Combine
    /// with [`GridCanvas::set_native_validation_tooltip`] to opt out of the
    /// built-in `title`-attribute fallback if it competes with your UI.
    ///
    /// # Re-entrancy
    ///
    /// Dispatching another `GridCommand` from inside this callback is
    /// safe (see [`GridCanvas::set_on_change`] for the mechanism).
    #[allow(clippy::type_complexity)]
    pub fn set_on_validation_state_changed(
        &self,
        cb: impl Fn(Option<(u64, String, String)>) + 'static,
    ) {
        *self.0.on_validation_state_changed.borrow_mut() = Some(Rc::new(cb));
    }

    /// Enable or disable the native `title` attribute on the inline edit
    /// `<input>` (default `true`). The attribute reflects the current
    /// validation error message, giving a zero-config browser tooltip.
    /// Disable it when building a custom validation UI via
    /// [`GridCanvas::set_on_validation_state_changed`] to avoid a competing
    /// native tooltip.
    pub fn set_native_validation_tooltip(&self, enabled: bool) {
        self.0.native_validation_tooltip.set(enabled);
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

#[cfg(test)]
mod tests {
    use rs_grid_core::{commands::GridCommand, sort::SortDir};

    use super::{invalidates_cache, triggers_fetch};

    /// Regression test for the bug where `SetSort` (menu ⋮ → "Sort
    /// Ascending"/"Descending") and `ClearSort` updated `GridState`
    /// in-memory but never re-fetched the server-backed page, unlike
    /// `ToggleSort` (direct header click) which did both.
    #[test]
    fn set_sort_and_clear_sort_trigger_fetch_and_invalidate_cache() {
        let set_sort = GridCommand::SetSort {
            col_key: "name".into(),
            dir: SortDir::Asc,
        };
        let clear_sort = GridCommand::ClearSort;
        let toggle_sort = GridCommand::ToggleSort {
            col_key: "name".into(),
        };

        for cmd in [&set_sort, &clear_sort, &toggle_sort] {
            assert!(triggers_fetch(cmd), "{cmd:?} should trigger a fetch");
            assert!(
                invalidates_cache(cmd),
                "{cmd:?} should invalidate the page cache"
            );
        }
    }
}
