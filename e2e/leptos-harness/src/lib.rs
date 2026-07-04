//! Minimal Leptos CSR fixture for rs-grid e2e tests.
//!
//! This is **not** the showcase demo — it is the smallest app that satisfies
//! the DOM contract exercised by the CI-run subset of `e2e/tests/grid.spec.ts`
//! (smoke + controls + canvas interaction + log scrollbar) and by
//! `e2e/tests/csp.spec.ts`. It deliberately drops the styled demo's theme
//! selector, language selector, toggles and layout persistence — those live in
//! the external `rs-grid-example-leptos` repo alongside the visual-regression
//! suite. Being a path-dep workspace member, it tracks `main` and catches
//! engine regressions on every push.

use std::rc::Rc;

use example_common::{
    build_model, class_map::resolve_classes, fmt_cols, fmt_rows,
};
use leptos::prelude::*;
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(20usize);

    // No theme selector: read whatever CSS vars are present (defaults to
    // Theme::light() when none are defined).
    let theme_memo = Memo::<Theme>::new(|_| theme_from_css_vars());
    let locale_sig = RwSignal::new(Locale::from_browser());
    // e2e-only: surfaces the live on_validation_state_changed callback
    // value in the DOM so Playwright can assert it fires on every
    // keystroke, not just on rejected commits.
    let validation_state = RwSignal::new(None::<(u64, String, String)>);

    view! {
        <main class="fixture-layout">
            // e2e-only: last on_validation_state_changed message, empty
            // string when the current value is valid / no active edit.
            <span
                data-testid="validation-state"
                style="position:absolute;width:1px;height:1px;overflow:hidden"
            >
                {move || {
                    validation_state.get().map(|(_, _, msg)| msg).unwrap_or_default()
                }}
            </span>
            <div class="fixture-header">
                <h1 class="fixture-title">"rs-grid basic example"</h1>
                <p class="fixture-subtitle">
                    "Use the "
                    <strong>{move || fmt_rows(row_count.get())}</strong>
                    " × "
                    <strong>{move || fmt_cols(col_count.get())}</strong>
                    " virtual dataset below to test windowed rendering."
                </p>
                <div class="fixture-controls">
                    // First <select> — dataset size (grid.spec queries .first()).
                    <select
                        on:change=move |e| {
                            let v = event_target_value(&e)
                                .parse::<u64>()
                                .unwrap_or(1_000);
                            row_count.set(v);
                        }
                    >
                        <option value="1000" selected=true>"1 000 rows"</option>
                        <option value="100000">"100 000 rows"</option>
                        <option value="1000000">"1 million rows"</option>
                        <option value="100000000">"100 million rows"</option>
                        <option value="1000000000">"1 billion rows"</option>
                        <option value="1000000000000">"1 trillion rows"</option>
                        <option value="1000000000000000">
                            "1 quadrillion rows"
                        </option>
                    </select>
                    // Second <select> — column count (grid.spec queries .nth(1)).
                    <select
                        on:change=move |e| {
                            let v = event_target_value(&e)
                                .parse::<usize>()
                                .unwrap_or(20);
                            col_count.set(v);
                        }
                    >
                        <option value="20" selected=true>"20 columns"</option>
                        <option value="100">"100 columns"</option>
                        <option value="1000">"1 000 columns"</option>
                    </select>
                </div>
            </div>
            <div class="fixture-grid">
                {move || {
                    let mut model = build_model(row_count.get(), col_count.get());
                    // e2e-only: row 10's "name" is required() but seeded
                    // empty, simulating data loaded already-invalid from
                    // an external source — exercises the at-rest
                    // validation border/tooltip without going through
                    // CommitEdit/PasteAt (both skip writing invalid
                    // values, so neither can produce this state). Row 0
                    // is left untouched — other specs (editing.spec.ts,
                    // this file's own edit-flow tests) dblclick it.
                    model.set_cell(10, "name", String::new());

                    let on_mount = Box::new(move |gc: WebGridCanvas| {
                        gc.set_class_resolver(Rc::new(resolve_classes));
                        gc.set_editable(true);
                        gc.set_selectable(true);
                        gc.set_column_reorderable(true);
                        // e2e-only: reproduces daisyUI's tooltip via
                        // the class hook — rs-grid renders no
                        // visual of its own, this is entirely
                        // caller-owned styling.
                        gc.set_validation_tooltip_class(Some(
                            "tooltip tooltip-open tooltip-error".to_string(),
                        ));
                    });
                    let on_validation_error = Box::new(
                        move |_row: u64, _col: String, _msg: String| {},
                    );
                    let on_validation_state_changed =
                        Box::new(move |state: Option<(u64, String, String)>| {
                            validation_state.set(state);
                        });
                    let on_cell_button_click = Box::new(
                        move |_row: u64, _col: String, _btn: String| {},
                    );

                    view! {
                        <GridCanvas
                            model=model
                            width="100%".into()
                            height="100%".into()
                            theme=Signal::derive(move || theme_memo.get())
                            locale=Signal::derive(move || locale_sig.get())
                            on_mount=on_mount
                            on_validation_error=on_validation_error
                            on_validation_state_changed=on_validation_state_changed
                            on_cell_button_click=on_cell_button_click
                        />
                    }
                }}
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
