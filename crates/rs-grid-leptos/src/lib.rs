//! Leptos CSR component integration for rs-grid.
//!
//! Provides a `<GridCanvas>` component that mounts a grid
//! into the DOM and keeps it in sync with Leptos reactive
//! signals. Top of the dependency chain (`core → scene →
//! render-canvas → web → **leptos**`).
//!
//! Re-exports: [`WebGridCanvas`], [`Locale`],
//! [`theme_from_css_vars`].

pub use rs_grid_web::theme_from_css_vars;
/// Re-exported so callers can name the type in `on_mount` closures without
/// depending on `rs-grid-web` directly.
pub use rs_grid_web::GridCanvas as WebGridCanvas;
pub use rs_grid_web::Locale;

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

/// Callback type for validation error events: `(row, col_key, message)`.
pub type ValidationErrorCb = Box<dyn Fn(u64, String, String)>;

/// Callback type for cell button click events:
/// `(row, col_key, button_id)`.
pub type CellButtonClickCb = Box<dyn Fn(u64, String, String)>;

/// A Leptos component that renders an rs-grid onto a `<canvas>` element.
///
/// # Props
/// - `model`: The `GridModel` to display.
/// - `width`: CSS width string (e.g. `"100%"` or `"800px"`).
/// - `height`: CSS height string (e.g. `"600px"`).
/// - `theme`: Optional reactive `Signal<Theme>`. When supplied, theme changes
///   are applied in-place via `set_theme()` without remounting the grid.
/// - `locale`: Optional reactive `Signal<Locale>`. When supplied, locale
///   changes are applied in-place via `set_locale()` without remounting.
#[component]
pub fn GridCanvas(
    model: GridModel,
    #[prop(default = "100%".into())] width: String,
    #[prop(default = "600px".into())] height: String,
    #[prop(optional)] theme: Option<Signal<Theme>>,
    #[prop(optional)] locale: Option<Signal<Locale>>,
    /// Called once after the grid is mounted with a cloned handle to the
    /// underlying `GridCanvas`. Use it to call `set_on_change`,
    /// `import_patches`, or `export_patches`.
    #[prop(optional)]
    on_mount: Option<Box<dyn FnOnce(rs_grid_web::GridCanvas)>>,
    /// Called when a per-column validator rejects an edit.
    /// Arguments: `(row, col_key, error_message)`.
    #[prop(optional)]
    on_validation_error: Option<ValidationErrorCb>,
    /// Called when a cell button is clicked.
    /// Arguments: `(row, col_key, button_id)`.
    #[prop(optional)]
    on_cell_button_click: Option<CellButtonClickCb>,
) -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // Wrap in RefCell<Option> so the model can be moved into the Effect
    // on first run without requiring GridModel: Clone.  This avoids
    // a panic when the data source is an FnDataSource (which is not cloneable).
    let model_slot = RefCell::new(Some(model));
    let on_mount_slot = RefCell::new(on_mount);
    let on_validation_error_slot = RefCell::new(on_validation_error);
    let on_cell_button_click_slot =
        RefCell::new(on_cell_button_click);

    // Holder for the mounted GridCanvas handle, shared across effects and cleanup.
    // SendWrapper allows Rc<RefCell<...>> to satisfy Send+Sync for on_cleanup;
    // it is safe because WASM is single-threaded and the value never actually
    // crosses thread boundaries.
    let gc_holder: Rc<RefCell<Option<rs_grid_web::GridCanvas>>> =
        Rc::new(RefCell::new(None));
    let gc_for_theme = gc_holder.clone();
    let gc_for_locale = gc_holder.clone();
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

        let mount_locale =
            locale.map(|s| s.get_untracked()).unwrap_or_default();

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
        let gc = rs_grid_web::GridCanvas::mount(
            canvas,
            state,
            mount_theme,
            mount_locale,
        );
        *gc_holder.borrow_mut() = Some(gc.clone());
        if let Some(cb) = on_validation_error_slot.borrow_mut().take() {
            gc.set_on_validation_error(move |row, col, msg| {
                cb(row, col.to_string(), msg.to_string());
            });
        }
        if let Some(cb) =
            on_cell_button_click_slot.borrow_mut().take()
        {
            gc.set_on_cell_button_click(
                move |row, col, btn| {
                    cb(row, col.to_string(), btn.to_string());
                },
            );
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

    // Reactive locale effect: same pattern as theme.
    if let Some(locale_sig) = locale {
        let first_run = Cell::new(true);
        Effect::new(move |_| {
            let l = locale_sig.get();
            if first_run.replace(false) {
                return; // mount already applied this locale
            }
            if let Some(gc) = gc_for_locale.borrow().as_ref() {
                gc.set_locale(l);
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

    let style = canvas_style(&width, &height);

    view! {
        <canvas
            node_ref=canvas_ref
            style=style
        />
    }
}

/// Build the inline CSS style applied to the `<canvas>`
/// element.
fn canvas_style(width: &str, height: &str) -> String {
    format!("width:{};height:{};display:block", width, height)
}

// ── Unit tests ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── canvas_style ─────────────────────────────────

    #[test]
    fn canvas_style_defaults() {
        assert_eq!(
            canvas_style("100%", "600px"),
            "width:100%;height:600px;display:block"
        );
    }

    #[test]
    fn canvas_style_fixed_dimensions() {
        assert_eq!(
            canvas_style("800px", "400px"),
            "width:800px;height:400px;display:block"
        );
    }

    #[test]
    fn canvas_style_empty_strings() {
        assert_eq!(canvas_style("", ""), "width:;height:;display:block");
    }

    #[test]
    fn canvas_style_viewport_units() {
        let s = canvas_style("50vw", "80vh");
        assert!(s.contains("50vw"));
        assert!(s.contains("80vh"));
        assert!(s.ends_with("display:block"));
    }

    // ── Re-export: Locale ────────────────────────────

    #[test]
    fn locale_default_is_english() {
        let loc = Locale::default();
        assert_eq!(loc.cut, "Cut");
        assert_eq!(loc.copy, "Copy");
        assert_eq!(loc.paste, "Paste");
    }

    #[test]
    fn locale_en() {
        let loc = Locale::en();
        assert_eq!(loc.cut, "Cut");
        assert_eq!(loc.copy_with_headers, "Copy with headers");
        assert!(!loc.search_placeholder.is_empty());
    }

    #[test]
    fn locale_fr() {
        let loc = Locale::fr();
        assert_eq!(loc.cut, "Couper");
        assert_eq!(loc.copy, "Copier");
    }

    #[test]
    fn locale_from_language_tag_exact() {
        let loc = Locale::from_language_tag("de");
        assert_eq!(loc.cut, "Ausschneiden");
    }

    #[test]
    fn locale_from_language_tag_with_region() {
        let loc = Locale::from_language_tag("es-MX");
        assert_eq!(loc.cut, "Cortar");
    }

    #[test]
    fn locale_from_language_tag_unknown_fallback() {
        let loc = Locale::from_language_tag("xx-ZZ");
        // Unknown tags fall back to English
        assert_eq!(loc.cut, "Cut");
    }

    #[test]
    fn locale_all_builtins_non_empty() {
        let locales: Vec<(&str, Locale)> = vec![
            ("en", Locale::en()),
            ("fr", Locale::fr()),
            ("de", Locale::de()),
            ("es", Locale::es()),
            ("it", Locale::it()),
            ("pt", Locale::pt()),
            ("nl", Locale::nl()),
            ("pl", Locale::pl()),
            ("tr", Locale::tr()),
            ("ru", Locale::ru()),
            ("uk", Locale::uk()),
            ("ar", Locale::ar()),
            ("ja", Locale::ja()),
            ("zh", Locale::zh()),
            ("ko", Locale::ko()),
        ];
        for (name, loc) in &locales {
            assert!(!loc.cut.is_empty(), "{name}: cut empty");
            assert!(!loc.copy.is_empty(), "{name}: copy empty");
            assert!(!loc.paste.is_empty(), "{name}: paste empty");
            assert!(
                !loc.search_placeholder.is_empty(),
                "{name}: search_placeholder empty"
            );
            assert!(
                !loc.sort_ascending.is_empty(),
                "{name}: sort_ascending empty"
            );
            assert!(!loc.pin_column.is_empty(), "{name}: pin_column empty");
        }
    }

    // ── Re-export: Theme ─────────────────────────────

    #[test]
    fn theme_light_defaults() {
        let t = Theme::light();
        assert!(t.font_size > 0.0);
        assert!(t.row_height > 0.0);
        assert!(t.header_height > 0.0);
        assert!(t.cell_padding > 0.0);
    }

    #[test]
    fn theme_dark_defaults() {
        let t = Theme::dark();
        assert!(t.font_size > 0.0);
        assert!(t.row_height > 0.0);
        assert!(t.header_height > 0.0);
    }

    #[test]
    fn theme_light_dark_differ() {
        let l = Theme::light();
        let d = Theme::dark();
        // Background colors must differ between themes
        assert_ne!(l.bg, d.bg);
        assert_ne!(l.header_bg, d.header_bg);
    }

    // ── GridModel / GridState construction ────────────

    #[test]
    fn grid_state_new_default_viewport() {
        use rs_grid_core::{column::ColumnDef, model::GridModel};

        let cols = vec![
            ColumnDef::new("a", "Col A", 100.0),
            ColumnDef::new("b", "Col B", 150.0),
        ];
        let model = GridModel::new(cols, vec![], 30.0, 40.0);
        let state = GridState::new(model, 800.0, 600.0);

        assert_eq!(state.viewport.width, 800.0);
        assert_eq!(state.viewport.height, 600.0);
        assert_eq!(state.viewport.scroll_x, 0.0);
        assert_eq!(state.viewport.scroll_y, 0.0);
    }

    #[test]
    fn grid_state_with_rows() {
        use rs_grid_core::{
            column::ColumnDef, model::GridModel, row::RowRecord,
        };

        let cols = vec![ColumnDef::new("name", "Name", 120.0)];
        let mut r1 = RowRecord::new(0);
        r1.set("name", "Alice");
        let mut r2 = RowRecord::new(1);
        r2.set("name", "Bob");
        let mut r3 = RowRecord::new(2);
        r3.set("name", "Carol");
        let model = GridModel::new(cols, vec![r1, r2, r3], 30.0, 40.0);
        let state = GridState::new(model, 500.0, 300.0);

        assert_eq!(state.model.data.row_count(), 3);
    }

    // ── Type alias ───────────────────────────────────

    #[test]
    fn validation_error_cb_callable() {
        let called = Rc::new(Cell::new(false));
        let called_c = called.clone();
        let cb: ValidationErrorCb = Box::new(move |row, col, msg| {
            assert_eq!(row, 42);
            assert_eq!(col, "price");
            assert_eq!(msg, "negative value");
            called_c.set(true);
        });
        cb(42, "price".into(), "negative value".into());
        assert!(called.get());
    }
}
