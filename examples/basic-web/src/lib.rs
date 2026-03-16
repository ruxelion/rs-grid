//! Minimal WASM example — no framework, just a `<canvas>` + rs-grid-web.

use rs_grid_core::{
    column::ColumnDef,
    model::GridModel,
    row::RowRecord,
    state::GridState,
};
use rs_grid_web::GridCanvas;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("grid-canvas")
        .expect("#grid-canvas not found")
        .dyn_into::<HtmlCanvasElement>()
        .expect("#grid-canvas is not a <canvas>");

    let model = build_model();
    // Viewport dimensions are corrected by GridCanvas::mount
    // using canvas.client_width/height at mount time.
    let state = GridState::new(model, 0.0, 0.0);

    let gc = GridCanvas::mount(canvas, state);
    gc.render();
}

fn build_model() -> GridModel {
    let columns = vec![
        ColumnDef::new("id",     "ID",         60.0),
        ColumnDef::new("name",   "Name",       200.0),
        ColumnDef::new("email",  "Email",      260.0),
        ColumnDef::new("role",   "Role",       140.0),
        ColumnDef::new("dept",   "Department", 160.0),
        ColumnDef::new("status", "Status",     100.0),
    ];

    let rows: Vec<RowRecord> = (0..100_000)
        .map(|i| {
            let mut row = RowRecord::new(i as u64);
            row.set("id",     i.to_string());
            row.set("name",   format!("User {i}"));
            row.set("email",  format!("user{i}@example.com"));
            row.set("role",   if i % 3 == 0 { "Admin" } else { "Member" });
            row.set("dept",   format!("Dept {}", i % 20));
            row.set("status", if i % 5 == 0 { "Inactive" } else { "Active" });
            row
        })
        .collect();

    GridModel::new(columns, rows, 28.0, 36.0)
}
