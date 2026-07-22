//! Header filter icon popup — an operator + value condition form,
//! anchored under the column's funnel icon. Reuses the context
//! menu's DOM shell (backdrop, position clamping, outside-click and
//! Escape close, all keyed on the same fixed element ids) instead of
//! building a parallel popup system — see `context_menu.rs`'s module
//! comment for why every listener here uses `Closure::forget()`.

use std::{cell::Cell, collections::HashSet, rc::Rc};

use rs_grid_core::{
    commands::GridCommand,
    filter::{FilterCondition, FilterOp, UniqueValues},
};
use rs_grid_scene::Theme;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{
    Event, HtmlElement, HtmlInputElement, KeyboardEvent, MouseEvent,
};

use super::{
    GridCanvas,
    context_menu::{
        CtxColors, create_menu_shell, read_ctx_colors, remove_ctx_menu,
    },
    dom_helpers::{document, make_el, set_styles},
    edit::{dd_idx_from_event, dd_scroll_into_view, dd_set_highlight},
};

/// Above this many distinct values, the checklist is replaced with a
/// message — matches [`rs_grid_core::model::GridModel::unique_values`]'s
/// own early-exit bound. Effectively unbounded (`usize::MAX`) per
/// explicit request — the `TooMany` fallback message stays wired
/// (`unique_values` itself is still bounded by
/// `GridModel::MAX_CLIENT_SORT_ROWS`, 1,000,000 rows scanned), but in
/// practice this cap is never hit anymore. Large, high-cardinality
/// columns (near-unique-per-row) will render one checkbox row per
/// distinct value — fine at the demo's default row counts, but can get
/// slow in the DOM well before the 1,000,000-row scan limit itself
/// does; lower this back down if that's a problem for a given dataset.
const MAX_VALUE_FILTER_OPTIONS: usize = usize::MAX;

/// Magnifying-glass glyph for the value checklist's search input —
/// same Feather Icons style (`stroke="currentColor"`,
/// `stroke-width="2.5"`) as `context_menu.rs`'s built-in action icons,
/// for visual consistency across the two DOM-shell-based popups.
const ICON_SEARCH: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2.5" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<circle cx="11" cy="11" r="8"/>"#,
    r#"<line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>"#
);

/// Right-chevron glyph for the "Text Filter" row that discloses the
/// condition-editor flyout (see `show_column_filter_popup`) — same
/// Feather Icons style as [`ICON_SEARCH`].
const ICON_CHEVRON_RIGHT: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2.5" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<polyline points="9 18 15 12 9 6"/></svg>"#
);

/// `Some((all_values, checkboxes))` when the value checklist was
/// rendered — `all_values` is every distinct value offered, `checkboxes`
/// pairs each with its live checkbox element. `None` when the column has
/// too many distinct values to list.
type Checklist = Option<(Vec<String>, Vec<(String, HtmlInputElement)>)>;

/// `(op, label)` — English labels, same precedent as the rest of the
/// grid's built-in chrome that isn't routed through `Locale` (e.g.
/// context-menu shortcut hints).
const OPS: &[(FilterOp, &str)] = &[
    (FilterOp::Contains, "Contains"),
    (FilterOp::NotContains, "Does not contain"),
    (FilterOp::StartsWith, "Starts with"),
    (FilterOp::EndsWith, "Ends with"),
    (FilterOp::Equals, "Equals"),
    (FilterOp::NotEquals, "Does not equal"),
    (FilterOp::Blank, "Is blank"),
    (FilterOp::NotBlank, "Is not blank"),
    (FilterOp::GreaterThan, "Greater than"),
    (FilterOp::GreaterThanOrEqual, "Greater than or equal"),
    (FilterOp::LessThan, "Less than"),
    (FilterOp::LessThanOrEqual, "Less than or equal"),
];

fn op_label(op: FilterOp) -> &'static str {
    OPS.iter()
        .find(|(o, _)| *o == op)
        .map(|(_, l)| *l)
        .unwrap_or("Contains")
}

/// `Blank`/`NotBlank` ignore the value entirely — mirrors
/// `FilterCondition::is_empty()`'s own semantics.
fn value_needed(op: FilterOp) -> bool {
    !matches!(op, FilterOp::Blank | FilterOp::NotBlank)
}

fn read_condition(op: FilterOp, input: &HtmlInputElement) -> FilterCondition {
    FilterCondition {
        op,
        value: input.value(),
    }
}

/// One shared `:focus-visible` rule for every [`make_daisy_button`] in
/// the popup — daisyUI's own `.btn` scopes its ring to `:focus-visible`
/// (real CSS, well-supported, unlike the Customizable Select API tried
/// and discarded for the operator combobox), so browsers apply their own
/// keyboard-vs-pointer heuristic for free instead of this crate
/// reimplementing it via `focus`/`blur` events. Scoped by `#rs-grid-ctx-menu
/// [role="button"]` rather than relying on the `<style>` tag's position in
/// the DOM (CSS rules aren't tree-scoped) — safe from colliding with an
/// unrelated `role="button"` elsewhere on the host page. Each button's own
/// ring color comes from its `--btn-ring-color` custom property (set in
/// `make_daisy_button`), so this one rule covers both Apply and Clear
/// despite their different colors. Meant to be appended once into the
/// popup shell, so it's torn down for free when `remove_ctx_menu()`
/// removes that subtree.
fn daisy_btn_focus_visible_style(doc: &web_sys::Document) -> HtmlElement {
    let style_el = make_el(doc, "style");
    style_el.set_text_content(Some(
        "#rs-grid-ctx-menu [role=\"button\"]:focus-visible { \
         outline: 2px solid var(--btn-ring-color, currentColor); \
         }",
    ));
    style_el
}

/// daisyUI's `--btn-shadow`/`--btn-inset` for a button whose current
/// background is `bg`: a barely-visible top highlight line plus two soft
/// drop shadows tinted with `bg` itself — copied verbatim from
/// `button.css` (`--depth: 1`).
fn btn_shadow(bg: &str) -> String {
    format!(
        "0 0.5px 0 0.5px oklch(100% 0 0 / 0.06) inset, \
         0 3px 2px -2px color-mix(in oklab, {bg} 30%, transparent), \
         0 4px 3px -2px color-mix(in oklab, {bg} 30%, transparent)"
    )
}

