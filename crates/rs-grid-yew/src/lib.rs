//! Yew CSR component integration for rs-grid.
//!
//! Provides a `GridCanvas` component that mounts a grid
//! into the DOM and keeps it in sync with Yew's reactive
//! state. Top of the dependency chain (`core → scene →
//! render-canvas → web → **yew**`).
//!
//! Re-exports: [`WebGridCanvas`], [`Locale`],
//! [`theme_from_css_vars`].

pub use rs_grid_web::theme_from_css_vars;
/// Re-exported so callers can name the type in callbacks
/// without depending on `rs-grid-web` directly.
pub use rs_grid_web::GridCanvas as WebGridCanvas;
pub use rs_grid_web::Locale;

use std::cell::RefCell;
use std::rc::Rc;

use rs_grid_core::{
    model::GridModel, state::GridState,
};
use rs_grid_scene::Theme;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;

/// Callback type for validation error events:
/// `(row, col_key, message)`.
pub type ValidationErrorCb =
    Rc<dyn Fn(u64, String, String)>;

/// Props for the [`GridCanvas`](GridCanvas) component.
#[derive(Properties)]
pub struct GridCanvasProps {
    /// Grid model (consumed on first mount).
    /// Wrap in `Rc<RefCell<Option<GridModel>>>` because
    /// Yew `Properties` requires `PartialEq`.
    pub model: Rc<RefCell<Option<GridModel>>>,
    /// CSS width. Defaults to `"100%"`.
    #[prop_or("100%".into())]
    pub width: AttrValue,
    /// CSS height. Defaults to `"600px"`.
    #[prop_or("600px".into())]
    pub height: AttrValue,
    /// Optional theme. When changed the grid updates
    /// in-place via `set_theme()`.
    #[prop_or_default]
    pub theme: Option<Theme>,
    /// Optional locale. When changed the grid updates
    /// in-place via `set_locale()`.
    #[prop_or_default]
    pub locale: Option<Locale>,
    /// Called once after the grid mounts.
    #[prop_or_default]
    pub on_mount: Option<Callback<WebGridCanvas>>,
    /// Called when a per-column validator rejects an edit.
    #[prop_or_default]
    pub on_validation_error: Option<ValidationErrorCb>,
}

impl PartialEq for GridCanvasProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.model, &other.model)
            && self.width == other.width
            && self.height == other.height
            && self.theme == other.theme
            && self.locale == other.locale
    }
}

/// A Yew function component that renders an rs-grid
/// onto a `<canvas>` element.
///
/// # Usage
/// ```ignore
/// use rs_grid_yew::{GridCanvas, wrap_model};
///
/// let model_slot = wrap_model(my_model);
/// html! {
///     <GridCanvas model={model_slot} />
/// }
/// ```
#[function_component]
pub fn GridCanvas(props: &GridCanvasProps) -> Html {
    let canvas_ref = use_node_ref();
    let gc_handle: Rc<
        RefCell<Option<rs_grid_web::GridCanvas>>,
    > = use_mut_ref(|| None);

    // Mount effect: runs once after the canvas DOM node
    // is available.
    {
        let canvas_ref = canvas_ref.clone();
        let gc_handle = gc_handle.clone();
        let model_slot = props.model.clone();
        let theme = props.theme.clone();
        let locale = props.locale.clone();
        let on_mount = props.on_mount.clone();
        let on_ve = props.on_validation_error.clone();

        use_effect_with((), move |_| {
            let gc_cleanup = gc_handle.clone();

            if let Some(canvas) =
                canvas_ref.cast::<HtmlCanvasElement>()
            {
                if let Some(model) =
                    model_slot.borrow_mut().take()
                {
                    let mount_theme =
                        theme.unwrap_or_else(
                            rs_grid_web::theme_from_css_vars,
                        );
                    let mount_locale =
                        locale.unwrap_or_default();

                    let rect = canvas
                        .get_bounding_client_rect();
                    let win = web_sys::window()
                        .expect("no window");
                    let w = {
                        let bw = rect.width();
                        if bw > 0.0 {
                            bw
                        } else {
                            win.inner_width()
                                .ok()
                                .and_then(|v| {
                                    v.as_f64()
                                })
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
                                .and_then(|v| {
                                    v.as_f64()
                                })
                                .map(|h| h - 80.0)
                                .unwrap_or(600.0)
                        }
                    };

                    let state =
                        GridState::new(model, w, h);
                    let gc =
                        rs_grid_web::GridCanvas::mount(
                            canvas,
                            state,
                            mount_theme,
                            mount_locale,
                        );
                    *gc_handle.borrow_mut() =
                        Some(gc.clone());

                    if let Some(cb) = on_ve {
                        gc.set_on_validation_error(
                            move |row, col, msg| {
                                cb(
                                    row,
                                    col.to_string(),
                                    msg.to_string(),
                                );
                            },
                        );
                    }
                    if let Some(cb) = on_mount {
                        cb.emit(gc);
                    }
                }
            }

            move || {
                if let Some(gc) =
                    gc_cleanup.borrow().as_ref()
                {
                    gc.detach();
                }
            }
        });
    }

    // Theme effect: update in-place when theme changes.
    {
        let gc_handle = gc_handle.clone();
        let theme = props.theme.clone();
        use_effect_with(theme.clone(), move |t| {
            if let Some(theme) = t {
                if let Some(gc) =
                    gc_handle.borrow().as_ref()
                {
                    gc.set_theme(theme.clone());
                }
            }
        });
    }

    // Locale effect: update in-place when locale changes.
    {
        let gc_handle = gc_handle.clone();
        let locale = props.locale.clone();
        use_effect_with(locale.clone(), move |l| {
            if let Some(locale) = l {
                if let Some(gc) =
                    gc_handle.borrow().as_ref()
                {
                    gc.set_locale(locale.clone());
                }
            }
        });
    }

    let style = format!(
        "width:{};height:{};display:block",
        props.width, props.height
    );

    html! {
        <canvas ref={canvas_ref} style={style} />
    }
}

/// Convenience wrapper to create a model slot from a
/// `GridModel`. Yew `Properties` requires `PartialEq`,
/// and `GridModel` is not `Clone`/`PartialEq`, so this
/// wraps it in `Rc<RefCell<Option>>`.
///
/// # Example
/// ```ignore
/// let model_slot = wrap_model(my_model);
/// html! { <GridCanvas model={model_slot} /> }
/// ```
pub fn wrap_model(
    model: GridModel,
) -> Rc<RefCell<Option<GridModel>>> {
    Rc::new(RefCell::new(Some(model)))
}
