use rs_grid_core::{commands::GridCommand, sort::SortDir};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlElement, MouseEvent};

use super::context_menu_config::{BuiltinAction, ContextMenuItem};
use super::dom_helpers::{document, make_el, set_styles};
use super::GridCanvas;
use crate::css_theme;
use crate::locale::Locale;

// ── context-menu icons (Feather Icons) ──────────────────

const ICON_SORT_ASC: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<line x1="12" y1="19" x2="12" y2="5"/>"#,
    r#"<polyline points="5 12 12 5 19 12"/></svg>"#
);
const ICON_SORT_DESC: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<line x1="12" y1="5" x2="12" y2="19"/>"#,
    r#"<polyline points="19 12 12 19 5 12"/></svg>"#
);
const ICON_CLEAR_SORT: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<line x1="18" y1="6" x2="6" y2="18"/>"#,
    r#"<line x1="6" y1="6" x2="18" y2="18"/></svg>"#
);
const ICON_AUTOSIZE: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<polyline points="15 3 21 3 21 9"/>"#,
    r#"<polyline points="9 21 3 21 3 15"/>"#,
    r#"<line x1="21" y1="3" x2="14" y2="10"/>"#,
    r#"<line x1="3" y1="21" x2="10" y2="14"/></svg>"#
);
const ICON_PIN: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<path d="M12 17v5"/>"#,
    r#"<path d="M9 11V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v7"/>"#,
    r#"<path d="M5 17h14l-1.5-6H6.5L5 17z"/></svg>"#
);
const ICON_CUT: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/>"#,
    r#"<line x1="20" y1="4" x2="8.12" y2="15.88"/>"#,
    r#"<line x1="14.47" y1="14.48" x2="20" y2="20"/>"#,
    r#"<line x1="8.12" y1="8.12" x2="12" y2="12"/></svg>"#
);
const ICON_COPY: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>"#,
    r#"<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>"#,
    r#"</svg>"#
);
const ICON_PASTE: &str = concat!(
    r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6"#,
    r#" a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>"#,
    r#"<rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>"#
);

// ── themed colors ───────────────────────────────────────

struct CtxColors {
    bg: String,
    border: String,
    shadow: String,
    text: String,
    text_disabled: String,
    hover_bg: String,
    separator: String,
    radius: String,
    font_size: String,
    min_width: String,
    shortcut_font_size: String,
    item_gap: String,
}

fn read_ctx_colors() -> CtxColors {
    let style = css_theme::root_computed_style();
    let v = |name: &str, fallback: &str| -> String {
        let val = style
            .as_ref()
            .map(|s| css_theme::get_var(s, name))
            .unwrap_or_default();
        if val.is_empty() {
            fallback.to_string()
        } else {
            val
        }
    };
    CtxColors {
        bg: v("--rs-grid-ctx-bg", "#ffffff"),
        border: v("--rs-grid-ctx-border", "#d1d5db"),
        shadow: v("--rs-grid-ctx-shadow", "0 4px 16px rgba(0,0,0,0.12)"),
        text: v("--rs-grid-ctx-text", "#111827"),
        text_disabled: v("--rs-grid-ctx-text-disabled", "#9ca3af"),
        hover_bg: v("--rs-grid-ctx-hover-bg", "#f3f4f6"),
        separator: v("--rs-grid-ctx-separator", "#e5e7eb"),
        radius: v("--rs-grid-ctx-radius", "6px"),
        font_size: v("--rs-grid-ctx-font-size", "13px"),
        min_width: v("--rs-grid-ctx-min-width", "160px"),
        shortcut_font_size: v("--rs-grid-ctx-shortcut-font-size", "11px"),
        item_gap: v("--rs-grid-ctx-item-gap", "8px"),
    }
}

// ── helpers ─────────────────────────────────────────────

fn make_menu_separator(
    doc: &web_sys::Document,
    colors: &CtxColors,
) -> HtmlElement {
    let sep = make_el(doc, "div");
    let border = format!("1px solid {}", colors.separator);
    set_styles(&sep, &[("border-top", &border), ("margin", "4px 0")]);
    sep
}

