use rs_grid_core::commands::GridCommand;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlElement, MouseEvent};

use super::dom_helpers::{document, make_el, set_styles};
use super::GridCanvas;

// ── context-menu icons (Feather Icons) ───────────────────────────────────────

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

pub(super) fn make_menu_separator(doc: &web_sys::Document) -> HtmlElement {
    let sep = make_el(doc, "div");
    set_styles(
        &sep,
        &[("border-top", "1px solid #e5e7eb"), ("margin", "4px 0")],
    );
    sep
}

pub(super) fn make_menu_item(
    doc: &web_sys::Document,
    icon: &str,
    label: &str,
    shortcut: &str,
    enabled: bool,
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
            ("color", "#9ca3af"),
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
        ("#111827", "pointer")
    } else {
        ("#9ca3af", "default")
    };
    set_styles(&item, &[("color", color), ("cursor", cursor)]);
    if enabled {
        let item_over = item.clone();
        let cb_over = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
            item_over
                .style()
                .set_property("background", "#f3f4f6")
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

// ── impl GridCanvas ──────────────────────────────────────────────────────────

impl GridCanvas {
    pub(super) fn attach_contextmenu(&self) {
        let gc = self.clone();
        let cb = Closure::<dyn FnMut(_)>::new(move |evt: MouseEvent| {
            evt.prevent_default();

            let (cx, cy) = gc.canvas_xy(&evt);

            // Right-click on column header → column-specific menu.
            let col = gc.0.state.borrow().hit_test_col_header(cx, cy);
            if let Some(col_idx) = col {
                gc.show_col_header_menu(
                    col_idx,
                    evt.client_x(),
                    evt.client_y(),
                );
                return;
            }

            // Select cell under right-click if nothing is selected yet
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
        self.0
            .canvas
            .add_event_listener_with_callback(
                "contextmenu",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
        cb.forget();
    }

    fn show_col_header_menu(&self, col_idx: usize, x: i32, y: i32) {
        let doc = document();
        remove_ctx_menu();
        let body = doc.body().expect("no body");

        // Backdrop
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

        // Menu
        let menu = make_el(&doc, "div");
        menu.set_id("rs-grid-ctx-menu");
        set_styles(
            &menu,
            &[
                ("position", "fixed"),
                ("left", &format!("{}px", x)),
                ("top", &format!("{}px", y)),
                ("z-index", "9999"),
                ("background", "#ffffff"),
                ("border", "1px solid #d1d5db"),
                ("border-radius", "6px"),
                ("box-shadow", "0 4px 16px rgba(0,0,0,0.12)"),
                ("padding", "4px 0"),
                ("font", "13px/1.4 system-ui,sans-serif"),
                ("min-width", "160px"),
                ("user-select", "none"),
            ],
        );

        let is_pinned = {
            let state = self.0.state.borrow();
            col_idx < state.model.pinned_count
        };

        if is_pinned {
            let item = make_menu_item(&doc, ICON_PIN, "Unpin Column", "", true);
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.set_pinned_count(0);
            });
            item.add_event_listener_with_callback(
                "click",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
            cb.forget();
            menu.append_child(&item).unwrap();
        } else {
            let item = make_menu_item(&doc, ICON_PIN, "Pin Column", "", true);
            let gc = self.clone();
            let ci = col_idx;
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.set_pinned_count(ci + 1);
            });
            item.add_event_listener_with_callback(
                "click",
                cb.as_ref().unchecked_ref(),
            )
            .unwrap();
            cb.forget();
            menu.append_child(&item).unwrap();
        }

        body.append_child(&backdrop).unwrap();
        body.append_child(&menu).unwrap();
    }

    fn show_context_menu(&self, x: i32, y: i32) {
        let doc = document();
        remove_ctx_menu();

        let body = doc.body().expect("no body");

        // ── backdrop ─────────────────────────────────────────
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

        // ── menu container ───────────────────────────────────
        let menu = make_el(&doc, "div");
        menu.set_id("rs-grid-ctx-menu");
        set_styles(
            &menu,
            &[
                ("position", "fixed"),
                ("left", &format!("{}px", x)),
                ("top", &format!("{}px", y)),
                ("z-index", "9999"),
                ("background", "#ffffff"),
                ("border", "1px solid #d1d5db"),
                ("border-radius", "6px"),
                ("box-shadow", "0 4px 16px rgba(0,0,0,0.12)"),
                ("padding", "4px 0"),
                ("font", "13px/1.4 system-ui,sans-serif"),
                ("min-width", "160px"),
                ("user-select", "none"),
            ],
        );

        let has_selection = self.0.state.borrow().selection.has_selection();

        // ── Cut ──────────────────────────────────────────────
        let cut_item =
            make_menu_item(&doc, ICON_CUT, "Couper", "Ctrl+X", has_selection);
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_cut();
            });
            cut_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&cut_item).unwrap();

        // ── Copy ─────────────────────────────────────────────
        let copy_item =
            make_menu_item(&doc, ICON_COPY, "Copier", "Ctrl+C", has_selection);
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_copy();
            });
            copy_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&copy_item).unwrap();

        // ── Copy with Headers ────────────────────────────────
        let copy_hdrs_item = make_menu_item(
            &doc,
            ICON_COPY,
            "Copier avec en-têtes",
            "",
            has_selection,
        );
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.handle_copy_headers();
            });
            copy_hdrs_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&copy_hdrs_item).unwrap();

        // ── separator ────────────────────────────────────────
        menu.append_child(&make_menu_separator(&doc)).unwrap();

        // ── Paste ────────────────────────────────────────────
        let paste_item =
            make_menu_item(&doc, ICON_PASTE, "Coller", "Ctrl+V", has_selection);
        if has_selection {
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                if !gc.0.state.borrow().selection.has_selection() {
                    return;
                }
                let win = web_sys::window().expect("no window");
                let promise = win.navigator().clipboard().read_text();
                let gc2 = gc.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match wasm_bindgen_futures::JsFuture::from(promise).await {
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
            });
            paste_item
                .add_event_listener_with_callback(
                    "click",
                    cb.as_ref().unchecked_ref(),
                )
                .unwrap();
            cb.forget();
        }
        menu.append_child(&paste_item).unwrap();

        body.append_child(&backdrop).unwrap();
        body.append_child(&menu).unwrap();
    }
}
