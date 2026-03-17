use leptos::prelude::*;
use rs_grid_core::{
    column::ColumnDef, datasource::FnDataSource, model::GridModel,
};
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

    let source =
        FnDataSource::new(9_007_199_254_740_99_u64, |row: u64, col_key| {
            match col_key {
                "id" => Some(row.to_string()),
                "name" => Some(format!("User {row}")),
                "email" => Some(format!("user{row}@example.com")),
                "role" => Some(
                    if row % 3 == 0 { "Admin" } else { "Member" }.to_owned(),
                ),
                "dept" => Some(format!("Dept {}", row % 20)),
                "status" => Some(
                    if row % 5 == 0 { "Inactive" } else { "Active" }.to_owned(),
                ),
                _ => None,
            }
        });

    GridModel::with_data_source(columns, Box::new(source), 40.0, 60.0)
}

#[component]
fn App() -> impl IntoView {
    let model = build_model();

    view! {
        <div class="app-layout">
            <header class="app-header">
                <div class="app-header-brand">
                    <span class="app-header-logo">"rs-grid"</span>
                    <span class="app-header-title">"Leptos CSR demo"</span>
                </div>
                <span class="app-header-badge">"9 000 000 000 000 rows · virtual"</span>
            </header>
            <div class="app-body">
                <div class="app-grid-wrapper">
                    <GridCanvas model=model width="100%".into() height="100%".into() />
                </div>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
