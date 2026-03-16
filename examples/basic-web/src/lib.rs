//! Minimal WASM example — no framework, just a `<canvas>` + rs-grid-web.

use rs_grid_core::{
    column::ColumnDef, datasource::FnDataSource, model::GridModel, state::GridState,
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
        ColumnDef::new("id", "ID", 100.0),
        ColumnDef::new("name", "Name", 200.0),
        ColumnDef::new("email", "Email", 260.0),
        ColumnDef::new("role", "Role", 140.0),
        ColumnDef::new("dept", "Department", 160.0),
        ColumnDef::new("status", "Status", 100.0),
    ];

    let source = FnDataSource::new(10_000_000_000_u64, |row: u64, col_key| match col_key {
        "id" => Some(row.to_string()),
        "name" => Some(format!("User {row}")),
        "email" => Some(format!("user{row}@example.com")),
        "role" => Some(if row % 3 == 0 { "Admin" } else { "Member" }.to_owned()),
        "dept" => Some(format!("Dept {}", row % 20)),
        "status" => Some(if row % 5 == 0 { "Inactive" } else { "Active" }.to_owned()),
        _ => None,
    });

    GridModel::with_data_source(columns, Box::new(source), 28.0, 36.0)
}
