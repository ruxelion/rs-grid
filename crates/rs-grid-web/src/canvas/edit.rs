use rs_grid_core::commands::GridCommand;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlInputElement, KeyboardEvent};

use super::dom_helpers::document;
use super::GridCanvas;

impl GridCanvas {
    /// Viewport rectangle `(x, y, w, h)` of a cell.
    fn cell_viewport_rect(
        &self,
        row: u64,
        col_idx: usize,
    ) -> (f64, f64, f64, f64) {
        let state = self.0.state.borrow();
        let model = &state.model;
        let off = model.column_offsets.offsets[col_idx];
        let cx = if col_idx < model.pinned_count {
            off + model.row_number_width
        } else {
            off - state.viewport.scroll_x + model.row_number_width
        };
        let cy = model.row_top(row) - state.viewport.scroll_y;
        let w = model.columns[col_idx].width;
        let h = model.row_height;
        (cx, cy, w, h)
    }

    /// Create a DOM `<input>` overlay for inline cell editing.
    pub(super) fn show_edit_input(&self) {
        self.remove_edit_input();

        let (row, col_key) = {
            let state = self.0.state.borrow();
            let edit = match &state.edit {
                Some(e) => e,
                None => return,
            };
            (edit.row, edit.col_key.clone())
        };

        let col_idx = {
            let state = self.0.state.borrow();
            match state.model.columns.iter().position(|c| c.key == col_key) {
                Some(i) => i,
                None => return,
            }
        };

        let (cx, cy, w, h) = self.cell_viewport_rect(row, col_idx);
        let canvas_rect = self.0.canvas.get_bounding_client_rect();
        let left = canvas_rect.left() + cx;
        let top = canvas_rect.top() + cy;

        let doc = document();
        let input: HtmlInputElement = doc
            .create_element("input")
            .expect("create input")
            .dyn_into()
            .expect("cast");

        let initial = self
            .0
            .state
            .borrow()
            .edit
            .as_ref()
            .map(|e| e.initial_value.clone())
            .unwrap_or_default();
        input.set_value(&initial);

        let style = input.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("width", &format!("{w}px"));
        let _ = style.set_property("height", &format!("{h}px"));
        let _ = style.set_property("z-index", "10000");
        let _ = style.set_property("border", "2px solid #2563eb");
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("padding", "0 4px");
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("box-sizing", "border-box");
        let _ = style.set_property("font", "inherit");
        let _ = style.set_property("background", "#fff");

        doc.body().expect("body").append_child(&input).unwrap();
        let _ = input.focus();
        let _ = input.select();

        // Enter → commit
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key.clone();
            let inp = input.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" => {
                            let val = inp.value();
                            gc.dispatch(GridCommand::CommitEdit {
                                row: r,
                                col_key: ck.clone(),
                                value: val,
                            });
                            gc.remove_edit_input();
                        }
                        "Escape" => {
                            gc.dispatch(GridCommand::CancelEdit);
                            gc.remove_edit_input();
                        }
                        _ => {}
                    },
                );
            input
                .add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }

        // Blur → commit
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key;
            let inp = input.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(move |_: web_sys::FocusEvent| {
                    if gc.0.state.borrow().edit.is_some() {
                        let val = inp.value();
                        gc.dispatch(GridCommand::CommitEdit {
                            row: r,
                            col_key: ck.clone(),
                            value: val,
                        });
                        gc.remove_edit_input();
                    }
                });
            input
                .add_event_listener_with_callback(
                    "blur",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }

        *self.0.edit_input.borrow_mut() = Some(input);
    }

    /// Remove the inline edit input from the DOM.
    pub(super) fn remove_edit_input(&self) {
        if let Some(input) = self.0.edit_input.borrow_mut().take() {
            input.remove();
        }
    }
}