fn make_menu_item(
    doc: &web_sys::Document,
    icon: &str,
    label: &str,
    shortcut: &str,
    enabled: bool,
    colors: &CtxColors,
) -> HtmlElement {
    let item = make_el(doc, "div");
    let row = make_el(doc, "div");
    set_styles(
        &row,
        &[
            ("display", "flex"),
            ("align-items", "center"),
            ("gap", &colors.item_gap),
            ("padding", "6px 12px"),
        ],
    );
    let icon_el = make_el(doc, "span");
    set_styles(
        &icon_el,
        &[
            ("width", "16px"),
            ("height", "16px"),
            ("display", "flex"),
            ("align-items", "center"),
            ("justify-content", "center"),
            ("flex-shrink", "0"),
            ("opacity", "0.6"),
        ],
    );
    icon_el.set_inner_html(icon);
    let label_el = make_el(doc, "span");
    set_styles(&label_el, &[("flex", "1")]);
    label_el.set_text_content(Some(label));
    let sc_el = make_el(doc, "span");
    set_styles(
        &sc_el,
        &[
            ("color", &colors.text_disabled),
            ("font-size", &colors.shortcut_font_size),
            ("white-space", "nowrap"),
        ],
    );
    sc_el.set_text_content(Some(shortcut));
    row.append_child(&icon_el).expect("append icon");
    row.append_child(&label_el).expect("append label");
    row.append_child(&sc_el).expect("append shortcut");
    item.append_child(&row).expect("append row");

    let (color, cursor) = if enabled {
        (colors.text.as_str(), "pointer")
    } else {
        (colors.text_disabled.as_str(), "default")
    };
    set_styles(&item, &[("color", color), ("cursor", cursor)]);
    if enabled {
        let hover = colors.hover_bg.clone();
        let item_over = item.clone();
        let cb_over = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_over
                .style()
                .set_property("background", &hover)
                .expect("set hover bg");
        });
        item.add_event_listener_with_callback(
            "mouseover",
            cb_over.as_ref().unchecked_ref(),
        )
        .expect("add mouseover listener");
        cb_over.forget();

        let item_out = item.clone();
        let cb_out = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_out
                .style()
                .set_property("background", "")
                .expect("clear hover bg");
        });
        item.add_event_listener_with_callback(
            "mouseout",
            cb_out.as_ref().unchecked_ref(),
        )
        .expect("add mouseout listener");
        cb_out.forget();
    }
    item
}

pub(super) fn remove_ctx_menu() {
    let doc = document();
    if let Some(el) = doc.get_element_by_id("rs-grid-ctx-backdrop") {
        el.remove();
    }
    if let Some(el) = doc.get_element_by_id("rs-grid-ctx-menu") {
        el.remove();
    }
}

// ── builtin defaults ────────────────────────────────────

fn builtin_icon(action: BuiltinAction) -> &'static str {
    match action {
        BuiltinAction::Cut => ICON_CUT,
        BuiltinAction::Copy | BuiltinAction::CopyWithHeaders => ICON_COPY,
        BuiltinAction::Paste => ICON_PASTE,
        BuiltinAction::PinColumn | BuiltinAction::UnpinColumn => ICON_PIN,
        BuiltinAction::SortAsc => ICON_SORT_ASC,
        BuiltinAction::SortDesc => ICON_SORT_DESC,
        BuiltinAction::ClearSort => ICON_CLEAR_SORT,
        BuiltinAction::AutoSizeColumn | BuiltinAction::AutoSizeAllColumns => {
            ICON_AUTOSIZE
        }
    }
}

fn builtin_label(action: BuiltinAction, locale: &Locale) -> &str {
    match action {
        BuiltinAction::Cut => &locale.cut,
        BuiltinAction::Copy => &locale.copy,
        BuiltinAction::CopyWithHeaders => &locale.copy_with_headers,
        BuiltinAction::Paste => &locale.paste,
        BuiltinAction::PinColumn => &locale.pin_column,
        BuiltinAction::UnpinColumn => &locale.unpin_column,
        BuiltinAction::SortAsc => &locale.sort_ascending,
        BuiltinAction::SortDesc => &locale.sort_descending,
        BuiltinAction::ClearSort => &locale.clear_sort,
        BuiltinAction::AutoSizeColumn => &locale.autosize_this_column,
        BuiltinAction::AutoSizeAllColumns => &locale.autosize_all_columns,
    }
}

