//! basic-js — rs-grid wasm-bindgen example (vanilla JS / library mode).
//!
//! Exposes a `JsGrid` class to JavaScript. The host page owns the
//! surrounding HTML chrome — this crate only mounts the grid on a
//! `<canvas>` element supplied by the caller.

use example_common::build_model;
use rs_grid_core::state::GridState;
use rs_grid_web::{theme_from_css_vars, GridCanvas, Locale};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// Handle to a mounted rs-grid instance, usable from JS.
#[wasm_bindgen]
pub struct JsGrid {
    inner: GridCanvas,
}

#[wasm_bindgen]
impl JsGrid {
    /// Mount a new grid on `canvas` with `row_count` rows
    /// and `col_count` columns of fake data.
    ///
    /// Uses `f64` instead of `u64` to avoid JS `BigInt`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        row_count: f64,
        col_count: f64,
    ) -> JsGrid {
        console_error_panic_hook::set_once();

        let model = build_model(row_count as u64, col_count as usize);
        let theme = theme_from_css_vars();

        let css_w = canvas.client_width() as f64;
        let css_h = canvas.client_height() as f64;
        let state = GridState::new(model, css_w, css_h);

        let gc = GridCanvas::mount(canvas, state, theme, Locale::default());
        gc.render();

        JsGrid { inner: gc }
    }

    /// Re-read the CSS theme variables and apply them.
    pub fn set_theme_from_css(&self) {
        self.inner.set_theme(theme_from_css_vars());
    }

    /// Set the number of pinned (frozen) columns.
    pub fn set_pinned_count(&self, count: usize) {
        self.inner.set_pinned_count(count);
    }

    /// Filter rows by text on a given column.
    pub fn set_filter(&self, col_key: &str, text: &str) {
        self.inner.set_filter(col_key, text);
    }

    /// Remove all active filters.
    pub fn clear_filters(&self) {
        self.inner.clear_filters();
    }

    /// Export edited cell patches as a TSV string.
    pub fn export_patches(&self) -> String {
        self.inner.export_patches()
    }

    /// Import cell patches from a TSV string.
    pub fn import_patches(&self, data: &str) {
        self.inner.import_patches(data);
    }

    /// Detach event listeners and clean up.
    pub fn detach(&self) {
        self.inner.detach();
    }
}
