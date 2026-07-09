# rs-grid-scene

Scene graph layer. Converts a `GridState` into a list of renderer-agnostic
drawing primitives.

## Modules

| Module | Role |
|---|---|
| `builder` | `SceneBuilder`: takes a `&GridState` + `Theme` and produces a `SceneFrame` |
| `frame` | `SceneFrame`: ordered list of `ScenePrimitive` for one frame |
| `primitives` | Primitive types: `RectPrimitive`, `TextPrimitive`, `LinePrimitive`, `PolygonPrimitive` |
| `theme` | `Theme`: colours and sizes for rendering |
| `css_vars` | `Theme` ↔ `--rs-grid-*` CSS variables (writer + reader, single source of truth; round-trip test enforces parity) |

## Critical invariants

- This crate **knows nothing about Canvas2D, WebGL, or any renderer**. It
  produces data — it does not draw.
- `SceneFrame` is an immutable value produced each frame — no mutable internal
  state between frames.
- Always reason in **logical coordinates** (DPR-independent pixels).
  The renderer applies the `devicePixelRatio`.
- The order of primitives in `SceneFrame` defines the draw order (back-to-front).

## Available primitives

- `ScenePrimitive::Rect` — filled rectangle, optional stroke, optional rounded corners
- `ScenePrimitive::Text` — clipped text, left/right alignment
- `ScenePrimitive::Line` — line segment
- `ScenePrimitive::Polygon` — filled convex polygon, optional rounded corners

## Cell content rendering (`builder/cells.rs`)

`emit_cell` dispatches on `ColumnDef.format`. Composite renderers build their
output from the primitives above — no new primitive type is added:

- `CellFormat::ProgressBar` → `emit_progress_bar`: a track `Rect`
  (`Theme::progress_track`) + a fill `Rect` scaled by the value fraction (colour
  from the resolved class `background`, else `Theme::progress_fill`) + an
  optional right-aligned `"NN%"` label. Geometry via `Theme::progress_height`
  and `Theme::progress_radius`.

### Locked-cell visual feedback

`emit_cell` takes a `model: &GridModel` parameter (in addition to `col`)
so it can call `ColumnDef::is_cell_editable(row, model)` — the single
source of truth combining `GridModel.editable` (grid-wide toggle), the
static `editable` flag, and the dynamic `editable_predicate`
(rs-grid-core). When a cell resolves to non-editable:

