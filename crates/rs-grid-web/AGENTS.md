# rs-grid-web

Browser integration. Manages the full lifecycle of a grid instance in the DOM:
mouse/keyboard events, rAF loop, resize, DPR, CSS theme, localisation.

## Modules

| Module | Role |
|---|---|
| `canvas` | `GridCanvas`: mounts the grid on an `HtmlCanvasElement`, manages rAF and events |
| `css_theme` | `theme_from_css_vars()`: reads CSS variables to build a `Theme` |
| `locale` | `Locale`: UI string translations (15 built-in languages, TOML-based) |
| `storage` | `get_item` / `set_item` / `remove_item`: graceful `localStorage` helpers for persisting small demo state (e.g. a column layout). No-op when storage is unavailable. Pair with `example_common::layout::LayoutSnapshot`. |

## Responsibilities of `GridCanvas`

- Resize via `ResizeObserver` (viewport update)
- `requestAnimationFrame` loop: `SceneBuilder` → `SceneFrame` → `CanvasRenderer`
- Event handling: `mousemove`, `mousedown`, `mouseup`, `wheel`, `keydown`,
  `copy`, `paste`
- Canvas DPR adjustment for HiDPI screens
- Auto-scroll during selection drag

## Critical invariants

- `GridCanvas::mount()` is the only public entry point — one canvas = one instance.
- `mount()` calls `console_error_panic_hook::set_once()` (idempotent across
  mounts) so a boundary panic surfaces a readable message + stack in the
  browser console instead of `RuntimeError: unreachable`. Embedders that
  install their own panic hook can disregard it.
- Events are converted to `GridCommand` before being applied to `GridState`.
  **Do not manipulate `GridState` directly from event handlers.**
- DPR is read once at mount and on each resize. Do not re-read it every frame.
- `theme_from_css_vars()` reads the DOM — call only at mount, not every frame.

## Editing contract (`show_edit_input`)

A `dblclick` on a cell dispatches `StartEdit` then calls `show_edit_input()`.
The DOM overlay opened depends on `ColumnDef.editor`:

| `column.editor` value | Result |
|---|---|
| `Some(CellEditor::Text)` | `<input>` or `<textarea>` positioned over the cell (see *Single-line vs. multiline* below) |
| `Some(CellEditor::Select { .. })` | Custom `<select>` dropdown with optional icons |
| `None` | `CancelEdit` is dispatched, no DOM overlay is created |

`None` means "this column is not user-editable via the overlay". Callers who
want plain-text editing must set `column.editor = Some(CellEditor::Text)`
explicitly — the grid does not fall back to a text input automatically.

### Single-line `<input>` vs. multiline `<textarea>`

`show_text_editor` (`canvas/edit.rs`) is a dispatcher, not an editor itself.
It decides once, from the value the cell already holds when the edit
opens, which of two sibling functions to call:

- **`show_single_line_editor`** — a classic `<input type="text">`, used
  when the value has no `\n`/`\r` and its measured width (see below) fits
  within `MAX_EDITOR_WIDTH` (520px) without wrapping. Behaves exactly like
  the editor did before multiline support existed: `Enter` (no modifier)
  always commits, height stays the cell's own height, vertical centering
  is the input's native rendering.
- **`show_multiline_editor`** — a `<textarea>`, used when the value
  already contains a line break or is too long to fit on one line. Supports
  wrapping, `Alt+Enter` for manual line breaks, and dynamic height (see
  *sizing* below). `Shift+Enter` is **not** a newline shortcut — it commits
  like plain `Enter`.

The initial choice is made once, from the value the cell already holds
when the edit opens — a value that grows past the single-line cap mid-edit
(by typing) keeps scrolling horizontally in its `<input>` rather than
morphing into a `<textarea>` under the user's cursor. The one exception is
an explicit `Alt+Enter` keypress: `show_single_line_editor`'s keydown
handler special-cases this (before its plain-`Enter`-commits arm) to splice
exactly one `\n` into the current value at the cursor (`selectionStart`,
clamped to the nearest UTF-8 char boundary — `selectionStart` is a UTF-16
code-unit offset, so this guards non-ASCII text against a slice panic),
then calls `remove_edit_input()` + `show_multiline_editor()` with that
spliced value and the same `EditorGeom`, and finally restores the cursor
right after the inserted `\n` on the new `<textarea>` (`select()` inside
`show_multiline_editor` would otherwise leave the whole value selected).
No new `StartEdit` is dispatched — it's the same edit session, only the
DOM overlay changes. `TEXT_PADDING` (24px) and `MAX_EDITOR_WIDTH` (520px)
are module-level consts in `canvas/edit.rs`, shared by both paths.

### Content-based sizing of the text editor

Neither editor opens smaller than the cell's own size, and both grow to
fit the actual value:

