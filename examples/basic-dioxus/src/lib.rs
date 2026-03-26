//! Demo application showcasing rs-grid with Dioxus 0.6 web.

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use example_common::build_model;
use rs_grid_core::state::GridState;
use rs_grid_web::{theme_from_css_vars, GridCanvas, Locale};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlCanvasElement;

// ── Types ──────────────────────────────────────────────────────────────────

type CanvasRef = Rc<RefCell<Option<HtmlCanvasElement>>>;
type GridRef = Rc<RefCell<Option<GridCanvas>>>;

const LS_KEY: &str = "rs-grid-patches";

// ── Helpers ────────────────────────────────────────────────────────────────

fn fmt_rows(n: u64) -> &'static str {
    match n {
        1_000 => "1 000 rows",
        100_000 => "100 000 rows",
        1_000_000 => "1 million rows",
        100_000_000 => "100 million rows",
        1_000_000_000 => "1 billion rows",
        1_000_000_000_000 => "1 trillion rows",
        1_000_000_000_000_000 => "1 quadrillion rows",
        _ => "rows",
    }
}

fn fmt_cols(n: usize) -> &'static str {
    match n {
        20 => "20 columns",
        100 => "100 columns",
        1000 => "1 000 columns",
        _ => "columns",
    }
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

// ── remount ────────────────────────────────────────────────────────────────

/// Detach the current GridCanvas if any and mount a fresh one with
/// `rows` × `cols` of virtual data.
fn remount(canvas_ref: &CanvasRef, grid_ref: &GridRef, rows: u64, cols: usize) {
    let Some(canvas) = canvas_ref.borrow().clone() else {
        return;
    };
    if let Some(old) = grid_ref.borrow().as_ref() {
        old.detach();
    }
    let model = build_model(rows, cols);
    let w = canvas.client_width() as f64;
    let h = canvas.client_height() as f64;
    let state = GridState::new(model, w, h);
    let gc = GridCanvas::mount(
        canvas,
        state,
        theme_from_css_vars(),
        Locale::default(),
    );
    if let Some(s) = local_storage() {
        if let Ok(Some(data)) = s.get_item(LS_KEY) {
            gc.import_patches(&data);
        }
    }
    let gc2 = gc.clone();
    gc.set_on_change(move || {
        if let Some(s) = local_storage() {
            let _ = s.set_item(LS_KEY, &gc2.export_patches());
        }
    });
    gc.render();
    *grid_ref.borrow_mut() = Some(gc);
}

