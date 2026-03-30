use rs_grid_core::scrollbar::{HScrollbarGeom, ScrollbarGeom};
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
        if vx < model.row_number_width {
            return None;
        }
        let scroll_x = state.viewport.scroll_x;
        let rnw = model.row_number_width;
        let pinned = model.pinned_count;
        for (i, col) in model.columns.iter().enumerate() {
            let off = model.column_offsets.offsets[i] + col.width;
            let sep_vx = if i < pinned {
                off + rnw
            } else {
                off - scroll_x + rnw
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
        let rnw = model.row_number_width;
        let pinned = model.pinned_count;
        let len = model.columns.len();

        let edge_vx = |i: usize| -> f64 {
            if i < len {
                let off = model.column_offsets.offsets[i];
                if i < pinned {
                    off + rnw
                } else {
                    off - sx + rnw
                }
            } else {
                let last = len - 1;
                let off = model.column_offsets.offsets[last]
                    + model.columns[last].width;
                if last < pinned {
                    off + rnw
                } else {
                    off - sx + rnw
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

    /// Returns `(char_width, header_char_width, cell_padding,
    /// header_right_reserve)` derived from the current theme —
    /// same values used by double-click auto-fit.
    pub(super) fn autofit_params(&self) -> (f64, f64, f64, f64) {
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
        let sort_zone =
            t.sort_arrow_width * 2.0 + t.cell_padding;
        let icon_zone =
            t.header_menu_icon_btn_w + t.header_menu_icon_margin_r;
        let header_right_reserve = sort_zone + icon_zone;
        (char_width, header_char_width, t.cell_padding, header_right_reserve)
    }

    /// Minimum column width that keeps the menu icon and sort arrow
    /// fully visible, derived from the current theme values.
    pub(super) fn min_col_width(&self) -> f64 {
        let b = self.0.builder.borrow();
        let t = &b.theme;
        // left padding  +  sort arrow (width + right gap)  +  icon button  +  right margin
        let sort_zone = t.sort_arrow_width * 2.0 + t.cell_padding;
        let icon_zone = t.header_menu_icon_btn_w + t.header_menu_icon_margin_r;
        t.cell_padding + sort_zone + icon_zone
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
        let rnw = model.row_number_width;
        let col_left_vx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - sx + rnw
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
    pub(super) fn menu_icon_anchor(
        &self,
        col_idx: usize,
    ) -> (f64, f64) {
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
        let rnw = model.row_number_width;
        let col_left_vx = if col_idx < model.pinned_count {
            off + rnw
        } else {
            off - sx + rnw
        };
        let col_right_vx =
            col_left_vx + model.columns[col_idx].width;
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
