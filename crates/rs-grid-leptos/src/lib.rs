//! Leptos component integration for rs-grid.
//!
//! Provides a `<GridCanvas>` component that mounts a `GridCanvas` into the DOM
//! and keeps it in sync with Leptos reactive signals.

pub use rs_grid_web::theme_from_css_vars;
/// Re-exported so callers can name the type in `on_mount` closures without
/// depending on `rs-grid-web` directly.
pub use rs_grid_web::GridCanvas as WebGridCanvas;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use leptos::prelude::*;
use rs_grid_core::{model::GridModel, state::GridState};
use rs_grid_scene::Theme;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// A Leptos component that renders an rs-grid onto a `<canvas>` element.
///
/// # Props
/// - `model`: The `GridModel` to display.
/// - `width`: CSS width string (e.g. `"100%"` or `"800px"`).
/// - `height`: CSS height string (e.g. `"600px"`).
/// - `theme`: Optional reactive `Signal<Theme>`. When supplied, theme changes
///   are applied in-place via `set_theme()` without remounting the grid.
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[prop(default = "100%".into())] width: String,
    #[prop(default = "600px".into())] height: String,
    #[prop(optional)] theme: Option<Signal<Theme>>,
    /// Called once after the grid is mounted with a cloned handle to the
    /// underlying `GridCanvas`. Use it to call `set_on_change`,
    /// `import_patches`, or `export_patches`.
    #[prop(optional)]
    on_mount: Option<Box<dyn FnOnce(rs_grid_web::GridCanvas)>>,
    /// Called when a per-column validator rejects an edit.
    /// Arguments: `(row, col_key, error_message)`.
    #[prop(optional)]
    on_validation_error: Option<Box<dyn Fn(u64, String, String)>>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // Wrap in RefCell<Option> so the model can be moved into the Effect
    // on first run without requiring GridModel: Clone.  This avoids
    // a panic when the data source is an FnDataSource (which is not cloneable).
    let model_slot = RefCell::new(Some(model));
    let on_mount_slot = RefCell::new(on_mount);
    let on_validation_error_slot = RefCell::new(on_validation_error);

    // Holder for the mounted GridCanvas handle, shared across effects and cleanup.
    // SendWrapper allows Rc<RefCell<...>> to satisfy Send+Sync for on_cleanup;
    // it is safe because WASM is single-threaded and the value never actually
    // crosses thread boundaries.
    let gc_holder: Rc<RefCell<Option<rs_grid_web::GridCanvas>>> =
        Rc::new(RefCell::new(None));
    let gc_for_theme = gc_holder.clone();
    let gc_for_cleanup = SendWrapper::new(gc_holder.clone());

    Effect::new(move |_| {
        let Some(canvas_el) = canvas_ref.get() else {
            return;
        };

        // Take the model out on the first run; subsequent runs are no-ops.
        let Some(model) = model_slot.borrow_mut().take() else {
            return;
        };

        // Read the initial theme: prefer the signal's current value, fall back
        // to CSS vars (so mount works without a theme prop too).
        let mount_theme = theme
            .map(|s| s.get_untracked())
            .unwrap_or_else(rs_grid_web::theme_from_css_vars);

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
        let gc = rs_grid_web::GridCanvas::mount(canvas, state, mount_theme);
        *gc_holder.borrow_mut() = Some(gc.clone());
        if let Some(cb) = on_validation_error_slot.borrow_mut().take() {
            gc.set_on_validation_error(move |row, col, msg| {
                cb(row, col.to_string(), msg.to_string());
            });
        }
        if let Some(cb) = on_mount_slot.borrow_mut().take() {
            cb(gc);
        }
    });

    // Reactive theme effect: when the theme signal changes,
    // update in-place without remounting. Skip the first run
    // because mount() already applied the initial theme.
    if let Some(theme_sig) = theme {
        let first_run = Cell::new(true);
        Effect::new(move |_| {
            let t = theme_sig.get();
            if first_run.replace(false) {
                return; // mount already applied this theme
            }
            if let Some(gc) = gc_for_theme.borrow().as_ref() {
                gc.set_theme(t);
            }
        });
    }

    // Detach document listeners when this component is unmounted.
    on_cleanup(move || {
        if let Some(gc) = gc_for_cleanup.borrow().as_ref() {
            gc.detach();
        }
        drop(gc_for_cleanup);
    });

    view! {
        <canvas
            node_ref=canvas_ref
            style=format!("width:{};height:{};display:block", width, height)
        />
    }
}
