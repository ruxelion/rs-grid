//! Demo application showcasing rs-grid with Leptos CSR.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use example_common::{
    build_model, class_map::resolve_classes, fmt_cols, fmt_rows,
};
use leptos::prelude::*;
use rs_grid_core::column::ColumnOffsets;
use rs_grid_core::model::GridModel;
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use wasm_bindgen::prelude::*;

const LS_KEY: &str = "rs-grid-basic-layout";

/// `(widths_by_key, order_by_key, pinned_count)`
type PersistedLayout = (Vec<(String, f64)>, Vec<String>, usize);

// Thread-local canvas ref — avoids Send requirements.
// WASM is single-threaded so this is always safe.
thread_local! {
    static CANVAS: RefCell<Option<WebGridCanvas>> = const { RefCell::new(None) };
}

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(20usize);
    let theme_class = RwSignal::new(String::new());
    let editable = RwSignal::new(true);
    let selectable = RwSignal::new(true);
    let column_reorderable = RwSignal::new(true);
    let detected_lang = web_sys::window()
        .and_then(|w| w.navigator().language())
        .unwrap_or_default();
    let initial_lang_code =
        match detected_lang.split('-').next().unwrap_or("en") {
            "fr" => "fr",
            "de" => "de",
            "es" => "es",
            _ => "en",
        };
    let lang_code = RwSignal::new(initial_lang_code.to_string());
    let locale_sig = RwSignal::new(Locale::from_browser());
    let validation_error = RwSignal::new(String::new());
    let last_button_action = RwSignal::new(String::new());

    let theme_memo = Memo::<Theme>::new(move |_| {
        let _ = theme_class.get();
        theme_from_css_vars()
    });

    Effect::new(move |_| {
        let cls = theme_class.get();
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            root.set_class_name(&cls);
        }
    });

    // Propagate the editable toggle to the live canvas.
    Effect::new(move |_| {
        let v = editable.get();
        CANVAS.with(|r| {
            if let Some(gc) = r.borrow().as_ref() {
                gc.set_editable(v);
            }
        });
    });

    // Propagate the selectable toggle to the live canvas.
    Effect::new(move |_| {
        let v = selectable.get();
        CANVAS.with(|r| {
            if let Some(gc) = r.borrow().as_ref() {
                gc.set_selectable(v);
            }
        });
    });

    // Propagate the column reorder toggle to the live canvas.
    Effect::new(move |_| {
        let v = column_reorderable.get();
        CANVAS.with(|r| {
            if let Some(gc) = r.borrow().as_ref() {
                gc.set_column_reorderable(v);
            }
        });
    });

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
                                let v = event_target_value(&e)
                                    .parse::<u64>()
                                    .unwrap_or(1_000);
                                row_count.set(v);
                            }
                        >
                            <option value="1000"   selected=true>"1 000 rows"</option>
                            <option value="100000">"100 000 rows"</option>
                            <option value="1000000">"1 million rows"</option>
                            <option value="100000000">"100 million rows"</option>
                            <option value="1000000000">"1 billion rows"</option>
                            <option value="1000000000000">"1 trillion rows"</option>
                            <option value="1000000000000000">"1 quadrillion rows"</option>
                        </select>
                    </div>
                    <div class="app-control">
                        <span class="app-control-label">"Column count"</span>
                        <select
                            class="app-control-select"
                            on:change=move |e| {
                                let v = event_target_value(&e)
                                    .parse::<usize>()
                                    .unwrap_or(20);
                                col_count.set(v);
                            }
                        >
                            <option value="20"  selected=true>"20 columns"</option>
                            <option value="100">"100 columns"</option>
                            <option value="1000">"1 000 columns"</option>
                        </select>
                    </div>

                    // Theme selector
                    <div class="app-control">
                        <span class="app-control-label">"Theme"</span>
                        <select
                            class="app-control-select"
                            on:change=move |e| {
                                theme_class.set(event_target_value(&e));
                            }
                        >
                            <option value="" selected=true>"Light"</option>
                            <option value="dark">"Dark"</option>
                            <option value="dimmed">"Dimmed"</option>
                        </select>
                    </div>

                    // Language selector
                    <div class="app-control">
                        <span class="app-control-label">"Language"</span>
                        <select
                            class="app-control-select"
                            prop:value=move || lang_code.get()
                            on:change=move |e| {
                                let v = event_target_value(&e);
                                locale_sig.set(
                                    Locale::from_language_tag(&v),
                                );
                                lang_code.set(v);
                            }
                        >
                            <option value="en">"English"</option>
                            <option value="fr">"Fran\u{e7}ais"</option>
                            <option value="de">"Deutsch"</option>
                            <option value="es">"Espa\u{f1}ol"</option>
                            <option value="it">"Italiano"</option>
                            <option value="pt">"Portugu\u{ea}s"</option>
                            <option value="nl">"Nederlands"</option>
                            <option value="pl">"Polski"</option>
                            <option value="tr">"T\u{fc}rk\u{e7}e"</option>
                            <option value="ru">"Русский"</option>
                            <option value="uk">"Українська"</option>
                            <option value="ar">"العربية"</option>
                            <option value="ja">"日本語"</option>
                            <option value="zh">"中文"</option>
                            <option value="ko">"한국어"</option>
                        </select>
                    </div>
                    // Editable toggle
                    <div class="app-control">
                        <span class="app-control-label">"Editable"</span>
                        <label class="app-switch">
                            <input
                                type="checkbox"
                                checked=move || editable.get()
                                on:change=move |e| {
                                    editable.set(
                                        event_target_checked(&e)
                                    );
                                }
                            />
                            <span class="app-switch-track"></span>
                        </label>
                    </div>
                    // Selectable toggle
                    <div class="app-control">
                        <span class="app-control-label">"Selectable"</span>
                        <label class="app-switch">
                            <input
                                type="checkbox"
                                checked=move || selectable.get()
                                on:change=move |e| {
                                    selectable.set(
                                        event_target_checked(&e)
                                    );
                                }
                            />
                            <span class="app-switch-track"></span>
                        </label>
                    </div>
                    // Column reorder toggle
                    <div class="app-control">
                        <span class="app-control-label">"Column reorder"</span>
                        <label class="app-switch">
                            <input
                                type="checkbox"
                                checked=move || column_reorderable.get()
                                on:change=move |e| {
                                    column_reorderable.set(
                                        event_target_checked(&e)
                                    );
                                }
                            />
                            <span class="app-switch-track"></span>
                        </label>
                    </div>
                </div>
            </div>
            {move || {
                let err = validation_error.get();
                if err.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! {
                        <div class="app-validation-error">
                            {err}
                        </div>
                    }.into_any()
                }
            }}
            {move || {
                let action = last_button_action.get();
                if action.is_empty() {
                    view! { <div></div> }.into_any()
                } else {
                    view! {
                        <div class="app-validation-error">
                            {"Button clicked: "}{action}
                        </div>
                    }.into_any()
                }
            }}
            <div class="app-body">
                <div class="app-grid-wrapper">
                    {move || {
                        let mut model =
                            build_model(row_count.get(), col_count.get());
                        if let Some(layout) = load_layout() {
                            apply_layout(&mut model, &layout);
                        }

                        let on_mount_cb =
                            Box::new(move |gc: WebGridCanvas| {
                                gc.set_class_resolver(
                                    Rc::new(resolve_classes),
                                );
                                gc.set_editable(editable.get_untracked());
                                gc.set_selectable(
                                    selectable.get_untracked(),
                                );
                                gc.set_column_reorderable(
                                    column_reorderable.get_untracked(),
                                );
                                // Persist column layout to localStorage so
                                // user-resized / reordered columns survive
                                // a page reload (F5).
                                let gc_save = gc.clone();
                                gc.set_on_columns_changed(move || {
                                    save_layout(&gc_save);
                                });
                                CANVAS.with(|r| *r.borrow_mut() = Some(gc));
                            });

                        let on_validation_error_cb = Box::new(
                            move |_row: u64, col: String, msg: String| {
                                validation_error.set(
                                    format!("[{col}] {msg}")
                                );
                            },
                        );

                        let on_cell_button_click_cb = Box::new(
                            move |row: u64, col: String, btn: String| {
                                last_button_action.set(format!(
                                    "[{btn}] row={row} col={col}"
                                ));
                            },
                        );

                        view! {
                            <GridCanvas
                                model=model
                                width="100%".into()
                                height="100%".into()
                                theme=Signal::derive(move || theme_memo.get())
                                locale=Signal::derive(move || locale_sig.get())
                                on_mount=on_mount_cb
                                on_validation_error=on_validation_error_cb
                                on_cell_button_click=on_cell_button_click_cb
                            />
                        }
                    }}
                </div>
            </div>
        </main>
    }
}

