//! Vanilla-JS bridge for rs-grid.
//!
//! Exposes a `JsGrid` wrapper around [`rs_grid_web::GridCanvas`]
//! via `#[wasm_bindgen]` so that a plain `<script type="module">`
//! can mount and drive the grid without any Rust framework.

use rs_grid_core::{
    column::{CellFormat, ColumnDef},
    datasource::FnDataSource,
    model::GridModel,
    state::GridState,
};
use rs_grid_web::{theme_from_css_vars, GridCanvas};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod fake_data;

// ── helpers ──────────────────────────────────────────

fn build_model(
    row_count: u64,
    col_count: usize,
) -> GridModel {
    let base: Vec<ColumnDef> = vec![
        ColumnDef::new("name", "Name", 200.0),
        ColumnDef::new("email", "Email", 260.0),
        ColumnDef::new("role", "Role", 140.0),
        ColumnDef::new("dept", "Department", 160.0),
        {
            let mut c =
                ColumnDef::new("salary", "Salary", 120.0);
            c.format = Some(CellFormat::Currency {
                symbol: "$".into(),
                decimal_places: 2,
                thousands_sep: Some(','),
                symbol_after: false,
            });
            c
        },
        {
            let mut c =
                ColumnDef::new("active", "Active", 80.0);
            c.format = Some(CellFormat::Boolean {
                true_label: "\u{2713}".into(),
                false_label: "\u{2717}".into(),
            });
            c
        },
        {
            let mut c =
                ColumnDef::new("avatar", "Avatar", 60.0);
            c.format = Some(CellFormat::Image {
                base_url: Some(
                    "https://ui-avatars.com/api/?size=40&name="
                        .into(),
                ),
                border_radius: 16.0,
                padding: 4.0,
            });
            c
        },
    ];

    let mut columns: Vec<ColumnDef> =
        base.into_iter().take(col_count.min(7)).collect();

    let extras_needed = col_count.saturating_sub(7);
    for col in
        fake_data::EXTRA_COLUMNS.iter().take(extras_needed)
    {
        let mut c =
            ColumnDef::new(col.key, col.label, col.width);
        c.format = match col.format_hint {
            fake_data::FormatHint::Text => None,
            fake_data::FormatHint::Integer => {
                Some(CellFormat::Number {
                    decimal_places: 0,
                    thousands_sep: Some(' '),
                    decimal_sep: '.',
                })
            }
            fake_data::FormatHint::Currency => {
                Some(CellFormat::Currency {
                    symbol: "$".into(),
                    decimal_places: 0,
                    thousands_sep: Some(','),
                    symbol_after: false,
                })
            }
            fake_data::FormatHint::Percent => {
                Some(CellFormat::Percent {
                    decimal_places: 0,
                })
            }
            fake_data::FormatHint::Boolean => {
                Some(CellFormat::Boolean {
                    true_label: "\u{2713}".into(),
                    false_label: "\u{2717}".into(),
                })
            }
            fake_data::FormatHint::ImageText => {
                Some(CellFormat::ImageText {
                    base_url: String::new(),
                    suffix: String::new(),
                    image_size: 20.0,
                    border_radius: 2.0,
                    gap: 6.0,
                })
            }
        };
        columns.push(c);
    }

    // Dynamic columns beyond the 92 hand-crafted extras
    let dynamic_needed =
        col_count.saturating_sub(7 + fake_data::EXTRA_COUNT);
    for i in 0..dynamic_needed {
        let (key, label, width, hint) =
            fake_data::dynamic_col_def(i);
        let mut c = ColumnDef::new(&key, &label, width);
        c.format = match hint {
            fake_data::FormatHint::Integer => {
                Some(CellFormat::Number {
                    decimal_places: 0,
                    thousands_sep: Some(' '),
                    decimal_sep: '.',
                })
            }
            fake_data::FormatHint::Currency => {
                Some(CellFormat::Currency {
                    symbol: "$".into(),
                    decimal_places: 0,
                    thousands_sep: Some(','),
                    symbol_after: false,
                })
            }
            fake_data::FormatHint::Percent => {
                Some(CellFormat::Percent {
                    decimal_places: 0,
                })
            }
            fake_data::FormatHint::Boolean => {
                Some(CellFormat::Boolean {
                    true_label: "\u{2713}".into(),
                    false_label: "\u{2717}".into(),
                })
            }
            _ => None,
        };
        columns.push(c);
    }

    let source = FnDataSource::new(
        row_count,
        move |row: u64, col_key: &str| {
            fake_data::fake_cell(row, col_key)
        },
    );

    GridModel::with_data_source(
        columns,
        Box::new(source),
        40.0,
        60.0,
    )
}

// ── wasm_bindgen wrapper ─────────────────────────────

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

        let row_count = row_count as u64;
        let col_count = col_count as usize;

        let model = build_model(row_count, col_count);
        let theme = theme_from_css_vars();

        let css_w = canvas.client_width() as f64;
        let css_h = canvas.client_height() as f64;
        let state = GridState::new(model, css_w, css_h);

        let gc = GridCanvas::mount(canvas, state, theme);
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