- **Width** (both editors) — `measure_text_width` (`canvas/edit.rs`)
  renders the initial text on an offscreen `<canvas>` 2D context with the
  same font string as `rs-grid-render-canvas`'s `draw_text` (`400 {size}px
  system-ui, sans-serif`, `theme.font_size`). The box width becomes
  `cell_width.max(measured_width + TEXT_PADDING)`, capped at
  `MAX_EDITOR_WIDTH` for the `<textarea>` path (the `<input>` path never
  needs the cap — that's precisely the condition that keeps it on the
  `<input>` path in the first place).
- **Height** (`<textarea>` only) — `resize_multiline_editor` re-measures
  the *current* value (not just the initial one) and re-applies the box
  height on every `input` event — typing, pasting, or an `Alt+Enter`
  newline insertion (which re-dispatches a synthetic `input` event
  precisely so this runs) — so the box grows as lines are added and
  shrinks back down as they're removed, live, not just once at open time.
  It calls `measure_wrapped_height`, which lays the text out in a
  throwaway offscreen `<textarea>` at the chosen width (`white-space:
  pre-wrap`, `word-break: break-word` — matching the real editor's CSS)
  and reads its `scrollHeight`, since the browser's own line-wrapping is
  the only reliable way to get this without reimplementing text shaping.
  The offscreen element sets `rows="1"` — a bare `<textarea>` defaults to
  `rows="2"`, which would floor `scrollHeight` at two lines even for
  single-line text — and its horizontal padding matches `EDITOR_H_PADDING`
  so wrap points line up with what's actually rendered. The overlay
  height becomes `cell_height.max(wrapped_height + 2.0 *
  EDITOR_V_PADDING)` — never below the cell's own height, even after
  deleting back down to one line — capped at 60% of the window's inner
  height (past which it scrolls internally, `overflow-y: auto`, rather
  than growing further). `show_multiline_editor`'s own setup calls the
  same method once, so there's a single code path for both the initial
  size and every live resize after.

Both editors override `apply_edit_style`'s shorthand `padding` with the
same explicit `padding-top`/`padding-bottom`/`padding-left`/`padding-right`
longhands, deliberately wider than the themed `--rs-grid-editor-padding`
default (4px) — so a value that flips between `<input>` and `<textarea>`
across edits (see *dispatcher* above) doesn't visibly shift position
within the box:

- **`show_single_line_editor`** — fixed at `EDITOR_V_PADDING`/
  `EDITOR_H_PADDING` (8px each). The `<input>`'s native rendering
  centers its one line regardless of the exact top/bottom value.
- **`show_multiline_editor`** — horizontal is the same fixed
  `EDITOR_H_PADDING`; vertical is `(box_height - wrapped_height) / 2`
  (real centering, since a `<textarea>` always top-aligns otherwise),
  floored at `EDITOR_V_PADDING` so it never collapses to ~0 when the box
  height already matches the content almost exactly (which reads as
  text jammed against the border).

`apply_edit_style` then clamps the `left` position so the box never runs
past the window's right edge (shifts left, does not shrink), for either
editor. This sizing does not touch the select dropdown, which has its own
independent `--rs-grid-dropdown-min-width` sizing in `show_select_editor`.

### Multiline editing

`show_multiline_editor`'s `<textarea>` accepts newlines: `Alt+Enter`
(Excel's convention) inserts exactly one, done by explicitly splicing
`\n` into the value at the cursor and manually re-dispatching an `input`
event (`set_value()` doesn't fire one on its own, and the existing
`input` listener needs to run to keep live validation current) — not by
relying on the browser's own default Enter-in-a-textarea behavior, so the
count is deterministic regardless of what a given browser does with the
Alt modifier held. Plain `Enter` (with or without Shift) commits;
`Shift+Enter` is deliberately **not** treated as a newline shortcut.
`white-space: pre-wrap` + `word-break: break-word` preserve manually-typed
line breaks and wrap long words; `resize: none` and `overflow-y: auto`
keep the box a fixed-then-clamped size (see *sizing* above) with a
scrollbar for content past the 60%-viewport-height cap.

`show_single_line_editor`'s `<input>` can never itself hold a newline —
but `Alt+Enter` there doesn't just do nothing: see *Single-line vs.
multiline* above for how it switches to `show_multiline_editor` mid-edit
instead, inserting the same single `\n`.

### Validation feedback on the text editor

Both `show_single_line_editor`'s `<input>` and `show_multiline_editor`'s
`<textarea>` (`canvas/edit.rs`) wire the same three listeners:

| Event | Behaviour |
|---|---|
| `input` | Dispatches `GridCommand::ValidateEdit` on every keystroke (no commit), then restyles the input via `apply_edit_validity_style` |
| `keydown` (`Enter` with no Alt/Shift) / `blur` | Dispatches `GridCommand::CommitEdit`, then calls `keep_or_close`: if `GridState.edit` is still `Some` (`InvalidEditMode::Block` kept it open), restyle as invalid and refocus; otherwise tear the overlay down as before |
| `keydown` (`Escape`) | Dispatches `GridCommand::CancelEdit` unconditionally, always tears the overlay down |

While a cell is being edited, its DOM overlay fully occludes the canvas
underneath (opaque `--rs-grid-editor-bg`), so the invalid-value indicator
for the *in-progress edit* is applied directly to that element's own
border/background instead of a `ScenePrimitive` — see *CSS theme* below. This
is separate from the canvas-rendered `invalid_cell_border` overlay
(`rs-grid-scene`'s `emit_cell`), which flags any cell whose *current* value
fails validation regardless of whether it's being edited — see
`rs-grid-scene/AGENTS.md`'s "At-rest invalid-value border". The two never
overlap: the DOM input hides the canvas cell underneath while editing.

### At-rest validation tooltip (hover)

A cell can be invalid without ever being edited — data loaded already-bad
from the source (see `rs-grid-scene/AGENTS.md`'s "At-rest invalid-value
border"). `refresh_validation_tooltip(vx, vy)` (`canvas/tooltip.rs`), called
from `attach_mousemove`'s hover branch right after `refresh_hover_cursor`,
surfaces this on hover via a single reused DOM `<div>`
(`tooltip_el: RefCell<Option<HtmlElement>>`, mirroring `edit_input`) — **not**
one element per invalid cell, so cost stays O(1) regardless of how many
cells in the grid are invalid. It is:

- Positioned via the existing `cell_client_rect(row, col_key)` (`canvas/
  edit.rs`) — no new geometry.
- Given `pointer-events: none` (the mouse stays over the `<canvas>`, never
  over this element) and `data-tip="<message>"` — the attribute daisyUI
  reads via `content: attr(data-tip)`.
- Change-guarded on `(row, col)` (`tooltip_cell`) so DOM writes only happen
  when the hovered cell actually changes, not on every `mousemove`.
- Hidden (not removed) on wheel/scroll (`attach_wheel`) since its position
  is computed once at hover time in fixed client coordinates and would
  otherwise desync from the cell it targets; it reappears on the next
  `mousemove` if still hovering an invalid cell.

rs-grid renders **no visual of its own** for this tooltip — the wrapper's
`class` attribute is fully caller-controlled via
`set_validation_tooltip_class(Option<String>)`; with no class set, the
element exists but is invisible. This is the same "rs-grid does not pick
where/how validation feedback renders" stance as `cell_client_rect`'s own
doc comment. To reproduce daisyUI's tooltip:
```rust
canvas.set_validation_tooltip_class(Some(
    "tooltip tooltip-open tooltip-error".into(),
));
```
`tooltip-open` (or an equivalent always-open modifier) is required in the
class — daisyUI's tooltip CSS normally shows on native `:hover`, which this
element can never receive (`pointer-events: none`); rs-grid owns
show/hide entirely via its own `display` toggle, not CSS `:hover`.
`GridCanvas::cell_validation_error(row, col_key) -> Option<String>` is also
public standalone (mirrors `validation_error()`, but for at-rest values) —
useful to build a fully custom indicator without the built-in hover
mechanism.

### Consuming validation state generically

rs-grid does not impose a validation-error widget for the *in-progress edit*
case (no built-in styled tooltip component, unlike e.g. AG Grid). Instead it
exposes the raw state so an integrator can build any UI with their own
framework/CSS:

- `GridCanvas::validation_error() -> Option<(u64, String, String)>` — reads
  the live state on demand (`GridState.edit.validation_error`).
- `set_on_validation_state_changed` — push notification on every keystroke
  (see *Public callbacks* above), for reactive UIs.
- `set_native_validation_tooltip(bool)` (default `true`) — toggles the
  zero-config native `title` attribute set by `apply_edit_validity_style`
  in `canvas/edit.rs`. Disable it when building a custom UI to avoid a
  competing browser tooltip.

The three framework wrappers (`rs-grid-leptos`/`dioxus`/`yew`) forward
`on_validation_state_changed` as a plain callback prop, mirroring the
existing `on_validation_error` plumbing exactly — no wrapper-managed
reactive signal exists for this or any other `GridCanvas` state today.

## Locked-cell cursor feedback

The mousemove hover handler (`canvas/events.rs`) sets `cursor: not-allowed`
when hovering a cell that resolves to non-editable via
`ColumnDef::is_cell_editable` — the single source of truth combining the
grid-wide `GridModel.editable` toggle, the column's static `editable` flag,
and a false-resolving `editable_predicate` (all in `rs-grid-core`) — using
the `hit_locked_cell(vx, vy)` helper in `canvas/hittest.rs` (delegates to
`GridState::hit_test` for an O(log n) row+col lookup). Because
`is_cell_editable` folds in `GridModel.editable`, a globally read-only grid
shows `not-allowed` on every cell, consistent with `emit_cell`'s locked
overlay (`rs-grid-scene`). Paired with the `locked_cell_bg`/`locked_cell_text`
`Theme` fields (see *CSS theme* below) for the themed visual.

## Row-selection checkbox column

`GridModel.show_checkbox_column` (opt-in, default `false`) inserts a
column at `GridModel.checkbox_column_width` (default
`GridModel::CHECKBOX_COLUMN_WIDTH`, runtime-configurable via
`GridCommand::SetCheckboxColumnWidth`/`GridCanvas::set_checkbox_column_width`),
as the first slot of the scrollable (unpinned) region — unlike the
row-number gutter, it is
**not** fixed on screen: it scrolls away with `scroll_x` like a real
column, and sits after any pinned real columns. Still outside
`ColumnOffsets`/`hit_column` (rs-grid-core's `hit_test`/
`hit_test_col_header` reserve its width as a scroll-shift term instead).
`attach_mousedown` (`canvas/events.rs`) hit-tests it in a dedicated
cascade block, right before the row-number gutter check:
`hit_test_checkbox_header` (→ `GridCommand::ToggleAllFilteredChecked`)
and `hit_test_checkbox_row` (→ `GridCommand::ToggleRowChecked`, or
`ExtendRowChecked` when `evt.shift_key()` is set — sets every row in
`[anchor, this row]` to the direction fixed by the last
`ToggleRowChecked` (the anchor click's own resulting state),
mirroring `ExtendRowSelection`'s anchor/focus range. A further
shift+click within the same gesture also reconciles against the
*previous* `ExtendRowChecked` call: rows it touched but the new range
no longer covers revert to the opposite state — checking rows 1-10,
then, still holding shift, clicking row 9 gives row 10 back its
earlier state, matching drag-selection behaviour instead of only ever
growing the checked set) — both `pub(super)` wrappers in
`canvas/hittest.rs` delegating to the
`GridState` methods of the same name (rs-grid-core), which now factor
in `scroll_x` since the
checkbox's on-screen position is scroll-dependent. A row-checkbox
click is a discrete toggle, not a drag gesture — unlike the
row-number gutter's `ActiveDrag::Row`.

`GridCanvas::checked_row_indices()` returns **physical** row ids (not
logical/display order, unlike `selected_row_indices()`) — checkbox state
is tracked by row identity so it survives sort/filter. Pair with
`set_on_checked_rows_changed` (see *Public callbacks* below) and
`checkbox_header_state()` (tri-state: `Checked`/`Unchecked`/
`Indeterminate`) to drive a bulk-action toolbar. Toggle the column itself
at runtime with `set_show_checkbox_column(bool)`.

## Column filter popup (`filter_popup.rs`)

The column header itself carries no filter-specific chrome — just the
column name and the "⋮" menu icon. The **only** click path to the
advanced condition/checklist popup this section documents is the
floating filter row's own mini funnel icon (opt-in, `GridModel.
show_filter_row`) — see "Floating filter row" below for its hit-test,
click wiring, and hover state. This section covers the popup's own
content and behavior, which the row's icon opens unchanged.

Its trigger's fill color doubles as the active-filter indicator: themed
`header_filter_icon_active` when `model.filters` **or**
`model.value_filters` has an entry for that column, `header_filter_icon`
otherwise (either filter mechanism alone activates it) — the theme
field names keep their original `header_*` prefix even though the
header no longer draws this icon itself, since renaming them would be
an unrelated, purely-cosmetic breaking change to the CSS variable
surface.

- **Popup reuses the context menu's DOM shell** rather than building a
  parallel popup system: `filter_popup.rs`'s
  `show_column_filter_popup(col_idx, x, y)` calls `context_menu.rs`'s
  `create_menu_shell`/`read_ctx_colors` (bumped to `pub(super)` for
  this cross-file reuse — the same visibility level `remove_ctx_menu`
  already uses so `keyboard.rs` can call it) — same backdrop, same
  position clamping, same fixed `rs-grid-ctx-backdrop`/`rs-grid-ctx-menu`
  element ids. This means outside-click-to-close works with zero new
  code, and only one popup (context menu or filter popup) can ever be
  open at once since they share ids. `create_menu_shell` takes an
  explicit `bg: &str` for the panel's own background rather than
  deriving it from `colors.bg` (`--rs-grid-ctx-bg`) — all three callers
  (this popup, `show_col_header_menu`, `show_context_menu`) pass
  `theme.header_bg.to_css()`, so every popup reads as an extension of
  the header it's anchored under instead of a separately-themed surface.
  The popup's form: a collapsed-by-default "Text Filter" disclosure row
  as its first element — no column-name heading above it (removed; the
  column is already identified by whichever filter-row cell/icon was
  clicked to open the popup); see the next bullet for the flyout it
  opens (the operator combobox and value `<input>` live there, not
  inline), the value checklist below (see further down), and
  Apply/Clear buttons. Apply dispatches `GridCommand::SetColumnFilter`
  (condition) plus, if the checklist rendered,
  `SetColumnValueFilter`/`ClearColumnValueFilter` (checklist); Clear
  always dispatches both a cleared condition and
  `ClearColumnValueFilter`, regardless of checklist state. Both read
  current state fresh each time the popup opens, no persistent DOM to
  keep in sync (unlike the old row, there's nothing to call
  `sync_filter_row_cell`-equivalent for). Apply/Clear are built by
  `make_daisy_button` — pixel-accurate daisyUI `.btn` (Apply is
  `.btn-primary`, filled with `theme.checkbox_checked_bg`; Clear is the
  neutral default `.btn`, filled with `colors.hover_bg` standing in for
  daisyUI's `base-200`). `:hover`/`:active` are wired via mouse listeners
  (`apply_btn_bg`/`btn_shadow`) since a `<div>` has no styling for those
  pseudo-classes for free — see `style_daisy_control`'s doc comment for
  why this crate replicates daisyUI's CSS values directly instead of
  depending on its stylesheet. The focus ring is **not** JS-driven,
  though: it's a real `:focus-visible` CSS rule
  (`daisy_btn_focus_visible_style`, one shared `<style>` tag for both
  buttons, each reading its own color from a `--btn-ring-color` custom
  property) — daisyUI's `.btn` deliberately scopes its ring to
  `:focus-visible` (keyboard/programmatic focus only, never a plain
  mouse click), unlike `.input`/`.select`, which ring on any `:focus`
  (`wire_daisy_focus_ring`). An earlier version used a `focus`/`blur` JS
  listener pair here too, which showed the ring on every mouse click —
  visibly wrong next to a real daisyUI button, and the reason this needs
  actual CSS instead of the JS-listener idiom used everywhere else in
  this file.
- **"Text Filter" row + condition-editor flyout** — AG-Grid-style
  disclosure: the operator combobox and value `<input>` (see next
  bullet) are built exactly as before, but appended into `tf_panel`, a
  `position: fixed` panel that starts `display: none`, instead of
  directly into the popup. `tf_row` (`role="button"`, a chevron icon
  via a new `ICON_CHEVRON_RIGHT`) toggles it. State is two independent
  `Rc<Cell<bool>>`s — never conflate them: `is_open` governs only the
  operator dropdown nested inside the flyout (unchanged from before
  this row existed); `submenu_open` governs the flyout itself, one
  level up. Closing the flyout force-closes the nested dropdown too
  (so it's never left open on the next reopen); the reverse doesn't
  happen. Position is computed in `tf_row`'s click handler, not at
  popup-construction time — `tf_row.get_bounding_client_rect()` needs
  the row already laid out, which only happens after
  `create_menu_shell` has placed and clamped the popup itself. Anchors
  to the row's right edge with a 4px gap, top-aligned; flips to the
  row's *left* edge instead if that would overflow the viewport's
  right edge (`window.inner_width()`, the same idiom `edit.rs` uses
  for its own inline-editor viewport clamp), then clamps both axes into
  the viewport — a **different** clamp model than `create_menu_shell`'s
  own (canvas-bounds, for positioning the whole popup at open time):
  the flyout is a sibling flyout of an already-placed, arbitrarily-
  positioned popup, so viewport bounds are the right frame of reference
  here, not the canvas's. An explicit `z-index: 10000` (one above the
  popup's own `9999`) is required since a `position: fixed` descendant
  doesn't reliably inherit its parent's stacking context. Outside-click
  closes the flyout via a `[data-textfilter-wrap]`-scoped `mousedown`
  listener on the popup, symmetric with (and independent of) the
  existing `[data-op-wrap]`-scoped one that closes just the nested
  dropdown — both coexist on the same event without conflict since they
  inspect disjoint DOM scopes.
- **Operator combobox is custom-built, not a native `<select>`** — a
  real `<select>`'s open dropdown list is OS-drawn with no reliable
  cross-browser CSS hook (Chrome's Customizable Select API,
  `appearance: base-select` + `::picker(select)`, was tried and
  discarded: computed styles resolved correctly but the popup didn't
  consistently paint them). Built instead as a trigger `div`
  (`role="combobox"`, styled via `style_daisy_select`) plus an
  absolutely-positioned option list (`role="listbox"`/`role="option"`
  rows) — mirrors `show_select_editor`'s dropdown in `edit.rs`, reusing
  its `dd_idx_from_event`/`dd_set_highlight`/`dd_scroll_into_view`
  helpers (bumped to `pub(super)` for this cross-file reuse) so hover
  highlight, click-to-select, and keyboard `ArrowUp`/`ArrowDown`
  navigation all share one implementation with the cell editor's own
  Select dropdown. Selected `FilterOp` is tracked in an `Rc<Cell<FilterOp>>`
  rather than read from a `<select>`'s `.value()`. `Enter`/`Space` open
  the list or commit the highlighted row; `Escape` closes the dropdown
  if open, otherwise falls through to closing the whole popup (checked
  first, since a native `<select>`'s own popup would have consumed
  `Escape` itself). e2e tests target it via `getByRole('combobox')`/
  `getByRole('option', { name })`, not `.selectOption()`. The trigger
  also sets its own `display: flex; align-items: center` on top of
  `style_daisy_select` — a `<div>` doesn't auto-center text vertically
  in its box the way an `<input>`/`<select>` does, so without this the
  label sat flush against the top of the 40px box instead of centered
  like every other control here.
- **Escape-to-close needs its own listener, not the document-level
  gate** — a genuine bug caught during manual (not Playwright) testing:
  something inside the popup always calls `.focus()` on open, which
  moves DOM focus off the canvas. `attach_keydown`'s document-level
  Escape handler (`canvas/keyboard.rs`) is gated on `gc.has_focus()`,
  which checks `document.activeElement === canvas` specifically — so
  with focus elsewhere, that gate is `false` and Escape silently no-ops
  through the document-level path, which (when it *does* fire) also
  dispatches `GridCommand::ClearSelection` — not just closing the popup.
  Fixed by wiring local `keydown` listeners directly on the value
  `<input>`, the operator `<select>`, and `tf_row` (covers the case
  right after switching to `Blank`/`NotBlank`, which hides the input
  and leaves the select focused, *and* the default collapsed state,
  where neither exists in the DOM's focus path yet), each calling
  `remove_ctx_menu()` on `"Escape"` — same pattern the inline edit
  `<input>` and the search bar already use for the identical reason.
  `tf_row.focus()` runs unconditionally at the end of
  `show_column_filter_popup` (so something always holds focus by
  default, even before the flyout is ever opened); `value_input.focus()`
  moved from there into the flyout's own open sequence, firing only once
  it's actually visible — focusing a `display: none` element is a
  no-op, so leaving the old unconditional call in place after the
  flyout became collapsed-by-default would have silently left focus on
  the canvas, reintroducing this exact bug for the "never opened Text
  Filter" case. Playwright's own `page.keyboard.press('Escape')` did
  not reproduce the original bug locally (window/CDP focus semantics
  differ from a real interactive browser session) — this was only
  caught by testing in an actual Chrome window, not by the automated
  suite passing.
- **Value checklist (AG-Grid-style "Set Filter")** — AND-combined with
  the condition form above, not a replacement for it (confirmed via
  `AskUserQuestion` when this was added). Built from
  `GridModel::unique_values(col_key, MAX_VALUE_FILTER_OPTIONS)`
  (`rs-grid-core`) — `MAX_VALUE_FILTER_OPTIONS` is `usize::MAX` (no
  practical cap, per explicit request; was `200`), so `UniqueValues::
  TooMany` is effectively unreachable now — `unique_values` itself is
  still bounded by `GridModel::MAX_CLIENT_SORT_ROWS` (1,000,000 rows
  scanned), so distinct-value counts stay finite, but a large,
  near-unique-per-row column will render one checkbox row per distinct
  value with no ceiling. Lower `MAX_VALUE_FILTER_OPTIONS` back down if
  that's a problem for a given dataset — the fallback path below is
  still fully wired, just dormant:
  - `UniqueValues::TooMany` → a message instead of a list; Apply leaves
    that column's `value_filters` entry untouched (the UI can't safely
    represent editing a restriction it can't display), but Clear still
    unconditionally clears it — "Clear Filter" must mean no filtering
    on the column, not "no filtering except what I couldn't show you."
  - `UniqueValues::Values` → a search `<input>`, a "(Select All)"
    checkbox, and one checkbox per value in a scrollable `<div>`.
    Initial checked state: `model.value_filters.get(col_key)` present →
    only its members checked; absent → every value checked (no
    restriction yet).
  - Search hides non-matching rows via `display: none` — it never
    unchecks a hidden checkbox, so re-clearing the search box restores
    whatever was checked before. The search `<input>` has a Feather
    magnifying-glass icon (`ICON_SEARCH`) absolutely positioned inside
    its own left padding (a wrapper `<div>` + `search.style()
    .set_property("padding-left", "32px")` on top of
    `style_daisy_control`'s own `0 12px`) — same icon style as
    `context_menu.rs`'s built-in action icons. The scrollable value list
    itself has **no border** — just `overflow-y: auto` + padding — so it
    reads as part of the popup, not a nested panel.
  - "(Select All)" is a real tri-state
    (`HtmlInputElement::set_indeterminate`), recomputed from the
    currently-*visible* rows after every search or per-value checkbox
    change (`update_select_all_state`) — acts on the visible subset
    when toggled, matching AG Grid rather than a global check-all.
  - On Apply, if every value ends up checked, the popup dispatches
    `ClearColumnValueFilter` instead of storing a no-op full set — keeps
    the filter row's own icon active/inactive color (and badge, below)
    meaningful after a user
    round-trips through "uncheck some → recheck all" via Select All.
  - Every checkbox (`select_all` and each value row) reuses the
    canvas-drawn row-selection checkbox column's own theme tokens
    (`Theme::checkbox_size`/`checkbox_checked_bg`, read from
    `self.0.builder.borrow().theme`) via the native CSS `accent-color`
    property (`style_checkbox`) — same size and checked-state color as
    the checkbox column, not a second, differently-styled checkbox
    widget.
  - The operator `<select>`, value `<input>`, and search `<input>` are
    styled pixel-for-pixel like daisyUI's `.input`/`.select` (md size:
    `style_daisy_control`/`style_daisy_select`/`wire_daisy_focus_ring`)
    — this crate has no Tailwind/daisyUI dependency, so the values
    (40px height, 12px horizontal padding, 4px corner radius, a 20%-
    opacity `color-mix` border, the `<select>`'s exact two-gradient
    chevron `background-image`, the `2px`-outline `focus`/`blur` ring)
    are copied directly from daisyUI's own component CSS source rather
    than assumed to be available via a `class` attribute on the host
    page. If daisyUI's upstream `.input`/`.select` CSS changes, re-pull
    the values from `packages/daisyui/src/components/{input,select}.css`
    in the `saadeghi/daisyui` repo rather than guessing.
- **Closures use `Closure::forget()`**, same policy as
  `context_menu.rs` (its module comment explains why: removing the
  shared shell root reclaims every attached listener via JS GC).
- Right-click a column header (or its "⋮" menu icon) → **Clear
  Filter** still appears in the existing context-menu extension point
  (`BuiltinAction::ClearColumnFilter`, `context_menu.rs`/
  `context_menu_config.rs`) only when that column has an active
  condition **or** value filter — same gating pattern as `ClearSort`'s
  `col_sorted` check, extended with `||
  state.model.value_filters.contains_key(col_key)`. Its handler
  dispatches both a cleared condition and `ClearColumnValueFilter`.
  This stays as a right-click/⋮-menu shortcut alongside the popup's own
  Clear button, matching AG Grid (which also exposes filter-clearing
  from both places).
- The popup itself needs no reserved header-band space and doesn't
  touch `GridModel.show_filter_row`/`filter_row_height` at all — those
  back the separate floating filter row below, which is the popup's
  only trigger.

## Floating filter row

A second sticky row directly under the column headers — AG-Grid's
floating filter row — opt-in via `GridModel.show_filter_row` (default
`false`) and `GridCanvas::set_show_filter_row(bool)`. The row is a fast
"contains" path; its own mini funnel icon is the **only** click path to
the advanced condition/checklist popup documented above — the header
itself has no funnel icon (removed; confirmed via `AskUserQuestion`
when the header/row split was decided — the header stays AG-Grid-plain,
name + "⋮" menu only).

- **Rendering** (closed state) is entirely canvas-drawn —
  `rs-grid-scene/builder.rs` draws a bordered "input-look" cell per
  column, the current filter value (nothing at all when empty — no
  placeholder text, same as the open-state overlay below), and a mini
  funnel icon reusing the header's own colors. See
  `rs-grid-scene/AGENTS.md`'s "Floating filter row" section.
- **Interaction** (open state) is a transient DOM `<input>` overlay,
  matching this crate's consistent architecture: canvas primary, DOM
  only for the ephemeral moment of interaction (every other overlay —
  inline cell editor, context menu, filter popup, search box — works
  the same way). Unlike real AG Grid's always-editable native input,
  a single click is needed to open it. Its geometry (`quick_filter.rs`)
  mirrors the closed box's own geometry exactly — same horizontal
  inset (`Theme::cell_padding` — the same distance a data cell's own
  text sits from the column edge, not an unrelated constant, so the
  box and the overlay both line up with the data cells' content),
  `filter_row_input_margin_v`, and icon-zone narrowing as `builder.rs`'s
  rendering block — rather than the whole cell. Using the whole cell
  was a real bug: the overlay would visibly jump/resize the instant it
  opened (covering the margins and the funnel icon's own reserved zone)
  instead of sitting flush over the box that was just clicked.
- **`style_daisy_control` must run *before* the geometry overrides, not
  after** — a second real bug, same file: `style_daisy_control` sets
  its own fixed `height: 40px` (correct for the popup's inputs, which
  are always 40px), and calling it *after* setting this overlay's own
  computed `width`/`height` silently clobbered them back to 40px —
  invisible as long as the filter row's box height happened to equal
  40px, and only surfaced once `filter_row_height`/
  `filter_row_input_margin_v` were tuned to a different box height.
  Caught by instrumenting `web_sys::console::log_1` at both the
  `mount()` dispatch site and inside `show_quick_filter_input` to
  compare the value actually flowing through against the DOM input's
  own `getBoundingClientRect()` — the Rust-side math was right the
  whole time; only the *order* of DOM calls was wrong. Fixed by calling
  `style_daisy_control`/`wire_daisy_focus_ring` first, then applying
  `position`/`left`/`top`/`width`/`height`/`z-index`/`box-sizing` after,
  so the computed geometry always wins.
- **Hit-testing**: `hit_filter_row_icon(vx, vy)` /
  `filter_row_icon_anchor(col_idx)` (`canvas/hittest.rs`) mirror the
  header's own menu-icon hit-test structure (`hit_header_menu_icon`/
  `menu_icon_anchor`) — narrow a whole-cell hit-test result to a small
  button rect — but checked against the filter row's own vertical band
  (`[effective_header_height(), effective_header_height() +
  effective_filter_row_height())`) instead of the header's, via
  `GridState::hit_test_filter_row_cell` (rs-grid-core) for column-cell
  resolution. **Hover-glow tracking mirrors the header's own menu icon
  exactly**: `hovered_filter_row_icon_col: RefCell<Option<usize>>`
  (`canvas/mod.rs`) is a second, independent copy of the
  `hovered_menu_col` pattern — updated in the same `attach_mousemove`
  branch right after the menu-icon check, reset on `mouseleave`, and
  passed as `SceneBuilder::build`'s 5th parameter. `builder.rs` draws
  the same hover-background `Rect` the menu icon uses, reusing its
  theme tokens as-is (`header_menu_icon_hover_bg`/`_radius`) rather than
  adding filter-icon-specific ones — the intent is for this button to
  read as the *same kind* of icon button, not a differently-styled one.
- **Click wiring** (`canvas/events.rs`, right after the column-header
  cascade): icon-first — the mini funnel icon opens the popup
  (`show_column_filter_popup`, unchanged); clicking anywhere else in
  the cell opens the quick-filter `<input>`
  (`show_quick_filter_input`, `canvas/quick_filter.rs`).
- **`show_quick_filter_input` calls `evt.prevent_default()` on the
  triggering mousedown, and this is load-bearing, not defensive
  boilerplate.** The overlay is positioned exactly over the click point
  (it covers the whole filter-row cell) and is focused synchronously on
  open. Without `prevent_default()`, the browser's native mousedown
  default action — focus the mousedown target, i.e. the canvas — runs
  right after this listener returns and steals focus straight back
  from the freshly-opened input, firing *its own* blur handler and
  tearing it down before the user (or a Playwright test) ever sees it.
  This only affects this overlay: the inline cell editor opens on
  `dblclick` (focus-stealing happens on a later, separate event, not
  the same click's remaining mouseup/click), and the filter row's own
  funnel icon opens a popup that isn't positioned under the cursor and
  isn't auto-focused on open — it didn't need this. Caught only via an actual
  Playwright run (`page.locator('canvas').click({position})` on the
  filter row consistently opened, then immediately closed, the input);
  a manual chrome-devtools MCP session didn't reproduce it reliably
  since manual clicks don't always replay the same mousedown→mouseup→
  click sequence at the exact same coordinates.
- Lifecycle mirrors `edit.rs`'s inline cell editor exactly: reuses the
  shared `edit_input`/`edit_closures`/`edit_listener_refs` bookkeeping
  and `remove_edit_input()` teardown, so only one transient overlay
  (cell editor or quick filter input) can ever be open at a time.
  Commit (`Enter` or blur) dispatches `GridCommand::SetColumnFilter`
  with `FilterCondition::contains(value)` — even an emptied value,
  which clears the filter via `FilterCondition::is_empty()`'s existing
  semantics. `Escape` cancels without dispatching. Prefills with the
  column's current `filters` value best-effort, regardless of the
  stored operator — same AG-Grid-style simplification the popup's own
  quick-input equivalent documents; typing into the row always
  overwrites to `Contains`.
- No new CSS theme variables for the overlay itself — styled via the
  same `style_daisy_control`/`wire_daisy_focus_ring` helpers the filter
  popup's value input uses (bumped to `pub(super)` in `filter_popup.rs`
  for this cross-file reuse), and left placeholder-less (no
  `set_attribute("placeholder", ...)`) to match the *closed*,
  canvas-drawn look, which also draws no hint text for an empty cell.
  Canvas-side `Theme` fields back that closed look — see
  `rs-grid-scene/AGENTS.md`.

## Server-side page fetcher (`FetchConfig`)

`canvas/fetcher.rs` implements the automatic fetch coordinator behind
`GridCanvas::enable_async_fetch(page_cache, FetchConfig { .. })` — detects
needed pages for the current viewport (`PageCacheDataSource::needed_pages`),
fetches them via `build_url`/`parse_response`, and inserts the response.
Full usage guide: `rs-grid-site`'s `data/page-cache.mdx` and
`howto/server-pagination.mdx` (see root `AGENTS.md`'s doc-sync rule — this
crate's own doc here only covers what a *contributor to this crate* needs,
not the end-user how-to).

On a successful response it dispatches **two** commands, not one:
`GridCommand::SetTotalRowCount(resp.total_rows)` then
`GridCommand::NotifyPageLoaded` — both after `cache_clone.set_total_rows(..)`
already updated the `PageCacheDataSource`'s own count. `SetTotalRowCount`
is what keeps `GridModel.row_number_width` in sync with the server-reported
total when the model was built from a placeholder count (see
`rs-grid-core/AGENTS.md`'s "Row-number gutter width" section) — dropping it
would silently regress the gutter back to whatever digit-count the
placeholder implied, even though `PageCacheDataSource::set_total_rows` on
its own already gives the *data* the right count.

## Success-flash feedback (paste, clear)

`GridCanvas::flash_cells(&[CellCoord])` (`canvas/dispatch.rs`) arms a
400 ms fading yellow overlay on exactly the given cells — **not** a
selection rectangle. Two call sites use it, both via `dispatch_with_output`
so they can read the `CommandOutput`:

- Paste — Ctrl+V in `canvas/keyboard.rs` and context-menu Paste in
  `canvas/context_menu.rs` dispatch `GridCommand::PasteAt { .. }` and pass
  `CommandOutput::PasteApplied { cells }` to `flash_cells`.
- Clear — Delete/Backspace in `canvas/keyboard.rs` dispatch
  `GridCommand::ClearCells` and pass `CommandOutput::CellsCleared { cells }`
  to `flash_cells`.

In both cases this matters because the *selection* still covers the full
target/cleared rectangle regardless of skips, but `cells` only contains
what was actually written — cells skipped for being locked or failing
validation must not get a "success" flash. `FlashState` (`canvas/mod.rs`)
stores the cell set; `compute_flash_hint` (`canvas/animation.rs`) clones it
into `FlashHint` each frame, and `rs-grid-scene`'s `emit_cell` checks
membership directly instead of reusing the selection highlight.

## Public callbacks

Callbacks fired during `dispatch()` after `GridState::apply()` returns:

| Callback | Triggers |
|---|---|
| `set_on_change` | `PasteAt`, `CommitEdit`, `ClearCells` (cell data mutations) — **not** fired when a `CommitEdit` is rejected by validation |
| `set_on_columns_changed` | `CommitColumnResize`, `MoveColumn`, `AutoFitColumn`, `AutoFitAllColumns`, `SetPinnedColumnCount` (layout mutations — **not** sort/filter) |
| `set_on_validation_error` | A `CommitEdit` was rejected by `ColumnDef.rules`/`validator` (`CommandOutput::ValidationError`) — fires for both `InvalidEditMode::Revert` and `::Block` |
| `set_on_validation_state_changed` | `StartEdit`, `ValidateEdit`, `CommitEdit`, `CancelEdit` — fires with the fresh `validation_error()` value on *every keystroke*, not just rejected commits |
| `set_on_cell_button_click` | User clicked a `ColumnDef.cell_buttons[i]` |
| `set_on_checked_rows_changed` | `ToggleRowChecked`, `ExtendRowChecked`, `ToggleAllFilteredChecked` (row-selection checkbox column) |

**Re-entrancy**: callbacks fire *after* the dispatch path has released its
borrow on `state` — `apply()` returns its output by value, and each callback
`Rc` is cloned out of its own cell before invocation. So a callback may both
read grid state (e.g. `selected_row_indices()`, `cell_at_logical()`) **and**
synchronously dispatch another `GridCommand` without panicking. (Guard against
unbounded recursion if a dispatch re-triggers the same callback.)

Layout getters callable from `on_columns_changed`:
`column_widths()`, `column_order()`, `pinned_count()`.

## CSS theme

CSS variables are prefixed `--rs-grid-*`. `light.css`, `dark.css`, and
`dimmed.css` in `examples/example-common/themes/` are **auto-generated**
— do not edit them directly. The variable ↔ `Theme` field mapping (both
directions) is the single source of truth in `rs-grid-scene/src/css_vars.rs`;
`css_theme.rs` here is only a thin DOM wrapper around its reader. The
progress-bar cell renderer adds `--rs-grid-progress-track`,
`--rs-grid-progress-fill`, `--rs-grid-progress-height`, and
`--rs-grid-progress-radius`. The locked-cell overlay (see *Locked-cell
cursor feedback* above) adds `--rs-grid-locked-cell-bg` and
`--rs-grid-locked-cell-text`. The at-rest invalid-cell background/border
(see *Validation feedback on the text editor* above) adds
`--rs-grid-invalid-cell-bg`, `--rs-grid-invalid-cell-border`, and
`--rs-grid-invalid-cell-border-width`. Cell buttons (`ColumnDef.
cell_buttons`, rendered by `emit_cell_buttons` in `rs-grid-scene`) add
`--rs-grid-cell-btn-{primary,secondary,danger,neutral,accent,info,
success,warning}-{bg,text}` plus `--rs-grid-cell-btn-ghost-color` (Ghost
has no fill, only a border/text color) and the shared geometry vars
`--rs-grid-cell-btn-{radius,padding-y,padding-x,gap,margin-r}`. In the
`light` theme these match DaisyUI's own semantic colors 1:1 (see
`examples/example-common/src/class_map_data.rs`); `dark`/`dimmed` match
DaisyUI's built-in `dark`/`dim` themes the same way.

### Inline editor overlay variables

`apply_edit_style`/`apply_edit_validity_style` (`canvas/edit.rs`) read a
**separate** set of `--rs-grid-editor-*` variables directly from the DOM
(via `css_theme::get_var`), with hard-coded Rust fallbacks — this is *not*
part of the `Theme`/`css_vars.rs` round-trip system above, since these
variables style a DOM overlay element, not a canvas primitive:

| Variable | Fallback | Applies to |
|---|---|---|
| `--rs-grid-editor-border` | `#2563eb` | Border colour, normal state |
| `--rs-grid-editor-border-width` | `2px` | Border width (both states) |
| `--rs-grid-editor-border-radius` | `0` | Border radius |
| `--rs-grid-editor-bg` | `#ffffff` | Background, normal state |
| `--rs-grid-editor-color` | `#000000` | Text colour |
| `--rs-grid-editor-padding` | `0 4px` | Padding shorthand — immediately overridden by both `show_single_line_editor` and `show_multiline_editor`'s explicit `padding-*` longhands (see *Single-line vs. multiline* below), so this var has no visible effect on the text editors today; it still applies as-is to the select dropdown. |
| `--rs-grid-editor-font-size` | `theme.font_size` (not `inherit`) | Font size. Must stay in sync with `theme.font_size` — `measure_text_width`/`measure_wrapped_height` always measure at that value, so overriding this var to something else desyncs the multiline `<textarea>`'s computed width/height from what's actually rendered (manifests as unwanted scrolling or excess whitespace). |
| `--rs-grid-editor-shadow` | `none` | Box shadow |
| *(none — fixed)* | `system-ui, sans-serif` | Font family. Not configurable via CSS var (unlike the rest of this table) — hard-coded in `apply_edit_style` to match `measure_text_width`/`measure_wrapped_height` and the canvas renderer's own `draw_text`. Was `inherit` before multiline support; changing it back would desync the same way an overridden font-size does. |
| `--rs-grid-editor-border-invalid` | `#dc2626` | Border colour while `EditCell.validation_error` is `Some` |
| `--rs-grid-editor-bg-invalid` | `#fef2f2` | Background while `EditCell.validation_error` is `Some` |

