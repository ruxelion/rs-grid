use rs_grid_core::commands::GridCommand;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{ClipboardEvent, KeyboardEvent};

use super::context_menu::remove_ctx_menu;
use super::dom_helpers::document;
use super::GridCanvas;

impl GridCanvas {
    pub(super) fn attach_keydown(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
            // Only handle keys when this grid has focus.
            if !gc.has_focus() {
                return;
            }
            // Ctrl+F always opens search, even during edit.
            if (evt.ctrl_key() || evt.meta_key()) && evt.key() == "f" {
                evt.prevent_default();
                gc.show_search_input();
                return;
            }
            if gc.0.edit_input.borrow().is_some() {
                return;
            }
            let key = evt.key();
            let shift = evt.shift_key();
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
                "Escape" => {
                    if gc.0.state.borrow().edit.is_some() {
                        gc.dispatch(GridCommand::CancelEdit);
                        gc.remove_edit_input();
                    } else {
                        remove_ctx_menu();
                        gc.dispatch(GridCommand::ClearSelection);
                    }
                }
                "z" if evt.ctrl_key() || evt.meta_key() => {
                    evt.prevent_default();
                    gc.dispatch(GridCommand::Undo);
                }
                "y" if evt.ctrl_key() || evt.meta_key() => {
                    evt.prevent_default();
                    gc.dispatch(GridCommand::Redo);
                }
                _ => {}
            }
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("keydown", &f)
            .unwrap();
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("keydown".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    pub(super) fn attach_copy(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(
            move |evt: ClipboardEvent| {
                // Always handle pending clipboard (context-menu
                // copy via execCommand). Otherwise require focus.
                let has_pending = gc.0.pending_clipboard.borrow().is_some();
                if !has_pending && !gc.has_focus() {
                    return;
                }
                gc.on_copy_event(&evt);
            },
        );
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("copy", &f)
            .unwrap();
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("copy".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    pub(super) fn attach_cut(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(
            move |evt: ClipboardEvent| {
                let has_pending = gc.0.pending_clipboard.borrow().is_some();
                if !has_pending && !gc.has_focus() {
                    return;
                }
                gc.on_cut_event(&evt);
            },
        );
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        document()
            .add_event_listener_with_callback("cut", &f)
            .unwrap();
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("cut".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    pub(super) fn attach_paste(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(
            move |evt: ClipboardEvent| {
                if !gc.has_focus() {
                    return;
                }
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
        self.0
            .doc_listeners
            .borrow_mut()
            .push(("paste".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }
}
