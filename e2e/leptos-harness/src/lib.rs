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

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use example_common::{
    build_columns, build_model, build_row, build_vec_model,
    class_map::resolve_classes, fmt_cols, fmt_rows,
};
use leptos::prelude::*;
use rs_grid_core::{
    column::ColumnDef,
    model::{DataSourceMode, GridModel},
    page_cache::PageCacheDataSource,
    row::RowRecord,
};
use rs_grid_leptos::{theme_from_css_vars, GridCanvas, Locale, WebGridCanvas};
use rs_grid_scene::Theme;
use send_wrapper::SendWrapper;
use wasm_bindgen::{prelude::*, JsCast};

/// e2e-only, manual QA: which `DataSource` backs the grid. Selected via
/// the "data-source-select" control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataSourceKind {
    /// `FnDataSource` — cells generated on demand, no eager allocation.
    /// Default, and what `build_model` (used by every other e2e spec in
    /// this fixture) already returns.
    Fn,
    /// `VecDataSource` — every row materialized eagerly up front, capped
    /// at `example_common::VEC_DEMO_MAX_ROWS`. See `build_vec_model`'s
    /// doc comment for why the cap exists.
    Vec,
    /// `PageCacheDataSource` — no real backend in this fixture, so
    /// `simulate_page_cache_stream` below hand-drives it instead of
    /// `FetchConfig`'s real `window.fetch()` path.
    PageCache,
}

/// Page size used by the `DataSourceKind::PageCache` demo.
const PAGE_SIZE: u64 = 100;

/// Schedule `f` to run once, after `delay_ms`, via `window.setTimeout`.
fn set_timeout_once(delay_ms: i32, f: impl FnOnce() + 'static) {
    let closure = Closure::once_into_js(f);
    let window = web_sys::window().expect("no window");
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.unchecked_ref(),
        delay_ms,
    );
}

/// e2e-only, manual QA: simulates a server streaming pages in over time
/// for a `PageCacheDataSource` — this fixture has no real backend, so it
/// hand-drives `insert_page`/`GridCanvas::set_total_row_count`/
/// `notify_page_loaded` on a timer instead of going through
/// `FetchConfig`'s real `window.fetch()` coordinator (see
/// `rs-grid-web/AGENTS.md`'s "Server-side page fetcher" section for the
/// production equivalent, and `rs-grid-core/AGENTS.md`'s "Row-number
/// gutter width" section for what `set_total_row_count` actually does).
///
/// Only streams in `PAGES_TO_STREAM` pages regardless of `real_total` —
/// scrolling past that stays in the `CellStatus::Loading` skeleton state
/// forever, since there's no viewport-aware fetch trigger wired up here
/// (that's `FetchConfig`'s job in a real app, driven by the actual
/// visible row range). Good enough to see pages stream in and the gutter
/// grow once the first "response" reveals `real_total`, without building
/// out scroll-driven fetching for a fixture that has nothing to fetch
/// from.
///
/// `generation`/`my_gen`: the reactive view block that calls this rebuilds
/// the whole model (a fresh `GridCanvas`/`PageCacheDataSource`) on every
/// row/column/data-source change, but does nothing to cancel timers
/// already scheduled by a previous call — `window.setTimeout` has no
/// owning Rust value to `Drop`. Without this guard, switching away from
/// PageCache mode (or just bumping row/col count) mid-stream leaves the
/// old timer chain running for up to `PAGES_TO_STREAM * DELAY_MS` more
/// milliseconds, still calling `insert_page`/`set_total_row_count` on the
/// now-orphaned `cache`/`gc` — harmless (nothing else holds a reference to
/// either), but wasted work and confusing if you're watching the gutter
/// resize live. `App`'s `stream_generation: Rc<Cell<u32>>` is bumped once
/// per rebuild; each tick checks it's still current before doing anything.
fn simulate_page_cache_stream(
    gc: WebGridCanvas,
    cache: PageCacheDataSource,
    columns: Vec<ColumnDef>,
    real_total: u64,
    generation: Rc<Cell<u32>>,
    my_gen: u32,
) {
    const PAGES_TO_STREAM: u64 = 5;
    const DELAY_MS: i32 = 400;

    fn stream_page(
        page_num: u64,
        gc: WebGridCanvas,
        cache: PageCacheDataSource,
        columns: Vec<ColumnDef>,
        real_total: u64,
        generation: Rc<Cell<u32>>,
        my_gen: u32,
    ) {
        set_timeout_once(DELAY_MS, move || {
            if generation.get() != my_gen {
                return; // Superseded by a later rebuild — stop here.
            }
            let rows: Vec<RowRecord> = (page_num * PAGE_SIZE
                ..(page_num + 1) * PAGE_SIZE)
                .map(|row| build_row(row, &columns))
                .collect();
            cache.insert_page(page_num, rows);
            if page_num == 0 {
                // The first "server response" is what reveals the real
                // total in a real app — both calls are required, not
                // just one: `cache.set_total_rows` is what the grid
                // actually scrolls against (PageCacheDataSource::
                // row_count, the DataSource's own count — without this
                // the grid stays bounded by the placeholder total passed
                // to `PageCacheDataSource::new`, however many pages get
                // streamed in afterward); `gc.set_total_row_count` only
                // updates the derived UI state (the gutter width) and
                // cannot touch the DataSource generically — see its doc
                // comment in rs-grid-web and canvas/fetcher.rs's real
                // FetchConfig coordinator, which also calls both.
                cache.set_total_rows(real_total);
                gc.set_total_row_count(real_total);
            }
            gc.notify_page_loaded();
            if page_num + 1 < PAGES_TO_STREAM {
                stream_page(
                    page_num + 1,
                    gc,
                    cache,
                    columns,
                    real_total,
                    generation,
                    my_gen,
                );
            }
        });
    }

    stream_page(0, gc, cache, columns, real_total, generation, my_gen);
}