### Adding a CSS variable to an existing theme

1. Add the field in `Theme` (`rs-grid-scene/src/theme.rs`) with a value
   in every constructor: `light()`, `dark()`, `dimmed()`
2. Wire the variable **both ways** in `rs-grid-scene/src/css_vars.rs`:
   `theme_to_css_vars` (writer) **and** `theme_from_css_vars_with` (reader).
   The `round_trips_every_field` test fails if you forget either side.
3. `cargo run -p rs-grid-scene --bin generate-theme`

### Adding a new theme (e.g. `solarized`)

1. **`theme.rs`** — add `Theme::solarized() -> Self` with all fields
2. **`generate_theme.rs`** — add `CTX_SOLARIZED` + call
   `render_overlay("solarized", &light_vars, &solarized_vars, CTX_SOLARIZED)`
   and write the output file
3. **`solarized-shell.css`** — create in `example-common/themes/` with
   `:root.solarized` overrides for `.app-*` and `body`
4. **3× `index.html`** — add links for `solarized.css` and
   `solarized-shell.css` after the existing theme links
5. **3× `src/lib.rs`** (Leptos, Dioxus, Yew) — add
   `<option value="solarized">Solarized</option>` in the theme select
6. `cargo run -p rs-grid-scene --bin generate-theme`
7. `cargo check --workspace`
