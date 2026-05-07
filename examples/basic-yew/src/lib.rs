//! Demo application showcasing rs-grid with Yew 0.21 CSR.

use example_common::{build_model, fmt_cols, fmt_rows};
use rs_grid_core::state::GridState;
use rs_grid_yew::{theme_from_css_vars, Locale, WebGridCanvas};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlCanvasElement, HtmlSelectElement};
use yew::prelude::*;

// ── App component ──────────────────────────────────────────────────────────

#[function_component]
fn App() -> Html {
    // ── Reactive state ────────────────────────────────────────────────────
    let row_count = use_state(|| 1_000u64);
    let col_count = use_state(|| 20usize);
    let theme_class = use_state(String::new);

    let canvas_ref = use_node_ref();
    let grid_ref = use_mut_ref(|| None::<WebGridCanvas>);

    // ── Effect: mount / remount on row or col count change ────────────────
    {
        let canvas_ref = canvas_ref.clone();
        let grid_ref = grid_ref.clone();
        use_effect_with((*row_count, *col_count), move |(rows, cols)| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Some(old) = grid_ref.borrow().as_ref() {
                    old.detach();
                }
                let model = build_model(*rows, *cols);
                let w = canvas.client_width() as f64;
                let h = canvas.client_height() as f64;
                let state = GridState::new(model, w, h);
                let gc = WebGridCanvas::mount(
                    canvas,
                    state,
                    theme_from_css_vars(),
                    Locale::default(),
                );
                gc.render();
                *grid_ref.borrow_mut() = Some(gc);
            }

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
            row_count.set(v);
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

                    // ── Theme ─────────────────────────────────────────
                    <div class="app-control">
                        <span class="app-control-label">{"Theme"}</span>
                        <select class="app-control-select"
                            onchange={on_theme_change}>
                            <option value="">{"Light"}</option>
                            <option value="dark">{"Dark"}</option>
                            <option value="dimmed">{"Dimmed"}</option>
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