/// Applies one visual state (rest, hover, or `:active`-pressed) of a
/// daisyUI button to `btn`, given the background that state uses.
/// `pressed` mirrors `.btn:active`: shadow flattens to none and the
/// button nudges down half a pixel, matching daisyUI's own press effect.
fn apply_btn_bg(btn: &HtmlElement, bg: &str, pressed: bool) {
    let border = format!("color-mix(in oklab, {bg}, black 5%)");
    let shadow = if pressed {
        "none".to_string()
    } else {
        btn_shadow(bg)
    };
    let style = btn.style();
    let _ = style.set_property("background-color", bg);
    let _ = style.set_property("border-color", &border);
    let _ = style.set_property("box-shadow", &shadow);
    let _ = style.set_property(
        "transform",
        if pressed { "translateY(0.5px)" } else { "none" },
    );
}

/// Builds a pixel-accurate daisyUI `.btn` (md size), `.btn-primary` when
/// `accent` is `Some(color)` (used for Apply — a filled, colored call to
/// action) or the neutral default `.btn` otherwise (Clear Filter — a
/// `base-200`-ish filled button, `colors.hover_bg` standing in for that
/// token same as elsewhere in this file). Values copied from daisyUI's
/// own `button.css`, same rationale as `style_daisy_control`: `--size`
/// gives the identical 40px height as `.input`/`.select`, `--btn-p: 1rem`
/// = 16px horizontal padding, `--radius-field: 0.25rem` = 4px corner
/// radius, `font-weight: 600`. `:hover`/`:active` are wired via mouse
/// listeners (inline styles can't express pseudo-classes) — hover mixes
/// in 7% black, active 5% black plus the half-pixel press nudge from
/// [`apply_btn_bg`]. The focus ring is a real `:focus-visible` CSS rule
/// instead (see `daisy_btn_focus_visible_style`) — a JS `focus` listener
/// can't replicate `:focus-visible`'s keyboard-only heuristic, and
/// daisyUI's own `.btn` deliberately uses it (unlike `.input`/`.select`,
/// which ring on any `:focus`). A real `<button>` gets all of this from
/// the browser for free; this is a `<div>`, so `tabindex="0"` plus an
/// `Enter`/`Space` → synthetic `click` listener restore keyboard
/// activation.
fn make_daisy_button(
    doc: &web_sys::Document,
    label: &str,
    colors: &CtxColors,
    accent: Option<&str>,
) -> HtmlElement {
    let base_bg = accent
        .map(str::to_string)
        .unwrap_or_else(|| colors.hover_bg.clone());
    let fg = if accent.is_some() {
        "#ffffff".to_string()
    } else {
        colors.text.clone()
    };
    let ring_color = accent.unwrap_or(&colors.text).to_string();

    let btn = make_el(doc, "div");
    let _ = btn.set_attribute("role", "button");
    let _ = btn.set_attribute("tabindex", "0");
    set_styles(
        &btn,
        &[
            ("display", "inline-flex"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("height", "40px"),
            ("padding", "0 16px"),
            ("box-sizing", "border-box"),
            ("font-size", &colors.font_size),
            ("font-weight", "600"),
            ("border-radius", "4px"),
            ("border-style", "solid"),
            ("border-width", "1px"),
            ("cursor", "pointer"),
            ("user-select", "none"),
            ("color", &fg),
            (
                "transition",
                "background-color .2s, border-color .2s, \
                 box-shadow .2s, transform .2s",
            ),
        ],
    );
    apply_btn_bg(&btn, &base_bg, false);
    btn.set_text_content(Some(label));

    let hover_bg = format!("color-mix(in oklab, {base_bg}, black 7%)");
    let active_bg = format!("color-mix(in oklab, {base_bg}, black 5%)");

    {
        let b = btn.clone();
        let hb = hover_bg.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            apply_btn_bg(&b, &hb, false);
        });
        btn.add_event_listener_with_callback(
            "mouseenter",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add mouseenter listener");
        cb.forget();
    }
    {
        let b = btn.clone();
        let base = base_bg.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            apply_btn_bg(&b, &base, false);
        });
        btn.add_event_listener_with_callback(
            "mouseleave",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add mouseleave listener");
        cb.forget();
    }
    {
        let b = btn.clone();
        let ab = active_bg.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            apply_btn_bg(&b, &ab, true);
        });
        btn.add_event_listener_with_callback(
            "mousedown",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add mousedown listener");
        cb.forget();
    }
    {
        let b = btn.clone();
        let hb = hover_bg.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            apply_btn_bg(&b, &hb, false);
        });
        btn.add_event_listener_with_callback(
            "mouseup",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add mouseup listener");
        cb.forget();
    }
    // The ring itself is a real CSS `:focus-visible` rule (see
    // `daisy_btn_focus_visible_style`), not a JS `focus`/`blur` pair like
    // `wire_daisy_focus_ring` uses for inputs — daisyUI deliberately
    // scopes `.btn`'s ring to `:focus-visible` (keyboard/programmatic
    // focus only, no ring on a mouse click), unlike `.input`/`.select`,
    // which use plain `:focus`. A JS `focus` listener can't tell the two
    // apart, so it would show the ring on every click — visibly wrong
    // next to a real daisyUI button. Only the color is set here, via a
    // custom property the shared rule reads.
    let _ = btn.style().set_property("--btn-ring-color", &ring_color);
    {
        let b = btn.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
            if evt.key() == "Enter" || evt.key() == " " {
                evt.prevent_default();
                let init = web_sys::MouseEventInit::new();
                init.set_bubbles(true);
                if let Ok(click) =
                    MouseEvent::new_with_mouse_event_init_dict("click", &init)
                {
                    let _ = b.dispatch_event(&click);
                }
            }
        });
        btn.add_event_listener_with_callback(
            "keydown",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add keydown listener");
        cb.forget();
    }

    btn
}

/// Pixel-accurate daisyUI `.input`/`.select` look (medium size), applied
/// via plain CSS rather than the `input`/`select` classes — this crate
/// has no Tailwind/daisyUI dependency (renderer-agnostic, CSS-var-driven
/// only), so the values are copied directly from daisyUI's own component
/// source instead of assuming its stylesheet is loaded on the host page:
/// `--size-field: 0.25rem` × the `md` size multiplier `10` = 40px height,
/// `px-3` = 12px horizontal padding, `--radius-field: 0.25rem` = 4px
/// corner radius, border color = `color-mix(in oklab, base-content 20%,
/// transparent)` (`colors.text` stands in for `base-content`).
pub(super) fn style_daisy_control(el: &HtmlElement, colors: &CtxColors) {
    let border_color =
        format!("color-mix(in oklab, {} 20%, transparent)", colors.text);
    let border = format!("1px solid {border_color}");
    set_styles(
        el,
        &[
            ("height", "40px"),
            ("padding", "0 12px"),
            ("box-sizing", "border-box"),
            ("font-size", &colors.font_size),
            ("color", &colors.text),
            ("border-radius", "4px"),
            ("border", &border),
            ("background-color", &colors.bg),
        ],
    );
}

