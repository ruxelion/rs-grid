//! Leptos component integration for rs-grid.
//!
//! Provides a `<GridCanvas>` component that mounts a `GridCanvas` into the DOM
//! and keeps it in sync with Leptos reactive signals.

use std::cell::RefCell;

use leptos::prelude::*;
use rs_grid_core::{model::GridModel, state::GridState};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// A Leptos component that renders an rs-grid onto a `<canvas>` element.
///
/// # Props
/// - `model`: The `GridModel` to display.
/// - `width`: CSS width string (e.g. `"100%"` or `"800px"`).
/// - `height`: CSS height string (e.g. `"600px"`).
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[prop(default = "100%".into())] width: String,
    #[prop(default = "600px".into())] height: String,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // Wrap in RefCell<Option> so the model can be moved into the Effect
    // on first run without requiring GridModel: Clone.  This avoids
    // a panic when the data source is an FnDataSource (which is not cloneable).
    let model_slot = RefCell::new(Some(model));

    Effect::new(move |_| {
        let Some(canvas_el) = canvas_ref.get() else {
            return;
        };

        // Take the model out on the first run; subsequent runs are no-ops.
        let Some(model) = model_slot.borrow_mut().take() else {
            return;
        };

        // getBoundingClientRect() is reliable even before first paint;
        // fall back to window dimensions if the element has no size yet.
        let rect = canvas_el.get_bounding_client_rect();
        let win = web_sys::window().expect("no window");

        let w = {
            let bw = rect.width();
            if bw > 0.0 {
                bw
            } else {
                win.inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(800.0)
            }
        };
        let h = {
            let bh = rect.height();
            if bh > 0.0 {
                bh
            } else {
                win.inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|h| h - 80.0)
                    .unwrap_or(600.0)
            }
        };

        let canvas: HtmlCanvasElement = canvas_el.unchecked_into();
        let state = GridState::new(model, w, h);
        rs_grid_web::GridCanvas::mount(canvas, state);
    });

    view! {
        <canvas
            node_ref=canvas_ref
            style=format!("width:{};height:{};display:block", width, height)
        />
    }
}
