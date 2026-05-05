use rs_grid_core::commands::{CommandOutput, GridCommand};
use wasm_bindgen::JsValue;

use super::{FlashState, GridCanvas};

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
                if let Some(cb) = self.0.on_validation_error.borrow().as_ref() {
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
            web_sys::console::warn_1(&JsValue::from_str(&format!(
                "rs-grid: sort skipped — {row_count} rows exceeds \
                 the {limit}-row client-side limit. Use a \
                 server-side data source for large datasets."
            )));
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

    /// Register a callback fired when a cell button is clicked.
    /// Arguments: `(row, col_key, button_id)`.
    pub fn set_on_cell_button_click(
        &self,
        cb: impl Fn(u64, &str, &str) + 'static,
    ) {
        *self.0.on_cell_button_click.borrow_mut() =
            Some(Box::new(cb));
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
            state
                .model
                .patches
                .insert((row, unescape(c)), unescape(v));
        }
        drop(state);
        self.render();
    }
}
