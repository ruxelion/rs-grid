use std::{cell::Cell, rc::Rc};

use rs_grid_core::{
    column::{CellEditor, SelectOption},
    commands::GridCommand,
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlImageElement, HtmlTextAreaElement, KeyboardEvent};

use super::{GridCanvas, dom_helpers::document};
use crate::css_theme;

/// Extra width reserved beyond the measured text so the caret/cursor
/// has room and text doesn't touch the box edge.
const TEXT_PADDING: f64 = 24.0;
/// Single-line values grow the editor box up to this width before
/// `show_text_editor` switches to a wrapping `<textarea>` instead.
const MAX_EDITOR_WIDTH: f64 = 520.0;
/// Left/right padding applied to both editors — wider than the themed
/// `--rs-grid-editor-padding` default (4px), which reads as cramped
/// once text wraps onto multiple lines. Kept identical between
/// `show_single_line_editor` and `show_multiline_editor` so switching
/// between them (see `show_text_editor`) doesn't shift the text's
/// position within the box. Also fed into `measure_wrapped_height` so
/// wrap points match what's actually rendered.
const EDITOR_H_PADDING: f64 = 8.0;
/// Top/bottom padding of both editors — for `show_single_line_editor`
/// it's fixed (the `<input>`'s native rendering centers its one line
/// regardless); for `show_multiline_editor` it's a floor under the
/// centering math, so it never collapses to ~0 when the box height
/// ends up matching the content almost exactly (which would read as
/// text jammed against the border). Kept identical between the two so
/// switching between them doesn't shift the text's position.
const EDITOR_V_PADDING: f64 = 8.0;

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
        let rnw = model.effective_row_number_width();
        let ccw = model.effective_checkbox_column_width();
        let cx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - state.viewport.scroll_x + rnw + ccw
        };
        let cy = model.row_top(row) - state.viewport.scroll_y;
        let w = model.columns[col_idx].width;
        let h = model.row_height;
        (cx, cy, w, h)
    }

    /// Client-space rectangle `(left, top, width, height)` of a cell, in
    /// CSS pixels relative to the page — ready to position a `position:
    /// fixed` element (e.g. a custom validation tooltip) above, below, or
    /// beside the cell. `None` if `col_key` is unknown. Geometry only —
    /// does not check whether the cell is currently scrolled into view.
    ///
    /// rs-grid does not pick where validation feedback renders — pair
    /// this with [`GridCanvas::validation_error`] (or the `(row,
    /// col_key, ..)` delivered by
    /// [`GridCanvas::set_on_validation_state_changed`]) to place your own
    /// tooltip/banner exactly where you want relative to the failing
    /// cell:
    /// ```ignore
    /// if let Some((row, col_key, _)) = canvas.validation_error()
    ///     && let Some((left, top, _w, h)) =
    ///         canvas.cell_client_rect(row, &col_key)
    /// {
    ///     // Below the cell:
    ///     position_tooltip(left, top + h + 4.0);
    ///     // Above the cell:
    ///     // position_tooltip(left, top - tooltip_height - 4.0);
    /// }
    /// ```
    pub fn cell_client_rect(
        &self,
        row: u64,
        col_key: &str,
    ) -> Option<(f64, f64, f64, f64)> {
        let col_idx = self
            .0
            .state
            .borrow()
            .model
            .columns
            .iter()
            .position(|c| c.key == col_key)?;
        let (cx, cy, w, h) = self.cell_viewport_rect(row, col_idx);
        let canvas_rect = self.0.canvas.get_bounding_client_rect();
        Some((canvas_rect.left() + cx, canvas_rect.top() + cy, w, h))
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
        let color = var("--rs-grid-editor-color", "#000000");
        let padding = var("--rs-grid-editor-padding", "0 4px");
        // Falls back to the theme's own font_size (not "inherit") so
        // the rendered font-size matches, by default, what
        // measure_text_width/measure_wrapped_height assume — those
        // always measure at theme.font_size, so a mismatch here would
        // silently throw off the multiline `<textarea>`'s computed
        // width/height (manifests as unwanted scrolling or excess
        // whitespace). Overriding the CSS var away from theme.font_size
        // reintroduces that mismatch — same caveat as any other
        // editor-overlay var that isn't part of the Theme round-trip.
        let theme_font_size = self.0.builder.borrow().theme.font_size;
        let font_size = var(
            "--rs-grid-editor-font-size",
            &format!("{theme_font_size}px"),
        );
        let shadow = var("--rs-grid-editor-shadow", "none");

        // Parse border width to offset the editor so the border
        // wraps around the cell rather than clipping inside it.
        let bw: f64 =
            border_width.trim_end_matches("px").parse().unwrap_or(2.0);

        // `w` may already be wider than the cell (see `show_text_editor`'s
        // content-based sizing) — keep it on screen by shifting left
        // instead of letting it run past the window's right edge.
        let win_w = web_sys::window()
            .and_then(|win| win.inner_width().ok())
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::MAX);
        let left = left.min((win_w - w - bw).max(0.0));

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
        // Must match the font used by `measure_text_width` and
        // `measure_wrapped_height` (and the canvas renderer's own
        // `draw_text`) — "inherit" would pick up whatever font the
        // host page happens to cascade, silently desyncing the
        // editor's actual line-wrapping/height from what was measured
        // for it (manifests as a `<textarea>` that scrolls even though
        // its content should fit within the computed height).
        let _ = style.set_property("font-family", "system-ui, sans-serif");
        let _ = style.set_property("background", &bg);
        let _ = style.set_property("color", &color);
        let _ = style.set_property("box-shadow", &shadow);
    }

    /// Swap the edit overlay's border/background between the normal
    /// and invalid-value look, without touching position/geometry, and
    /// sync the native `title` attribute (unless disabled via
    /// [`GridCanvas::set_native_validation_tooltip`]). Cheap enough to
    /// call on every keystroke.
    fn apply_edit_validity_style(
        &self,
        el: &web_sys::HtmlElement,
        message: Option<&str>,
    ) {
        let invalid = message.is_some();
        let css_style = css_theme::root_computed_style();
        let var = |name: &str, fb: &str| -> String {
            css_style
                .as_ref()
                .map(|s| css_theme::get_var(s, name))
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| fb.to_string())
        };
        let border_width = var("--rs-grid-editor-border-width", "2px");
        let (border_color, bg) = if invalid {
            (
                var("--rs-grid-editor-border-invalid", "#dc2626"),
                var("--rs-grid-editor-bg-invalid", "#fef2f2"),
            )
        } else {
            (
                var("--rs-grid-editor-border", "#2563eb"),
                var("--rs-grid-editor-bg", "#ffffff"),
            )
        };
        let style = el.style();
        let _ = style.set_property(
            "border",
            &format!("{border_width} solid {border_color}"),
        );
        let _ = style.set_property("background", &bg);

        if self.0.native_validation_tooltip.get() {
            match message {
                Some(msg) => {
                    let _ = el.set_attribute("title", msg);
                }
                None => {
                    let _ = el.remove_attribute("title");
                }
            }
        }
    }

    /// Pixel width of `text` rendered at the grid's current font size,
    /// via an offscreen canvas — mirrors the font string used by
    /// `rs-grid-render-canvas`'s `draw_text` so the measurement matches
    /// what's actually on screen.
    fn measure_text_width(&self, text: &str) -> f64 {
        let font_size = self.0.builder.borrow().theme.font_size;
        document()
            .create_element("canvas")
            .ok()
            .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .and_then(|c| c.get_context("2d").ok().flatten())
            .and_then(|ctx| {
                ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().ok()
            })
            .and_then(|ctx| {
                ctx.set_font(&format!(
                    "400 {}px system-ui, sans-serif",
                    font_size.round() as u32
                ));
                ctx.measure_text(text).ok()
            })
            .map(|m| m.width())
            .unwrap_or(0.0)
    }

    /// Height `text` would wrap to at the given pixel `width`, measured
    /// via a throwaway offscreen `<textarea>` (kept out of the a11y tree
    /// and never focusable) styled to match the real editor's font/
    /// padding — the browser's own line-wrapping is the only reliable
    /// way to get this without reimplementing text shaping.
    fn measure_wrapped_height(&self, text: &str, width: f64) -> f64 {
        let font_size = self.0.builder.borrow().theme.font_size;
        let doc = document();
        let Some(el) = doc
            .create_element("textarea")
            .ok()
            .and_then(|e| e.dyn_into::<HtmlTextAreaElement>().ok())
        else {
            return 0.0;
        };
        let style = el.style();
        let _ = style.set_property("position", "fixed");
        let _ = style.set_property("visibility", "hidden");
        let _ = style.set_property("left", "-9999px");
        let _ = style.set_property("width", &format!("{width}px"));
        let _ = style.set_property("font-size", &format!("{font_size}px"));
        let _ = style.set_property("font-family", "system-ui, sans-serif");
        let _ =
            style.set_property("padding", &format!("0 {EDITOR_H_PADDING}px"));
        let _ = style.set_property("box-sizing", "border-box");
        let _ = style.set_property("white-space", "pre-wrap");
        let _ = style.set_property("word-break", "break-word");
        let _ = style.set_property("border", "none");
        // A bare <textarea> defaults to rows="2" — without this,
        // scrollHeight would floor out at two lines even for
        // single-line text, undercounting how much shorter the real
        // content is (and starving the caller's vertical-centering
        // math of the padding it needs).
        el.set_rows(1);
        el.set_value(text);
        let Some(body) = doc.body() else {
            return 0.0;
        };
        let _ = body.append_child(&el);
        let h = el.scroll_height() as f64;
        el.remove();
        h
    }

    /// Create the appropriate DOM overlay for inline
    /// cell editing (text `<input>` or custom dropdown).
    pub(super) fn show_edit_input(&self) {
        self.remove_edit_input();
        self.0.edit_closures.borrow_mut().clear();

        let (row, col_key, col_idx) = {
            let state = self.0.state.borrow();
            let edit = match &state.edit {
                Some(e) => e,
                None => return,
            };
            (edit.row, edit.col_key.clone(), edit.col_idx)
        };
        if col_idx >= self.0.state.borrow().model.columns.len() {
            return;
        }

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
            Some(_) => {
                self.show_text_editor(row, &col_key, col_idx, geom, &raw_value);
            }
            None => {
                // No editor configured for this column: cancel the edit state
                // opened by StartEdit and show no DOM overlay. This prevents a
                // spurious text <input> when column.editor is None even though
                // the column is technically editable (model.editable=true,
                // column.editable=true). Users who want plain-text editing must
                // set column.editor = Some(CellEditor::Text).
                self.dispatch(GridCommand::CancelEdit);
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
        // Pixel-matched to daisyUI's `::picker(select)` + `option` look
        // (no Tailwind/daisyUI dependency in this crate, so the shape is
        // replicated rather than assumed available via a `class`) —
        // border = its `base-200` stand-in (`Theme::grid_line`, the
        // grid's own subtle divider color), highlight = its
        // `bg-base-content/10` hover token, derived from the themed
        // `cell_text` color via `color-mix` rather than a new hardcoded
        // color so it still tracks a theme swap.
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
            let shadow = var(
                "--rs-grid-overlay-shadow",
                "0 20px 25px -5px rgba(0,0,0,.1), \
                 0 8px 10px -6px rgba(0,0,0,.1)",
            );
            let dd_min_w: f64 = var("--rs-grid-dropdown-min-width", "220")
                .trim_end_matches("px")
                .parse()
                .unwrap_or(220.0);
            let dd_max_h = var("--rs-grid-dropdown-max-height", "240px");
            let text_c = t.cell_text.to_css();
            let sel_c =
                format!("color-mix(in oklab, {text_c} 10%, transparent)");
            (
                t.bg.to_css(),
                text_c,
                sel_c,
                t.grid_line.to_css(),
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
        let _ = s.set_property("border", &format!("1px solid {border_c}"));
        // daisyUI `rounded-box` (`--radius-box: 0.5rem`).
        let _ = s.set_property("border-radius", "8px");
        let _ = s.set_property("background", &bg);
        let _ = s.set_property("color", &text_c);
        let _ = s.set_property("font-size", &format!("{fsz}px"));
        let _ = s.set_property("font-family", "inherit");
        let _ = s.set_property("box-shadow", &shadow);
        let _ = s.set_property("outline", "none");
        let _ = s.set_property("box-sizing", "border-box");
        let _ = s.set_property("margin", "0");
        // daisyUI `p-2`.
        let _ = s.set_property("padding", "8px");

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
            // daisyUI `option`: `py-1.5` (6px) + `--option-px: 3` (12px).
            let _ = rs.set_property("padding", "6px 12px");
            // daisyUI `rounded-field` (`--radius-field: 0.25rem`).
            let _ = rs.set_property("border-radius", "4px");
            let _ = rs
                .set_property("transition", "color .2s, background-color .2s");
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

    /// Show a text editor (default) — a classic single-line `<input>`
    /// when the value fits on one line without wrapping, or a
    /// `<textarea>` when it doesn't (needs wrapping) or already
    /// contains a manual line break. Decided once, from the value the
    /// cell already holds when the edit opens — not re-decided while
    /// typing, so a value that grows past the single-line cap mid-edit
    /// keeps scrolling horizontally in its `<input>` rather than
    /// morphing into a `<textarea>` under the user's cursor.
    fn show_text_editor(
        &self,
        row: u64,
        col_key: &str,
        col_idx: usize,
        geom: EditorGeom,
        raw_value: &str,
    ) {
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

        let fits_one_line = !initial.contains('\n')
            && !initial.contains('\r')
            && self.measure_text_width(&initial) + TEXT_PADDING
                <= MAX_EDITOR_WIDTH;

        if fits_one_line {
            self.show_single_line_editor(
                row,
                col_key,
                geom,
                &initial,
                &img_prefix,
            );
        } else {
            self.show_multiline_editor(
                row,
                col_key,
                geom,
                &initial,
                &img_prefix,
            );
        }
    }

    /// Classic single-line `<input>` editor — used when the cell's
    /// value fits without wrapping (see `show_text_editor`). Only
    /// grows the box up to `MAX_EDITOR_WIDTH`; height stays the cell's
    /// own, vertically centered by the input's native rendering.
    fn show_single_line_editor(
        &self,
        row: u64,
        col_key: &str,
        geom: EditorGeom,
        initial: &str,
        img_prefix: &str,
    ) {
        let EditorGeom {
            left,
            top,
            width: w,
            height: h,
        } = geom;
        let doc = document();
        let input: web_sys::HtmlInputElement = doc
            .create_element("input")
            .expect("create input")
            .dyn_into()
            .expect("cast");
        // Explicit type="text" so CSS selectors like input[type="text"]
        // match (without it the attribute is absent, only the IDL default).
        let _ = input.set_attribute("type", "text");
        input.set_value(initial);

        let w = w.max(self.measure_text_width(initial) + TEXT_PADDING);
        self.apply_edit_style(
            input.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
            left,
            top,
            w,
            h,
        );
        // Same padding as `show_multiline_editor`'s <textarea> (see
        // EDITOR_H_PADDING/EDITOR_V_PADDING) — overrides the themed
        // `--rs-grid-editor-padding` default so text sits at the same
        // position in the box whichever editor `show_text_editor`
        // picked. The input's native rendering centers its one line
        // regardless of the exact top/bottom value.
        let style = input.style();
        let _ =
            style.set_property("padding-top", &format!("{EDITOR_V_PADDING}px"));
        let _ = style
            .set_property("padding-bottom", &format!("{EDITOR_V_PADDING}px"));
        let _ = style
            .set_property("padding-left", &format!("{EDITOR_H_PADDING}px"));
        let _ = style
            .set_property("padding-right", &format!("{EDITOR_H_PADDING}px"));

        doc.body()
            .expect("body")
            .append_child(&input)
            .expect("append");
        let _ = input.focus();
        input.select();

        let col_key_owned = col_key.to_owned();
        let img_prefix = img_prefix.to_owned();

        // After a CommitEdit attempt: if the edit session is still
        // active (InvalidEditMode::Block kept it open because the
        // value is still invalid), keep the overlay mounted, reflect
        // the error visually, and refocus it. Otherwise (success, or
        // InvalidEditMode::Revert) tear the overlay down as before.
        fn keep_or_close(gc: &GridCanvas, inp: &web_sys::HtmlInputElement) {
            let still_editing = gc.0.state.borrow().edit.is_some();
            if still_editing {
                let message =
                    gc.0.state
                        .borrow()
                        .edit
                        .as_ref()
                        .and_then(|e| e.validation_error.clone());
                gc.apply_edit_validity_style(
                    inp.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
                    message.as_deref(),
                );
                let _ = inp.focus();
            } else {
                gc.remove_edit_input();
            }
        }

        // Alt+Enter → switch to the multiline editor with a single
        // newline inserted at the cursor, Enter (no modifier, or with
        // Shift — Shift+Enter is not a newline shortcut here) →
        // commit, Escape → cancel
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key_owned.clone();
            let inp = input.clone();
            let pfx = img_prefix.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" if evt.alt_key() => {
                            evt.prevent_default();
                            let val = inp.value();
                            // selectionStart is a UTF-16 code-unit offset;
                            // clamp to the nearest UTF-8 char boundary so
                            // slicing below can't panic on non-ASCII text.
                            let mut idx = inp
                                .selection_start()
                                .ok()
                                .flatten()
                                .map(|n| n as usize)
                                .unwrap_or(val.len())
                                .min(val.len());
                            while idx > 0 && !val.is_char_boundary(idx) {
                                idx -= 1;
                            }
                            let new_val =
                                format!("{}\n{}", &val[..idx], &val[idx..]);
                            gc.remove_edit_input();
                            gc.show_multiline_editor(
                                r,
                                &ck,
                                EditorGeom {
                                    left,
                                    top,
                                    width: w,
                                    height: h,
                                },
                                &new_val,
                                &pfx,
                            );
                            if let Some(ta) = gc
                                .0
                                .edit_input
                                .borrow()
                                .as_ref()
                                .and_then(|el| {
                                    el.dyn_ref::<web_sys::HtmlTextAreaElement>()
                                })
                            {
                                let pos = (idx + 1) as u32;
                                let _ = ta.set_selection_range(pos, pos);
                            }
                        }
                        "Enter" => {
                            let val = format!("{}{}", pfx, inp.value());
                            gc.dispatch(GridCommand::CommitEdit {
                                row: r,
                                col_key: ck.clone(),
                                value: val,
                            });
                            keep_or_close(&gc, &inp);
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

        // Input → live validation feedback (no commit).
        {
            let gc = self.clone();
            let inp = input.clone();
            let pfx = img_prefix.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                let val = format!("{}{}", pfx, inp.value());
                gc.dispatch(GridCommand::ValidateEdit { value: val });
                let message =
                    gc.0.state
                        .borrow()
                        .edit
                        .as_ref()
                        .and_then(|e| e.validation_error.clone());
                gc.apply_edit_validity_style(
                    inp.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
                    message.as_deref(),
                );
            });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("input", &func)
                .expect("input");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("input".into(), func));
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
                        keep_or_close(&gc, &inp);
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

    /// Re-measures `ta`'s current value at width `w` and adjusts its
    /// height (and the vertical-centering padding that depends on it)
    /// to fit — called once at `show_multiline_editor`'s setup and
    /// again on every `input` event, so the box grows as `Alt+Enter`
    /// or wrapping adds lines and shrinks back down if they're
    /// removed. Never smaller than `min_h` (the cell's own height) or
    /// larger than `max_h` (the 60%-viewport-height cap).
    fn resize_multiline_editor(
        &self,
        ta: &HtmlTextAreaElement,
        left: f64,
        top: f64,
        w: f64,
        min_h: f64,
        max_h: f64,
    ) {
        // Content-only height (no vertical padding baked in — see
        // measure_wrapped_height). The box needs at least this plus a
        // comfortable padding margin on both sides.
        let wrapped_h = self.measure_wrapped_height(&ta.value(), w);
        let required_h = wrapped_h + 2.0 * EDITOR_V_PADDING;
        let h = min_h.max(required_h).min(max_h);

        self.apply_edit_style(
            ta.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
            left,
            top,
            w,
            h,
        );

        // Unlike a single-line <input>, a <textarea> always aligns its
        // content to the top. Center it vertically instead of leaving
        // it stuck at the top, with at least EDITOR_V_PADDING of
        // breathing room even when the box height already matches the
        // content almost exactly — plain centering alone can collapse
        // to ~0 there, which reads as text jammed against the border.
        // Also widens the horizontal padding beyond the themed
        // `--rs-grid-editor-padding` default (4px), which reads as
        // cramped once text wraps onto multiple lines. All four
        // override the shorthand `padding` `apply_edit_style` just
        // applied above (it resets all four sides every call).
        let style = ta.style();
        let vpad = ((h - wrapped_h) / 2.0).max(EDITOR_V_PADDING);
        let _ = style.set_property("padding-top", &format!("{vpad}px"));
        let _ = style.set_property("padding-bottom", &format!("{vpad}px"));
        let _ = style
            .set_property("padding-left", &format!("{EDITOR_H_PADDING}px"));
        let _ = style
            .set_property("padding-right", &format!("{EDITOR_H_PADDING}px"));
    }

    /// `<textarea>` editor for values that need wrapping (see
    /// `show_text_editor`) — grows width up to `MAX_EDITOR_WIDTH`,
    /// then height to fit the wrapped line count (dynamically, as
    /// lines are added or removed — see `resize_multiline_editor`),
    /// and supports `Alt+Enter` for manual line breaks.
    fn show_multiline_editor(
        &self,
        row: u64,
        col_key: &str,
        geom: EditorGeom,
        initial: &str,
        img_prefix: &str,
    ) {
        let EditorGeom {
            left,
            top,
            width: w,
            height: h,
        } = geom;
        let doc = document();
        let input: HtmlTextAreaElement = doc
            .create_element("textarea")
            .expect("create textarea")
            .dyn_into()
            .expect("cast");
        input.set_value(initial);

        // Width grows up to MAX_EDITOR_WIDTH before text starts wrapping
        // instead of running off arbitrarily wide, and is fixed for the
        // life of this editor — only height re-adjusts live as lines are
        // added/removed (see resize_multiline_editor).
        let w = w.max(
            (self.measure_text_width(initial) + TEXT_PADDING)
                .min(MAX_EDITOR_WIDTH),
        );
        let max_h = web_sys::window()
            .and_then(|win| win.inner_height().ok())
            .and_then(|v| v.as_f64())
            .map(|vh| vh * 0.6)
            .unwrap_or(f64::MAX);
        // `h` (the cell's own height) is the permanent floor — never
        // shrinks below it even if the value is later cleared down to
        // one line.
        let min_h = h;
        self.resize_multiline_editor(&input, left, top, w, min_h, max_h);

        let style = input.style();
        let _ = style.set_property("resize", "none");
        let _ = style.set_property("overflow-y", "auto");
        let _ = style.set_property("white-space", "pre-wrap");
        let _ = style.set_property("word-break", "break-word");

        doc.body()
            .expect("body")
            .append_child(&input)
            .expect("append");
        let _ = input.focus();
        input.select();

        let col_key_owned = col_key.to_owned();
        let img_prefix = img_prefix.to_owned();

        // After a CommitEdit attempt: if the edit session is still
        // active (InvalidEditMode::Block kept it open because the
        // value is still invalid), keep the overlay mounted, reflect
        // the error visually, and refocus it. Otherwise (success, or
        // InvalidEditMode::Revert) tear the overlay down as before.
        fn keep_or_close(gc: &GridCanvas, inp: &HtmlTextAreaElement) {
            let still_editing = gc.0.state.borrow().edit.is_some();
            if still_editing {
                let message =
                    gc.0.state
                        .borrow()
                        .edit
                        .as_ref()
                        .and_then(|e| e.validation_error.clone());
                gc.apply_edit_validity_style(
                    inp.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
                    message.as_deref(),
                );
                let _ = inp.focus();
            } else {
                gc.remove_edit_input();
            }
        }

        // Enter (with or without Shift — Shift+Enter is not a newline
        // shortcut here) → commit, Alt+Enter → insert exactly one
        // newline at the cursor (Excel's convention), Escape → cancel
        {
            let gc = self.clone();
            let r = row;
            let ck = col_key_owned.clone();
            let inp = input.clone();
            let pfx: String = img_prefix.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" if evt.alt_key() => {
                            evt.prevent_default();
                            let val = inp.value();
                            // selectionStart is a UTF-16 code-unit
                            // offset; clamp to the nearest UTF-8 char
                            // boundary so slicing below can't panic on
                            // non-ASCII text.
                            let mut idx = inp
                                .selection_start()
                                .ok()
                                .flatten()
                                .map(|n| n as usize)
                                .unwrap_or(val.len())
                                .min(val.len());
                            while idx > 0 && !val.is_char_boundary(idx) {
                                idx -= 1;
                            }
                            inp.set_value(&format!(
                                "{}\n{}",
                                &val[..idx],
                                &val[idx..]
                            ));
                            let pos = (idx + 1) as u32;
                            let _ = inp.set_selection_range(pos, pos);
                            // set_value() doesn't fire a native
                            // "input" event — dispatch one so the
                            // existing listener below still runs live
                            // validation on the new value.
                            if let Ok(ev) = web_sys::Event::new("input") {
                                let _ = inp.dispatch_event(&ev);
                            }
                        }
                        "Enter" => {
                            evt.prevent_default();
                            let val = format!("{}{}", pfx, inp.value());
                            gc.dispatch(GridCommand::CommitEdit {
                                row: r,
                                col_key: ck.clone(),
                                value: val,
                            });
                            keep_or_close(&gc, &inp);
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

        // Input → live validation feedback (no commit) + grow/shrink
        // the box to fit the current line count.
        {
            let gc = self.clone();
            let inp = input.clone();
            let pfx = img_prefix.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
                let val = format!("{}{}", pfx, inp.value());
                gc.dispatch(GridCommand::ValidateEdit { value: val });
                let message =
                    gc.0.state
                        .borrow()
                        .edit
                        .as_ref()
                        .and_then(|e| e.validation_error.clone());
                gc.apply_edit_validity_style(
                    inp.dyn_ref::<web_sys::HtmlElement>().expect("cast"),
                    message.as_deref(),
                );
                gc.resize_multiline_editor(&inp, left, top, w, min_h, max_h);
            });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("input", &func)
                .expect("input");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("input".into(), func));
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
                        keep_or_close(&gc, &inp);
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
pub(super) fn dd_idx_from_event(evt: &web_sys::MouseEvent) -> Option<usize> {
    let target = evt.target()?;
    let el: web_sys::Element = target.dyn_into().ok()?;
    let row = el.closest("[data-idx]").ok()??;
    row.get_attribute("data-idx")?.parse::<usize>().ok()
}

/// Update the highlight background on two option rows.
pub(super) fn dd_set_highlight(
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
pub(super) fn dd_scroll_into_view(
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