fn builtin_shortcut(action: BuiltinAction, locale: &Locale) -> &str {
    match action {
        BuiltinAction::Cut => &locale.shortcut_cut,
        BuiltinAction::Copy => &locale.shortcut_copy,
        BuiltinAction::Paste => &locale.shortcut_paste,
        _ => "",
    }
}

// ── menu container ──────────────────────────────────────
//
// All closures below use `Closure::forget()` rather than
// storing in `GridCanvas`'s closure Vec.  This is
// intentional: the context menu is a self-contained DOM
// subtree rooted at `#rs-grid-ctx-backdrop`.
// `remove_ctx_menu()` removes that root element; JS GC then
// releases the element, its children, and all attached event
// listeners — including the forgotten closures.
//
// The alternative (explicit `removeEventListener` before
// dropping) would require threading a `Vec<Closure>` through
// every helper and calling it from `remove_ctx_menu()`.  The
// complexity gain is not justified: each closure captures only
// lightweight DOM handles or a ref-counted `GridCanvas` clone,
// and the menu lifetime is bounded by the user's single
// right-click gesture.

fn create_menu_shell(
    x: i32,
    y: i32,
    colors: &CtxColors,
) -> (HtmlElement, HtmlElement) {
    let doc = document();
    let body = doc.body().expect("no body");

    let backdrop = make_el(&doc, "div");
    backdrop.set_id("rs-grid-ctx-backdrop");
    set_styles(
        &backdrop,
        &[("position", "fixed"), ("inset", "0"), ("z-index", "9998")],
    );
    {
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            remove_ctx_menu();
        });
        backdrop
            .add_event_listener_with_callback(
                "click",
                cb.as_ref().unchecked_ref(),
            )
            .expect("add backdrop click listener");
        // Intentional forget — see module comment above.
        cb.forget();
    }
    {
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            evt.prevent_default();
            remove_ctx_menu();
        });
        backdrop
            .add_event_listener_with_callback(
                "contextmenu",
                cb.as_ref().unchecked_ref(),
            )
            .expect("add backdrop contextmenu listener");
        cb.forget();
    }

    let menu = make_el(&doc, "div");
    menu.set_id("rs-grid-ctx-menu");
    let border_val = format!("1px solid {}", colors.border);
    let font_val = format!("{}/1.4 system-ui,sans-serif", colors.font_size);
    set_styles(
        &menu,
        &[
            ("position", "fixed"),
            ("left", &format!("{}px", x)),
            ("top", &format!("{}px", y)),
            ("z-index", "9999"),
            ("background", &colors.bg),
            ("border", &border_val),
            ("border-radius", &colors.radius),
            ("box-shadow", &colors.shadow),
            ("padding", "4px 0"),
            ("font", &font_val),
            ("min-width", &colors.min_width),
            ("user-select", "none"),
        ],
    );

    body.append_child(&backdrop).expect("append backdrop");
    body.append_child(&menu).expect("append menu");

    (backdrop, menu)
}

// ── impl GridCanvas ─────────────────────────────────────

