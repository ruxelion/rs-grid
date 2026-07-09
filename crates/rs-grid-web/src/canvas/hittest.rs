use rs_grid_core::{
    commands::GridCommand,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
};
use rs_grid_scene::builder::ColumnDragHint;
use web_sys::MouseEvent;

use super::{ActiveDrag, GridCanvas};

impl GridCanvas {
    pub(super) fn scrollbar(&self) -> Option<ScrollbarGeom> {
        let s = self.0.state.borrow();
        let track_w = self.0.builder.borrow().theme.scrollbar_width;
        ScrollbarGeom::compute(
            s.viewport.scroll_y,
            s.viewport.width,
            s.viewport.height,
            s.model.header_height,
            s.model.total_height(),
            track_w,
        )
    }

    pub(super) fn hscrollbar(&self) -> Option<HScrollbarGeom> {
        let s = self.0.state.borrow();
        let track_h = self.0.builder.borrow().theme.scrollbar_width;
        let vsb_w = if ScrollbarGeom::compute(
            s.viewport.scroll_y,
            s.viewport.width,
            s.viewport.height,
            s.model.header_height,
            s.model.total_height(),
            track_h,
        )
        .is_some()
        {
            track_h
        } else {
            0.0
        };
        HScrollbarGeom::compute(
            s.viewport.scroll_x,
            s.viewport.width,
            s.viewport.height,
            s.model.row_number_width,
            s.model.total_width(),
            vsb_w,
            track_h,
        )
    }

    pub(super) fn canvas_xy(&self, evt: &MouseEvent) -> (f64, f64) {
        let rect = self.0.canvas.get_bounding_client_rect();
        (
            evt.client_x() as f64 - rect.left(),
            evt.client_y() as f64 - rect.top(),
        )
    }

    /// Returns `Some(col_idx)` when `(vx, vy)` is within `HIT_ZONE` px of a
    /// column separator in the header, enabling the resize cursor / drag.
    pub(super) fn hit_col_resize_separator(
        &self,
        vx: f64,
        vy: f64,
    ) -> Option<usize> {
        const HIT_ZONE: f64 = 4.0;
        let state = self.0.state.borrow();
        let model = &state.model;
        if vy >= model.header_height {
            return None;
        }
        let rnw = model.effective_row_number_width();
        if vx < rnw {
            return None;
        }
        let ccw = model.effective_checkbox_column_width();
        let scroll_x = state.viewport.scroll_x;
        let pinned = model.pinned_count;
        for (i, col) in model.columns.iter().enumerate() {
            let off = model.column_offsets.offsets[i] + col.width;
            let sep_vx = if i < pinned {
                off + rnw
            } else {
                off - scroll_x + rnw + ccw
            };
            if (vx - sep_vx).abs() <= HIT_ZONE {
                return Some(i);
            }
        }
        None
    }

    pub(super) fn set_cursor(&self, cursor: &str) {
        let _ = self.0.canvas.style().set_property("cursor", cursor);
    }

    /// Returns `true` when the data cell under viewport point `(vx, vy)`
    /// resolves to non-editable via `ColumnDef::is_cell_editable` — the
    /// grid-wide `GridModel.editable` toggle, the column's static
    /// `editable=false`, or a false-resolving `editable_predicate`.
    /// `false` (not locked) for any point outside a data cell (header,
    /// gutter, out of bounds) — callers should treat that as "no
    /// override", falling through to the existing cursor logic.
    pub(super) fn hit_locked_cell(&self, vx: f64, vy: f64) -> bool {
        let state = self.0.state.borrow();
        let Some(coord) = state.hit_test(vx, vy) else {
            return false;
        };
        state
            .model
            .columns
            .get(coord.col)
            .is_some_and(|c| !c.is_cell_editable(coord.row, &state.model))
    }

    /// Recomputes and applies the hover cursor for viewport point
    /// `(vx, vy)` — the same precedence used by the mousemove handler
    /// (header menu icon > resize separator > locked cell > default).
    /// Also called after a column drag/resize ends, so the cursor
    /// doesn't stay stale (e.g. showing `default` over a locked cell,
    /// or over a resize separator) until the next `mousemove` fires.
    pub(super) fn refresh_hover_cursor(&self, vx: f64, vy: f64) {
        if self.hit_header_menu_icon(vx, vy).is_some() {
            self.set_cursor("pointer");
        } else if self.hit_col_resize_separator(vx, vy).is_some() {
            self.set_cursor("w-resize");
        } else if self.hit_locked_cell(vx, vy) {
            self.set_cursor("not-allowed");
        } else {
            self.set_cursor("default");
        }
    }

