use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    selection::CopyError,
};
use wasm_bindgen::JsCast;

use super::dom_helpers::document;
use super::GridCanvas;

impl GridCanvas {
    // ── native event handlers (Ctrl+C / Ctrl+X) ──────────

    /// Called from the document-level `copy` event.
    pub(super) fn on_copy_event(
        &self,
        evt: &web_sys::ClipboardEvent,
    ) {
        // Pending text from context-menu copy/cut takes priority.
        if let Some(text) =
            self.0.pending_clipboard.borrow_mut().take()
        {
            if let Some(dt) = evt.clipboard_data() {
                evt.prevent_default();
                let _ = dt.set_data("text/plain", &text);
            }
            return;
        }
        // Don't intercept while editing a cell.
        if self.0.edit_input.borrow().is_some() {
            return;
        }
        if !self.0.state.borrow().selection.has_selection() {
            return;
        }
        match self
            .dispatch_with_output(GridCommand::CopySelection)
        {
            CommandOutput::CopyText(text) => {
                if let Some(dt) = evt.clipboard_data() {
                    evt.prevent_default();
                    let _ =
                        dt.set_data("text/plain", &text);
                }
            }
            CommandOutput::CopyError(
                CopyError::TooManyRows { actual, max },
            ) => Self::warn_too_many("Copy", actual, max),
            _ => {}
        }
    }

    /// Called from the document-level `cut` event.
    pub(super) fn on_cut_event(
        &self,
        evt: &web_sys::ClipboardEvent,
    ) {
        // Pending text from context-menu cut takes priority.
        if let Some(text) =
            self.0.pending_clipboard.borrow_mut().take()
        {
            if let Some(dt) = evt.clipboard_data() {
                evt.prevent_default();
                let _ = dt.set_data("text/plain", &text);
            }
            return;
        }
        if self.0.edit_input.borrow().is_some() {
            return;
        }
        if !self.0.state.borrow().selection.has_selection() {
            return;
        }
        match self
            .dispatch_with_output(GridCommand::CutSelection)
        {
            CommandOutput::CopyText(text) => {
                if let Some(dt) = evt.clipboard_data() {
                    evt.prevent_default();
                    let _ =
                        dt.set_data("text/plain", &text);
                }
            }
            CommandOutput::CopyError(
                CopyError::TooManyRows { actual, max },
            ) => Self::warn_too_many("Cut", actual, max),
            _ => {}
        }
    }

    // ── context-menu actions ─────────────────────────────

    pub(super) fn handle_copy(&self) {
        match self
            .dispatch_with_output(GridCommand::CopySelection)
        {
            CommandOutput::CopyText(text) => {
                self.write_clipboard(text);
            }
            CommandOutput::CopyError(
                CopyError::TooManyRows { actual, max },
            ) => Self::warn_too_many("Copy", actual, max),
            _ => {}
        }
    }

    pub(super) fn handle_cut(&self) {
        match self
            .dispatch_with_output(GridCommand::CutSelection)
        {
            CommandOutput::CopyText(text) => {
                self.write_clipboard(text);
            }
            CommandOutput::CopyError(
                CopyError::TooManyRows { actual, max },
            ) => Self::warn_too_many("Cut", actual, max),
            _ => {}
        }
    }

    pub(super) fn handle_copy_headers(&self) {
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
        match self
            .dispatch_with_output(GridCommand::CopySelection)
        {
            CommandOutput::CopyText(data) => {
                self.write_clipboard(format!(
                    "{header_row}\n{data}"
                ));
            }
            CommandOutput::CopyError(
                CopyError::TooManyRows { actual, max },
            ) => Self::warn_too_many("Copy", actual, max),
            _ => {}
        }
    }

    // ── clipboard write (execCommand fallback) ───────────

    /// Write text to the clipboard without requiring a
    /// secure context.  Stores the text in
    /// `pending_clipboard`, creates a temporary `<textarea>`
    /// with a DOM selection, and fires
    /// `document.execCommand("copy")`.  The synchronous
    /// `copy` event handler picks up the pending text and
    /// writes it via `clipboardData.setData`.
    ///
    /// Falls back to the async Clipboard API when
    /// `execCommand` fails (e.g. the browser removed it).
    fn write_clipboard(&self, text: String) {
        *self.0.pending_clipboard.borrow_mut() = Some(text);

        let doc = document();
        let html_doc: web_sys::HtmlDocument =
            doc.clone().dyn_into().expect("cast to HtmlDocument");
        let ta: web_sys::HtmlTextAreaElement = doc
            .create_element("textarea")
            .expect("create textarea")
            .dyn_into()
            .expect("cast to textarea");
        ta.set_value(" ");
        let style = ta.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("left", "-9999px");
        let _ = style.set_property("top", "-9999px");
        let _ = style.set_property("opacity", "0");
        doc.body()
            .expect("no body")
            .append_child(&ta)
            .expect("append textarea");
        ta.select();
        let _ = html_doc.exec_command("copy");
        ta.remove();

        // If execCommand didn't trigger copy (pending still
        // set), fall back to async Clipboard API.
        if let Some(text) =
            self.0.pending_clipboard.borrow_mut().take()
        {
            let window = web_sys::window().expect("no window");
            let clipboard = window.navigator().clipboard();
            let promise = clipboard.write_text(&text);
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) =
                    wasm_bindgen_futures::JsFuture::from(
                        promise,
                    )
                    .await
                {
                    web_sys::console::warn_1(&e);
                }
            });
        }
    }

    fn warn_too_many(op: &str, actual: u64, max: u64) {
        let msg = format!(
            "{op} annulé : {actual} lignes sélectionnées \
             (max {max})"
        );
        web_sys::console::warn_1(
            &wasm_bindgen::JsValue::from_str(&msg),
        );
    }
}
