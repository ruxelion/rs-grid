mod cells;
mod scrollbars;

use std::collections::HashSet;

use rs_grid_core::{sort::SortDir, state::GridState};

use crate::{
    frame::SceneFrame,
    primitives::{
        Color, LinePrimitive, PolygonPrimitive, RectPrimitive, ScenePrimitive,
        TextAlign, TextPrimitive,
    },
    theme::Theme,
};

// ── column drag hint ─────────────────────────────────────────────────────────

/// Transient UI hint for column-drag visual feedback.
///
/// Computed by the web layer, consumed by the scene builder to
/// render a dimmed source header, an insertion line, and a ghost.
pub struct ColumnDragHint {
    /// Index of the column being dragged.
    pub source_col: usize,
    /// The column will be inserted *before* this index.
    /// Equal to `columns.len()` means "insert at end".
    pub insert_before: usize,
    /// Viewport-relative X of the cursor (positions the ghost).
    pub cursor_vx: f64,
    /// Viewport-relative Y of the cursor (positions the ghost).
    pub cursor_vy: f64,
    /// Animated column offsets (`col_idx → cumulative left offset`)
    /// for smooth lerp transitions. When non-empty the builder uses
    /// these directly instead of computing from `insert_before`.
    pub animated_offsets: Vec<f64>,
}

// ── flash hint ───────────────────────────────────────────────────────────────

/// Transient hint for the flash-cells animation.
///
/// Computed by the web layer from elapsed time; consumed by the
/// scene builder to render a fading golden-yellow overlay on
/// selected cells.
pub struct FlashHint {
    /// Normalised intensity: 1.0 = full flash, 0.0 = invisible.
    pub alpha_factor: f64,
}

// ── builder ───────────────────────────────────────────────────────────────────

/// Transforms a `GridState` snapshot into a `SceneFrame`.
///
/// Instantiate once and reuse; all state is read from `GridState` on each
/// `build()` call.
pub struct SceneBuilder {
    /// Device pixel ratio — hardware property, not a theme property.
    pub dpr: f64,
    /// Active visual theme.
    pub theme: Theme,
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self {
            dpr: 1.0,
            theme: Theme::default(),
        }
    }
}

impl SceneBuilder {
    /// Create a builder with the given DPR and the default theme.
    pub fn new(dpr: f64) -> Self {
        Self {
            dpr,
            theme: Theme::default(),
        }
    }

    /// Create a builder with the given DPR and a custom theme.
    pub fn with_theme(dpr: f64, theme: Theme) -> Self {
        Self { dpr, theme }
    }