/// Extends [`style_daisy_control`] with the `.select`-only parts of
/// daisyUI's look: the native arrow hidden (`appearance: none`), extra
/// right padding to make room for it, and the same two-gradient chevron
/// daisyUI paints via `background-image` (copied verbatim from its
/// source — a plain CSS declaration, not a class). Applied to the
/// operator combobox's trigger div (see `show_column_filter_popup` — a
/// real `<select>`'s open dropdown list is OS-drawn with no reliable
/// cross-browser CSS hook, so the whole control is custom-built rather
/// than styled on top of a native element).
fn style_daisy_select(el: &HtmlElement, colors: &CtxColors) {
    style_daisy_control(el, colors);
    let text = &colors.text;
    let bg_image = format!(
        "linear-gradient(45deg, transparent 50%, {text} 50%), \
         linear-gradient(135deg, {text} 50%, transparent 50%)"
    );
    set_styles(
        el,
        &[
            ("appearance", "none"),
            ("-webkit-appearance", "none"),
            ("-moz-appearance", "none"),
            ("padding-right", "28px"),
            ("background-image", &bg_image),
            (
                "background-position",
                "calc(100% - 20px) calc(1px + 50%), \
                 calc(100% - 16.1px) calc(1px + 50%)",
            ),
            ("background-size", "4px 4px, 4px 4px"),
            ("background-repeat", "no-repeat"),
        ],
    );
}

/// daisyUI's focus state for `.input`/`.select`: the (already
/// 20%-opacity) border becomes fully opaque, plus a `2px` outline offset
/// `2px` outside it — a real CSS `:focus` rule can't be expressed via
/// inline styles, so this wires the equivalent via `focus`/`blur`
/// listeners, the same idiom [`make_daisy_button`] uses for its own
/// hover/active/focus states.
pub(super) fn wire_daisy_focus_ring(el: &HtmlElement, colors: &CtxColors) {
    let focus_color = colors.text.clone();
    let blur_color =
        format!("color-mix(in oklab, {} 20%, transparent)", colors.text);

    let el_focus = el.clone();
    let fc = focus_color.clone();
    let cb_focus = Closure::<dyn FnMut(_)>::new(move |_: Event| {
        let style = el_focus.style();
        let _ = style.set_property("border-color", &fc);
        let _ = style.set_property("outline", &format!("2px solid {fc}"));
        let _ = style.set_property("outline-offset", "2px");
    });
    el.add_event_listener_with_callback(
        "focus",
        cb_focus.as_ref().unchecked_ref(),
    )
    .expect("add focus listener");
    cb_focus.forget();

    let el_blur = el.clone();
    let cb_blur = Closure::<dyn FnMut(_)>::new(move |_: Event| {
        let style = el_blur.style();
        let _ = style.set_property("border-color", &blur_color);
        let _ = style.set_property("outline", "none");
    });
    el.add_event_listener_with_callback(
        "blur",
        cb_blur.as_ref().unchecked_ref(),
    )
    .expect("add blur listener");
    cb_blur.forget();
}

/// Sizes and colors a checklist checkbox to match the canvas-drawn
/// row-selection checkbox column (`Theme::checkbox_*`) — same size, same
/// checked-state accent — via the native `accent-color` CSS property
/// rather than a custom-drawn checkbox, so it stays a real, accessible
/// `<input type=checkbox>`.
fn style_checkbox(cb: &HtmlInputElement, theme: &Theme) {
    let size = format!("{}px", theme.checkbox_size);
    let accent = theme.checkbox_checked_bg.to_css();
    set_styles(
        cb.unchecked_ref(),
        &[
            ("width", &size),
            ("height", &size),
            ("accent-color", &accent),
            ("margin", "0"),
            ("cursor", "pointer"),
            ("flex", "none"),
        ],
    );
}

/// Recomputes the "(Select All)" checkbox's checked/indeterminate
/// tri-state from the currently-*visible* value rows — visibility is
/// read directly from each row's own `display` style (already updated
/// by the search listener before this runs), so this is the single
/// source of truth rather than tracking a parallel query string.
fn update_select_all_state(
    select_all: &HtmlInputElement,
    rows: &[(String, HtmlElement, HtmlInputElement)],
) {
    let visible: Vec<&HtmlInputElement> = rows
        .iter()
        .filter(|(_, row, _)| {
            row.style()
                .get_property_value("display")
                .unwrap_or_default()
                != "none"
        })
        .map(|(_, _, cb)| cb)
        .collect();
    if visible.is_empty() {
        select_all.set_checked(false);
        select_all.set_indeterminate(false);
        return;
    }
    let checked_count = visible.iter().filter(|cb| cb.checked()).count();
    if checked_count == 0 {
        select_all.set_checked(false);
        select_all.set_indeterminate(false);
    } else if checked_count == visible.len() {
        select_all.set_checked(true);
        select_all.set_indeterminate(false);
    } else {
        select_all.set_checked(false);
        select_all.set_indeterminate(true);
    }
}