- A `locked_cell_bg` overlay `Rect` is pushed right before the cell's
  content, **regardless of `CellFormat`** — it applies to
  `Styled`/`Image`/`ImageText`/`ProgressBar` composite cells exactly the
  same as plain text, since it's pushed before the format dispatch.
  Skipped entirely when `Theme::locked_cell_bg.a == 0` (mirrors the
  `row_hover_bg` "transparent = disabled" convention — no extra draw call
  for themes that don't opt in).
- The **text color** swap (`Theme::locked_cell_text` instead of
  `Theme::cell_text`) is scoped to the default plain-text renderer only —
  the `Styled`/`Image`/`ImageText`/`ProgressBar` composite branches keep
  their own colors (badge class, image, progress-bar fill). Extending the
  text-color treatment to those is a separate, explicit follow-up if
  needed; the background wash already gives every locked cell a visible
  affordance regardless of format.

### At-rest invalid-value background/border

`emit_cell` also calls `ColumnDef::validate_value` against the cell's
current `CellStatus::Ready` value (peeked by reference before the
format-dispatch `match` consumes it) — a cell with no `rules`/`validator`
always resolves `Ok`, so this is a no-op for unvalidated columns. When it
resolves `Err`:

- A fill-only `Rect` (`fill: Theme::invalid_cell_bg`, no stroke) is
  pushed right after the locked-cell overlay, before the format
  dispatch — same format-agnostic placement as the locked overlay, so
  it applies to composite `CellFormat`s too.
- The border is **not** pushed here as a `Rect` — `emit_cell` only
  collects its bounds into the `invalid_borders: &mut Vec<(f64, f64,
  f64, f64)>` accumulator threaded through its signature (right after
  the bg overlay, same placement precedent as before). `SceneBuilder::
  build` draws it later as four boundary `Line`s via the shared
  `push_boundary_lines` helper (`builder.rs`, also used for the
  selection outer border) — placed right after the grid lines / column
  separators / pinned-column overlay but *before* the header/gutter, so
  it wins the draw-order race against this row's own trailing grid
  line (the reason it isn't a `Rect` pushed inline: a stroked `Rect`
  at the cell's exact bounds would otherwise have its bottom/right
  edge painted over by that line) while still being masked by the
  header/gutter for a row/column that's only partially scrolled into
  view, the same way ordinary cell content is.
- The bg and border overlays are independent primitives, each skipped
  entirely when its own color has `.a == 0` (same "transparent =
  disabled" convention as `locked_cell_bg`) — a consumer can theme
  either, both, or neither.
- This fires independently of an active edit session — a cell that's
  invalid because the *data source* loaded it that way is flagged
  immediately, unlike the DOM editor's invalid style
  (`rs-grid-web`'s `apply_edit_validity_style`), which only exists while
  `GridState.edit` is `Some`. A locked cell and an invalid cell aren't
  mutually exclusive (e.g. read-only column seeded with bad data) — the
  overlays layer without conflict.

### At-rest cell decoration

`emit_cell` also calls `ColumnDef::cell_decoration(row, model)` right
after the invalid-value border block, before the format dispatch — same
format-agnostic placement, so a decoration applies uniformly to
composite `CellFormat`s too. When it resolves `Some(CellDecoration)`:

- A fill-only `Rect` (`fill: Color::rgba` from `background_tint`,
  `stroke: None`) is pushed first, only if `background_tint` is set.
- A border-only `Rect` (`fill` fully transparent, `stroke: Color::rgba`
  from `border_color`, `stroke_width: Theme::decoration_border_width`)
  is pushed second, only if `border_color` is set. Two independent
  primitives, so a decoration can set either, both, or neither without
  an empty draw call for the unset one.
- Unlike `locked_cell_bg`/`invalid_cell_border`, the colors themselves
  are **not** themed — they're consumer-supplied `[u8; 4]` RGBA read
  straight from `CellDecoration`, the same "app controls the value"
  precedent as `FormattedCell::color`. Only the border's stroke width
  is themed (`Theme::decoration_border_width`), since it's uniform
  across every decorated cell regardless of which color the consumer
  picked.
- Resolved (and pushed) after the locked/invalid overlays, so a
  decoration layers on top of them rather than being suppressed — same
  "layer, don't suppress" precedent as invalid-and-locked composing
  above. `CellDecoration::message` is not consumed here — no tooltip
  rendering exists yet.

### Cell button visibility

`emit_cell_buttons` (called from `emit_cell`, right after the format
dispatch) now takes `model: &GridModel` and early-returns — pushing no
`Rect`/`Text` primitives and registering no `ButtonZone` hit-test entry —
when `col.cell_buttons.is_empty()` **or**
`!col.are_cell_buttons_visible(ri, model)` (rs-grid-core). A hidden row's
buttons don't just render invisibly: they don't exist in the frame at
all, so `SceneFrame::hit_button` (used by `rs-grid-web`'s click dispatch)
correctly finds nothing there — no separate "disabled" click state to
maintain.

### Column separators (data rows)

A themed vertical line at each column boundary in the data-row area
(`Theme::column_separator_color` / `Theme::column_separator_width`),
drawn in `builder.rs` right after the per-row loop closes — **once per
column boundary, not once per cell**. Column x-positions don't vary by
row (only scroll/drag-preview do, both already resolved before the row
loop starts), so looping `row_count × column_count` times to draw this
line would be an unjustified O(rows × columns) cost for geometry that's
actually O(columns). This is the precedent to follow for any *future*
per-column (not per-cell) decoration: hoist it out of the row loop
rather than adding it inside `emit_cell`.

- Guarded by `t.column_separator_width > 0.0` ("width = 0 disables it"
  — there's no natural alpha channel to gate on for a line, unlike a
  fill, so width is the off-switch).
- `column_separator_color` defaults to the same RGB as `grid_line` in
  every theme, so enabling this feature causes **no visual change**
  until a consumer explicitly diverges the two — locked in by
  `theme.rs`'s `*_column_separator_color_matches_grid_line_by_default`
  tests.
- Pinned columns get separators between themselves too (`0..pinned_count
  - 1`), explicitly excluding the pinned band's own right edge — that
  edge already has a dedicated line (`pinned_separator_color`/`width`,
  pushed a few lines later). Drawing both there would double-stack or
  z-fight; the two features must not overlap.

### Success-flash cell scoping (paste, clear)

`FlashHint.cells: HashSet<(u64, usize)>` (`builder.rs`) carries the exact
coordinates to flash, supplied by `rs-grid-web` from either
`CommandOutput::PasteApplied` (paste) or `CommandOutput::CellsCleared`
(Delete/Backspace clear). `emit_cell` checks `flash.cells.contains(&(ri,
ci))` directly — it does **not** reuse `SelectionState::is_selected`, since
both `PasteAt` and `ClearCells` always expand the selection to the full
target/cleared rectangle even when some cells were skipped (locked or
failing validation). Flashing by selection would give a skipped cell the
same "success" overlay as a cell that was actually written.

### Row-selection checkbox column (`builder/checkbox.rs`)

Gated by `GridModel.show_checkbox_column`, width
`GridModel.checkbox_column_width` (default `GridModel::CHECKBOX_COLUMN_WIDTH`,
runtime-configurable — `emit_checkbox` always centers the box within it,
so widening it grows the margin symmetrically). Unlike the row-number
gutter, this
column is **not** a fixed/pinned band — it's the first slot of the
scrollable (unpinned) region, so it scrolls away with `scroll_x` exactly
like a real column, and pinned real columns (if any) render immediately
after the gutter, unaffected by it, with the checkbox appearing after
them. It's still never part of `columns: Vec<ColumnDef>` or
`ColumnOffsets` — `rs-grid-core`'s `hit_test`/`hit_test_col_header`
reserve its width (`effective_checkbox_column_width()`) as a scroll-shift
term instead, so `ColumnOffsets::hit_column`'s O(log n) search never
needs to know about it. Drawn in its own pass in `builder.rs` (row
checkboxes right before the pinned-column overlay section; the header
checkbox right before the pinned-header block) so the pinned band's
overlay — rendered after, on top — correctly masks the checkbox once it
scrolls underneath, the same z-order relationship real unpinned columns
already have with the pinned band.

- `emit_checkbox` (`builder/checkbox.rs`) draws a themed box (`Rect` with
  `Theme::checkbox_border`/`checkbox_radius`/`checkbox_border_width`) plus,
  for `Checked`/`Indeterminate`, a mark built from plain `Line` segments
  (a two-segment check mark, or a single dash) — no new primitive type,
  same "reuse an existing primitive" precedent as the sort-arrow's
  `Polygon`.
- The pinned band's z-order masking above only covers the pinned-width
  region. As the checkbox column scrolls left it also dips under the
  **row-number gutter** (`rnw`) itself, so `emit_checkbox` additionally
  takes a `clip_left` parameter (both call sites pass `rnw`) and clamps
  the box's own `Rect.clip` to `[band_x.max(clip_left), ...]` — mirroring
  `cells.rs`/header-text's own `clip_x = cx.max(rnw)` clamp. Without it,
  the box bleeds into the gutter/corner whenever `Theme::gutter_bg` has
  any transparency, the same risk already called out for cell content and
  header text. The check-mark `Line` segments have no `clip` field at all
  (`LinePrimitive` doesn't support one), so they're skipped outright
  whenever the box is left-clipped, rather than left to bleed past the
  box's own clipped edge.
- Per-row state (`Checked`/`Unchecked`) comes from `GridState.checked_rows:
  HashSet<u64>`, keyed by **physical** row id (survives sort/filter,
  mirroring `set_cell`'s logical→physical translation via
  `model.logical_to_physical`) — never `Indeterminate` for a single row.
- The header cell's tri-state comes from `GridState::checkbox_header_state()`
  (rs-grid-core), scoped to `model.filtered_indices` when a filter is
  active, otherwise all rows.
- A checked row also gets a full-width background overlay
  (`Theme::checked_row_bg`, transparent = disabled), drawn in the main
  row loop right after the hover overlay — same "layer above alt-bg,
  below selection" ordering as `row_hover_bg`, keyed by the same
  physical-id lookup as the checkbox itself.

## Adding a primitive

1. Add the struct in `primitives.rs`
2. Add the variant in `ScenePrimitive`
3. Implement rendering in `rs-grid-render-canvas/src/renderer.rs`

## Useful commands

```sh
cargo test -p rs-grid-scene
cargo clippy -p rs-grid-scene -- -D warnings
cargo bench -p rs-grid-scene --bench scene_builder
cargo bench -p rs-grid-scene --bench scroll_frame
```
