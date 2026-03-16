use leptos::prelude::*;
use rs_grid_core::{column::ColumnDef, model::GridModel, row::RowRecord};
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

    let rows: Vec<RowRecord> = (0..500_001)
        .map(|i| {
            let mut row = RowRecord::new(i as u64);
            row.set("id", i.to_string());
            row.set("name", format!("User {}", i));
            row.set("email", format!("user{}@example.com", i));
            row.set("role", if i % 3 == 0 { "Admin" } else { "Member" });
            row.set("dept", format!("Dept {}", i % 20));
            row.set("status", if i % 5 == 0 { "Inactive" } else { "Active" });
            row
        })
        .collect();

    GridModel::new(columns, rows, 28.0, 36.0)
}

#[component]
fn App() -> impl IntoView {
    let model = build_model();

    view! {
        <div style="display:flex;flex-direction:column;height:100vh">
            <header style="height:48px;display:flex;align-items:center;padding:0 16px;background:#1e1e2e;color:#cdd6f4;font:600 15px system-ui">
                "rs-grid · Leptos CSR · 500 000 rows"
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
