---
name: rs-grid
description: >
  High-performance, renderer-agnostic data grid engine built with Rust and
  compiled to WebAssembly. Use when embedding a virtualized data grid in a
  Leptos, Dioxus, or Yew CSR application, adding Canvas2D-based grid rendering,
  or implementing large-dataset table views in the browser.
license: MIT
compatibility: >
  Requires Rust 2024 edition, wasm32-unknown-unknown target, and Trunk for
  WASM builds. Leptos/Dioxus/Yew CSR only (no SSR). Browser Canvas2D API at
  runtime.
metadata:
  author: ruxelion
  version: "0.1"
  repository: https://github.com/ruxelion/rs-grid
---

# rs-grid

High-performance data grid engine for the browser, written in Rust and compiled
to WebAssembly. Handles millions of rows with smooth 60 fps rendering via a
virtualized Canvas2D renderer.

## Capabilities

- Render large datasets (up to ~9×10¹⁴ rows) with viewport virtualization
- O(log n) hit-testing via precomputed column offset arrays
- Cell, row, and column selection with anchor/focus model (keyboard + mouse)
- Renderer-agnostic scene graph (ScenePrimitive) — Canvas2D backend included
- Inline cell editing: Text input, Select dropdown with optional icons
- Per-column validation with real-time error feedback
- Column sorting (single column, asc/desc, togglable), per-column condition
  filtering (contains, equals, greater than, ...) via a header filter icon
  + popup, plus an optional AG-Grid-style floating filter row for a quick
  "contains" filter per column
- Full-text search with match highlighting (Ctrl+F pattern)
- Undo/redo history stack (edit operations only)
- Clipboard support: copy/cut/paste TSV (RFC 4180)
- Context menu (built-in actions + custom items)
- Server-side pagination via PageCache + async fetch callback
- CSS variable theming (light, dark, dimmed built-in; fully customizable)
- 15 built-in locales (en, fr, de, es, it, pt, nl, pl, ru, uk, ar, zh, ja, ko, tr)
- Callbacks: on_change, on_columns_changed, on_selection_changed,
  on_validation_error, on_cell_button_click
- Cell buttons (ButtonDef: Primary, Secondary, Danger, Ghost styles)
- Cell formats: Number, Percent, Currency, Boolean, Image, ImageText,
  ProgressBar, Custom, Styled
- Pinned (frozen) columns on the left
- Column reordering (drag-and-drop), resizing, auto-fit
- Leptos CSR, Dioxus CSR, and Yew CSR component wrappers out of the box
- Custom renderer backend by consuming SceneFrame primitives

## Skills

### Add rs-grid to a Leptos project

1. Add dependencies to `Cargo.toml`:

   ```toml
   rs-grid-core   = { git = "https://github.com/ruxelion/rs-grid", tag = "rs-grid-core-v0.1.3" }
   rs-grid-leptos = { git = "https://github.com/ruxelion/rs-grid", tag = "rs-grid-core-v0.1.3" }
   ```

2. Build a `GridModel` and mount `<GridCanvas>` in a Leptos view:

   ```rust
   use rs_grid_core::{column::ColumnDef, model::GridModel, row::RowRecord};
   use rs_grid_leptos::GridCanvas;

   let rows: Vec<RowRecord> = (0..1000).map(|i| {
       let mut r = RowRecord::new(i);
       r.set("name", format!("User {i}"));
       r.set("age", format!("{}", 20 + i % 50));
       r
   }).collect();

   let columns = vec![
       ColumnDef::new("name", "Name", 200.0),
       ColumnDef::new("age", "Age", 80.0),
   ];
   let model = GridModel::new(columns, rows, 32.0, 40.0);

   view! { <GridCanvas model=model width="100%" height="600px" /> }
   ```

3. Apply `--rs-grid-*` CSS custom properties on `:root` for theming.

### Mutate grid state via GridCommand

All mutations use `GridState::apply(GridCommand)`. In Leptos/Dioxus/Yew
components, access state via the `on_mount` callback which provides a
`GridCanvas` handle with an `apply()` method:

