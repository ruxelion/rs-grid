use rs_grid_core::commands::GridCommand;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlInputElement, KeyboardEvent};

use super::dom_helpers::document;
use super::GridCanvas;

impl GridCanvas {
    /// Show the floating search bar above the canvas.
    pub(super) fn show_search_input(&self) {
        // Already open — just re-focus.
        if let Some(input) = self.0.search_input.borrow().as_ref() {
            let _ = input.focus();
            input.select();
            return;
        }

        let canvas_rect = self.0.canvas.get_bounding_client_rect();
        let left = canvas_rect.right() - 260.0;
        let top = canvas_rect.top() + 4.0;

        let doc = document();
        let input: HtmlInputElement = doc
            .create_element("input")
            .expect("create input")
            .dyn_into()
            .expect("cast");

        input.set_placeholder("Find\u{2026}");

        let style = input.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("width", "250px");
        let _ = style.set_property("height", "28px");
        let _ = style.set_property("z-index", "10001");
        let _ = style.set_property("border", "1px solid #babfc7");
        let _ = style.set_property("border-radius", "4px");
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("padding", "0 8px");
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("box-sizing", "border-box");
        let _ = style.set_property("font", "13px system-ui, sans-serif");
        let _ = style.set_property("background", "#fff");
        let _ = style.set_property("box-shadow", "0 2px 8px rgba(0,0,0,.15)");

        doc.body()
            .expect("body")
            .append_child(&input)
            .expect("append search input");
        let _ = input.focus();

        // Input → search on every keystroke
        {
            let gc = self.clone();
            let inp = input.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                let query = inp.value();
                gc.dispatch(GridCommand::Search { query });
            });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("input", &func)
                .expect("add input listener");
            self.0.search_listener_refs.borrow_mut().push(("input".into(), func));
            self.0.search_closures.borrow_mut().push(Box::new(cb));
        }

        // Keydown → Enter=next, Shift+Enter=prev, Escape=close
        {
            let gc = self.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" => {
                            evt.prevent_default();
                            if evt.shift_key() {
                                gc.dispatch(GridCommand::SearchPrev);
                            } else {
                                gc.dispatch(GridCommand::SearchNext);
                            }
                        }
                        "Escape" => {
                            gc.dispatch(GridCommand::ClearSearch);
                            gc.remove_search_input();
                        }
                        _ => {}
                    },
                );
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("keydown", &func)
                .expect("add keydown listener");
            self.0.search_listener_refs.borrow_mut().push(("keydown".into(), func));
            self.0.search_closures.borrow_mut().push(Box::new(cb));
        }

        *self.0.search_input.borrow_mut() = Some(input);
    }

    /// Remove the search bar from the DOM and drop its closures.
    ///
    /// Explicitly calls `removeEventListener` before removal to avoid
    /// dangling Rust closure references on the JS side.
    pub(super) fn remove_search_input(&self) {
        if let Some(input) = self.0.search_input.borrow().as_ref() {
            for (event, func) in self.0.search_listener_refs.borrow().iter() {
                let _ = input.remove_event_listener_with_callback(event, func);
            }
        }
        self.0.search_listener_refs.borrow_mut().clear();
        if let Some(input) = self.0.search_input.borrow_mut().take() {
            input.remove();
        }
        self.0.search_closures.borrow_mut().clear();
    }
}