impl GridCanvas {
    pub(super) fn attach_contextmenu(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            evt.prevent_default();
            let _ = gc.0.canvas.focus();

            let (cx, cy) = gc.canvas_xy(&evt);

            // Right-click on column header → column menu.
            let col = gc.0.state.borrow().hit_test_col_header(cx, cy);
            if let Some(col_idx) = col {
                gc.show_col_header_menu(
                    col_idx,
                    evt.client_x(),
                    evt.client_y(),
                );
                return;
            }

            // Select cell under right-click if needed.
            let has_sel = gc.0.state.borrow().selection.has_selection();
            if !has_sel {
                let row = gc.0.state.borrow().hit_test_row_header(cx, cy);
                if let Some(row) = row {
                    gc.dispatch(GridCommand::SelectRow(row));
                } else {
                    let coord = gc.0.state.borrow().hit_test(cx, cy);
                    if let Some(coord) = coord {
                        gc.dispatch(GridCommand::SelectCell(coord));
                    }
                }
            }

            gc.show_context_menu(evt.client_x(), evt.client_y());
        });
        let f: js_sys::Function =
            cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
        self.0
            .canvas
            .add_event_listener_with_callback("contextmenu", &f)
            .expect("add contextmenu listener");
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("contextmenu".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    pub(super) fn show_col_header_menu(&self, col_idx: usize, x: i32, y: i32) {
        remove_ctx_menu();
        let colors = read_ctx_colors();
        let doc = document();
        let (_, menu) = create_menu_shell(x, y, &colors);

        let config = self.0.ctx_menu_config.borrow();
        let (is_pinned, col_sorted) = {
            let state = self.0.state.borrow();
            let pinned = col_idx < state.model.pinned_count;
            let col_key = state
                .model
                .columns
                .get(col_idx)
                .map(|c| c.key.as_str())
                .unwrap_or("");
            let sorted =
                state.sort.as_ref().is_some_and(|s| s.col_key == col_key);
            (pinned, sorted)
        };

        // Build item list.
        let default_items;
        let items: &[ContextMenuItem] = match config.col_header_items.as_deref()
        {
            Some(list) => list,
            None => {
                let mut v = vec![
                    ContextMenuItem::sort_asc(),
                    ContextMenuItem::sort_desc(),
                ];
                if col_sorted {
                    v.push(ContextMenuItem::clear_sort());
                }
                v.push(ContextMenuItem::separator());
                v.push(ContextMenuItem::autosize_column());
                v.push(ContextMenuItem::autosize_all_columns());
                v.push(ContextMenuItem::separator());
                if is_pinned {
                    v.push(ContextMenuItem::unpin_column());
                } else {
                    v.push(ContextMenuItem::pin_column());
                }
                default_items = v;
                &default_items
            }
        };

        let locale = self.0.locale.borrow();
        for item_cfg in items {
            match item_cfg {
                ContextMenuItem::Separator => {
                    menu.append_child(&make_menu_separator(&doc, &colors))
                        .expect("append separator");
                }
                ContextMenuItem::Builtin {
                    action,
                    label,
                    icon,
                    shortcut,
                } => {
                    let action = *action;
                    // For PinColumn in config, show Pin or
                    // Unpin based on actual state.
                    let effective_action = if action == BuiltinAction::PinColumn
                        && is_pinned
                    {
                        BuiltinAction::UnpinColumn
                    } else if action == BuiltinAction::UnpinColumn && !is_pinned
                    {
                        BuiltinAction::PinColumn
                    } else {
                        action
                    };
                    let lbl = label
                        .as_deref()
                        .unwrap_or(builtin_label(effective_action, &locale));
                    let ico = icon
                        .as_deref()
                        .unwrap_or(builtin_icon(effective_action));
                    let sc = shortcut
                        .as_deref()
                        .unwrap_or(builtin_shortcut(effective_action, &locale));

                    let el = make_menu_item(&doc, ico, lbl, sc, true, &colors);
                    self.wire_builtin(&el, effective_action, col_idx);
                    menu.append_child(&el).expect("append menu item");
                }
            }
        }
    }

    fn show_context_menu(&self, x: i32, y: i32) {
        remove_ctx_menu();
        let colors = read_ctx_colors();
        let doc = document();
        let (_, menu) = create_menu_shell(x, y, &colors);

        let config = self.0.ctx_menu_config.borrow();
        let has_selection = self.0.state.borrow().selection.has_selection();

        let default_items;
        let items: &[ContextMenuItem] = match config.cell_items.as_deref() {
            Some(list) => list,
            None => {
                default_items = vec![
                    ContextMenuItem::cut(),
                    ContextMenuItem::copy(),
                    ContextMenuItem::copy_with_headers(),
                    ContextMenuItem::separator(),
                    ContextMenuItem::paste(),
                ];
                &default_items
            }
        };

        let locale = self.0.locale.borrow();
        for item_cfg in items {
            match item_cfg {
                ContextMenuItem::Separator => {
                    menu.append_child(&make_menu_separator(&doc, &colors))
                        .expect("append separator");
                }
                ContextMenuItem::Builtin {
                    action,
                    label,
                    icon,
                    shortcut,
                } => {
                    let action = *action;
                    let secure = web_sys::window()
                        .map(|w| w.is_secure_context())
                        .unwrap_or(false);
                    let enabled = match action {
                        BuiltinAction::Cut
                        | BuiltinAction::Copy
                        | BuiltinAction::CopyWithHeaders => has_selection,
                        BuiltinAction::Paste => has_selection && secure,
                        _ => true,
                    };
                    let lbl = label
                        .as_deref()
                        .unwrap_or(builtin_label(action, &locale));
                    let ico = icon.as_deref().unwrap_or(builtin_icon(action));
                    let sc = shortcut
                        .as_deref()
                        .unwrap_or(builtin_shortcut(action, &locale));

                    let el =
                        make_menu_item(&doc, ico, lbl, sc, enabled, &colors);
                    if enabled {
                        self.wire_builtin(&el, action, 0);
                    }
                    menu.append_child(&el).expect("append menu item");
                }
            }
        }
    }

    /// Attach the click handler for a built-in action to a
    /// menu item element.
    fn wire_builtin(
        &self,
        el: &HtmlElement,
        action: BuiltinAction,
        col_idx: usize,
    ) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            remove_ctx_menu();
            match action {
                BuiltinAction::Cut => gc.handle_cut(),
                BuiltinAction::Copy => gc.handle_copy(),
                BuiltinAction::CopyWithHeaders => gc.handle_copy_headers(),
                BuiltinAction::Paste => {
                    if !gc.0.state.borrow().selection.has_selection() {
                        return;
                    }
                    let win = web_sys::window().expect("no window");
                    let promise = win.navigator().clipboard().read_text();
                    let gc2 = gc.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match wasm_bindgen_futures::JsFuture::from(promise)
                            .await
                        {
                            Ok(val) => {
                                if let Some(text) = val.as_string() {
                                    gc2.dispatch(GridCommand::PasteAt { text });
                                    gc2.flash_selection();
                                }
                            }
                            Err(e) => {
                                web_sys::console::warn_1(&e);
                            }
                        }
                    });
                }
                BuiltinAction::PinColumn => {
                    let pinned_count = gc.0.state.borrow().model.pinned_count;
                    // Move the column to the end of the pinned zone
                    // before expanding it, so only this column is
                    // added — not every column to its left.
                    if col_idx != pinned_count {
                        gc.dispatch(GridCommand::MoveColumn {
                            from_idx: col_idx,
                            to_idx: pinned_count,
                        });
                    }
                    gc.set_pinned_count(pinned_count + 1);
                }
                BuiltinAction::UnpinColumn => {
                    let pinned_count = gc.0.state.borrow().model.pinned_count;
                    let new_count = pinned_count.saturating_sub(1);
                    // Move the column just past the new pinned zone
                    // so it appears at the left of the scrollable
                    // area and only this column is unpinned.
                    if col_idx != new_count {
                        gc.dispatch(GridCommand::MoveColumn {
                            from_idx: col_idx,
                            to_idx: new_count,
                        });
                    }
                    gc.set_pinned_count(new_count);
                }
                BuiltinAction::SortAsc | BuiltinAction::SortDesc => {
                    let col_key =
                        gc.0.state
                            .borrow()
                            .model
                            .columns
                            .get(col_idx)
                            .map(|c| c.key.clone())
                            .unwrap_or_default();
                    let dir = if action == BuiltinAction::SortAsc {
                        SortDir::Asc
                    } else {
                        SortDir::Desc
                    };
                    gc.dispatch(GridCommand::SetSort { col_key, dir });
                }
                BuiltinAction::ClearSort => {
                    gc.dispatch(GridCommand::ClearSort);
                }
                BuiltinAction::AutoSizeColumn => {
                    let (cw, hcw, cp) = gc.autofit_params();
                    gc.dispatch(GridCommand::AutoFitColumn {
                        col_idx,
                        char_width: cw,
                        header_char_width: hcw,
                        cell_padding: cp,
                    });
                }
                BuiltinAction::AutoSizeAllColumns => {
                    let (cw, hcw, cp) = gc.autofit_params();
                    gc.dispatch(GridCommand::AutoFitAllColumns {
                        char_width: cw,
                        header_char_width: hcw,
                        cell_padding: cp,
                    });
                }
            }
        });
        el.add_event_listener_with_callback(
            "click",
            cb.as_ref().unchecked_ref(),
        )
        .expect("add click listener");
        cb.forget();
    }
}
