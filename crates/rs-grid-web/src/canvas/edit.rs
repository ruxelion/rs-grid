use std::{cell::Cell, rc::Rc};

use rs_grid_core::{
    column::{CellEditor, SelectOption},
    commands::GridCommand,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{HtmlImageElement, HtmlInputElement, KeyboardEvent};

use super::{dom_helpers::document, GridCanvas};
use crate::css_theme;

/// Geometry of an inline cell editor (screen coordinates).
struct EditorGeom {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl GridCanvas {
    /// Viewport rectangle `(x, y, w, h)` of a cell.
    fn cell_viewport_rect(
        &self,
        row: u64,
        col_idx: usize,
    ) -> (f64, f64, f64, f64) {
        let state = self.0.state.borrow();
        let model = &state.model;
        let off = model.column_offsets.offsets[col_idx];
        let cx = if col_idx < model.pinned_count {
            off + model.row_number_width
        } else {
            off - state.viewport.scroll_x + model.row_number_width
        };
        let cy = model.row_top(row) - state.viewport.scroll_y;
        let w = model.columns[col_idx].width;
        let h = model.row_height;
        (cx, cy, w, h)
    }

    /// Apply shared positioning styles to an edit overlay.
    fn apply_edit_style(
        &self,
        el: &web_sys::HtmlElement,
        left: f64,
        top: f64,
        w: f64,
        h: f64,
    ) {
        let css_style = css_theme::root_computed_style();
        let var = |name: &str, fb: &str| -> String {
            css_style
                .as_ref()
                .map(|s| css_theme::get_var(s, name))
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| fb.to_string())
        };
        let border_color = var("--rs-grid-editor-border", "#2563eb");
        let border_width = var("--rs-grid-editor-border-width", "2px");
        let border_radius = var("--rs-grid-editor-border-radius", "0");
        let bg = var("--rs-grid-editor-bg", "#ffffff");
        let padding = var("--rs-grid-editor-padding", "0 4px");
        let font_size = var("--rs-grid-editor-font-size", "inherit");
        let shadow = var("--rs-grid-editor-shadow", "none");

        // Parse border width to offset the editor so the border
        // wraps around the cell rather than clipping inside it.
        let bw: f64 =
            border_width.trim_end_matches("px").parse().unwrap_or(2.0);

        let style = el.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("left", &format!("{}px", left - bw));
        let _ = style.set_property("top", &format!("{}px", top - bw));
        let _ = style.set_property("width", &format!("{}px", w + 2.0 * bw));
        let _ = style.set_property("height", &format!("{}px", h + 2.0 * bw));
        let _ = style.set_property("z-index", "10000");
        let _ = style.set_property(
            "border",
            &format!("{border_width} solid {border_color}"),
        );
        let _ = style.set_property("border-radius", &border_radius);
        let _ = style.set_property("outline", "none");
        let _ = style.set_property("padding", &padding);
        let _ = style.set_property("margin", "0");
        let _ = style.set_property("box-sizing", "border-box");
        let _ = style.set_property("font-size", &font_size);
        let _ = style.set_property("font-family", "inherit");
        let _ = style.set_property("background", &bg);
        let _ = style.set_property("box-shadow", &shadow);
    }

    /// Create the appropriate DOM overlay for inline
    /// cell editing (text `<input>` or custom dropdown).
    pub(super) fn show_edit_input(&self) {
        self.remove_edit_input();
        self.0.edit_closures.borrow_mut().clear();

        let (row, col_key) = {
            let state = self.0.state.borrow();
            let edit = match &state.edit {
                Some(e) => e,
                None => return,
            };
            (edit.row, edit.col_key.clone())
        };

        let col_idx = {
            let state = self.0.state.borrow();
            match state.model.columns.iter().position(|c| c.key == col_key) {
                Some(i) => i,
                None => return,
            }
        };

        let (cx, cy, w, h) = self.cell_viewport_rect(row, col_idx);
        let canvas_rect = self.0.canvas.get_bounding_client_rect();
        let geom = EditorGeom {
            left: canvas_rect.left() + cx,
            top: canvas_rect.top() + cy,
            width: w,
            height: h,
        };

        // Read editor type and raw initial value.
        let (editor, raw_value) = {
            let state = self.0.state.borrow();
            let editor = state.model.columns[col_idx].editor.clone();
            let raw = state
                .edit
                .as_ref()
                .map(|e| e.initial_value.clone())
                .unwrap_or_default();
            (editor, raw)
        };

        match editor {
            Some(CellEditor::Select { ref options }) => {
                self.show_select_editor(
                    row, &col_key, options, geom, &raw_value,
                );
            }
            _ => {
                self.show_text_editor(row, &col_key, col_idx, geom, &raw_value);
            }
        }
    }

    /// Show a custom HTML dropdown editor with optional
    /// icons (e.g. flag SVGs).
    fn show_select_editor(
        &self,
        row: u64,
        col_key: &str,
        options: &[SelectOption],
        geom: EditorGeom,
        current_value: &str,
    ) {
        let EditorGeom {
            left,
            top,
            width: w,
            height: h,
        } = geom;
        let doc = document();
        let n = options.len();
        if n == 0 {
            return;
        }

        // ── theme colours ─────────────────────────────
        let (bg, text_c, sel_c, border_c, fsz, shadow, dd_min_w, dd_max_h) = {
            let b = self.0.builder.borrow();
            let t = &b.theme;
            let css_style = css_theme::root_computed_style();
            let var = |name: &str, fb: &str| -> String {
                css_style
                    .as_ref()
                    .map(|s| css_theme::get_var(s, name))
                    .filter(|v| !v.is_empty())
                    .unwrap_or_else(|| fb.to_string())
            };
            let shadow =
                var("--rs-grid-overlay-shadow", "0 4px 12px rgba(0,0,0,.15)");
            let dd_min_w: f64 = var("--rs-grid-dropdown-min-width", "220")
                .trim_end_matches("px")
                .parse()
                .unwrap_or(220.0);
            let dd_max_h = var("--rs-grid-dropdown-max-height", "240px");
            (
                t.bg.to_css(),
                t.cell_text.to_css(),
                t.selection_fill.to_css(),
                t.selection_border.to_css(),
                t.font_size,
                shadow,
                dd_min_w,
                dd_max_h,
            )
        };

        let cur = options
            .iter()
            .position(|o| o.value == current_value)
            .unwrap_or(0);

        // ── container div ─────────────────────────────
        let ctr: web_sys::HtmlElement = doc
            .create_element("div")
            .expect("div")
            .dyn_into()
            .expect("cast");
        let _ = ctr.set_attribute("tabindex", "-1");

        let s = ctr.style();
        let _ = s.set_property("position", "fixed");
        let _ = s.set_property("left", &format!("{left}px"));
        let _ = s.set_property("width", &format!("{}px", w.max(dd_min_w)));
        let _ = s.set_property("max-height", &dd_max_h);
        let _ = s.set_property("overflow-y", "auto");
        let _ = s.set_property("z-index", "10000");
        let _ = s.set_property("border", &format!("2px solid {border_c}"));
        let _ = s.set_property("border-radius", "4px");
        let _ = s.set_property("background", &bg);
        let _ = s.set_property("color", &text_c);
        let _ = s.set_property("font-size", &format!("{fsz}px"));
        let _ = s.set_property("font-family", "inherit");
        let _ = s.set_property("box-shadow", &shadow);
        let _ = s.set_property("outline", "none");
        let _ = s.set_property("box-sizing", "border-box");
        let _ = s.set_property("margin", "0");
        let _ = s.set_property("padding", "2px 0");

        // Below cell, or above if no room.
        let win_h = web_sys::window()
            .and_then(|w| w.inner_height().ok())
            .and_then(|v| v.as_f64())
            .unwrap_or(600.0);
        if win_h - (top + h) >= 120.0 {
            let _ = s.set_property("top", &format!("{}px", top + h));
        } else {
            let _ = s.set_property("bottom", &format!("{}px", win_h - top));
        }

        // ── option rows ───────────────────────────────
        let highlight = Rc::new(Cell::new(cur));
        let mut opt_els: Vec<web_sys::HtmlElement> = Vec::with_capacity(n);

        for (i, opt) in options.iter().enumerate() {
            let el: web_sys::HtmlElement = doc
                .create_element("div")
                .expect("div")
                .dyn_into()
                .expect("cast");
            let _ = el.set_attribute("data-idx", &i.to_string());

            let rs = el.style();
            let _ = rs.set_property("display", "flex");
            let _ = rs.set_property("align-items", "center");
            let _ = rs.set_property("padding", "4px 8px");
            let _ = rs.set_property("cursor", "pointer");
            let _ = rs.set_property("white-space", "nowrap");

            if i == cur {
                let _ = rs.set_property("background", &sel_c);
            }

            // Optional icon (e.g. flag SVG)
            if let Some(ref url) = opt.icon {
                let img: HtmlImageElement = doc
                    .create_element("img")
                    .expect("img")
                    .dyn_into()
                    .expect("cast");
                img.set_src(url);
                let is = img.style();
                let _ = is.set_property("width", "20px");
                let _ = is.set_property("height", "15px");
                let _ = is.set_property("border-radius", "2px");
                let _ = is.set_property("margin-right", "6px");
                let _ = is.set_property("flex-shrink", "0");
                let _ = el.append_child(&img);
            }

            // Label
            let span = doc.create_element("span").expect("span");
            span.set_text_content(Some(&opt.label));
            let _ = el.append_child(&span);

            let _ = ctr.append_child(&el);
            opt_els.push(el);
        }

        // Append and focus
        doc.body()
            .expect("body")
            .append_child(&ctr)
            .expect("append");
        let _ = ctr.focus();

        // Scroll selected into view
        dd_scroll_into_view(&ctr, &opt_els, cur);

        // ── shared state for closures ─────────────────
        let opts_rc = Rc::new(opt_els);
        let vals: Vec<String> =
            options.iter().map(|o| o.value.clone()).collect();
        let vals_rc = Rc::new(vals);
        let labels: Vec<String> =
            options.iter().map(|o| o.label.clone()).collect();
        let labels_rc = Rc::new(labels);
        let sel_css = Rc::new(sel_c);
        let ctr_rc: Rc<web_sys::HtmlElement> = Rc::new(ctr.clone());

        // ── mousedown → commit ────────────────────────
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key.to_owned();
            let vals = Rc::clone(&vals_rc);
            let cb = Closure::<dyn FnMut(_)>::new(
                move |evt: web_sys::MouseEvent| {
                    evt.prevent_default();
                    let idx = dd_idx_from_event(&evt);
                    let Some(idx) = idx else {
                        return;
                    };
                    if let Some(val) = vals.get(idx) {
                        gc.dispatch(GridCommand::CommitEdit {
                            row: r,
                            col_key: ck.clone(),
                            value: val.clone(),
                        });
                        gc.remove_edit_input();
                    }
                },
            );
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            ctr.add_event_listener_with_callback("mousedown", &func)
                .expect("mousedown");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("mousedown".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        // ── mouseover → highlight ─────────────────────
        {
            let hl = Rc::clone(&highlight);
            let opts = Rc::clone(&opts_rc);
            let sc = Rc::clone(&sel_css);
            let cb = Closure::<dyn FnMut(_)>::new(
                move |evt: web_sys::MouseEvent| {
                    let Some(idx) = dd_idx_from_event(&evt) else {
                        return;
                    };
                    let old = hl.get();
                    if old != idx {
                        dd_set_highlight(&opts, old, idx, &sc);
                        hl.set(idx);
                    }
                },
            );
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            ctr.add_event_listener_with_callback("mouseover", &func)
                .expect("mouseover");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("mouseover".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        // ── keydown → navigate / commit / cancel ──────
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key.to_owned();
            let hl = Rc::clone(&highlight);
            let opts = Rc::clone(&opts_rc);
            let vals = Rc::clone(&vals_rc);
            let lbls = Rc::clone(&labels_rc);
            let sc = Rc::clone(&sel_css);
            let c = Rc::clone(&ctr_rc);
            let count = n;
            let cb = Closure::<dyn FnMut(_)>::new(move |evt: KeyboardEvent| {
                match evt.key().as_str() {
                    "ArrowDown" => {
                        evt.prevent_default();
                        let old = hl.get();
                        let nw = if old + 1 < count { old + 1 } else { 0 };
                        dd_set_highlight(&opts, old, nw, &sc);
                        hl.set(nw);
                        dd_scroll_into_view(&c, &opts, nw);
                    }
                    "ArrowUp" => {
                        evt.prevent_default();
                        let old = hl.get();
                        let nw = if old > 0 {
                            old - 1
                        } else {
                            count.saturating_sub(1)
                        };
                        dd_set_highlight(&opts, old, nw, &sc);
                        hl.set(nw);
                        dd_scroll_into_view(&c, &opts, nw);
                    }
                    "Enter" => {
                        let idx = hl.get();
                        if let Some(v) = vals.get(idx) {
                            gc.dispatch(GridCommand::CommitEdit {
                                row: r,
                                col_key: ck.clone(),
                                value: v.clone(),
                            });
                            gc.remove_edit_input();
                        }
                    }
                    "Escape" => {
                        gc.dispatch(GridCommand::CancelEdit);
                        gc.remove_edit_input();
                    }
                    key if key.len() == 1 => {
                        // Type-ahead search
                        let ch = key.to_lowercase();
                        let cur_i = hl.get();
                        let found = lbls
                            .iter()
                            .enumerate()
                            .skip(cur_i + 1)
                            .chain(lbls.iter().enumerate().take(cur_i + 1))
                            .find(|(_, l)| l.to_lowercase().starts_with(&ch))
                            .map(|(i, _)| i);
                        if let Some(nw) = found {
                            dd_set_highlight(&opts, cur_i, nw, &sc);
                            hl.set(nw);
                            dd_scroll_into_view(&c, &opts, nw);
                        }
                    }
                    _ => {}
                }
            });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            ctr.add_event_listener_with_callback("keydown", &func)
                .expect("keydown");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("keydown".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        // ── blur → cancel ─────────────────────────────
        {
            let gc = self.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(move |_: web_sys::FocusEvent| {
                    if gc.0.state.borrow().edit.is_some() {
                        gc.dispatch(GridCommand::CancelEdit);
                        gc.remove_edit_input();
                    }
                });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            ctr.add_event_listener_with_callback("blur", &func)
                .expect("blur");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("blur".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        *self.0.edit_input.borrow_mut() = Some(ctr);
    }

    /// Show a text `<input>` editor (default).
    fn show_text_editor(
        &self,
        row: u64,
        col_key: &str,
        col_idx: usize,
        geom: EditorGeom,
        raw_value: &str,
    ) {
        let EditorGeom {
            left,
            top,
            width: w,
            height: h,
        } = geom;
        let doc = document();
        let input: HtmlInputElement = doc
            .create_element("input")
            .expect("create input")
            .dyn_into()
            .expect("cast");

        // For ImageText cells the raw value is
        // "{data_uri} {label}". Show only the label
        // in the input and restore the prefix on
        // commit.
        let (initial, img_prefix) = {
            let state = self.0.state.borrow();
            let is_img_text = state
                .model
                .columns
                .get(col_idx)
                .and_then(|c| c.format.as_ref())
                .map(|f| f.is_image_text())
                .unwrap_or(false);
            if is_img_text {
                if let Some(i) = raw_value.find(' ') {
                    let prefix = raw_value[..=i].to_owned();
                    let label = raw_value[i + 1..].to_owned();
                    (label, prefix)
                } else {
                    (raw_value.to_owned(), String::new())
                }
            } else {
                (raw_value.to_owned(), String::new())
            }
        };
        input.set_value(&initial);

        self.apply_edit_style(
            input.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
            left,
            top,
            w,
            h,
        );

        doc.body()
            .expect("body")
            .append_child(&input)
            .expect("append");
        let _ = input.focus();
        input.select();

        let col_key_owned = col_key.to_owned();

        // Enter → commit, Escape → cancel
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key_owned.clone();
            let inp = input.clone();
            let pfx: String = img_prefix.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" => {
                            let val = format!("{}{}", pfx, inp.value());
                            gc.dispatch(GridCommand::CommitEdit {
                                row: r,
                                col_key: ck.clone(),
                                value: val,
                            });
                            gc.remove_edit_input();
                        }
                        "Escape" => {
                            gc.dispatch(GridCommand::CancelEdit);
                            gc.remove_edit_input();
                        }
                        _ => {}
                    },
                );
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("keydown", &func)
                .expect("keydown");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("keydown".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        // Blur → commit
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key_owned;
            let inp = input.clone();
            let pfx = img_prefix;
            let cb =
                Closure::<dyn FnMut(_)>::new(move |_: web_sys::FocusEvent| {
                    if gc.0.state.borrow().edit.is_some() {
                        let val = format!("{}{}", pfx, inp.value());
                        gc.dispatch(GridCommand::CommitEdit {
                            row: r,
                            col_key: ck.clone(),
                            value: val,
                        });
                        gc.remove_edit_input();
                    }
                });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("blur", &func)
                .expect("blur");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("blur".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        *self.0.edit_input.borrow_mut() =
            Some(input.unchecked_into::<web_sys::HtmlElement>());
    }

    /// Remove the inline edit overlay from the DOM and drop its closures.
    ///
    /// Explicitly calls `removeEventListener` before removal to avoid
    /// dangling Rust closure references on the JS side.
    /// Cancel the current edit and remove the overlay.
    pub(super) fn cancel_and_close_edit(&self) {
        if self.0.state.borrow().edit.is_some() {
            self.dispatch(GridCommand::CancelEdit);
        }
        self.remove_edit_input();
    }

    pub(super) fn remove_edit_input(&self) {
        if let Some(el) = self.0.edit_input.borrow().as_ref() {
            for (event, func) in self.0.edit_listener_refs.borrow().iter() {
                let _ = el.remove_event_listener_with_callback(event, func);
            }
        }
        self.0.edit_listener_refs.borrow_mut().clear();
        if let Some(el) = self.0.edit_input.borrow_mut().take() {
            el.remove();
        }
        self.0.edit_closures.borrow_mut().clear();
    }
}

// ── dropdown helpers ──────────────────────────────────

/// Extract the `data-idx` of the closest option row
/// from a mouse event target.
fn dd_idx_from_event(evt: &web_sys::MouseEvent) -> Option<usize> {
    let target = evt.target()?;
    let el: web_sys::Element = target.dyn_into().ok()?;
    let row = el.closest("[data-idx]").ok()??;
    row.get_attribute("data-idx")?.parse::<usize>().ok()
}

/// Update the highlight background on two option rows.
fn dd_set_highlight(
    opts: &[web_sys::HtmlElement],
    old: usize,
    new: usize,
    sel_css: &str,
) {
    if let Some(el) = opts.get(old) {
        let _ = el.style().remove_property("background");
    }
    if let Some(el) = opts.get(new) {
        let _ = el.style().set_property("background", sel_css);
    }
}

/// Scroll an option row into the visible area of the
/// dropdown container.
fn dd_scroll_into_view(
    container: &web_sys::HtmlElement,
    opts: &[web_sys::HtmlElement],
    idx: usize,
) {
    if let Some(el) = opts.get(idx) {
        let et = el.offset_top();
        let eh = el.offset_height();
        let st = container.scroll_top();
        let vh = container.client_height();
        if et < st {
            container.set_scroll_top(et);
        } else if et + eh > st + vh {
            container.set_scroll_top(et + eh - vh);
        }
    }
}