    /// Build a complete `SceneFrame` from the current
    /// `GridState`, with optional column-drag visual hints.
    pub fn build(
        &self,
        state: &GridState,
        col_drag: Option<&ColumnDragHint>,
        flash: Option<&FlashHint>,
        hovered_menu_col: Option<usize>,
    ) -> SceneFrame {
        let vp = &state.viewport;
        let model = &state.model;
        let sel = &state.selection;
        let t = &self.theme;

        let mut frame = SceneFrame::new(vp.width, vp.height, self.dpr);

        // ── background ───────────────────────────────────────────────────────
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: vp.width,
            height: vp.height,
            fill: t.bg,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
        }));

        let col_widths: Vec<f64> =
            model.columns.iter().map(|c| c.width).collect();
        let pinned_count = model.pinned_count;
        let pinned_width = model.pinned_width();

        // Scrollable columns — exclude pinned ones and account for
        // the narrower scrollable viewport band.
        let (col_start, col_end) = if pinned_count == 0 {
            vp.visible_columns(&model.column_offsets, &col_widths)
        } else {
            vp.visible_scrollable_columns(
                &model.column_offsets,
                &col_widths,
                pinned_count,
                pinned_width,
                model.row_number_width,
            )
        };
        let (row_start, row_end) = vp.visible_rows(
            model.display_row_count(),
            model.row_height,
            model.header_height,
        );

        let sx = vp.scroll_x;
        let sy = vp.scroll_y;
        let rnw = model.row_number_width; // row-number gutter width

        // Compute viewport-y for a row without large absolute
        // values.  row_top(ri) − sy  =  header + ri*rh − sy.
        // When ri and sy are both huge the subtraction loses
        // f64 precision.  Instead compute the offset from
        // row_start whose viewport-y we derive from the small
        // fractional remainder.
        let rh = model.row_height;
        // Guard: a zero row height would cause division by zero below.
        if rh <= 0.0 {
            return frame;
        }
        let hh = model.header_height;
        // sy_content = scroll_y that falls inside the data
        // area (past the header).
        let sy_content = (sy - hh).max(0.0);
        // Use fmod to get the sub-row fractional offset
        // without subtracting two large f64 values.
        // visible_rows computes first_row = floor(sy_content / rh),
        // and row_start = first_row - overscan.
        // frac_first = sy_content mod rh (offset within
        //              the first fully visible row).
        let frac_first = sy_content % rh;
        // row_start may include overscan rows before
        // first_row, so adjust by the overscan distance.
        let first_no_os = (sy_content / rh) as u64;
        let os_rows = first_no_os.saturating_sub(row_start);
        let frac = frac_first + os_rows as f64 * rh;
        // ry of row_start in viewport coords:
        let ry_base = (hh - sy).max(0.0) - frac;

        let row_vy =
            |ri: u64| -> f64 { ry_base + (ri as f64 - row_start as f64) * rh };

        // During a drag, compute preview offsets: columns laid out
        // as-if the drop has already happened, so they render at
        // their target positions while dragging.
        // Prefer pre-computed animated offsets from the hint
        // (smooth lerp); fall back to instant computation.
        let preview_offsets: Option<Vec<f64>> = col_drag.and_then(|hint| {
            if !hint.animated_offsets.is_empty() {
                return Some(hint.animated_offsets.clone());
            }
            let cols = &model.columns;
            let src = hint.source_col;
            if src >= cols.len() {
                return None;
            }
            let dst = if hint.insert_before > src {
                hint.insert_before.saturating_sub(1)
            } else {
                hint.insert_before
            };
            if dst == src {
                return None;
            }
            let mut order: Vec<usize> =
                (0..cols.len()).filter(|&i| i != src).collect();
            let ins = dst.min(order.len());
            order.insert(ins, src);
            let mut offs = vec![0.0_f64; cols.len()];
            let mut cum = 0.0_f64;
            for &ci in &order {
                offs[ci] = cum;
                cum += cols[ci].width;
            }
            Some(offs)
        });

        // Helper: viewport x of the left edge of column `ci`.
        // Uses preview offsets when a drag is active so columns
        // render at their drop-target positions.
        // Pinned columns are not shifted by scroll_x.
        let col_vx = |ci: usize| -> f64 {
            let off = match &preview_offsets {
                Some(po) => po[ci],
                None => model.column_offsets.offsets[ci],
            };
            if ci < pinned_count {
                off + rnw
            } else {
                off - sx + rnw
            }
        };

        // ── search match set ─────────────────────────────────────────────────
        let search_set: HashSet<(u64, usize)> = state
            .search
            .matches
            .iter()
            .map(|c| (c.row, c.col))
            .collect();
        let search_current: Option<(u64, usize)> = state
            .search
            .matches
            .get(state.search.current)
            .map(|c| (c.row, c.col));

        // ── data rows ────────────────────────────────────────────────────────
        for ri in row_start..row_end {
            let ry = row_vy(ri);

            // Skip rows that are fully outside the clip zone (overscan may
            // produce rows above the header).
            if ry + model.row_height < model.header_height || ry > vp.height {
                continue;
            }

            let mid_y = ry + model.row_height * 0.5 + t.font_size * 0.35;

            // Alternating row background (odd rows, behind selection)
            if ri % 2 == 1 {
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: 0.0,
                    y: ry,
                    width: vp.width,
                    height: model.row_height,
                    fill: t.row_alt_bg,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));
            }

            // Hover highlight (above alt-bg, below selection)
            if state.hovered_row == Some(ri) {
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: 0.0,
                    y: ry,
                    width: vp.width,
                    height: model.row_height,
                    fill: t.row_hover_bg,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));
            }

            // During a drag, render all non-pinned columns so
            // that columns shifted to new preview positions are
            // not missed by the normal visible-range culling.
            let drag_end = if preview_offsets.is_some() {
                model.columns.len()
            } else {
                col_end
            };
            for ci in col_start..drag_end {
                let col = &model.columns[ci];
                let cx = col_vx(ci);
                let status = model.cell_status(ri, &col.key);
                cells::emit_cell(
                    &mut frame,
                    col,
                    ri,
                    ci,
                    cx,
                    ry,
                    mid_y,
                    model.row_height,
                    status,
                    sel,
                    &search_set,
                    search_current,
                    t,
                    flash,
                );
            }

            // Horizontal grid line
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: 0.0,
                y1: ry + model.row_height - 0.5,
                x2: vp.width,
                y2: ry + model.row_height - 0.5,
                color: t.grid_line,
                width: 1.0,
            }));
        }

        // ── pinned-column data overlay ────────────────────────────────────────
        // Rendered after scrollable rows so pinned cells appear on top.
        if pinned_count > 0 && pinned_count <= model.columns.len() {
            // Solid background covering the full pinned band.
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: rnw,
                y: model.header_height,
                width: pinned_width,
                height: vp.height - model.header_height,
                fill: t.bg,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));

            for ri in row_start..row_end {
                let ry = row_vy(ri);
                if ry + model.row_height < model.header_height || ry > vp.height
                {
                    continue;
                }
                let mid_y = ry + model.row_height * 0.5 + t.font_size * 0.35;

                if ri % 2 == 1 {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: rnw,
                        y: ry,
                        width: pinned_width,
                        height: model.row_height,
                        fill: t.row_alt_bg,
                        stroke: None,
                        stroke_width: 0.0,
                        corner_radius: 0.0,
                    }));
                }
                if state.hovered_row == Some(ri) {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: rnw,
                        y: ry,
                        width: pinned_width,
                        height: model.row_height,
                        fill: t.row_hover_bg,
                        stroke: None,
                        stroke_width: 0.0,
                        corner_radius: 0.0,
                    }));
                }

                for ci in 0..pinned_count {
                    let col = &model.columns[ci];
                    let cx = col_vx(ci);
                    let status = model.cell_status(ri, &col.key);
                    cells::emit_cell(
                        &mut frame,
                        col,
                        ri,
                        ci,
                        cx,
                        ry,
                        mid_y,
                        model.row_height,
                        status,
                        sel,
                        &search_set,
                        search_current,
                        t,
                        flash,
                    );
                }
            }

            // Separator line on the right edge of the pinned band.
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: rnw + pinned_width - 0.5,
                y1: model.header_height,
                x2: rnw + pinned_width - 0.5,
                y2: vp.height,
                color: t.header_border,
                width: 1.0,
            }));
        }

        // ── selection outer border ───────────────────────────────────────────
        if let Some((tl, br)) = sel.range() {
            let x1 = col_vx(tl.col);
            let y1 = row_vy(tl.row);
            let x2 = col_vx(br.col) + model.columns[br.col].width;
            let y2 = row_vy(br.row) + model.row_height;

            // During a flash the border adopts the flash colour;
            // otherwise use the normal selection border colour.
            let border_color = if let Some(f) = flash {
                let a =
                    (t.flash_border.a as f64 * f.alpha_factor).round() as u8;
                Color::rgba(
                    t.flash_border.r,
                    t.flash_border.g,
                    t.flash_border.b,
                    a,
                )
            } else {
                t.selection_border
            };

            // top
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1,
                y1: y1 - 0.5,
                x2,
                y2: y1 - 0.5,
                color: border_color,
                width: 1.0,
            }));
            // bottom
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1,
                y1: y2 - 0.5,
                x2,
                y2: y2 - 0.5,
                color: border_color,
                width: 1.0,
            }));
            // left
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x1 + 0.5,
                y1,
                x2: x1 + 0.5,
                y2,
                color: border_color,
                width: 1.0,
            }));
            // right
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x2 - 0.5,
                y1,
                x2: x2 - 0.5,
                y2,
                color: border_color,
                width: 1.0,
            }));
        }

        // ── header (sticky, drawn on top of scrolled data) ───────────────────
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: vp.width,
            height: model.header_height,
            fill: t.header_bg,
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
        }));

        // Render column headers for a given index range.
        let render_col_headers =
            |frame: &mut SceneFrame, range: std::ops::Range<usize>| {
                let mid_y =
                    model.header_height * 0.5 + t.header_font_size * 0.35;
                for ci in range {
                    let col = &model.columns[ci];
                    let cx = col_vx(ci);

                    let col_in_sel = sel
                        .range()
                        .is_some_and(|(tl, br)| ci >= tl.col && ci <= br.col);
                    if col_in_sel {
                        frame.push(ScenePrimitive::Rect(RectPrimitive {
                            x: cx,
                            y: 0.0,
                            width: col.width,
                            height: model.header_height,
                            fill: t.selection_fill,
                            stroke: None,
                            stroke_width: 0.0,
                            corner_radius: 0.0,
                        }));
                    }

                    // Text must end before the icon dots (which
                    // are at the horizontal center of the button).
                    // icon_center_offset = margin_r + btn_w / 2
                    let icon_center_offset = t.header_menu_icon_margin_r
                        + t.header_menu_icon_btn_w / 2.0;
                    let label_max_w =
                        (col.width - t.cell_padding - icon_center_offset)
                            .max(0.0);
                    frame.push(ScenePrimitive::Text(TextPrimitive {
                        x: cx + t.cell_padding,
                        y: mid_y,
                        text: col.label.clone(),
                        color: t.header_text,
                        font_size: t.header_font_size,
                        bold: t.header_font_bold,
                        clip: Some([cx, 0.0, col.width, model.header_height]),
                        align: TextAlign::Left,
                        max_width: Some(label_max_w),
                    }));

                    // Three-dot menu icon (⋮) — three small circles at
                    // the right edge of the header cell.
                    {
                        let mr = t.header_menu_icon_margin_r;
                        let btn_w = t.header_menu_icon_btn_w;
                        let btn_h = if t.header_menu_icon_btn_h > 0.0 {
                            t.header_menu_icon_btn_h
                        } else {
                            (model.header_height - 12.0).max(8.0)
                        };
                        // Right edge of button, inset by margin_r.
                        let btn_rx = cx + col.width - mr;
                        let btn_lx = btn_rx - btn_w;
                        let btn_ty = (model.header_height - btn_h) / 2.0;

                        // Hover background.
                        if hovered_menu_col == Some(ci) {
                            frame.push(ScenePrimitive::Rect(RectPrimitive {
                                x: btn_lx,
                                y: btn_ty,
                                width: btn_w,
                                height: btn_h,
                                fill: t.header_menu_icon_hover_bg,
                                stroke: None,
                                stroke_width: 0.0,
                                corner_radius: t.header_menu_icon_radius,
                            }));
                        }

                        let dot_r = t.header_menu_icon_dot_r;
                        let dot_gap = dot_r * 3.75;
                        // Icon center x = horizontal center of button.
                        let icon_cx = btn_lx + btn_w / 2.0;
                        let icon_mid_y = model.header_height / 2.0;
                        for i in -1i32..=1 {
                            let dot_y = icon_mid_y + i as f64 * dot_gap;
                            frame.push(ScenePrimitive::Rect(RectPrimitive {
                                x: icon_cx - dot_r,
                                y: dot_y - dot_r,
                                width: dot_r * 2.0,
                                height: dot_r * 2.0,
                                fill: t.header_menu_icon,
                                stroke: None,
                                stroke_width: 0.0,
                                corner_radius: dot_r,
                            }));
                        }
                    }

                    // Sort indicator ▲ / ▼
                    if let Some(s) = &state.sort {
                        if s.col_key == col.key {
                            let aw = t.sort_arrow_width;
                            let ah = t.sort_arrow_height;
                            // Shifted left to leave room for menu icon.
                            let ax = cx + col.width
                                - t.header_menu_icon_margin_r
                                - t.header_menu_icon_btn_w
                                - t.cell_padding
                                - aw;
                            let ay = mid_y - t.header_font_size * 0.35;
                            let points = if s.dir == SortDir::Asc {
                                vec![
                                    [ax, ay - ah],
                                    [ax + aw, ay + ah * 0.6],
                                    [ax - aw, ay + ah * 0.6],
                                ]
                            } else {
                                vec![
                                    [ax, ay + ah],
                                    [ax + aw, ay - ah * 0.6],
                                    [ax - aw, ay - ah * 0.6],
                                ]
                            };
                            frame.push(ScenePrimitive::Polygon(
                                PolygonPrimitive {
                                    points,
                                    fill: t.header_text,
                                    corner_radius: 0.5,
                                },
                            ));
                        }
                    }

                    let sep_x = cx + col.width - 0.5;
                    frame.push(ScenePrimitive::Line(LinePrimitive {
                        x1: sep_x,
                        y1: t.header_separator_inset,
                        x2: sep_x,
                        y2: model.header_height - t.header_separator_inset,
                        color: t.header_border,
                        width: t.header_separator_width,
                    }));
                }
            };

        // Scrollable column headers
        render_col_headers(&mut frame, col_start..col_end);
        // Pinned column headers (on top)
        if pinned_count > 0 {
            // Solid background masking scrollable headers underneath.
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: rnw,
                y: 0.0,
                width: pinned_width,
                height: model.header_height,
                fill: t.header_bg,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));
            render_col_headers(&mut frame, 0..pinned_count);
        }

        frame.push(ScenePrimitive::Line(LinePrimitive {
            x1: 0.0,
            y1: model.header_height - 0.5,
            x2: vp.width,
            y2: model.header_height - 0.5,
            color: t.header_border,
            width: 1.0,
        }));

        // ── row-number gutter (sticky, drawn on top of scrolled data) ────────
        if rnw > 0.0 {
            // Header corner + gutter background
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: 0.0,
                y: 0.0,
                width: rnw,
                height: vp.height,
                fill: t.gutter_bg,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));

            for ri in row_start..row_end {
                let ry = row_vy(ri);
                if ry + model.row_height < model.header_height || ry > vp.height
                {
                    continue;
                }

                let is_selected = sel
                    .range()
                    .is_some_and(|(tl, br)| ri >= tl.row && ri <= br.row);

                // Clamp everything to below the sticky header.
                let clip_y = ry.max(model.header_height);
                let clip_h = (ry + model.row_height - clip_y).max(0.0);
                if clip_h == 0.0 {
                    continue;
                }

                if is_selected {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: 0.0,
                        y: clip_y,
                        width: rnw,
                        height: clip_h,
                        fill: t.selection_fill,
                        stroke: None,
                        stroke_width: 0.0,
                        corner_radius: 0.0,
                    }));
                }

                let mid_y =
                    ry + model.row_height * 0.5 + t.gutter_font_size * 0.35;
                frame.push(ScenePrimitive::Text(TextPrimitive {
                    x: rnw - t.cell_padding,
                    y: mid_y,
                    text: (ri + 1).to_string(),
                    color: t.gutter_text,
                    font_size: t.gutter_font_size,
                    bold: t.gutter_font_bold,
                    clip: Some([0.0, clip_y, rnw, clip_h]),
                    align: TextAlign::Right,
                    max_width: None,
                }));

                // Horizontal grid line inside gutter — only below header.
                let line_y = ry + model.row_height - 0.5;
                if line_y > model.header_height {
                    frame.push(ScenePrimitive::Line(LinePrimitive {
                        x1: 0.0,
                        y1: line_y,
                        x2: rnw,
                        y2: line_y,
                        color: t.grid_line,
                        width: 1.0,
                    }));
                }
            }

            // Gutter right border (full height)
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: rnw - 0.5,
                y1: 0.0,
                x2: rnw - 0.5,
                y2: vp.height,
                color: t.gutter_border,
                width: 1.0,
            }));

            // Header bottom border re-drawn on top of gutter
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: 0.0,
                y1: model.header_height - 0.5,
                x2: rnw,
                y2: model.header_height - 0.5,
                color: t.gutter_border,
                width: 1.0,
            }));
        }

        // ── scrollbars ───────────────────────────────────────────────────────
        scrollbars::emit_scrollbars(&mut frame, vp, model, rnw, t);

        // ── column drag preview ────────────────────────────────
        if let Some(hint) = col_drag {
            let cols = &model.columns;
            if hint.source_col < cols.len() {
                let src_w = cols[hint.source_col].width;
                let src_vx = col_vx(hint.source_col);

                // 1. Dim the source column (header + all rows)
                // — with preview offsets, src_vx is already the
                //   target position, so the overlay shows where
                //   the column will land.
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: src_vx,
                    y: 0.0,
                    width: src_w,
                    height: vp.height,
                    fill: t.drag_overlay,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));

                // 3. Ghost badge (follows cursor in both X and Y)
                let ghost_w = src_w;
                let ghost_h = model.header_height;
                let ghost_x = (hint.cursor_vx - ghost_w / 2.0)
                    .max(0.0)
                    .min(vp.width - ghost_w);
                let ghost_y = (hint.cursor_vy - ghost_h / 2.0)
                    .max(0.0)
                    .min(vp.height - ghost_h);
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: ghost_x,
                    y: ghost_y,
                    width: ghost_w,
                    height: ghost_h,
                    fill: t.drag_ghost_bg,
                    stroke: Some(t.header_border),
                    stroke_width: t.drag_ghost_border_width,
                    corner_radius: t.drag_ghost_radius,
                }));

                // Ghost label — vertically centred inside the badge
                let mid_y = ghost_y + ghost_h * 0.5 + t.header_font_size * 0.35;
                frame.push(ScenePrimitive::Text(TextPrimitive {
                    x: ghost_x + t.cell_padding,
                    y: mid_y,
                    text: cols[hint.source_col].label.clone(),
                    color: t.drag_ghost_text,
                    font_size: t.header_font_size,
                    bold: t.header_font_bold,
                    clip: Some([ghost_x, ghost_y, ghost_w, ghost_h]),
                    align: TextAlign::Left,
                    max_width: None,
                }));
            }
        }

        frame
    }
}
