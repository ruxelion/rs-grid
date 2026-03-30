//! Demo application showcasing rs-grid with Leptos CSR.

use std::{cell::RefCell, rc::Rc};

use example_common::build_model;
use leptos::prelude::*;
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;

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

const LS_KEY: &str = "rs-grid-patches";

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(20usize);
    let theme_class = RwSignal::new(String::new());
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

    // Shared handle to the mounted web GridCanvas (for Export/Import buttons).
    // Wrapped in SendWrapper so the Rc can be captured in Leptos closures.
    let gc_ref: Rc<RefCell<Option<WebGridCanvas>>> =
        Rc::new(RefCell::new(None));
    // The view closure and event handlers all need Send captures in Leptos 0.7.
    let gc_for_mount = SendWrapper::new(gc_ref.clone());


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
            <div class="app-body">
                <div class="app-grid-wrapper">
                    {move || {
                        let model =
                            build_model(row_count.get(), col_count.get());

                        // Clone the SendWrapper to move into on_mount_cb.
                        // SendWrapper<Rc<...>> is Send, so this closure is Send.
                        // Inside on_mount_cb (a WASM callback), dereffing it
                        // gives the inner Rc — safe because WASM is
                        // single-threaded.
                        let gc_holder = gc_for_mount.clone();
                        let on_mount_cb = Box::new(
                            move |gc: WebGridCanvas| {
                                if let Some(s) = local_storage() {
                                    if let Ok(Some(data)) =
                                        s.get_item(LS_KEY)
                                    {
                                        gc.import_patches(&data);
                                    }
                                }
                                let gc2 = gc.clone();
                                gc.set_on_change(move || {
                                    if let Some(s) = local_storage() {
                                        let _ = s.set_item(
                                            LS_KEY,
                                            &gc2.export_patches(),
                                        );
                                    }
                                });
                                *gc_holder.borrow_mut() = Some(gc);
                            },
                        );

                        let on_validation_error_cb = Box::new(
                            move |_row: u64, col: String, msg: String| {
                                validation_error.set(
                                    format!("[{col}] {msg}")
                                );
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