#[component]
fn App() -> impl IntoView {
    let row_count = RwSignal::new(1_000u64);
    let col_count = RwSignal::new(20usize);
    // e2e-only: whether the row-selection checkbox column is shown. A plain
    // `<button>` (not an `<input>`) so it doesn't trip editing.spec.ts's
    // "no <input> exists in the DOM" assertion for the editor=None case.
    let show_checkboxes = RwSignal::new(false);
    // e2e-only: whether the floating filter row is shown — same rationale
    // as `show_checkboxes` (plain `<button>`, off by default).
    let show_filter_row = RwSignal::new(false);
    // e2e-only, manual QA: which DataSource backs the grid — see the
    // "data-source-select" control and DataSourceKind below.
    let data_source = RwSignal::new(DataSourceKind::Fn);
    // e2e-only: bumped once per model rebuild so a stale
    // simulate_page_cache_stream timer chain from a previous rebuild can
    // tell it's been superseded and stop — see that function's doc
    // comment. SendWrapper for the same reason as gc_holder below.
    let stream_generation: SendWrapper<Rc<Cell<u32>>> =
        SendWrapper::new(Rc::new(Cell::new(0)));

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
                    // e2e-only, manual QA: switches which DataSource
                    // backs the grid — Fn (on-demand, the default),
                    // Vec (eager, row-count capped), or PageCache
                    // (simulated server streaming, see
                    // simulate_page_cache_stream below). Changing this
                    // rebuilds the model from scratch (read inside the
                    // `<div class="fixture-grid">` closure below), same
                    // as changing the row/column count selects above.
                    <select
                        data-testid="data-source-select"
                        on:change=move |e| {
                            let v = event_target_value(&e);
                            data_source.set(match v.as_str() {
                                "vec" => DataSourceKind::Vec,
                                "pagecache" => DataSourceKind::PageCache,
                                _ => DataSourceKind::Fn,
                            });
                        }
                    >
                        <option value="fn" selected=true>
                            "Data source: Fn (virtual, on-demand)"
                        </option>
                        <option value="vec">
                            "Data source: Vec (in-memory, capped)"
                        </option>
                        <option value="pagecache">
                            "Data source: PageCache (simulated streaming)"
                        </option>
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
                // e2e-only: toggles the floating filter row live — same
                // rationale/positioning as the checkbox-column toggle above.
                <button
                    data-testid="show-filter-row-toggle"
                    style="position:absolute; top:44px; right:16px;"
                    on:click={
                        let gc_holder = gc_holder.clone();
                        move |_| {
                            let next = !show_filter_row.get_untracked();
                            show_filter_row.set(next);
                            if let Some(gc) = gc_holder.borrow().as_ref() {
                                gc.set_show_filter_row(next);
                            }
                        }
                    }
                >
                    {move || {
                        if show_filter_row.get() {
                            "Filter row: on"
                        } else {
                            "Filter row: off"
                        }
                    }}
                </button>
            </div>
            <div class="fixture-grid">
                {move || {
                    // Invalidates any simulate_page_cache_stream timer
                    // chain scheduled by a previous rebuild — see that
                    // function's doc comment.
                    let my_gen = stream_generation.get().wrapping_add(1);
                    stream_generation.set(my_gen);

                    let kind = data_source.get();
                    // Only Fn/Vec build the demo columns internally
                    // (build_model/build_vec_model each call
                    // build_columns themselves); PageCache needs its own
                    // handle to the column set to synthesize streamed
                    // rows later, so it's built once here and reused.
                    let page_cache_setup = match kind {
                        DataSourceKind::PageCache => {
                            let columns = build_columns(col_count.get());
                            let cache =
                                PageCacheDataSource::new(PAGE_SIZE, PAGE_SIZE);
                            Some((columns, cache))
                        }
                        DataSourceKind::Vec | DataSourceKind::Fn => None,
                    };
                    let mut model = match kind {
                        DataSourceKind::Vec => {
                            build_vec_model(row_count.get(), col_count.get())
                        }
                        DataSourceKind::Fn => {
                            build_model(row_count.get(), col_count.get())
                        }
                        DataSourceKind::PageCache => {
                            let (columns, cache) =
                                page_cache_setup.clone().expect("set above");
                            let mut m = GridModel::with_data_source(
                                columns,
                                Box::new(cache),
                                40.0,
                                60.0,
                            );
                            m.mode = DataSourceMode::ServerSide;
                            m
                        }
                    };
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
                        let page_cache_setup = page_cache_setup.clone();
                        let real_total = row_count.get();
                        let stream_generation = stream_generation.clone();
                        Box::new(move |gc: WebGridCanvas| {
                            let theme = theme_memo.get_untracked();
                            gc.set_class_resolver(Rc::new(move |raw| {
                                resolve_classes(raw, &theme)
                            }));
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
                            if let Some((columns, cache)) = page_cache_setup {
                                simulate_page_cache_stream(
                                    gc.clone(),
                                    cache,
                                    columns,
                                    real_total,
                                    (*stream_generation).clone(),
                                    my_gen,
                                );
                            }
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