```rust
<GridCanvas
    model=model
    width="100%"
    height="600px"
    on_mount=Box::new(move |canvas| {
        canvas.apply(GridCommand::SelectCell(CellCoord { row: 0, col: 0 }));
    })
/>
```

#### Selection

```rust
SelectCell(CellCoord { row: u64, col: usize })
ExtendSelection(CellCoord { row: u64, col: usize }) // Shift+click equivalent
ClearSelection
MoveSelection { delta_row: i64, delta_col: i64, extend: bool }
SelectRow(u64)
ExtendRowSelection(u64)
SelectCol(usize)
ExtendColSelection(usize)
```

#### Row checkboxes

```rust
ToggleRowChecked(u64)          // logical row index — click on the checkbox column
ToggleAllFilteredChecked       // toggles all rows passing the active filter (or all rows, if unfiltered)
SetShowCheckboxColumn(bool)    // show/hide the row-selection checkbox column
SetCheckboxColumnWidth(f64)    // width in logical px (checkbox stays centered — controls the margin too)
```

#### Scrolling & Viewport

```rust
ScrollTo { x: f64, y: f64 }        // absolute scroll position in pixels
ScrollBy { dx: f64, dy: f64 }       // relative delta
Resize { width: f64, height: f64 }  // update canvas dimensions
```

#### Columns

```rust
ResizeColumn { col_idx: usize, new_width: f64 }
CommitColumnResize { col_idx: usize, old_width: f64, old_flex: Option<f64> }
AutoFitColumn {
    col_idx: usize, char_width: f64,
    header_char_width: f64, cell_padding: f64, header_right_reserve: f64,
}
AutoFitAllColumns {
    char_width: f64, header_char_width: f64,
    cell_padding: f64, header_right_reserve: f64,
}
MoveColumn { from_idx: usize, to_idx: usize }
SetPinnedColumnCount { count: usize }
```

#### Sorting

```rust
ToggleSort { col_key: String }               // cycles asc → desc → off
SetSort { col_key: String, dir: SortDir }    // SortDir::Asc or SortDir::Desc
ClearSort
```

#### Filtering

```rust
SetColumnFilter { col_key: String, condition: FilterCondition } // operator + value
SetColumnValueFilter { col_key: String, values: HashSet<String> } // checklist filter — AND-combined with the condition
ClearColumnValueFilter { col_key: String }
ClearAllFilters
SetShowFilterRow(bool)      // show/hide the floating filter row (AG-Grid-style quick "contains" row)
SetFilterRowHeight(f64)     // height in logical px, ignored if <= 0.0
```

`FilterCondition { op: FilterOp, value: String }`. `FilterOp` variants:
`Contains`, `NotContains`, `StartsWith`, `EndsWith`, `Equals`, `NotEquals`,
`Blank`, `NotBlank`, `GreaterThan`, `GreaterThanOrEqual`, `LessThan`,
`LessThanOrEqual`. `Equals`/`NotEquals` compare numerically when the
column's `CellFormat::is_numeric_like()`, otherwise case-insensitive
string equality; the four `*Than*` operators always compare numerically
(non-numeric cells never match, never panic).

`GridModel::unique_values(col_key, cap) -> UniqueValues` returns
`Values(Vec<String>)` (sorted distinct values, up to `cap`) or
`TooMany { cap }` once exceeded — backs the checklist filter's value
list. `GridModel::is_filter_applied() -> bool` distinguishes "no filter
active" from "a filter is active and matches zero rows" — both leave
`filtered_indices` empty otherwise.

#### Editing

```rust
StartEdit { row: u64, col_key: String }
CommitEdit { row: u64, col_key: String, value: String }
CancelEdit
ValidateEdit { value: String }  // live re-check of the in-progress edit, no commit
ClearCells  // Delete/Backspace — clears selected cells to "" (→ CommandOutput::CellsCleared), no clipboard involved
```

#### Clipboard

```rust
CopySelection    // → CommandOutput::CopyText(tsv) or CopyError
CutSelection     // copy then clear cells
PasteAt { text: String }  // TSV text pasted at selection anchor
```

#### Search

```rust
Search { query: String }  // set query, highlights all matches
SearchNext                // move to next match
SearchPrev                // move to previous match
ClearSearch
```