impl GridCanvas {
    /// Open the column filter popup for `col_idx`, anchored at
    /// `(x, y)` — client/viewport coordinates, the same convention
    /// `show_col_header_menu` uses.
    pub(super) fn show_column_filter_popup(
        &self,
        col_idx: usize,
        x: i32,
        y: i32,
    ) {
        remove_ctx_menu();

        let (col_key, current, existing_value_filter, unique) = {
            let state = self.0.state.borrow();
            let model = &state.model;
            let Some(col) = model.columns.get(col_idx) else {
                return;
            };
            let current =
                model.filters.get(&col.key).cloned().unwrap_or_default();
            let existing_value_filter =
                model.value_filters.get(&col.key).cloned();
            let unique =
                model.unique_values(&col.key, MAX_VALUE_FILTER_OPTIONS);
            (col.key.clone(), current, existing_value_filter, unique)
        };

        let colors = read_ctx_colors();
        let theme = self.0.builder.borrow().theme.clone();
        let header_bg = theme.header_bg.to_css();
        let doc = document();
        let (_backdrop, menu) =
            create_menu_shell(x, y, &colors, &header_bg, &self.0.canvas);
        // A form needs more room than the context menu's label-driven
        // min-width, and its own padding (the menu shell's `6px 0` is
        // meant for a list of full-bleed rows).
        let _ = menu.style().set_property("min-width", "220px");
        let _ = menu.style().set_property("padding", "10px 12px");

        // ── operator combobox (custom, not a native <select>) ──────
        // A real `<select>`'s open dropdown list is OS-drawn with no
        // reliable cross-browser CSS hook (Chrome's `::picker(select)`
        // API was tried and discarded — computed styles resolved
        // correctly but the popup didn't consistently paint them; see
        // `style_daisy_select`'s doc comment). Built as a trigger div +
        // an absolutely-positioned option list instead, mirroring
        // `show_select_editor`'s dropdown in `edit.rs` (reusing its
        // `dd_*` highlight/scroll helpers) — pixel-perfect daisyUI on
        // every browser, at the cost of reimplementing combobox keyboard
        // semantics that a native `<select>` gets for free.
        let selected_op = Rc::new(Cell::new(current.op));
        let is_open = Rc::new(Cell::new(false));
        // Independent of `is_open` above: `is_open` governs only the
        // *inner* op-list dropdown (Contains/Greater than/...);
        // `submenu_open` governs the *outer* "Text Filter" flyout that
        // now hosts `op_wrap`/`value_input` (see below). Never conflate
        // the two — closing the flyout should also force-close the
        // dropdown nested inside it, but the reverse isn't true.
        let submenu_open = Rc::new(Cell::new(false));
        let hover_bg =
            format!("color-mix(in oklab, {} 10%, transparent)", colors.text);

        let op_wrap = make_el(&doc, "div");
        let _ = op_wrap.set_attribute("data-op-wrap", "");
        set_styles(
            &op_wrap,
            &[
                ("position", "relative"),
                ("width", "100%"),
                ("margin-bottom", "8px"),
            ],
        );

        let op_trigger = make_el(&doc, "div");
        let _ = op_trigger.set_attribute("role", "combobox");
        let _ = op_trigger.set_attribute("aria-haspopup", "listbox");
        let _ = op_trigger.set_attribute("aria-expanded", "false");
        let _ = op_trigger.set_attribute("tabindex", "0");
        set_styles(
            &op_trigger,
            &[
                ("width", "100%"),
                ("box-sizing", "border-box"),
                ("cursor", "pointer"),
                ("user-select", "none"),
                ("white-space", "nowrap"),
                ("overflow", "hidden"),
                ("text-overflow", "ellipsis"),
                // A `<div>` doesn't auto-center its text vertically the
                // way an `<input>`/`<select>` does — without this the
                // label sits flush against the top of the 40px box
                // instead of centered like every other control here.
                ("display", "flex"),
                ("align-items", "center"),
            ],
        );
        style_daisy_select(&op_trigger, &colors);
        wire_daisy_focus_ring(&op_trigger, &colors);
        op_trigger.set_text_content(Some(op_label(current.op)));
        op_wrap
            .append_child(&op_trigger)
            .expect("append op trigger");

        let op_list = make_el(&doc, "div");
        let _ = op_list.set_attribute("role", "listbox");
        let op_list_border = format!("1px solid {}", colors.border);
        set_styles(
            &op_list,
            &[
                ("position", "absolute"),
                ("top", "calc(100% + 4px)"),
                ("left", "0"),
                ("right", "0"),
                ("z-index", "1"),
                ("max-height", "240px"),
                ("overflow-y", "auto"),
                ("display", "none"),
                ("box-sizing", "border-box"),
                ("border", &op_list_border),
                ("border-radius", "8px"),
                ("padding", "8px"),
                ("background-color", &colors.bg),
                (
                    "box-shadow",
                    "0 20px 25px -5px rgba(0,0,0,.1), \
                     0 8px 10px -6px rgba(0,0,0,.1)",
                ),
            ],
        );
        op_wrap.append_child(&op_list).expect("append op list");

        let cur_idx =
            OPS.iter().position(|(o, _)| *o == current.op).unwrap_or(0);
        let highlight = Rc::new(Cell::new(cur_idx));
        let mut op_rows: Vec<HtmlElement> = Vec::with_capacity(OPS.len());
        for (i, (_, label)) in OPS.iter().enumerate() {
            let row = make_el(&doc, "div");
            let _ = row.set_attribute("role", "option");
            let _ = row.set_attribute("data-idx", &i.to_string());
            set_styles(
                &row,
                &[
                    ("padding", "6px 12px"),
                    ("border-radius", "4px"),
                    ("cursor", "pointer"),
                    ("white-space", "nowrap"),
                    ("transition", "background-color .2s"),
                ],
            );
            row.set_text_content(Some(label));
            if i == cur_idx {
                let _ = row.style().set_property("background", &hover_bg);
                let _ = row.set_attribute("aria-selected", "true");
            }
            op_list.append_child(&row).expect("append op row");
            op_rows.push(row);
        }
        let op_rows = Rc::new(op_rows);

        let value_input: HtmlInputElement =
            make_el(&doc, "input").dyn_into().expect("input element");
        value_input.set_value(&current.value);
        set_styles(
            value_input.unchecked_ref(),
            &[("width", "100%"), ("margin-bottom", "10px")],
        );
        style_daisy_control(value_input.unchecked_ref(), &colors);
        wire_daisy_focus_ring(value_input.unchecked_ref(), &colors);
        if !value_needed(current.op) {
            let _ = value_input.style().set_property("display", "none");
        }

        // ── "Text Filter" row + condition-editor flyout ────────────
        // AG-Grid-style disclosure: `op_wrap`/`value_input` built above
        // are hidden inside a collapsed-by-default flyout (`tf_panel`)
        // rather than shown inline, reached via this always-visible
        // "Text Filter" row. `tf_panel` is `position: fixed` (not
        // relative to `tf_wrap`) so it can float beside the row
        // regardless of the row's own position in the popup — but it's
        // still a DOM descendant of `menu`, so `remove_ctx_menu()` tears
        // it down for free along with everything else.
        let tf_wrap = make_el(&doc, "div");
        let _ = tf_wrap.set_attribute("data-textfilter-wrap", "");

        let tf_row = make_el(&doc, "div");
        let _ = tf_row.set_attribute("role", "button");
        let _ = tf_row.set_attribute("aria-haspopup", "true");
        let _ = tf_row.set_attribute("aria-expanded", "false");
        let _ = tf_row.set_attribute("tabindex", "0");
        set_styles(
            &tf_row,
            &[
                ("height", "40px"),
                ("box-sizing", "border-box"),
                ("display", "flex"),
                ("align-items", "center"),
                ("padding", "0 12px"),
                ("border-radius", "4px"),
                ("cursor", "pointer"),
                ("user-select", "none"),
                ("font-weight", "600"),
                ("margin-bottom", "8px"),
                ("transition", "background-color .2s"),
            ],
        );
        let tf_label = make_el(&doc, "span");
        let _ = tf_label.style().set_property("flex", "1");
        // Plain hardcoded English, not routed through `Locale` — same
        // precedent as the `OPS` labels above (no per-column-type
        // distinction like AG Grid's own "Number Filter" either; every
        // column shares the same operator set, so one generic label
        // covers all of them).
        tf_label.set_text_content(Some("Text Filter"));
        tf_row.append_child(&tf_label).expect("append tf label");
        let tf_chevron = make_el(&doc, "span");
        set_styles(
            &tf_chevron,
            &[
                ("width", "18px"),
                ("height", "18px"),
                ("flex-shrink", "0"),
                ("opacity", "0.75"),
                ("display", "flex"),
                ("align-items", "center"),
                ("justify-content", "center"),
            ],
        );
        tf_chevron.set_inner_html(ICON_CHEVRON_RIGHT);
        tf_row.append_child(&tf_chevron).expect("append tf chevron");
        tf_wrap.append_child(&tf_row).expect("append tf row");

        {
            let row = tf_row.clone();
            let hb = hover_bg.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                let _ = row.style().set_property("background", &hb);
            });
            tf_row
                .add_event_listener_with_callback(
                    "mouseover",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add tf row mouseover listener");
            cb.forget();
        }
        {
            let row = tf_row.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                let _ = row.style().set_property("background", "");
            });
            tf_row
                .add_event_listener_with_callback(
                    "mouseout",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add tf row mouseout listener");
            cb.forget();
        }

        let tf_panel = make_el(&doc, "div");
        let panel_border = format!("1px solid {}", colors.border);
        set_styles(
            &tf_panel,
            &[
                ("position", "fixed"),
                ("display", "none"),
                ("background-color", &header_bg),
                ("border", &panel_border),
                ("border-radius", &colors.radius),
                ("box-shadow", &colors.shadow),
                ("padding", "10px 12px"),
                ("min-width", "220px"),
                ("box-sizing", "border-box"),
                // One above `menu`'s own `9999` (see `create_menu_shell`)
                // — a `position: fixed` descendant doesn't reliably
                // inherit its parent's stacking context, so this must
                // be explicit rather than assumed.
                ("z-index", "10000"),
            ],
        );
        tf_panel.append_child(&op_wrap).expect("append op wrap");
        tf_panel
            .append_child(&value_input)
            .expect("append value input");
        tf_wrap.append_child(&tf_panel).expect("append tf panel");
        menu.append_child(&tf_wrap).expect("append tf wrap");

        // Opens/closes the flyout, keeping `tf_row`'s own
        // `aria-expanded` and the nested op-list dropdown in sync so
        // the dropdown is never left dangling open on the next open.
        let set_submenu_open = {
            let panel = tf_panel.clone();
            let row = tf_row.clone();
            let open = Rc::clone(&submenu_open);
            let list = op_list.clone();
            let trigger = op_trigger.clone();
            let dd_open = Rc::clone(&is_open);
            move |now_open: bool| {
                open.set(now_open);
                let _ = panel.style().set_property(
                    "display",
                    if now_open { "block" } else { "none" },
                );
                let _ = row.set_attribute(
                    "aria-expanded",
                    if now_open { "true" } else { "false" },
                );
                if !now_open {
                    dd_open.set(false);
                    let _ = list.style().set_property("display", "none");
                    let _ = trigger.set_attribute("aria-expanded", "false");
                }
            }
        };

        {
            let row = tf_row.clone();
            let panel = tf_panel.clone();
            let open = Rc::clone(&submenu_open);
            let set_open = set_submenu_open.clone();
            let input = value_input.clone();
            let selected_op = Rc::clone(&selected_op);
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                evt.stop_propagation();
                let now_open = !open.get();
                if now_open {
                    // Must show before measuring — `get_bounding_client_rect`/
                    // `offset_width`/`offset_height` need a laid-out
                    // element, and `tf_row`'s own final position isn't
                    // known until `create_menu_shell`'s clamp has
                    // already placed `menu`, so this can only run here,
                    // at click time.
                    let _ = panel.style().set_property("display", "block");
                    let row_rect = row.get_bounding_client_rect();
                    let panel_w = panel.offset_width() as f64;
                    let panel_h = panel.offset_height() as f64;
                    let win_w = web_sys::window()
                        .and_then(|w| w.inner_width().ok())
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::MAX);
                    let win_h = web_sys::window()
                        .and_then(|w| w.inner_height().ok())
                        .and_then(|v| v.as_f64())
                        .unwrap_or(f64::MAX);
                    let mut left = row_rect.right() + 4.0;
                    if left + panel_w > win_w {
                        left = row_rect.left() - panel_w - 4.0;
                    }
                    left = left.max(0.0).min((win_w - panel_w).max(0.0));
                    let top =
                        row_rect.top().max(0.0).min((win_h - panel_h).max(0.0));
                    let _ = panel
                        .style()
                        .set_property("left", &format!("{left}px"));
                    let _ =
                        panel.style().set_property("top", &format!("{top}px"));
                }
                set_open(now_open);
                if now_open && value_needed(selected_op.get()) {
                    let _ = input.focus();
                }
            });
            tf_row
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add tf row click listener");
            cb.forget();
        }
        {
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
                if evt.key() == "Enter" || evt.key() == " " {
                    evt.prevent_default();
                    // Reuses `make_daisy_button`'s own idiom: dispatch
                    // a synthetic click rather than duplicating the
                    // toggle/positioning logic in a second listener.
                    let init = web_sys::MouseEventInit::new();
                    init.set_bubbles(true);
                    if let Ok(click) =
                        MouseEvent::new_with_mouse_event_init_dict(
                            "click", &init,
                        )
                    {
                        let _ = evt
                            .target()
                            .and_then(|t| t.dyn_into::<HtmlElement>().ok())
                            .map(|el| el.dispatch_event(&click));
                    }
                }
            });
            tf_row
                .add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add tf row keydown listener");
            cb.forget();
        }
        // Local Escape handler, same rationale as `value_input`'s own
        // below: focusing `tf_row` (the default focus target once this
        // popup opens — see the end of this function) moves DOM focus
        // off the canvas, so the document-level Escape handler (which
        // also dispatches `ClearSelection`) can't fire; this closes
        // just the popup, matching what `value_input`'s own local
        // Escape handler does when the flyout is open instead.
        {
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
                if evt.key() == "Escape" {
                    remove_ctx_menu();
                }
            });
            tf_row
                .add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add tf row escape listener");
            cb.forget();
        }

        // Outside click (anywhere else in the popup — the checklist,
        // search box, Apply/Clear) closes the flyout, symmetric with
        // the `[data-op-wrap]`-scoped listener below that closes just
        // the nested dropdown. Independent scopes, so both listeners
        // coexist on the same `mousedown` without conflict.
        {
            let open = Rc::clone(&submenu_open);
            let set_open = set_submenu_open.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                if !open.get() {
                    return;
                }
                let inside = evt
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .and_then(|el| el.closest("[data-textfilter-wrap]").ok())
                    .is_some_and(|found| found.is_some());
                if inside {
                    return;
                }
                set_open(false);
            });
            menu.add_event_listener_with_callback(
                "mousedown",
                cb.as_ref().unchecked_ref(),
            )
            .expect("add menu mousedown listener (text filter)");
            cb.forget();
        }

        // Toggle open/close on trigger click.
        {
            let list = op_list.clone();
            let trigger = op_trigger.clone();
            let open = Rc::clone(&is_open);
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                evt.stop_propagation();
                let now_open = !open.get();
                open.set(now_open);
                let _ = list.style().set_property(
                    "display",
                    if now_open { "block" } else { "none" },
                );
                let _ = trigger.set_attribute(
                    "aria-expanded",
                    if now_open { "true" } else { "false" },
                );
            });
            op_trigger
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add op trigger click listener");
            cb.forget();
        }

        // Hover highlights the row under the cursor (event delegation
        // on the list, same pattern as `show_select_editor`'s own
        // mouseover handler).
        {
            let hl = Rc::clone(&highlight);
            let rows = Rc::clone(&op_rows);
            let hb = hover_bg.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                let Some(idx) = dd_idx_from_event(&evt) else {
                    return;
                };
                let old = hl.get();
                if old != idx {
                    dd_set_highlight(&rows, old, idx, &hb);
                    hl.set(idx);
                }
            });
            op_list
                .add_event_listener_with_callback(
                    "mouseover",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add op list mouseover listener");
            cb.forget();
        }

        // Click a row → commit and close.
        {
            let list = op_list.clone();
            let trigger = op_trigger.clone();
            let open = Rc::clone(&is_open);
            let selected = Rc::clone(&selected_op);
            let input = value_input.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                evt.stop_propagation();
                let Some(idx) = dd_idx_from_event(&evt) else {
                    return;
                };
                let Some((op, label)) = OPS.get(idx).map(|(o, l)| (*o, *l))
                else {
                    return;
                };
                selected.set(op);
                trigger.set_text_content(Some(label));
                let display = if value_needed(op) { "" } else { "none" };
                let _ = input.style().set_property("display", display);
                open.set(false);
                let _ = list.style().set_property("display", "none");
                let _ = trigger.set_attribute("aria-expanded", "false");
            });
            op_list
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add op list click listener");
            cb.forget();
        }

        // Outside click (anywhere else in the popup) closes the
        // dropdown without applying — `evt.target()`'s closest
        // `[data-op-wrap]` ancestor is `None` for anything outside it.
        {
            let list = op_list.clone();
            let trigger = op_trigger.clone();
            let open = Rc::clone(&is_open);
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
                if !open.get() {
                    return;
                }
                let inside = evt
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .and_then(|el| el.closest("[data-op-wrap]").ok())
                    .is_some_and(|found| found.is_some());
                if inside {
                    return;
                }
                open.set(false);
                let _ = list.style().set_property("display", "none");
                let _ = trigger.set_attribute("aria-expanded", "false");
            });
            menu.add_event_listener_with_callback(
                "mousedown",
                cb.as_ref().unchecked_ref(),
            )
            .expect("add menu mousedown listener");
            cb.forget();
        }

        // Keyboard: Enter/Space toggles (or commits the highlighted row
        // when open), ArrowUp/Down opens-if-closed and moves the
        // highlight, Escape closes the dropdown if open — otherwise
        // falls through to closing the whole popup, same rationale as
        // the value input's own local Escape handler below (focusing
        // this trigger moves DOM focus off the canvas, so the
        // document-level `has_focus()`-gated handler can't fire).
        {
            let list = op_list.clone();
            let trigger = op_trigger.clone();
            let open = Rc::clone(&is_open);
            let hl = Rc::clone(&highlight);
            let rows = Rc::clone(&op_rows);
            let selected = Rc::clone(&selected_op);
            let input = value_input.clone();
            let hb = hover_bg.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
                let key = evt.key();
                match key.as_str() {
                    "ArrowDown" | "ArrowUp" => {
                        evt.prevent_default();
                        if !open.get() {
                            open.set(true);
                            let _ =
                                list.style().set_property("display", "block");
                            let _ =
                                trigger.set_attribute("aria-expanded", "true");
                        }
                        let old = hl.get();
                        let nw = if key == "ArrowDown" {
                            if old + 1 < rows.len() { old + 1 } else { 0 }
                        } else if old > 0 {
                            old - 1
                        } else {
                            rows.len().saturating_sub(1)
                        };
                        dd_set_highlight(&rows, old, nw, &hb);
                        hl.set(nw);
                        dd_scroll_into_view(&list, &rows, nw);
                    }
                    "Enter" | " " => {
                        evt.prevent_default();
                        if open.get() {
                            let idx = hl.get();
                            if let Some((op, label)) =
                                OPS.get(idx).map(|(o, l)| (*o, *l))
                            {
                                selected.set(op);
                                trigger.set_text_content(Some(label));
                                let display =
                                    if value_needed(op) { "" } else { "none" };
                                let _ = input
                                    .style()
                                    .set_property("display", display);
                            }
                            open.set(false);
                            let _ =
                                list.style().set_property("display", "none");
                            let _ =
                                trigger.set_attribute("aria-expanded", "false");
                        } else {
                            open.set(true);
                            let _ =
                                list.style().set_property("display", "block");
                            let _ =
                                trigger.set_attribute("aria-expanded", "true");
                        }
                    }
                    "Escape" => {
                        if open.get() {
                            evt.stop_propagation();
                            open.set(false);
                            let _ =
                                list.style().set_property("display", "none");
                            let _ =
                                trigger.set_attribute("aria-expanded", "false");
                        } else {
                            remove_ctx_menu();
                        }
                    }
                    _ => {}
                }
            });
            op_trigger
                .add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add op trigger keydown listener");
            cb.forget();
        }

        // Value checklist (AG-Grid-style "Set Filter") — AND-combined
        // with the condition form above, not a replacement for it. Only
        // read by the Apply closure below; `None` means the column has
        // too many distinct values to list, in which case Apply leaves
        // `value_filters` for this column untouched (Clear still always
        // clears it — see the Clear closure).
        let checklist: Checklist = match unique {
            UniqueValues::TooMany { cap } => {
                let msg = make_el(&doc, "div");
                let border = format!("1px solid {}", colors.separator);
                set_styles(
                    &msg,
                    &[
                        ("border-top", &border),
                        ("margin-top", "10px"),
                        ("padding-top", "10px"),
                        ("color", &colors.text_disabled),
                        ("font-size", &colors.shortcut_font_size),
                    ],
                );
                msg.set_text_content(Some(&format!(
                    "Too many distinct values (> {cap}) to list — \
                         use the condition filter above."
                )));
                menu.append_child(&msg).expect("append too-many message");
                None
            }
            UniqueValues::Values(values) => {
                let divider = make_el(&doc, "div");
                let border = format!("1px solid {}", colors.separator);
                set_styles(
                    &divider,
                    &[("border-top", &border), ("margin", "10px 0")],
                );
                menu.append_child(&divider).expect("append divider");

                // Wrapper positions the magnifying-glass icon over the
                // input's left padding (added below) rather than
                // stacking it as separate flow content.
                let search_wrap = make_el(&doc, "div");
                set_styles(
                    &search_wrap,
                    &[
                        ("position", "relative"),
                        ("width", "100%"),
                        ("margin-bottom", "6px"),
                    ],
                );
                let search_icon = make_el(&doc, "span");
                set_styles(
                    &search_icon,
                    &[
                        ("position", "absolute"),
                        ("left", "10px"),
                        ("top", "50%"),
                        ("transform", "translateY(-50%)"),
                        ("display", "flex"),
                        ("pointer-events", "none"),
                        ("opacity", "0.55"),
                    ],
                );
                search_icon.set_inner_html(ICON_SEARCH);
                search_wrap
                    .append_child(&search_icon)
                    .expect("append search icon");

                let search: HtmlInputElement =
                    make_el(&doc, "input").dyn_into().expect("input element");
                search
                    .set_attribute("placeholder", "Search...")
                    .expect("set placeholder");
                set_styles(search.unchecked_ref(), &[("width", "100%")]);
                style_daisy_control(search.unchecked_ref(), &colors);
                wire_daisy_focus_ring(search.unchecked_ref(), &colors);
                // Room for the icon, on top of style_daisy_control's own
                // `padding: 0 12px`.
                let _ = search.style().set_property("padding-left", "32px");
                search_wrap
                    .append_child(&search)
                    .expect("append search input");
                menu.append_child(&search_wrap).expect("append search wrap");

                let list = make_el(&doc, "div");
                set_styles(
                    &list,
                    &[
                        ("max-height", "180px"),
                        ("overflow-y", "auto"),
                        ("padding", "4px 6px"),
                        ("margin-bottom", "10px"),
                    ],
                );

                let row_style: &[(&str, &str)] = &[
                    ("display", "flex"),
                    ("align-items", "center"),
                    ("gap", "6px"),
                    ("padding", "2px 0"),
                    ("cursor", "pointer"),
                ];

                let select_all_row = make_el(&doc, "label");
                set_styles(&select_all_row, row_style);
                let _ =
                    select_all_row.style().set_property("font-weight", "600");
                let select_all: HtmlInputElement =
                    make_el(&doc, "input").dyn_into().expect("input element");
                select_all.set_type("checkbox");
                style_checkbox(&select_all, &theme);
                select_all_row
                    .append_child(&select_all)
                    .expect("append select-all checkbox");
                let select_all_label = make_el(&doc, "span");
                select_all_label.set_text_content(Some("(Select All)"));
                select_all_row
                    .append_child(&select_all_label)
                    .expect("append select-all label");
                list.append_child(&select_all_row)
                    .expect("append select-all row");

                let mut rows: Vec<(String, HtmlElement, HtmlInputElement)> =
                    Vec::with_capacity(values.len());
                for value in &values {
                    let row = make_el(&doc, "label");
                    set_styles(&row, row_style);
                    let cb: HtmlInputElement = make_el(&doc, "input")
                        .dyn_into()
                        .expect("input element");
                    cb.set_type("checkbox");
                    style_checkbox(&cb, &theme);
                    // Absent entry (no value filter yet) → every
                    // value starts checked (no restriction).
                    let checked = existing_value_filter
                        .as_ref()
                        .map(|allowed| allowed.contains(value))
                        .unwrap_or(true);
                    cb.set_checked(checked);
                    row.append_child(&cb).expect("append checkbox");
                    let label_el = make_el(&doc, "span");
                    label_el.set_text_content(Some(value));
                    row.append_child(&label_el).expect("append label");
                    list.append_child(&row).expect("append value row");
                    rows.push((value.clone(), row, cb));
                }
                menu.append_child(&list).expect("append list");

                // Derive the select-all checkbox's initial tri-state
                // from the rows just built, rather than duplicating
                // the "is every value checked" logic separately.
                update_select_all_state(&select_all, &rows);

                // Search: hide non-matching rows, then refresh
                // select-all's tri-state over the now-visible subset
                // (AG-Grid behavior — select-all acts on the
                // search-filtered subset, not hidden rows).
                {
                    let rows = rows.clone();
                    let select_all = select_all.clone();
                    let search_el = search.clone();
                    let cb = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                        let query = search_el.value().to_lowercase();
                        for (value, row, _) in &rows {
                            let visible = query.is_empty()
                                || value.to_lowercase().contains(&query);
                            // "flex", not "" — an empty value clears the
                            // inline style override entirely, falling
                            // back to <label>'s default `display: inline`
                            // and breaking the row's checkbox/text
                            // alignment (and, since every row would then
                            // be `inline`, wrapping them side-by-side
                            // instead of one per line).
                            let _ = row.style().set_property(
                                "display",
                                if visible { "flex" } else { "none" },
                            );
                        }
                        update_select_all_state(&select_all, &rows);
                    });
                    search
                        .add_event_listener_with_callback(
                            "input",
                            cb.as_ref().unchecked_ref(),
                        )
                        .expect("add search input listener");
                    cb.forget();
                }

                // Select-all: check/uncheck every currently-visible
                // row.
                {
                    let rows = rows.clone();
                    let select_all_el = select_all.clone();
                    let cb = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                        let checked = select_all_el.checked();
                        for (_, row, cb) in &rows {
                            let visible = row
                                .style()
                                .get_property_value("display")
                                .unwrap_or_default()
                                != "none";
                            if visible {
                                cb.set_checked(checked);
                            }
                        }
                    });
                    select_all
                        .add_event_listener_with_callback(
                            "change",
                            cb.as_ref().unchecked_ref(),
                        )
                        .expect("add select-all change listener");
                    cb.forget();
                }

                // Each value checkbox recomputes select-all's
                // tri-state.
                for (_, _, value_cb) in &rows {
                    let rows2 = rows.clone();
                    let select_all = select_all.clone();
                    let listener =
                        Closure::<dyn FnMut(_)>::new(move |_: Event| {
                            update_select_all_state(&select_all, &rows2);
                        });
                    value_cb
                        .add_event_listener_with_callback(
                            "change",
                            listener.as_ref().unchecked_ref(),
                        )
                        .expect("add value checkbox change listener");
                    listener.forget();
                }

                let checkboxes: Vec<(String, HtmlInputElement)> =
                    rows.into_iter().map(|(v, _, cb)| (v, cb)).collect();
                Some((values, checkboxes))
            }
            // `UniqueValues` is `#[non_exhaustive]` — treat any
            // future variant conservatively, same as `TooMany`
            // without a message: don't render a checklist for it.
            _ => None,
        };

        let buttons = make_el(&doc, "div");
        set_styles(
            &buttons,
            &[
                ("display", "flex"),
                ("gap", "8px"),
                ("justify-content", "flex-end"),
            ],
        );
        let locale = self.0.locale.borrow();
        let apply_label = locale.filter_apply.clone();
        let clear_label = locale.clear_filter.clone();
        drop(locale);
        let primary = theme.checkbox_checked_bg.to_css();
        let apply_btn =
            make_daisy_button(&doc, &apply_label, &colors, Some(&primary));
        let clear_btn = make_daisy_button(&doc, &clear_label, &colors, None);
        buttons
            .append_child(&apply_btn)
            .expect("append apply button");
        buttons
            .append_child(&clear_btn)
            .expect("append clear button");
        buttons
            .append_child(&daisy_btn_focus_visible_style(&doc))
            .expect("append button focus-visible style");
        menu.append_child(&buttons).expect("append buttons");

        // Apply — dispatch the current operator/value, plus the value
        // checklist if it was rendered, and close.
        {
            let gc = self.clone();
            let col_key = col_key.clone();
            let selected_op = Rc::clone(&selected_op);
            let input = value_input.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                let condition = read_condition(selected_op.get(), &input);
                gc.dispatch(GridCommand::SetColumnFilter {
                    col_key: col_key.clone(),
                    condition,
                });
                if let Some((all_values, checkboxes)) = &checklist {
                    // Read every checkbox, not just currently-visible
                    // ones — the search box only hides rows, it doesn't
                    // discard their checked state.
                    let checked: HashSet<String> = checkboxes
                        .iter()
                        .filter(|(_, cb)| cb.checked())
                        .map(|(v, _)| v.clone())
                        .collect();
                    if checked.len() == all_values.len() {
                        // Every value still checked = no restriction —
                        // clear instead of storing a no-op full set, so
                        // the header icon's active color stays accurate.
                        gc.dispatch(GridCommand::ClearColumnValueFilter {
                            col_key: col_key.clone(),
                        });
                    } else {
                        gc.dispatch(GridCommand::SetColumnValueFilter {
                            col_key: col_key.clone(),
                            values: checked,
                        });
                    }
                }
                remove_ctx_menu();
            });
            apply_btn
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add apply click listener");
            cb.forget();
        }

        // Clear — reset to an empty condition, remove any value-set
        // restriction (regardless of whether the checklist could be
        // rendered — "Clear Filter" must mean no filtering on this
        // column at all), and close.
        {
            let gc = self.clone();
            let col_key = col_key.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                gc.dispatch(GridCommand::SetColumnFilter {
                    col_key: col_key.clone(),
                    condition: FilterCondition::default(),
                });
                gc.dispatch(GridCommand::ClearColumnValueFilter {
                    col_key: col_key.clone(),
                });
                remove_ctx_menu();
            });
            clear_btn
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add clear click listener");
            cb.forget();
        }

        // Enter in the value input applies immediately, same as
        // clicking Apply. Escape closes without applying — handled
        // here rather than relying on the document-level keydown
        // handler's `has_focus()` gate, which checks for canvas
        // focus specifically: focusing this input (below) moves DOM
        // focus away from the canvas, the same reason the inline
        // edit `<input>` and the search bar each wire their own
        // local Escape listener instead of depending on that gate.
        {
            let gc = self.clone();
            let col_key = col_key.clone();
            let selected_op = Rc::clone(&selected_op);
            let input = value_input.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" => {
                            let condition =
                                read_condition(selected_op.get(), &input);
                            gc.dispatch(GridCommand::SetColumnFilter {
                                col_key: col_key.clone(),
                                condition,
                            });
                            remove_ctx_menu();
                        }
                        "Escape" => remove_ctx_menu(),
                        _ => {}
                    },
                );
            value_input
                .add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                )
                .expect("add keydown listener");
            cb.forget();
        }

        // `value_input` only gets focus once the flyout is actually
        // opened (see the "Text Filter" row's click handler above) —
        // it starts hidden inside the collapsed flyout, so focusing it
        // here would be a no-op. `tf_row` holds focus by default
        // instead, so something inside the popup always does (its own
        // local Escape handler above closes the popup, same as
        // `value_input`'s — see the comment there for why this can't
        // just rely on the document-level handler).
        let _ = tf_row.focus();
    }
}