/// WASM entry point — mount the Leptos app.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

// ── column-layout persistence (localStorage) ──────────────────────────────────

fn load_layout() -> Option<PersistedLayout> {
    let ls = web_sys::window()?.local_storage().ok().flatten()?;
    let raw = ls.get_item(LS_KEY).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

fn save_layout(gc: &WebGridCanvas) {
    let payload: PersistedLayout =
        (gc.column_widths(), gc.column_order(), gc.pinned_count());
    if let Ok(json) = serde_json::to_string(&payload) {
        if let Some(ls) =
            web_sys::window().and_then(|w| w.local_storage().ok().flatten())
        {
            let _ = ls.set_item(LS_KEY, &json);
        }
    }
}

fn apply_layout(model: &mut GridModel, layout: &PersistedLayout) {
    let (widths, order, pinned) = layout;

    let width_map: HashMap<&str, f64> =
        widths.iter().map(|(k, w)| (k.as_str(), *w)).collect();
    for col in model.columns.iter_mut() {
        if let Some(w) = width_map.get(col.key.as_str()) {
            col.width = *w;
        }
    }

    let order_idx: HashMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, k)| (k.as_str(), i))
        .collect();
    model.columns.sort_by_key(|c| {
        order_idx.get(c.key.as_str()).copied().unwrap_or(usize::MAX)
    });

    model.pinned_count = (*pinned).min(model.columns.len());

    // Hit-testing reads `column_offsets`; keep it in sync after mutating
    // widths and reordering.
    model.column_offsets = ColumnOffsets::compute(&model.columns);
}
