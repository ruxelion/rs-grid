use rs_grid_core::commands::GridCommand;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlElement, MouseEvent};

use super::context_menu_config::{BuiltinAction, ContextMenuItem};
use super::dom_helpers::{document, make_el, set_styles};
use super::GridCanvas;
use crate::css_theme;

// ── context-menu icons (Feather Icons) ──────────────────

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
            ("gap", "8px"),
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
            ("font-size", "11px"),
            ("white-space", "nowrap"),
        ],
    );
    sc_el.set_text_content(Some(shortcut));
    row.append_child(&icon_el).unwrap();
    row.append_child(&label_el).unwrap();
    row.append_child(&sc_el).unwrap();
    item.append_child(&row).unwrap();

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
                .unwrap();
        });
        item.add_event_listener_with_callback(
            "mouseover",
            cb_over.as_ref().unchecked_ref(),
        )
        .unwrap();
        cb_over.forget();

        let item_out = item.clone();
        let cb_out = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_out.style().set_property("background", "").unwrap();
        });
        item.add_event_listener_with_callback(
            "mouseout",
            cb_out.as_ref().unchecked_ref(),
        )
        .unwrap();
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
    }
}

fn builtin_label(action: BuiltinAction) -> &'static str {
    match action {
        BuiltinAction::Cut => "Cut",
        BuiltinAction::Copy => "Copy",
        BuiltinAction::CopyWithHeaders => "Copy with headers",
        BuiltinAction::Paste => "Paste",
        BuiltinAction::PinColumn => "Pin Column",
        BuiltinAction::UnpinColumn => "Unpin Column",
    }
}

fn builtin_shortcut(action: BuiltinAction) -> &'static str {
    match action {
        BuiltinAction::Cut => "Ctrl+X",
        BuiltinAction::Copy => "Ctrl+C",
        BuiltinAction::Paste => "Ctrl+V",
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
            .unwrap();
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
            .unwrap();
        cb.forget();
    }

    let menu = make_el(&doc, "div");
    menu.set_id("rs-grid-ctx-menu");
    let border_val = format!("1px solid {}", colors.border);
    set_styles(
        &menu,
        &[
            ("position", "fixed"),
            ("left", &format!("{}px", x)),
            ("top", &format!("{}px", y)),
            ("z-index", "9999"),
            ("background", &colors.bg),
            ("border", &border_val),
            ("border-radius", "6px"),
            ("box-shadow", &colors.shadow),
            ("padding", "4px 0"),
            ("font", "13px/1.4 system-ui,sans-serif"),
            ("min-width", "160px"),
            ("user-select", "none"),
        ],
    );

    body.append_child(&backdrop).unwrap();
    body.append_child(&menu).unwrap();

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
            .unwrap();
        self.0
            .canvas_listeners
            .borrow_mut()
            .push(("contextmenu".to_string(), f));
        self.0.closures.borrow_mut().push(Box::new(cb));
    }

    fn show_col_header_menu(&self, col_idx: usize, x: i32, y: i32) {
        remove_ctx_menu();
        let colors = read_ctx_colors();
        let doc = document();
        let (_, menu) = create_menu_shell(x, y, &colors);

        let config = self.0.ctx_menu_config.borrow();
        let is_pinned = {
            let state = self.0.state.borrow();
            col_idx < state.model.pinned_count
        };

        // Build item list.
        let default_items;
        let items: &[ContextMenuItem] = match config.col_header_items.as_deref()
        {
            Some(list) => list,
            None => {
                default_items = if is_pinned {
                    vec![ContextMenuItem::unpin_column()]
                } else {
                    vec![ContextMenuItem::pin_column()]
                };
                &default_items
            }
        };

        for item_cfg in items {
            match item_cfg {
                ContextMenuItem::Separator => {
                    menu.append_child(&make_menu_separator(&doc, &colors))
                        .unwrap();
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
                        .unwrap_or(builtin_label(effective_action));
                    let ico = icon
                        .as_deref()
                        .unwrap_or(builtin_icon(effective_action));
                    let sc = shortcut
                        .as_deref()
                        .unwrap_or(builtin_shortcut(effective_action));

                    let el = make_menu_item(&doc, ico, lbl, sc, true, &colors);
                    self.wire_builtin(&el, effective_action, col_idx);
                    menu.append_child(&el).unwrap();
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

        for item_cfg in items {
            match item_cfg {
                ContextMenuItem::Separator => {
                    menu.append_child(&make_menu_separator(&doc, &colors))
                        .unwrap();
                }
                ContextMenuItem::Builtin {
                    action,
                    label,
                    icon,
                    shortcut,
                } => {
                    let action = *action;
                    let enabled = match action {
                        BuiltinAction::Cut
                        | BuiltinAction::Copy
                        | BuiltinAction::CopyWithHeaders
                        | BuiltinAction::Paste => has_selection,
                        _ => true,
                    };
                    let lbl = label.as_deref().unwrap_or(builtin_label(action));
                    let ico = icon.as_deref().unwrap_or(builtin_icon(action));
                    let sc =
                        shortcut.as_deref().unwrap_or(builtin_shortcut(action));

                    let el =
                        make_menu_item(&doc, ico, lbl, sc, enabled, &colors);
                    if enabled {
                        self.wire_builtin(&el, action, 0);
                    }
                    menu.append_child(&el).unwrap();
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
                                }
                            }
                            Err(e) => {
                                web_sys::console::warn_1(&e);
                            }
                        }
                    });
                }
                BuiltinAction::PinColumn => {
                    gc.set_pinned_count(col_idx + 1);
                }
                BuiltinAction::UnpinColumn => {
                    gc.set_pinned_count(0);
                }
            }
        });
        el.add_event_listener_with_callback(
            "click",
            cb.as_ref().unchecked_ref(),
        )
        .unwrap();
        cb.forget();
    }
}