    /// Returns the data row index under viewport point `(vx, vy)`, or `None`
    /// if the point is in the header, gutter, or below the last row.
    pub(super) fn row_at(&self, vx: f64, vy: f64) -> Option<u64> {
        let state = self.0.state.borrow();
        let model = &state.model;
        if vy < model.header_height {
            return None;
        }
        if vx < 0.0 || vx > state.viewport.width {
            return None;
        }
        let abs_y = vy - model.header_height + state.viewport.scroll_y;
        let row = (abs_y / model.row_height) as u64;
        if row < model.display_row_count() {
            Some(row)
        } else {
            None
        }
    }

    /// Compute which column gap the cursor is closest to.
    /// Returns the index to insert *before* (0..=columns.len()).
    pub(super) fn insertion_index(&self, vx: f64) -> usize {
        let state = self.0.state.borrow();
        let model = &state.model;
        let sx = state.viewport.scroll_x;
        let rnw = model.effective_row_number_width();
        let ccw = model.effective_checkbox_column_width();
        let pinned = model.pinned_count;
        let len = model.columns.len();

        let edge_vx = |i: usize| -> f64 {
            if i < len {
                let off = model.column_offsets.offsets[i];
                if i < pinned {
                    off + rnw
                } else {
                    off - sx + rnw + ccw
                }
            } else {
                let last = len - 1;
                let off = model.column_offsets.offsets[last]
                    + model.columns[last].width;
                if last < pinned {
                    off + rnw
                } else {
                    off - sx + rnw + ccw
                }
            }
        };

        let mut best_idx = 0;
        let mut best_dist = f64::MAX;
        for i in 0..=len {
            let d = (vx - edge_vx(i)).abs();
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Minimum column width that keeps all header UI elements
    /// (padding, sort arrows, menu icon) visible.
    ///
    /// Used as a floor during interactive resize so a column
    /// can never be dragged smaller than its own chrome.
    pub(super) fn header_min_col_width(&self) -> f64 {
        let b = self.0.builder.borrow();
        let t = &b.theme;
        let sort_zone = t.sort_arrow_width * 2.0 + t.cell_padding;
        let icon_zone = t.header_menu_icon_btn_w + t.header_menu_icon_margin_r;
        t.cell_padding + sort_zone + icon_zone
    }

    /// Returns `(char_width, header_char_width, cell_padding,
    /// header_right_reserve, btn_char_width, btn_padding_x, btn_gap)`
    /// derived from the current theme — same values used by
    /// double-click auto-fit and the context-menu auto-size actions.
    pub(super) fn autofit_params(&self) -> (f64, f64, f64, f64, f64, f64, f64) {
        let b = self.0.builder.borrow();
        let t = &b.theme;
        let char_width = t.font_size * 0.6;
        let header_char_width = if t.header_font_bold {
            t.header_font_size * 0.65
        } else {
            t.header_font_size * 0.6
        };
        // Space reserved at the right of the header for the
        // sort arrow, menu icon button, and their margins.
        let sort_zone = t.sort_arrow_width * 2.0 + t.cell_padding;
        let icon_zone = t.header_menu_icon_btn_w + t.header_menu_icon_margin_r;
        let header_right_reserve = sort_zone + icon_zone;
        // Same per-character ratio `emit_cell_buttons`
        // (rs-grid-scene/builder/cells.rs) uses for button labels.
        let btn_char_width = t.font_size * 0.65;
        (
            char_width,
            header_char_width,
            t.cell_padding,
            header_right_reserve,
            btn_char_width,
            t.cell_btn_padding_x,
            t.cell_btn_gap,
        )
    }

    /// Auto-fit a single column's width to its content — the same
    /// computation double-clicking its header separator or the
    /// context-menu "auto-size column" action triggers, callable
    /// directly. Useful to size a column (e.g. one with `cell_buttons`)
    /// once at mount, without requiring the user to interact first.
    pub fn auto_fit_column(&self, col_idx: usize) {
        let (
            char_width,
            header_char_width,
            cell_padding,
            header_right_reserve,
            btn_char_width,
            btn_padding_x,
            btn_gap,
        ) = self.autofit_params();
        self.dispatch(GridCommand::AutoFitColumn {
            col_idx,
            char_width,
            header_char_width,
            cell_padding,
            header_right_reserve,
            btn_char_width,
            btn_padding_x,
            btn_gap,
        });
    }

    /// Auto-fit every column's width to its content.
    pub fn auto_fit_all_columns(&self) {
        let (
            char_width,
            header_char_width,
            cell_padding,
            header_right_reserve,
            btn_char_width,
            btn_padding_x,
            btn_gap,
        ) = self.autofit_params();
        self.dispatch(GridCommand::AutoFitAllColumns {
            char_width,
            header_char_width,
            cell_padding,
            header_right_reserve,
            btn_char_width,
            btn_padding_x,
            btn_gap,
        });
    }

    /// Returns `Some(col_idx)` when `(vx, vy)` falls inside the
    /// three-dot menu icon zone at the right edge of a column header.
    pub(super) fn hit_header_menu_icon(
        &self,
        vx: f64,
        vy: f64,
    ) -> Option<usize> {
        let col_idx = self.0.state.borrow().hit_test_col_header(vx, vy)?;
        let theme = self.0.builder.borrow();
        let mr = theme.theme.header_menu_icon_margin_r;
        let bw = theme.theme.header_menu_icon_btn_w;
        let bh_cfg = theme.theme.header_menu_icon_btn_h;
        drop(theme);
        let state = self.0.state.borrow();
        let model = &state.model;
        // Compute button vertical bounds (same formula as builder.rs).
        let btn_h = if bh_cfg > 0.0 {
            bh_cfg
        } else {
            (model.header_height - 12.0).max(8.0)
        };
        let btn_ty = (model.header_height - btn_h) / 2.0;
        // Reject if the pointer is not within the button's height.
        if vy < btn_ty || vy >= btn_ty + btn_h {
            return None;
        }
        let off = model.column_offsets.offsets[col_idx];
        let sx = state.viewport.scroll_x;
        let rnw = model.effective_row_number_width();
        let ccw = model.effective_checkbox_column_width();
        let col_left_vx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - sx + rnw + ccw
        };
        let col_right_vx = col_left_vx + model.columns[col_idx].width;
        if vx >= col_right_vx - mr - bw && vx < col_right_vx - mr {
            Some(col_idx)
        } else {
            None
        }
    }

    /// Returns the bottom-left corner of the menu icon button
    /// for `col_idx` in canvas-local coordinates, suitable for
    /// anchoring the context menu at a fixed position.
    pub(super) fn menu_icon_anchor(&self, col_idx: usize) -> (f64, f64) {
        let theme = self.0.builder.borrow();
        let mr = theme.theme.header_menu_icon_margin_r;
        let bw = theme.theme.header_menu_icon_btn_w;
        let bh_cfg = theme.theme.header_menu_icon_btn_h;
        drop(theme);
        let state = self.0.state.borrow();
        let model = &state.model;
        let btn_h = if bh_cfg > 0.0 {
            bh_cfg
        } else {
            (model.header_height - 12.0).max(8.0)
        };
        let btn_ty = (model.header_height - btn_h) / 2.0;
        let off = model.column_offsets.offsets[col_idx];
        let sx = state.viewport.scroll_x;
        let rnw = model.effective_row_number_width();
        let ccw = model.effective_checkbox_column_width();
        let col_left_vx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - sx + rnw + ccw
        };
        let col_right_vx = col_left_vx + model.columns[col_idx].width;
        let btn_left_vx = col_right_vx - mr - bw;
        (btn_left_vx, btn_ty + btn_h)
    }

    /// Build a `ColumnDragHint` from the current drag state,
    /// or `None` if no column drag is active.
    pub(super) fn column_drag_hint(&self) -> Option<ColumnDragHint> {
        let drag = self.0.drag.borrow();
        match *drag {
            Some(ActiveDrag::ColumnDrag {
                col_idx,
                current_vx,
                current_vy,
            }) => {
                drop(drag);
                let insert = self.insertion_index(current_vx);
                let animated_offsets = self.0.drag_col_offsets.borrow().clone();
                Some(ColumnDragHint {
                    source_col: col_idx,
                    insert_before: insert,
                    cursor_vx: current_vx,
                    cursor_vy: current_vy,
                    animated_offsets,
                })
            }
            _ => None,
        }
    }
}
