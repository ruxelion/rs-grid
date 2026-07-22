//! Quick filter-row input — a transient DOM `<input>` overlay opened by
//! clicking a cell in the floating filter row (`GridModel.show_filter_row`,
//! `builder.rs`'s filter-row rendering block draws the *closed* look; this
//! module is the *open* state). Mirrors `edit.rs`'s inline cell editor
//! lifecycle exactly — created on demand, reuses the shared
//! `edit_input`/`edit_closures`/`edit_listener_refs` bookkeeping and
//! `remove_edit_input()` teardown, so only one transient overlay (cell
//! editor or quick filter input) can ever be open at a time. This is
//! deliberately *not* a persistent DOM row: see `rs-grid-web/AGENTS.md`'s
//! "Floating filter row" section for why.

use rs_grid_core::{commands::GridCommand, filter::FilterCondition};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use super::{
    GridCanvas,
    context_menu::read_ctx_colors,
    dom_helpers::{document, make_el},
    filter_popup::{style_daisy_control, wire_daisy_focus_ring},
};

impl GridCanvas {
    /// Opens a transient `<input>` over the floating filter row's cell
    /// for `col_idx`. Prefills with the column's current `filters` value
    /// (raw, regardless of stored operator — the same best-effort
    /// AG-Grid-style simplification the closed canvas-drawn cell already
    /// makes). Commit (`Enter` or blur) dispatches `SetColumnFilter`
    /// with `FilterOp::Contains` — even an emptied value, which clears
    /// the filter via `FilterCondition::is_empty()`'s existing semantics.
    /// `Escape` cancels without dispatching.
    pub(super) fn show_quick_filter_input(&self, col_idx: usize) {
        self.remove_edit_input();

        let (col_key, current_value, left, top, width, height) = {
            let state = self.0.state.borrow();
            let model = &state.model;
            let Some(col) = model.columns.get(col_idx) else {
                return;
            };
            let sx = state.viewport.scroll_x;
            let rnw = model.effective_row_number_width();
            let ccw = model.effective_checkbox_column_width();
            let Some(base) = model.column_screen_x(col_idx, sx) else {
                return;
            };
            let cx = if col_idx < model.pinned_count {
                base + rnw
            } else {
                base + rnw + ccw
            };
            let hh = model.effective_header_height();
            let fh = model.effective_filter_row_height();
            // Match the closed, canvas-drawn "input look" box exactly
            // (`builder.rs`'s filter-row rendering block) rather than
            // spanning the whole cell — otherwise the overlay jumps to
            // fill the full cell (over the margins and the funnel
            // icon's own reserved zone) the moment it opens, instead of
            // sitting flush over the box that was just clicked.
            let theme = self.0.builder.borrow().theme.clone();
            let icon_mr = theme.header_filter_icon_margin_r;
            let icon_bw = theme.header_filter_icon_btn_w;
            let icon_zone = icon_mr + icon_bw;
            // Same as `builder.rs`'s filter-row box: horizontal inset
            // reuses `cell_padding` so this overlay lines up with the
            // data cells' own left/right content edges below.
            let inset = theme.cell_padding;
            let margin_v = theme.filter_row_input_margin_v;
            let box_x = cx + inset;
            let box_w = (col.width - inset * 2.0 - icon_zone).max(0.0);
            let box_y = hh + margin_v;
            let box_h = (fh - margin_v * 2.0).max(0.0);
            let rect = self.0.canvas.get_bounding_client_rect();
            let current_value = model
                .filters
                .get(&col.key)
                .map(|c| c.value.clone())
                .unwrap_or_default();
            (
                col.key.clone(),
                current_value,
                rect.left() + box_x,
                rect.top() + box_y,
                box_w,
                box_h,
            )
        };

        let doc = document();
        let colors = read_ctx_colors();
        let input: HtmlInputElement =
            make_el(&doc, "input").dyn_into().expect("input element");
        input.set_value(&current_value);
        // `style_daisy_control` first — it sets its own fixed `height:
        // 40px` (correct for the popup's inputs, which are always
        // 40px), so the geometry below must be applied *after* it to
        // win: this overlay's height comes from the filter row's own
        // themed geometry (`filter_row_height`/`filter_row_input_margin_v`),
        // not a fixed 40px, and previously got silently clobbered back
        // to 40px by running before this call.
        style_daisy_control(input.unchecked_ref(), &colors);
        wire_daisy_focus_ring(input.unchecked_ref(), &colors);
        {
            let style = input.style();
            let _ = style.set_property("position", "fixed");
            let _ = style.set_property("left", &format!("{left}px"));
            let _ = style.set_property("top", &format!("{top}px"));
            let _ = style.set_property("width", &format!("{width}px"));
            let _ = style.set_property("height", &format!("{height}px"));
            let _ = style.set_property("z-index", "10000");
            let _ = style.set_property("box-sizing", "border-box");
        }

        doc.body()
            .expect("no body")
            .append_child(&input)
            .expect("append quick filter input");
        let _ = input.focus();
        input.select();

        // Commit on blur.
        {
            let gc = self.clone();
            let ck = col_key.clone();
            let inp = input.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                gc.dispatch(GridCommand::SetColumnFilter {
                    col_key: ck.clone(),
                    condition: FilterCondition::contains(inp.value()),
                });
                gc.remove_edit_input();
            });
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("blur", &func)
                .expect("add blur listener");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("blur".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }
        // Enter commits, Escape cancels without applying.
        {
            let gc = self.clone();
            let ck = col_key.clone();
            let inp = input.clone();
            let cb =
                Closure::<dyn FnMut(_)>::new(
                    move |evt: KeyboardEvent| match evt.key().as_str() {
                        "Enter" => {
                            gc.dispatch(GridCommand::SetColumnFilter {
                                col_key: ck.clone(),
                                condition: FilterCondition::contains(
                                    inp.value(),
                                ),
                            });
                            gc.remove_edit_input();
                        }
                        "Escape" => {
                            gc.remove_edit_input();
                        }
                        _ => {}
                    },
                );
            let func: js_sys::Function =
                cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
            input
                .add_event_listener_with_callback("keydown", &func)
                .expect("add keydown listener");
            self.0
                .edit_listener_refs
                .borrow_mut()
                .push(("keydown".into(), func));
            self.0.edit_closures.borrow_mut().push(Box::new(cb));
        }

        *self.0.edit_input.borrow_mut() =
            Some(input.dyn_into::<web_sys::HtmlElement>().expect("cast"));
    }
}