// ── App component ──────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let mut row_count = use_signal(|| 1_000u64);
    let mut col_count = use_signal(|| 20usize);
    let mut theme_class = use_signal(String::new);

    // GridCanvas and HtmlCanvasElement are Rc-backed and not Send.
    // use_hook stores them in the component scope without Send requirements.
    // `.clone()` via auto-deref calls Rc::clone on the &mut Rc returned
    // by use_hook, yielding an owned Rc pointing to the same allocation.
    let canvas_ref: CanvasRef =
        use_hook(|| Rc::new(RefCell::new(None::<HtmlCanvasElement>))).clone();
    let grid_ref: GridRef =
        use_hook(|| Rc::new(RefCell::new(None::<GridCanvas>))).clone();

    // Per-closure clones of the shared handles
    let cr_mount = Rc::clone(&canvas_ref);
    let gr_mount = Rc::clone(&grid_ref);
    let cr_rows = Rc::clone(&canvas_ref);
    let gr_rows = Rc::clone(&grid_ref);
    let cr_cols = Rc::clone(&canvas_ref);
    let gr_cols = Rc::clone(&grid_ref);
    let gr_theme = Rc::clone(&grid_ref);
    let gr_pinned = Rc::clone(&grid_ref);
    let gr_filter = Rc::clone(&grid_ref);
    let gr_export = Rc::clone(&grid_ref);
    let gr_import = Rc::clone(&grid_ref);

    // Apply theme class to <html> and refresh grid colours whenever
    // theme_class signal changes.
    use_effect(move || {
        let cls = theme_class.read().clone();
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            root.set_class_name(&cls);
        }
        if let Some(gc) = gr_theme.borrow().as_ref() {
            gc.set_theme(theme_from_css_vars());
        }
    });

    rsx! {
        main { class: "app-layout",
            div { class: "app-page-header",
                h1 { class: "app-title", "rs-grid basic example" }
                p { class: "app-subtitle",
                    "Use the "
                    strong { class: "app-highlight",
                        { fmt_rows(*row_count.read()) }
                    }
                    " × "
                    strong { class: "app-highlight",
                        { fmt_cols(*col_count.read()) }
                    }
                    " virtual dataset below to test windowed rendering."
                }
                div { class: "app-controls",

                    // ── Dataset size ──────────────────────────────────
                    div { class: "app-control",
                        span { class: "app-control-label", "Dataset size" }
                        select {
                            class: "app-control-select",
                            onchange: move |e| {
                                let v = e.value()
                                    .parse::<u64>()
                                    .unwrap_or(1_000);
                                row_count.set(v);
                                remount(
                                    &cr_rows,
                                    &gr_rows,
                                    v,
                                    *col_count.peek(),
                                );
                            },
                            option { value: "1000",             "1 000 rows" }
                            option { value: "100000",           "100 000 rows" }
                            option { value: "1000000",          "1 million rows" }
                            option { value: "100000000",        "100 million rows" }
                            option { value: "1000000000",       "1 billion rows" }
                            option { value: "1000000000000",    "1 trillion rows" }
                            option { value: "1000000000000000","1 quadrillion rows" }
                        }
                    }

                    // ── Column count ──────────────────────────────────
                    div { class: "app-control",
                        span { class: "app-control-label", "Column count" }
                        select {
                            class: "app-control-select",
                            onchange: move |e| {
                                let v = e.value()
                                    .parse::<usize>()
                                    .unwrap_or(20);
                                col_count.set(v);
                                remount(
                                    &cr_cols,
                                    &gr_cols,
                                    *row_count.peek(),
                                    v,
                                );
                            },
                            option { value: "20",   "20 columns" }
                            option { value: "100",  "100 columns" }
                            option { value: "1000", "1 000 columns" }
                        }
                    }

                    // ── Export ────────────────────────────────────────
                    button {
                        class: "app-btn",
                        onclick: move |_| {
                            let Some(gc) = gr_export.borrow().clone()
                            else {
                                return;
                            };
                            let data = gc.export_patches();
                            let encoded =
                                js_sys::encode_uri_component(&data);
                            let url = format!(
                                "data:text/tab-separated-values;\
                                 charset=utf-8,{encoded}"
                            );
                            let doc = web_sys::window()
                                .unwrap()
                                .document()
                                .unwrap();
                            let a = doc
                                .create_element("a")
                                .unwrap()
                                .dyn_into::<web_sys::HtmlAnchorElement>()
                                .unwrap();
                            a.set_href(&url);
                            a.set_download("rs-grid-patches.tsv");
                            doc.body().unwrap().append_child(&a).unwrap();
                            a.click();
                            doc.body().unwrap().remove_child(&a).unwrap();
                        },
                        "Export"
                    }

                    // Hidden file input driven by the Import button
                    input {
                        r#type: "file",
                        id: "dioxus-file-import",
                        accept: ".tsv,.txt",
                        style: "display:none",
                        onchange: move |_| {
                            let input_el = web_sys::window()
                                .unwrap()
                                .document()
                                .unwrap()
                                .get_element_by_id("dioxus-file-import")
                                .unwrap()
                                .dyn_into::<web_sys::HtmlInputElement>()
                                .unwrap();
                            let file = input_el
                                .files()
                                .and_then(|fl| fl.get(0));
                            if let Some(file) = file {
                                let reader =
                                    web_sys::FileReader::new().unwrap();
                                let reader2 = reader.clone();
                                let gc =
                                    gr_import.borrow().clone().unwrap();
                                let cb = Closure::once(move || {
                                    if let Ok(result) = reader2.result() {
                                        if let Some(text) =
                                            result.as_string()
                                        {
                                            gc.import_patches(&text);
                                            if let Some(s) = local_storage()
                                            {
                                                let _ = s.set_item(
                                                    LS_KEY,
                                                    &gc.export_patches(),
                                                );
                                            }
                                        }
                                    }
                                });
                                reader.set_onloadend(Some(
                                    cb.as_ref().unchecked_ref(),
                                ));
                                reader.read_as_text(&file).unwrap();
                                cb.forget();
                            }
                        },
                    }

                    // ── Import ────────────────────────────────────────
                    button {
                        class: "app-btn",
                        onclick: move |_| {
                            if let Some(el) = web_sys::window()
                                .unwrap()
                                .document()
                                .unwrap()
                                .get_element_by_id("dioxus-file-import")
                            {
                                let _ = el
                                    .dyn_into::<web_sys::HtmlInputElement>()
                                    .map(|i| i.click());
                            }
                        },
                        "Import"
                    }

                    // ── Pinned cols ───────────────────────────────────
                    div { class: "app-control",
                        span { class: "app-control-label", "Pinned cols" }
                        select {
                            class: "app-control-select",
                            onchange: move |e| {
                                let v = e.value()
                                    .parse::<usize>()
                                    .unwrap_or(0);
                                if let Some(gc) =
                                    gr_pinned.borrow().as_ref()
                                {
                                    gc.set_pinned_count(v);
                                }
                            },
                            option { value: "0", "None" }
                            option { value: "1", "1" }
                            option { value: "2", "2" }
                            option { value: "3", "3" }
                        }
                    }

                    // ── Filter ────────────────────────────────────────
                    div { class: "app-control",
                        span { class: "app-control-label", "Filter Name" }
                        input {
                            r#type: "text",
                            class: "app-control-select",
                            placeholder: "type to filter\u{2026}",
                            oninput: move |e| {
                                let text = e.value();
                                if let Some(gc) =
                                    gr_filter.borrow().as_ref()
                                {
                                    gc.set_filter("name", &text);
                                }
                            },
                        }
                    }

                    // ── Theme ─────────────────────────────────────────
                    div { class: "app-control",
                        span { class: "app-control-label", "Theme" }
                        select {
                            class: "app-control-select",
                            onchange: move |e| {
                                theme_class.set(e.value());
                            },
                            option { value: "",              "Light" }
                            option { value: "dark",          "Dark" }
                            option { value: "material",      "Material 3" }
                            option { value: "material-dark", "Material 3 Dark" }
                        }
                    }
                }
            }

            // ── Body: grid canvas ─────────────────────────────────────
            div { class: "app-body",
                div { class: "app-grid-wrapper",
                    canvas {
                        id: "rs-grid-canvas",
                        style: "width:100%;height:100%;display:block",
                        onmounted: move |_| {
                            // Retrieve the canvas by id — simpler and
                            // more reliable than downcasting MountedData
                            // whose raw-element API varies across Dioxus
                            // versions.
                            if let Some(canvas) = web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|d| {
                                    d.get_element_by_id("rs-grid-canvas")
                                })
                                .and_then(|el| {
                                    el.dyn_into::<HtmlCanvasElement>().ok()
                                })
                            {
                                *cr_mount.borrow_mut() = Some(canvas);
                                remount(
                                    &cr_mount,
                                    &gr_mount,
                                    *row_count.peek(),
                                    *col_count.peek(),
                                );
                            }
                        },
                    }
                }
            }
        }
    }
}

// ── WASM entry point ───────────────────────────────────────────────────────

/// WASM entry point — mount the Dioxus app.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}