#### Undo / Redo

```rust
Undo
Redo
```

#### Display

```rust
SetHeaderHeight(f64)
SetRowHeight(f64)
SetShowHeader(bool)
SetShowRowNumbers(bool)
SetHoveredRow(Option<u64>)
```

#### Behaviour toggles

```rust
SetEditable(bool)           // global inline-edit on/off
SetSelectable(bool)         // selection on/off; clears selection when false
SetColumnReorderable(bool)  // header drag-to-reorder on/off
SetInvalidEditMode(InvalidEditMode)  // Revert (default) or Block on invalid CommitEdit
```

#### Server-side data

```rust
NotifyPageLoaded         // signal that a page fetch completed
SetTotalRowCount(u64)    // update total row count (server-side mode)
```

### Configure event callbacks

Attach callbacks on the `GridCanvas` handle returned by `on_mount`:

```rust
canvas.set_on_change(move || {
    // fires after any state mutation (edit commit, sort, filter, …)
    let widths = canvas.column_widths(); // Vec<(String, f64)>
    save_to_localstorage(&widths);
});

canvas.set_on_selection_changed(move || {
    let rows = canvas.selected_row_indices(); // Vec<u64> logical indices
});

canvas.set_on_validation_error(move |row, col_key, message| {
    // Fires whenever a CommitEdit is rejected, both when the value
    // reverts (InvalidEditMode::Revert, the default) and when the
    // editor stays open (InvalidEditMode::Block).
    log::warn!("Validation error at [{row},{col_key}]: {message}");
});

canvas.set_on_validation_state_changed(move |state| {
    // Fires on every StartEdit/ValidateEdit/CommitEdit/CancelEdit —
    // i.e. on every keystroke, not just rejected commits. `state` is
    // `Some((row, col_key, message))` while the in-progress edit is
    // invalid, `None` otherwise. rs-grid does not impose a widget for
    // this — build your own tooltip/banner/icon with it:
    match state {
        Some((row, col_key, message)) => show_my_tooltip(row, &col_key, &message),
        None => hide_my_tooltip(),
    }
});
// A zero-config native `title` attribute is applied to the edit
// <input> by default. Disable it if it competes with your own UI:
canvas.set_native_validation_tooltip(false);

canvas.set_on_cell_button_click(move |row, col_key, button_id| {
    // triggered when a ButtonDef cell button is clicked
});
```

The current validation state can also be read on demand (not just via the
callback) with `canvas.validation_error() -> Option<(u64, String, String)>`.

A cell can be invalid *at rest* — never edited, just loaded that way from the
data source. `canvas.cell_validation_error(row, col_key) -> Option<String>`
checks this independently of any active edit session. rs-grid also has a
built-in (no-visual-of-its-own) hover mechanism for it: a single reused DOM
tooltip, positioned automatically, shown only while hovering an invalid
cell — reproduce daisyUI's tooltip by giving it a class:

```rust
canvas.set_validation_tooltip_class(Some(
    "tooltip tooltip-open tooltip-error".into(),
));
// `tooltip-open` is required — this element never receives a real
// `:hover` (pointer-events: none; the mouse stays over the canvas).
// Without a class set, the tooltip is positioned but invisible.
```

### Set up inline editing

```rust
use rs_grid_core::column::{ColumnDef, CellEditor, CellValidator, SelectOption};

// Text editor with a legacy free-form validator
let name_col = ColumnDef::new("name", "Name", 200.0)
    .with_editor(CellEditor::Text)
    .with_validator(CellValidator::new(|v| {
        if v.trim().is_empty() {
            Err("Name cannot be empty".into())
        } else {
            Ok(())
        }
    }));

// Select dropdown editor with options
let role_col = ColumnDef::new("role", "Role", 150.0)
    .with_editor(CellEditor::Select {
        options: vec![
            SelectOption {
                value: "admin".into(),
                label: "Admin".into(),
                icon: None,
            },
            SelectOption {
                value: "user".into(),
                label: "User".into(),
                icon: None,
            },
        ],
    });
```

### Declarative validation rules (`ValidationRule`)

