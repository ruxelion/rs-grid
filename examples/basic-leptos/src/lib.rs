use leptos::prelude::*;
use rs_grid_core::{
    column::ColumnDef, datasource::FnDataSource, model::GridModel,
};
use rs_grid_leptos::GridCanvas;
use wasm_bindgen::prelude::*;

fn build_model(row_count: u64, col_count: usize) -> GridModel {
    let base: Vec<(&'static str, &'static str, f64)> = vec![
        ("id", "ID", 60.0),
        ("name", "Name", 200.0),
        ("email", "Email", 260.0),
        ("role", "Role", 140.0),
        ("dept", "Department", 160.0),
        ("status", "Status", 100.0),
    ];

    let mut columns: Vec<ColumnDef> = base
        .iter()
        .take(col_count.min(base.len()))
        .map(|(k, l, w)| ColumnDef::new(*k, *l, *w))
        .collect();

    for i in (columns.len() + 1)..=col_count {
        columns.push(ColumnDef::new(
            format!("col{i}"),
            format!("Col {i}"),
            100.0,
        ));
    }

    let source =
        FnDataSource::new(row_count, move |row: u64, col_key: &str| {
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
                key if key.starts_with("col") => {
                    key[3..].parse::<u64>().ok().map(|n| format!("{row}×{n}"))
                }
                _ => None,
            }
        });

    GridModel::with_data_source(columns, Box::new(source), 40.0, 60.0)
}

fn fmt_rows(n: u64) -> &'static str {
    match n {
        1_000 => "1,000 rows",
        100_000 => "100,000 rows",
        _ => "rows",
    }
}

fn fmt_cols(n: usize) -> &'static str {
    match n {
        10 => "10 columns",
        100 => "100 columns",
        _ => "columns",
    }
}

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(10usize);

    view! {
        <main class="app-layout">
            <div class="app-page-header">
                <h1 class="app-title">"rs-grid basic example"</h1>
                <p class="app-subtitle">
                    "Use the "
                    <strong class="app-highlight">{move || fmt_rows(row_count.get())}</strong>
                    " × "
                    <strong class="app-highlight">{move || fmt_cols(col_count.get())}</strong>
                    " virtual dataset below to test windowed rendering."
                </p>
                <div class="app-controls">
                    <div class="app-control">
                        <span class="app-control-label">"Dataset size"</span>
                        <select
                            class="app-control-select"
                            on:change=move |e| {
                                let v = event_target_value(&e).parse::<u64>().unwrap_or(1_000);
                                row_count.set(v);
                            }
                        >
                            <option value="1000"   selected=true>"1,000 rows"</option>
                            <option value="100000">"100,000 rows"</option>
                        </select>
                    </div>
                    <div class="app-control">
                        <span class="app-control-label">"Column count"</span>
                        <select
                            class="app-control-select"
                            on:change=move |e| {
                                let v = event_target_value(&e).parse::<usize>().unwrap_or(10);
                                col_count.set(v);
                            }
                        >
                            <option value="10"  selected=true>"10 columns"</option>
                            <option value="100">"100 columns"</option>
                        </select>
                    </div>
                </div>
            </div>
            <div class="app-body">
                <div class="app-grid-wrapper">
                    {move || {
                        let model = build_model(row_count.get(), col_count.get());
                        view! { <GridCanvas model=model width="100%".into() height="100%".into() /> }
                    }}
                </div>
            </div>
        </main>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
