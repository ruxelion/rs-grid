//! Dioxus CSR component integration for rs-grid.
//!
//! Provides a `GridCanvas` component that mounts a grid
//! into the DOM and keeps it in sync with Dioxus reactive
//! signals. Top of the dependency chain (`core → scene →
//! render-canvas → web → **dioxus**`).
//!
//! Re-exports: [`WebGridCanvas`], [`Locale`],
//! [`theme_from_css_vars`].

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

use dioxus::prelude::*;
use rs_grid_core::{model::GridModel, state::GridState};
use rs_grid_scene::Theme;
/// Re-exported so callers can name the type in `on_mount`
/// closures without depending on `rs-grid-web` directly.
pub use rs_grid_web::GridCanvas as WebGridCanvas;
pub use rs_grid_web::{theme_from_css_vars, Locale};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

/// Monotonic counter for unique canvas element IDs,
/// allowing multiple `GridCanvas` instances on one page.
static CANVAS_ID: AtomicU32 = AtomicU32::new(0);

/// Wrapper for passing a non-Clone `GridModel` into the
/// `GridCanvas` component. Dioxus 0.7 requires all props
/// to be `Clone + PartialEq`; this wrapper satisfies both
/// via `Rc` sharing.
///
/// # Example
/// ```ignore
/// let model = build_model(rows, cols);
/// rsx! {
///     GridCanvas {
///         model: ModelSlot::new(model),
///     }
/// }
/// ```
#[derive(Clone)]
pub struct ModelSlot(Rc<RefCell<Option<GridModel>>>);

impl ModelSlot {
    /// Wrap a `GridModel` for passing as a component prop.
    pub fn new(model: GridModel) -> Self {
        Self(Rc::new(RefCell::new(Some(model))))
    }

    fn take(&self) -> Option<GridModel> {
        self.0.borrow_mut().take()
    }
}

impl PartialEq for ModelSlot {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// A Dioxus component that renders an rs-grid onto a
/// `<canvas>` element.
///
/// # Props
/// - `model`: A [`ModelSlot`] wrapping the `GridModel`.
/// - `width`: CSS width (e.g. `"100%"` or `"800px"`).
/// - `height`: CSS height (e.g. `"600px"`).
/// - `theme`: Optional reactive `Signal<Theme>`. Changes are applied in-place
///   via `set_theme()`.
/// - `locale`: Optional reactive `Signal<Locale>`. Changes are applied in-place
///   via `set_locale()`.
/// - `on_mount`: Called once after mount with the `GridCanvas` handle.
/// - `on_validation_error`: Called when a validator rejects an edit. Args:
///   `(row, col_key, message)`.
#[component]
pub fn GridCanvas(
    model: ModelSlot,
    #[props(default = "100%".into())] width: String,
    #[props(default = "600px".into())] height: String,
    #[props(optional)] theme: Option<Signal<Theme>>,
    #[props(optional)] locale: Option<Signal<Locale>>,
    #[props(default)] on_mount: EventHandler<WebGridCanvas>,
    #[props(default)] on_validation_error: EventHandler<(u64, String, String)>,
) -> Element {
    // Unique canvas id for this component instance.
    let canvas_id = use_hook(|| {
        let n = CANVAS_ID.fetch_add(1, Ordering::Relaxed);
        format!("rs-grid-canvas-{n}")
    });

    // Shared handle to the mounted GridCanvas.
    let gc_holder: Rc<RefCell<Option<rs_grid_web::GridCanvas>>> =
        use_hook(|| Rc::new(RefCell::new(None))).clone();
    let gc_for_theme = gc_holder.clone();
    let gc_for_locale = gc_holder.clone();
    let gc_for_cleanup = gc_holder.clone();

    // Reactive theme: when the signal changes, update
    // in-place. Skip the first run (mount already
    // applied the initial theme).
    if let Some(theme_sig) = theme {
        let first_run = use_hook(|| Cell::new(true));
        use_effect(move || {
            let t = theme_sig.read().clone();
            if first_run.replace(false) {
                return;
            }
            if let Some(gc) = gc_for_theme.borrow().as_ref() {
                gc.set_theme(t);
            }
        });
    }

    // Reactive locale: same pattern as theme.
    if let Some(locale_sig) = locale {
        let first_run = use_hook(|| Cell::new(true));
        use_effect(move || {
            let l = locale_sig.read().clone();
            if first_run.replace(false) {
                return;
            }
            if let Some(gc) = gc_for_locale.borrow().as_ref() {
                gc.set_locale(l);
            }
        });
    }

    // Detach listeners when this component unmounts.
    use_drop(move || {
        if let Some(gc) = gc_for_cleanup.borrow().as_ref() {
            gc.detach();
        }
    });

    let cid = canvas_id.clone();

    rsx! {
        canvas {
            id: "{canvas_id}",
            style: "width:{width};height:{height};\
                    display:block",
            onmounted: move |_| {
                // Retrieve the canvas by id — simpler
                // and more reliable than downcasting
                // MountedData across Dioxus versions.
                let Some(canvas) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| {
                        d.get_element_by_id(&cid)
                    })
                    .and_then(|el| {
                        el.dyn_into::<HtmlCanvasElement>()
                            .ok()
                    })
                else {
                    return;
                };

                // Take model on first mount;
                // subsequent calls are no-ops.
                let Some(m) = model.take() else {
                    return;
                };

                let mount_theme = theme
                    .map(|s| s.peek().clone())
                    .unwrap_or_else(
                        rs_grid_web::theme_from_css_vars,
                    );
                let mount_locale = locale
                    .map(|s| s.peek().clone())
                    .unwrap_or_default();

                let rect =
                    canvas.get_bounding_client_rect();
                let win = web_sys::window()
                    .expect("no window");

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

                let state = GridState::new(m, w, h);
                let gc =
                    rs_grid_web::GridCanvas::mount(
                        canvas,
                        state,
                        mount_theme,
                        mount_locale,
                    );
                *gc_holder.borrow_mut() =
                    Some(gc.clone());

                let ve = on_validation_error;
                gc.set_on_validation_error(
                    move |row, col, msg| {
                        ve.call((
                            row,
                            col.to_string(),
                            msg.to_string(),
                        ));
                    },
                );

                on_mount.call(gc);
            },
        }
    }
}
