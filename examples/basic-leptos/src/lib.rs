use leptos::prelude::*;
use rs_grid_core::{column::ColumnDef, datasource::FnDataSource, model::GridModel};
use rs_grid_leptos::GridCanvas;
use wasm_bindgen::prelude::*;

fn build_model() -> GridModel {
    let columns = vec![
        ColumnDef::new("id", "ID", 60.0),
        ColumnDef::new("name", "Name", 200.0),
        ColumnDef::new("email", "Email", 260.0),
        ColumnDef::new("role", "Role", 140.0),
        ColumnDef::new("dept", "Department", 160.0),
        ColumnDef::new("status", "Status", 100.0),
    ];

    let source = FnDataSource::new(429_496_729_5, |row, col_key| match col_key {
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

#[component]
fn App() -> impl IntoView {
    let model = build_model();

    view! {
        <div style="display:flex;flex-direction:column;height:100vh">
            <header style="height:48px;display:flex;align-items:center;padding:0 16px;background:#1e1e2e;color:#cdd6f4;font:600 15px system-ui">
                "rs-grid · Leptos CSR · 429 496 729 5 rows (virtual)"
            </header>
            <div style="flex:1;padding:16px;min-height:0">
                <GridCanvas model=model width="100%".into() height="calc(100vh - 80px)".into() />
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