Business-rule validation, checked (in order) before every `CommitEdit`.
Each built-in rule has a default message, overridable with
`.with_message(...)`. Combine with sugar builders on `ColumnDef`, or the
enum directly via `.with_rules(...)`:

```rust
use rs_grid_core::{
    column::{CellEditor, ColumnDef},
    validation::{InvalidEditMode, ValidationRule},
};

let doc_label = ColumnDef::new("doc_label", "Document", 200.0)
    .with_editor(CellEditor::Text)
    .required()
    .with_max_length(20)
    .with_allowed_values(vec!["INV".into(), "PO".into(), "CN".into()])
    .with_range(0.0, 100.0) // combine as many rules as needed
    .with_rules(vec![
        ValidationRule::one_of(vec!["A".into(), "B".into()])
            .with_message("Must be A or B"),
    ]);
```

`ValidationRule` variants: `Required`, `MinLength`, `MaxLength`, `Range`,
`OneOf` (allowed-value list), `Custom(CellValidator)` (arbitrary closure,
for regex-like patterns or cross-field checks). Rules run before the
legacy `validator` field, so both can coexist during a migration.

By default an invalid `CommitEdit` reverts the cell to its previous value
(`InvalidEditMode::Revert`). Switch to `InvalidEditMode::Block` to keep
the editor open until the value is corrected:

```rust
use rs_grid_core::{commands::GridCommand, validation::InvalidEditMode};

canvas.dispatch(GridCommand::SetInvalidEditMode(InvalidEditMode::Block));
```

Validation isn't limited to `CommitEdit`: `GridCommand::PasteAt` also
validates each target cell and silently skips writing the ones that fail,
applying the rest of the pasted block normally (mirrors Excel/AG Grid — a
rejection, when it exists, is always per-cell, never all-or-nothing). And a
cell doesn't need an active edit session to be flagged — `rs-grid-scene`'s
cell renderer checks every visible cell's current value and draws a themed
`Theme::invalid_cell_border` around it when invalid, so data that's already
bad when loaded is visible immediately, not just after the user clicks in.

The grid also validates live, on every keystroke (not just on commit),
via `GridCommand::ValidateEdit` dispatched internally by `rs-grid-web`.
While the value is invalid, the inline editor's border and background
switch from `--rs-grid-editor-border` / `--rs-grid-editor-bg` to
`--rs-grid-editor-border-invalid` (default `#dc2626`) /
`--rs-grid-editor-bg-invalid` (default `#fef2f2`).

### Per-cell editability (`EditablePredicate`)

