//! Demo application showcasing rs-grid with Yew 0.21 CSR.

use example_common::build_model;
use rs_grid_core::state::GridState;
use rs_grid_web::{theme_from_css_vars, GridCanvas};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, HtmlCanvasElement, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

// ── Constants ──────────────────────────────────────────────────────────────

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

// ── App component ──────────────────────────────────────────────────────────

#[function_component]
fn App() -> Html {
    // ── Reactive state (drives re-renders) ────────────────────────────────
    let row_count = use_state(|| 1_000u64);
    let col_count = use_state(|| 20usize);
    let theme_class = use_state(String::new);

    // ── Non-reactive handles ──────────────────────────────────────────────
    // use_node_ref: Yew's built-in typed reference to a DOM element.
    let canvas_ref = use_node_ref();
    // use_mut_ref: Rc<RefCell<T>> stored across renders, no Send required.
    let grid_ref = use_mut_ref(|| None::<GridCanvas>);

    // ── Effect: mount / remount on row or col count change ────────────────
    // use_effect_with fires after every render where the deps changed,
    // including the first render (when the canvas is already in the DOM).
    {
        let canvas_ref = canvas_ref.clone();
        let grid_ref = grid_ref.clone();
        use_effect_with((*row_count, *col_count), move |(rows, cols)| {
            // canvas_ref is valid after the first render
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                // Detach previous instance
                if let Some(old) = grid_ref.borrow().as_ref() {
                    old.detach();
                }
                let model = build_model(*rows, *cols);
                let w = canvas.client_width() as f64;
                let h = canvas.client_height() as f64;
                let state = GridState::new(model, w, h);
                let gc =
                    GridCanvas::mount(canvas, state, theme_from_css_vars());
                // Restore patches from localStorage
                if let Some(s) = local_storage() {
                    if let Ok(Some(data)) = s.get_item(LS_KEY) {
                        gc.import_patches(&data);
                    }
                }
                // Persist on every edit
                let gc2 = gc.clone();
                gc.set_on_change(move || {
                    if let Some(s) = local_storage() {
                        let _ = s.set_item(LS_KEY, &gc2.export_patches());
                    }
                });
                gc.render();
                *grid_ref.borrow_mut() = Some(gc);
            }

            // Cleanup: detach when deps change or component unmounts.
            // Always returned so the closure type is consistent.
            let grid_ref2 = grid_ref.clone();
            move || {
                if let Some(gc) = grid_ref2.borrow().as_ref() {
                    gc.detach();
                }
            }
        });
    }

    // ── Effect: theme class → CSS + grid repaint ──────────────────────────
    {
        let grid_ref = grid_ref.clone();
        use_effect_with((*theme_class).clone(), move |cls| {
            if let Some(root) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.document_element())
            {
                root.set_class_name(cls);
            }
            if let Some(gc) = grid_ref.borrow().as_ref() {
                gc.set_theme(theme_from_css_vars());
            }
        });
    }

    // ── Callbacks ─────────────────────────────────────────────────────────

    let on_rows_change = {
        let row_count = row_count.clone();
        Callback::from(move |e: Event| {
            let v = e
                .target_unchecked_into::<HtmlSelectElement>()
                .value()
                .parse::<u64>()
                .unwrap_or(1_000);
            row_count.set(v); // triggers re-render → effect remounts grid
        })
    };

    let on_cols_change = {
        let col_count = col_count.clone();
        Callback::from(move |e: Event| {
            let v = e
                .target_unchecked_into::<HtmlSelectElement>()
                .value()
                .parse::<usize>()
                .unwrap_or(20);
            col_count.set(v);
        })
    };

    let on_theme_change = {
        let theme_class = theme_class.clone();
        Callback::from(move |e: Event| {
            let v = e.target_unchecked_into::<HtmlSelectElement>().value();
            theme_class.set(v);
        })
    };

    let on_pinned_change = {
        let grid_ref = grid_ref.clone();
        Callback::from(move |e: Event| {
            let v = e
                .target_unchecked_into::<HtmlSelectElement>()
                .value()
                .parse::<usize>()
                .unwrap_or(0);
            if let Some(gc) = grid_ref.borrow().as_ref() {
                gc.set_pinned_count(v);
            }
        })
    };

    let on_filter_input = {
        let grid_ref = grid_ref.clone();
        Callback::from(move |e: InputEvent| {
            let text = e.target_unchecked_into::<HtmlInputElement>().value();
            if let Some(gc) = grid_ref.borrow().as_ref() {
                gc.set_filter("name", &text);
            }
        })
    };

    let on_export = {
        let grid_ref = grid_ref.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(gc) = grid_ref.borrow().clone() else {
                return;
            };
            let data = gc.export_patches();
            let encoded = js_sys::encode_uri_component(&data);
            let url = format!(
                "data:text/tab-separated-values;charset=utf-8,{encoded}"
            );
            let doc = web_sys::window().unwrap().document().unwrap();
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
        })
    };

    let on_import_click = Callback::from(|_: MouseEvent| {
        if let Some(el) = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("yew-file-import")
        {
            let _ = el.dyn_into::<HtmlInputElement>().map(|i| i.click());
        }
    });

    let on_import_change = {
        let grid_ref = grid_ref.clone();
        Callback::from(move |_: Event| {
            let input_el = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("yew-file-import")
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap();
            let file = input_el.files().and_then(|fl| fl.get(0));
            if let Some(file) = file {
                let reader = web_sys::FileReader::new().unwrap();
                let reader2 = reader.clone();
                let gc = grid_ref.borrow().clone().unwrap();
                let cb = Closure::once(move || {
                    if let Ok(result) = reader2.result() {
                        if let Some(text) = result.as_string() {
                            gc.import_patches(&text);
                            if let Some(s) = local_storage() {
                                let _ =
                                    s.set_item(LS_KEY, &gc.export_patches());
                            }
                        }
                    }
                });
                reader.set_onloadend(Some(cb.as_ref().unchecked_ref()));
                reader.read_as_text(&file).unwrap();
                cb.forget();
            }
        })
    };

    // ── View ──────────────────────────────────────────────────────────────

    html! {
        <main class="app-layout">
            <div class="app-page-header">
                <h1 class="app-title">{"rs-grid basic example"}</h1>
                <p class="app-subtitle">
                    {"Use the "}
                    <strong class="app-highlight">
                        { fmt_rows(*row_count) }
                    </strong>
                    {" × "}
                    <strong class="app-highlight">
                        { fmt_cols(*col_count) }
                    </strong>
                    {" virtual dataset below to test windowed rendering."}
                </p>
                <div class="app-controls">

                    // ── Dataset size ──────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">{"Dataset size"}</span>
                        <select class="app-control-select"
                            onchange={on_rows_change}>
                            <option value="1000"
                                selected={*row_count == 1_000}>
                                {"1 000 rows"}
                            </option>
                            <option value="100000"
                                selected={*row_count == 100_000}>
                                {"100 000 rows"}
                            </option>
                            <option value="1000000"
                                selected={*row_count == 1_000_000}>
                                {"1 million rows"}
                            </option>
                            <option value="100000000"
                                selected={*row_count == 100_000_000}>
                                {"100 million rows"}
                            </option>
                            <option value="1000000000"
                                selected={*row_count == 1_000_000_000}>
                                {"1 billion rows"}
                            </option>
                            <option value="1000000000000"
                                selected={*row_count == 1_000_000_000_000}>
                                {"1 trillion rows"}
                            </option>
                            <option value="1000000000000000"
                                selected={*row_count
                                    == 1_000_000_000_000_000}>
                                {"1 quadrillion rows"}
                            </option>
                        </select>
                    </div>

                    // ── Column count ──────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">
                            {"Column count"}
                        </span>
                        <select class="app-control-select"
                            onchange={on_cols_change}>
                            <option value="20"
                                selected={*col_count == 20}>
                                {"20 columns"}
                            </option>
                            <option value="100"
                                selected={*col_count == 100}>
                                {"100 columns"}
                            </option>
                            <option value="1000"
                                selected={*col_count == 1000}>
                                {"1 000 columns"}
                            </option>
                        </select>
                    </div>

                    // ── Export ────────────────────────────────────────
                    <button class="app-btn" onclick={on_export}>
                        {"Export"}
                    </button>

                    // Hidden file input for import
                    <input
                        type="file"
                        id="yew-file-import"
                        accept=".tsv,.txt"
                        style="display:none"
                        onchange={on_import_change}
                    />

                    // ── Import ────────────────────────────────────────
                    <button class="app-btn" onclick={on_import_click}>
                        {"Import"}
                    </button>

                    // ── Pinned cols ───────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">
                            {"Pinned cols"}
                        </span>
                        <select class="app-control-select"
                            onchange={on_pinned_change}>
                            <option value="0">{"None"}</option>
                            <option value="1">{"1"}</option>
                            <option value="2">{"2"}</option>
                            <option value="3">{"3"}</option>
                        </select>
                    </div>

                    // ── Filter ────────────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">
                            {"Filter Name"}
                        </span>
                        <input
                            type="text"
                            class="app-control-select"
                            placeholder="type to filter\u{2026}"
                            oninput={on_filter_input}
                        />
                    </div>

                    // ── Theme ─────────────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">{"Theme"}</span>
                        <select class="app-control-select"
                            onchange={on_theme_change}>
                            <option value="">{"Light"}</option>
                            <option value="dark">{"Dark"}</option>
                            <option value="material">
                                {"Material 3"}
                            </option>
                            <option value="material-dark">
                                {"Material 3 Dark"}
                            </option>
                        </select>
                    </div>
                </div>
            </div>

            // ── Body: grid canvas ─────────────────────────────────────
            <div class="app-body">
                <div class="app-grid-wrapper">
                    <canvas
                        ref={canvas_ref}
                        style="width:100%;height:100%;display:block"
                    />
                </div>
            </div>
        </main>
    }
}

// ── WASM entry point ───────────────────────────────────────────────────────

/// WASM entry point — mount the Yew app.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
