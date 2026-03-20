use rs_grid_core::{
    commands::{CommandOutput, GridCommand},
    selection::CopyError,
};

use super::GridCanvas;

impl GridCanvas {
    pub(super) fn handle_copy(&self) {
        match self.dispatch_with_output(GridCommand::CopySelection) {
            CommandOutput::CopyText(text) => self.write_clipboard(text),
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Copy annulé : {actual} lignes sélectionnées \
                     (max {max})"
                );
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                    &msg,
                ));
            }
            _ => {}
        }
    }

    pub(super) fn handle_cut(&self) {
        match self.dispatch_with_output(GridCommand::CutSelection) {
            CommandOutput::CopyText(text) => self.write_clipboard(text),
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Cut annulé : {actual} lignes sélectionnées \
                     (max {max})"
                );
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                    &msg,
                ));
            }
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
        match self.dispatch_with_output(GridCommand::CopySelection) {
            CommandOutput::CopyText(data) => {
                self.write_clipboard(format!("{header_row}\n{data}"));
            }
            CommandOutput::CopyError(CopyError::TooManyRows {
                actual,
                max,
            }) => {
                let msg = format!(
                    "Copy annulé : {actual} lignes sélectionnées \
                     (max {max})"
                );
                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(
                    &msg,
                ));
            }
            _ => {}
        }
    }

    pub(super) fn write_clipboard(&self, text: String) {
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
}