`ColumnDef::editable` locks an entire column. For per-cell (row+col)
locking — AG Grid's `colDef.editable` callback equivalent — attach a
dynamic predicate with `.editable_when(...)`. The closure receives the
row index and the full `GridModel`, so it can implement cross-column
logic (not just this column's own value):

```rust
use rs_grid_core::column::ColumnDef;

let notes = ColumnDef::new("notes", "Notes", 160.0).editable_when(
    |row, model| model.get_cell(row, "status").as_deref() != Some("locked"),
);
```

Checked only when the grid-wide `GridModel.editable` and the static
per-column `editable` flag are both `true` — either one short-circuits
before the predicate is ever called (mirrors the `rules` → `validator`
layering above). `ColumnDef::is_cell_editable(row, model)` resolves all
three layers and is the single source of truth reused by `rs-grid-web`'s
cursor logic, `PasteAt`/`CutSelection`, and `rs-grid-scene`'s cell
renderer.

When a cell resolves to non-editable: `rs-grid-web` shows a
`not-allowed` cursor on hover and `PasteAt`/`CutSelection` skip the cell,
and the canvas paints a `--rs-grid-locked-cell-bg` background wash behind
the cell regardless of its `CellFormat` (transparent `locked_cell_bg` = no
visual change). The `--rs-grid-locked-cell-text` color swap is scoped to
the default plain-text renderer only — `Styled`/`Image`/`ImageText`/
`ProgressBar` composite formats keep their own colors, but still get the
background wash.

### Per-cell decoration (`CellDecorator`)

Persistent, at-rest visual annotation for a single cell — a red border or
tinted background that stays visible whenever the cell is on screen, not
just while it's being edited. Purely cosmetic: unlike `rules`/`validator`,
it never blocks a commit, only changes how the cell is painted. Attach a
dynamic decorator with `.decorated_when(...)`, mirroring `.editable_when`'s
`Fn(row, &GridModel) -> ...` shape so the closure can implement
cross-column logic — e.g. flagging a row where two paired columns are
inconsistent:

```rust
use rs_grid_core::column::{CellDecoration, ColumnDef};

let doc1_file = ColumnDef::new("doc1_file", "Doc 1 file", 160.0)
    .decorated_when(|row, model| {
        let file = model.get_cell(row, "doc1_file").unwrap_or_default();
        let label = model.get_cell(row, "doc1_label").unwrap_or_default();
        (file.is_empty() != label.is_empty()).then(|| {
            CellDecoration::default().with_border_color([239, 68, 68, 255])
        })
    });
```

`#[non_exhaustive]` blocks struct-literal construction for `CellDecoration`
from outside `rs-grid-core` — build one from `CellDecoration::default()`
chained with `.with_border_color(...)`, `.with_background_tint(...)`, and/or
`.with_message(...)`.

Evaluated every frame for every visible cell, same cost profile as
`CellFormat::Custom` and `EditablePredicate` — no static gate (unlike
`is_cell_editable`'s 3-layer stack), since decoration is purely cosmetic. The
border/tint colors are consumer-supplied `[u8; 4]` RGBA, not read from the
theme (same precedent as `FormattedCell::color`) — only the border's stroke
width is themed, via `--rs-grid-decoration-border-width` (default `1.5px`).
Rendered after the built-in locked/invalid overlays, so a decoration layers
on top of them rather than being suppressed. `CellDecoration::message` is
reserved for a future native hover tooltip — it's not rendered anywhere yet.

### Per-row cell-button visibility (`CellButtonsVisible`)

`ColumnDef::cell_buttons` is column-level by default — the same buttons are
drawn on every row. To hide buttons on rows with no meaningful action (e.g.
a product with no known URL), attach a dynamic predicate with
`.cell_buttons_visible_when(...)`, mirroring `.editable_when`'s
`Fn(row, &GridModel) -> bool` shape:

```rust
use rs_grid_core::column::{ButtonDef, ButtonStyle, ColumnDef};

let actions = ColumnDef::new("url", "Actions", 160.0)
    .with_cell_buttons(vec![ButtonDef::new("open", "Open", ButtonStyle::Primary)])
    .cell_buttons_visible_when(|row, model| {
        model.get_cell(row, "url").as_deref().is_some_and(|u| !u.is_empty())
    });
```

No static gate (unlike `is_cell_editable`'s 3-layer stack) — same shape as
`CellDecorator`, since an empty `cell_buttons` vec already serves as the
static "no buttons" case. `ColumnDef::are_cell_buttons_visible(row, model)`
resolves the predicate (`true` when none is set, matching prior behaviour)
and is checked by `rs-grid-scene`'s `emit_cell_buttons` before drawing
anything or registering a click hit-zone — a hidden row emits no primitives
at all for that cell's buttons.

### Enable server-side pagination

```rust
use rs_grid_web::canvas::fetcher::{FetchConfig, PageFetchRequest, PageFetchResponse};

canvas.enable_async_fetch(
    page_cache, // PageCacheDataSource
    FetchConfig {
        build_url: Box::new(|req: &PageFetchRequest| {
            format!(
                "/api/rows?page={}&size={}", req.page_num, req.page_size
            )
        }),
        parse_response: Box::new(|js_val| {
            // deserialize JSON → PageFetchResponse { rows, total_rows }
            todo!()
        }),
        buffer_pages: 2,
    },
);
```

### Apply CSS variable theming

```css
/* In your global CSS */
:root {
    --rs-grid-bg: #ffffff;
    --rs-grid-header-bg: #f5f5f5;
    --rs-grid-cell-text: #1a1a1a;
    --rs-grid-selection-bg: rgba(66, 133, 244, 0.2);
    /* see /theming/css-variables for the full list */
}
```

```rust
use rs_grid_web::css_theme::theme_from_css_vars;

let theme = theme_from_css_vars(); // reads CSS vars from document
canvas.set_theme(theme);
```

### Read current state from GridCanvas

```rust
let widths: Vec<(String, f64)> = canvas.column_widths();
let order:  Vec<String>         = canvas.column_order();
let sel:    Vec<u64>            = canvas.selected_row_indices();
let pinned: usize               = canvas.pinned_count();
let cell:   Option<String>      = canvas.cell_at_logical(row, "col_key");
```

### Add a new renderer backend

1. Create a crate depending on `rs-grid-scene`
2. Call `SceneBuilder::build(&grid_state, &theme)` → `SceneFrame`
3. Iterate over `SceneFrame::primitives()` (Rect, Text, Line, Polygon, Image)
4. Issue draw calls — never modify `rs-grid-core` or `rs-grid-scene`

## Constraints

- `rs-grid-core` has no WASM dependency — keep it that way
- Row indices must use `u64`, not `usize` (WASM32 address space is 4 GB)
- Hit-testing must remain O(log n) via precomputed column offsets
- All mutations go exclusively through `GridState::apply(GridCommand)` —
  never mutate fields directly
- Max line width: 80 characters
- No `unwrap()` in production code — use `expect("reason")` or `?`
- All public enums are `#[non_exhaustive]` — always add a `_` wildcard arm
- Callbacks are `Fn`/`FnOnce` closures, not `Send + Sync` — WASM is
  single-threaded; no `Arc`, no channels
- `GridCanvas::mount()` is one-time; call `detach()` before re-mounting
- Sorting is skipped when row count exceeds `GridModel::MAX_CLIENT_SORT_ROWS`
  (1,000,000); use server-side sort via PageCache for larger datasets
- `CommitColumnResize` is for undo history only — use `ResizeColumn` for
  programmatic resizing

## Workflows

### Run tests

```sh
# Native crates (WASM crates excluded — require a browser)
cargo nextest run --workspace \
  --exclude rs-grid-web --exclude rs-grid-leptos \
  --exclude rs-grid-dioxus --exclude rs-grid-yew \
  --exclude rs-grid-render-canvas \
  --exclude leptos-harness --exclude example-common

cargo clippy --workspace -- -D warnings
```

### Run the e2e fixture locally

```sh
cd e2e/leptos-harness && trunk serve
# → http://localhost:9079
```

### Inline editing lifecycle

1. User double-clicks a cell (or programmatic `StartEdit { row, col_key }`)
2. Editor opens — `GridState.edit` is `Some(EditCell { row, col_key, … })`
3. User confirms → `CommitEdit { row, col_key, value }` — triggers on_change
4. User presses Escape → `CancelEdit` — no state change, editor closes

### Sort + filter programmatically

```rust
// Sort "revenue" descending
state.apply(GridCommand::SetSort {
    col_key: "revenue".into(),
    dir: SortDir::Desc,
});

// Filter "country" to "France" (contains, the common case)
state.apply(GridCommand::SetColumnFilter {
    col_key: "country".into(),
    condition: FilterCondition::contains("France"),
});

// Filter "revenue" greater than 100000
state.apply(GridCommand::SetColumnFilter {
    col_key: "revenue".into(),
    condition: FilterCondition {
        op: FilterOp::GreaterThan,
        value: "100000".into(),
    },
});

// Clear everything
state.apply(GridCommand::ClearSort);
state.apply(GridCommand::ClearAllFilters);
```

### Server-side pagination with PageCache

1. Build a `PageCacheDataSource` with the initial total row count
2. Call `canvas.enable_async_fetch(cache, config)` (URL builder + parser)
3. Grid fetches pages on demand; cells show `Loading` during fetches
4. JS callback calls `GridCommand::NotifyPageLoaded` on completion
5. Update count with `GridCommand::SetTotalRowCount(n)` after server response

### Read current state

```rust
// In on_change callback (or any time after a mutation):
let widths  = canvas.column_widths();          // Vec<(String, f64)>
let order   = canvas.column_order();           // Vec<String>
let pinned  = canvas.pinned_count();           // usize
let sel     = canvas.selected_row_indices();   // Vec<u64>
let cell    = canvas.cell_at_logical(row, "key"); // Option<String>
```
