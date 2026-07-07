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

use std::{cell::RefCell, rc::Rc};

use example_common::{
    build_model, class_map::resolve_classes, fmt_cols, fmt_rows,
};
use leptos::prelude::*;
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(20usize);
    // e2e-only: whether the row-selection checkbox column is shown. A plain
    // `<button>` (not an `<input>`) so it doesn't trip editing.spec.ts's
    // "no <input> exists in the DOM" assertion for the editor=None case.
    let show_checkboxes = RwSignal::new(false);

    // No theme selector: read whatever CSS vars are present (defaults to
    // Theme::light() when none are defined).
    let theme_memo = Memo::<Theme>::new(|_| theme_from_css_vars());
    let locale_sig = RwSignal::new(Locale::from_browser());
    // e2e-only: surfaces the live on_validation_state_changed callback
    // value in the DOM so Playwright can assert it fires on every
    // keystroke, not just on rejected commits.
    let validation_state = RwSignal::new(None::<(u64, String, String)>);
    // e2e-only: lets the header-height/gutter-width selects below call
    // methods on the mounted grid to reproduce the resize-clipping bug
    // (rs-grid-scene's body_clip_tracks_header_height_after_resize /
    // body_clip_tracks_row_number_width_after_resize) visually.
    // SendWrapper: WASM is single-threaded, this never actually crosses a
    // thread boundary — needed because the reactive view closure below
    // requires its captures to be `Send`.
    let gc_holder: SendWrapper<Rc<RefCell<Option<WebGridCanvas>>>> =
        SendWrapper::new(Rc::new(RefCell::new(None)));

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
                    // e2e-only: resize the header/gutter live to exercise
                    // the clip-clamp bug (see rs-grid-scene's
                    // body_clip_tracks_header_height_after_resize /
                    // body_clip_tracks_row_number_width_after_resize).
                    <select
                        data-testid="header-height-select"
                        on:change={
                            let gc_holder = gc_holder.clone();
                            move |e| {
                                let h = event_target_value(&e)
                                    .parse::<f64>()
                                    .unwrap_or(40.0);
                                if let Some(gc) = gc_holder.borrow().as_ref() {
                                    let mut theme = theme_memo.get_untracked();
                                    theme.header_height = h;
                                    gc.set_theme(theme);
                                }
                            }
                        }
                    >
                        <option value="40" selected=true>"Header: 40px"</option>
                        <option value="150">"Header: 150px"</option>
                    </select>
                    <select
                        data-testid="gutter-width-select"
                        on:change={
                            let gc_holder = gc_holder.clone();
                            move |e| {
                                let w = event_target_value(&e)
                                    .parse::<f64>()
                                    .unwrap_or(60.0);
                                if let Some(gc) = gc_holder.borrow().as_ref() {
                                    gc.set_row_number_width(w);
                                }
                            }
                        }
                    >
                        <option value="60" selected=true>"Gutter: 60px"</option>
                        <option value="150">"Gutter: 150px"</option>
                    </select>
                </div>
                // e2e-only: toggles the row-selection checkbox column live.
                // `position: absolute` (see fixture.css) takes it out of
                // flow so it can't grow `.fixture-header`'s height and shift
                // every pixel-coordinate-based test/snapshot below it. Off
                // by default, so other specs are unaffected unless a test
                // explicitly clicks this button.
                <button
                    data-testid="show-checkbox-column-toggle"
                    style="position:absolute; top:12px; right:16px;"
                    on:click={
                        let gc_holder = gc_holder.clone();
                        move |_| {
                            let next = !show_checkboxes.get_untracked();
                            show_checkboxes.set(next);
                            if let Some(gc) = gc_holder.borrow().as_ref() {
                                gc.set_show_checkbox_column(next);
                            }
                        }
                    }
                >
                    {move || {
                        if show_checkboxes.get() {
                            "Row checkboxes: on"
                        } else {
                            "Row checkboxes: off"
                        }
                    }}
                </button>
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

                    let on_mount = {
                        let gc_holder = gc_holder.clone();
                        Box::new(move |gc: WebGridCanvas| {
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
                            *gc_holder.borrow_mut() = Some(gc);
                        })
                    };
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
