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

use std::{cell::RefCell, rc::Rc, time::Duration};

use example_common::{
    build_model, class_map::resolve_classes, fmt_cols, fmt_rows,
};
use leptos::prelude::*;
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;

/// Which side of the cell the dev toast is anchored to — demonstrates
/// that `GridCanvas::cell_client_rect` gives the integrator full control
/// over placement; rs-grid itself has no opinion here.
#[derive(Clone, Copy, PartialEq)]
enum TooltipPlacement {
    Top,
    Bottom,
    Left,
    Right,
}

/// Compute `(left, top, css_transform)` for `rect` (the cell's client
/// rect) so the toast sits `gap` px to the given side, centered on the
/// cell's other axis. The `transform` offsets the toast by its own size
/// so callers never need to know the toast's dimensions up front.
fn tooltip_position(
    placement: TooltipPlacement,
    rect: (f64, f64, f64, f64),
    gap: f64,
) -> (f64, f64, &'static str) {
    let (l, t, w, h) = rect;
    match placement {
        TooltipPlacement::Bottom => {
            (l + w / 2.0, t + h + gap, "translateX(-50%)")
        }
        TooltipPlacement::Top => {
            (l + w / 2.0, t - gap, "translate(-50%, -100%)")
        }
        TooltipPlacement::Left => {
            (l - gap, t + h / 2.0, "translate(-100%, -50%)")
        }
        TooltipPlacement::Right => {
            (l + w + gap, t + h / 2.0, "translateY(-50%)")
        }
    }
}

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
    // Dev-only mini toast, bottom-right, built entirely from the generic
    // on_validation_state_changed API — demonstrates that rs-grid does
    // not impose a validation-error widget. Fades out after 3s.
    let toast_text = RwSignal::new(String::new());
    let toast_visible = RwSignal::new(false);
    let toast_generation = RwSignal::new(0u64);
    // (left, top, width, height) of the cell that emitted the current
    // error, from GridCanvas::cell_client_rect — lets the toast be
    // positioned relative to that cell instead of a fixed corner.
    let toast_rect = RwSignal::new(None::<(f64, f64, f64, f64)>);
    let toast_placement = RwSignal::new(TooltipPlacement::Bottom);
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
            // Dev-only mini toast built entirely from the generic
            // on_validation_state_changed + cell_client_rect APIs — shows
            // rs-grid does not impose a validation-error widget or its
            // position. Placed just below the failing cell; falls back
            // to the bottom-right corner if no rect is available.
            <div
                style=move || {
                    let (left, top, transform) = match toast_rect.get() {
                        Some(rect) => {
                            tooltip_position(toast_placement.get(), rect, 6.0)
                        }
                        None => (16.0, 16.0, "none"),
                    };
                    format!(
                        "position:fixed;left:{left}px;top:{top}px;\
                         transform:{transform};padding:8px 14px;\
                         background:#dc2626;color:#fff;border-radius:6px;\
                         font-size:13px;box-shadow:0 2px 8px rgba(0,0,0,.25);\
                         transition:opacity 300ms ease;\
                         pointer-events:none;z-index:20000;opacity:{}",
                        if toast_visible.get() { 1 } else { 0 },
                    )
                }
            >
                {move || toast_text.get()}
            </div>
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
                    // Dev-only: which side of the cell the validation
                    // toast is anchored to.
                    <select
                        data-testid="toast-placement"
                        on:change=move |e| {
                            let p = match event_target_value(&e).as_str() {
                                "top" => TooltipPlacement::Top,
                                "left" => TooltipPlacement::Left,
                                "right" => TooltipPlacement::Right,
                                _ => TooltipPlacement::Bottom,
                            };
                            toast_placement.set(p);
                        }
                    >
                        <option value="bottom" selected=true>"Toast: bottom"</option>
                        <option value="top">"Toast: top"</option>
                        <option value="left">"Toast: left"</option>
                        <option value="right">"Toast: right"</option>
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
                                "tooltip tooltip-open tooltip-error"
                                    .to_string(),
                            ));
                            *gc_holder.borrow_mut() = Some(gc);
                        })
                    };
                    let on_validation_error = Box::new(
                        move |_row: u64, _col: String, _msg: String| {},
                    );
                    let on_validation_state_changed = {
                        let gc_holder = gc_holder.clone();
                        Box::new(move |state: Option<(u64, String, String)>| {
                            validation_state.set(state.clone());
                            match state {
                                Some((row, col_key, msg)) => {
                                    let rect = gc_holder
                                        .borrow()
                                        .as_ref()
                                        .and_then(|gc| {
                                            gc.cell_client_rect(row, &col_key)
                                        });
                                    toast_rect.set(rect);
                                    toast_text.set(msg);
                                    toast_visible.set(true);
                                    let gen = toast_generation.get_untracked() + 1;
                                    toast_generation.set(gen);
                                    set_timeout(
                                        move || {
                                            if toast_generation.get_untracked() == gen {
                                                toast_visible.set(false);
                                            }
                                        },
                                        Duration::from_secs(3),
                                    );
                                }
                                None => toast_visible.set(false),
                            }
                        })
                    };
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
