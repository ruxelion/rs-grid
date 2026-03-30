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

/// Transient render-time hint for column-drag visual
/// feedback.
///
/// This is **not** persistent state — it exists only for a
/// single frame render cycle. Computed by the web layer,
/// consumed by the scene builder to render a dimmed source
/// header, an insertion line, and a ghost.
#[derive(Debug, Clone)]
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

/// Transient render-time hint for the paste-flash animation.
///
/// This is **not** persistent state — it exists only for a
/// single frame render cycle. Computed by the web layer
/// from elapsed time; consumed by the scene builder to
/// render a fading overlay on selected cells.
#[derive(Debug, Clone, Copy)]
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
                fill: t.pinned_bg,
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
                color: t.pinned_separator_color,
                width: t.pinned_separator_width,
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
                            fill: t.header_selection_fill,
                            stroke: None,
                            stroke_width: 0.0,
                            corner_radius: 0.0,
                        }));
                    }

                    // Text must end before the left edge of the
                    // menu icon button to prevent the ellipsis
                    // from overlapping the button.
                    let icon_btn_offset = t.header_menu_icon_margin_r
                        + t.header_menu_icon_btn_w;
                    let label_max_w =
                        (col.width - t.cell_padding - icon_btn_offset)
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
                fill: t.pinned_header_bg,
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
                        fill: t.gutter_selection_fill,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{
        Color, LinePrimitive, PolygonPrimitive, RectPrimitive,
        ScenePrimitive, TextPrimitive,
    };
    use rs_grid_core::{
        column::ColumnDef,
        commands::GridCommand,
        model::GridModel,
        row::RowRecord,
        selection::CellCoord,
        sort::{SortDir, SortState},
        state::GridState,
    };

    // ── helpers ──────────────────────────────────────────────

    /// Build a small 3-column × 10-row grid state.
    fn make_state() -> GridState {
        let cols = vec![
            ColumnDef::new("a", "Alpha", 100.0),
            ColumnDef::new("b", "Beta", 150.0),
            ColumnDef::new("c", "Gamma", 200.0),
        ];
        let rows: Vec<RowRecord> = (0..10)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r.set("c", format!("c{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        GridState::new(model, 800.0, 600.0)
    }

    /// Extract all Text primitives from a frame.
    fn text_primitives(
        frame: &crate::frame::SceneFrame,
    ) -> Vec<&TextPrimitive> {
        frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Text(t) => Some(t),
                _ => None,
            })
            .collect()
    }

    /// Extract all Rect primitives from a frame.
    fn rect_primitives(
        frame: &crate::frame::SceneFrame,
    ) -> Vec<&RectPrimitive> {
        frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Rect(r) => Some(r),
                _ => None,
            })
            .collect()
    }

    /// Extract all Line primitives from a frame.
    fn line_primitives(
        frame: &crate::frame::SceneFrame,
    ) -> Vec<&LinePrimitive> {
        frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Line(l) => Some(l),
                _ => None,
            })
            .collect()
    }

    /// Extract all Polygon primitives from a frame.
    fn polygon_primitives(
        frame: &crate::frame::SceneFrame,
    ) -> Vec<&PolygonPrimitive> {
        frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Polygon(pg) => Some(pg),
                _ => None,
            })
            .collect()
    }

    // ── SceneBuilder construction ────────────────────────────

    #[test]
    fn builder_default() {
        let b = SceneBuilder::default();
        assert_eq!(b.dpr, 1.0);
        assert_eq!(b.theme, Theme::default());
    }

    #[test]
    fn builder_new_stores_dpr() {
        let b = SceneBuilder::new(2.0);
        assert_eq!(b.dpr, 2.0);
    }

    #[test]
    fn builder_with_theme() {
        let dark = Theme::dark();
        let b = SceneBuilder::with_theme(1.5, dark.clone());
        assert_eq!(b.dpr, 1.5);
        assert_eq!(b.theme, dark);
    }

    // ── basic build ──────────────────────────────────────────

    #[test]
    fn build_produces_nonempty_frame() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        assert!(frame.primitive_count() > 0);
    }

    #[test]
    fn build_frame_dimensions_match_viewport() {
        let state = make_state();
        let b = SceneBuilder::new(2.0);
        let frame = b.build(&state, None, None, None);
        assert_eq!(frame.viewport_width, 800.0);
        assert_eq!(frame.viewport_height, 600.0);
        assert_eq!(frame.dpr, 2.0);
    }

    #[test]
    fn build_first_primitive_is_background_rect() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.x, 0.0);
                assert_eq!(r.y, 0.0);
                assert_eq!(r.width, 800.0);
                assert_eq!(r.height, 600.0);
                assert_eq!(r.fill, Theme::light().bg);
            }
            _ => panic!("first primitive should be background Rect"),
        }
    }

    // ── header ───────────────────────────────────────────────

    #[test]
    fn build_contains_header_background() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let t = Theme::light();
        let header_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.y == 0.0
                    && r.height == 40.0
                    && r.fill == t.header_bg
            })
            .collect();
        assert!(
            !header_rects.is_empty(),
            "should have a header background rect"
        );
    }

    #[test]
    fn build_contains_column_header_labels() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let texts = text_primitives(&frame);
        let labels: Vec<&str> =
            texts.iter().map(|t| t.text.as_str()).collect();
        assert!(labels.contains(&"Alpha"), "missing header Alpha");
        assert!(labels.contains(&"Beta"), "missing header Beta");
        assert!(labels.contains(&"Gamma"), "missing header Gamma");
    }

    #[test]
    fn build_header_bottom_border() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let lines = line_primitives(&frame);
        let hh = 40.0;
        let header_border = lines.iter().any(|l| {
            (l.y1 - (hh - 0.5)).abs() < 0.01
                && (l.y2 - (hh - 0.5)).abs() < 0.01
                && l.x1 == 0.0
        });
        assert!(header_border, "should have header bottom border");
    }

    // ── data rows ────────────────────────────────────────────

    #[test]
    fn build_contains_cell_text() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let texts = text_primitives(&frame);
        // Row 0 data should appear.
        assert!(
            texts.iter().any(|t| t.text == "a0"),
            "should render cell a0"
        );
        assert!(
            texts.iter().any(|t| t.text == "b0"),
            "should render cell b0"
        );
    }

    #[test]
    fn build_alternating_row_backgrounds() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let t = Theme::light();
        let alt_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == t.row_alt_bg && r.height == 30.0)
            .collect();
        // Odd rows should have alt-bg (rows 1, 3, 5, 7, 9).
        assert!(
            !alt_rects.is_empty(),
            "should have alternating row backgrounds"
        );
    }

    #[test]
    fn build_horizontal_grid_lines() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let lines = line_primitives(&frame);
        // Horizontal grid lines span full viewport width.
        let h_lines: Vec<_> = lines
            .iter()
            .filter(|l| {
                l.x1 == 0.0
                    && (l.x2 - 800.0).abs() < 0.01
                    && (l.y1 - l.y2).abs() < 0.01
                    && l.y1 > 40.0 // below header
            })
            .collect();
        assert!(
            !h_lines.is_empty(),
            "should have horizontal grid lines"
        );
    }

    // ── column separators ────────────────────────────────────

    #[test]
    fn build_column_separators_in_header() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        let lines = line_primitives(&frame);
        // Vertical separators in header (x1 == x2, within header
        // height range, with inset).
        let seps: Vec<_> = lines
            .iter()
            .filter(|l| {
                (l.x1 - l.x2).abs() < 0.01
                    && l.y1 >= t.header_separator_inset - 0.01
                    && l.y2 <= 40.0 - t.header_separator_inset + 0.01
                    && l.y1 < l.y2
            })
            .collect();
        // 3 columns → 3 separators.
        assert_eq!(
            seps.len(),
            3,
            "expected 3 column separators, got {}",
            seps.len()
        );
    }

    // ── menu icon dots ───────────────────────────────────────

    #[test]
    fn build_menu_icon_dots_per_column() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        // Each column header has 3 dots (small rects with
        // corner_radius == dot_r).
        let dot_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                (r.corner_radius - t.header_menu_icon_dot_r).abs()
                    < 0.01
                    && r.fill == t.header_menu_icon
            })
            .collect();
        // 3 columns × 3 dots = 9.
        assert_eq!(
            dot_rects.len(),
            9,
            "expected 9 menu dots, got {}",
            dot_rects.len()
        );
    }

    // ── hovered menu icon ────────────────────────────────────

    #[test]
    fn build_hovered_menu_shows_hover_bg() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame_no_hover = b.build(&state, None, None, None);
        let frame_hover = b.build(&state, None, None, Some(1));

        let hover_rects = |f: &crate::frame::SceneFrame| {
            rect_primitives(f)
                .into_iter()
                .filter(|r| r.fill == t.header_menu_icon_hover_bg)
                .count()
        };
        assert_eq!(hover_rects(&frame_no_hover), 0);
        assert_eq!(hover_rects(&frame_hover), 1);
    }

    // ── selection ────────────────────────────────────────────

    #[test]
    fn build_selection_adds_fill_and_border() {
        let mut state = make_state();
        state.apply(GridCommand::SelectCell(CellCoord {
            row: 1,
            col: 1,
        }));
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Selection fill rect.
        let sel_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == t.selection_fill)
            .collect();
        assert!(
            !sel_rects.is_empty(),
            "should have selection fill rects"
        );

        // Selection border (4 lines).
        let border_lines: Vec<_> = line_primitives(&frame)
            .into_iter()
            .filter(|l| l.color == t.selection_border)
            .collect();
        assert_eq!(
            border_lines.len(),
            4,
            "selection should produce 4 border lines"
        );
    }

    #[test]
    fn build_no_selection_no_border_lines() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        let border_lines: Vec<_> = line_primitives(&frame)
            .into_iter()
            .filter(|l| l.color == t.selection_border)
            .collect();
        assert!(
            border_lines.is_empty(),
            "no selection → no border lines"
        );
    }

    #[test]
    fn build_multi_cell_selection() {
        let mut state = make_state();
        state.apply(GridCommand::SelectCell(CellCoord {
            row: 0,
            col: 0,
        }));
        state.apply(GridCommand::ExtendSelection(CellCoord {
            row: 2,
            col: 1,
        }));
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Multiple cells selected → multiple fill rects.
        let sel_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == t.selection_fill)
            .collect();
        // 3 rows × 2 cols = 6 cell fills + possibly header
        // highlight.
        assert!(
            sel_rects.len() >= 6,
            "multi-cell selection should produce ≥6 fill rects, \
             got {}",
            sel_rects.len()
        );
    }

    // ── hover row ────────────────────────────────────────────

    #[test]
    fn build_hovered_row_overlay() {
        let mut state = make_state();
        state.hovered_row = Some(2);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        let hover_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == t.row_hover_bg)
            .collect();
        assert!(
            !hover_rects.is_empty(),
            "hovered row should produce hover overlay"
        );
    }

    #[test]
    fn build_no_hover_no_overlay() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        let hover_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == t.row_hover_bg)
            .collect();
        assert!(
            hover_rects.is_empty(),
            "no hover → no hover overlay"
        );
    }

    // ── sort indicator ───────────────────────────────────────

    #[test]
    fn build_sort_indicator_polygon() {
        let mut state = make_state();
        state.sort = Some(SortState {
            col_key: "a".into(),
            dir: SortDir::Asc,
        });
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let polys = polygon_primitives(&frame);
        // At least one polygon for the sort arrow.
        assert!(
            !polys.is_empty(),
            "sort should produce a polygon arrow"
        );
    }

    #[test]
    fn build_no_sort_no_polygon() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let polys = polygon_primitives(&frame);
        // No sort → no polygons in data area (scrollbar arrows
        // may still produce polygons).
        // Check: no polygon has the header_text fill.
        let t = Theme::light();
        let sort_polys: Vec<_> = polys
            .iter()
            .filter(|p| p.fill == t.header_text)
            .collect();
        assert!(
            sort_polys.is_empty(),
            "no sort → no sort indicator polygon"
        );
    }

    #[test]
    fn build_sort_desc_different_points() {
        let mut state = make_state();
        state.sort = Some(SortState {
            col_key: "b".into(),
            dir: SortDir::Desc,
        });
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        let t = Theme::light();
        let polys: Vec<_> = polygon_primitives(&frame)
            .into_iter()
            .filter(|p| p.fill == t.header_text)
            .collect();
        assert_eq!(polys.len(), 1, "one sort polygon");
        // Desc: tip points downward (max y among vertices is
        // the tip).
        let pts = &polys[0].points;
        assert_eq!(pts.len(), 3);
    }

    // ── flash ────────────────────────────────────────────────

    #[test]
    fn build_flash_changes_selection_border_color() {
        let mut state = make_state();
        state.apply(GridCommand::SelectCell(CellCoord {
            row: 0,
            col: 0,
        }));
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let flash = FlashHint { alpha_factor: 0.5 };
        let frame = b.build(&state, None, Some(&flash), None);

        // Border lines should use flash_border color (alpha-adjusted).
        let expected_a =
            (t.flash_border.a as f64 * 0.5).round() as u8;
        let expected_color = Color::rgba(
            t.flash_border.r,
            t.flash_border.g,
            t.flash_border.b,
            expected_a,
        );
        let flash_borders: Vec<_> = line_primitives(&frame)
            .into_iter()
            .filter(|l| l.color == expected_color)
            .collect();
        assert_eq!(
            flash_borders.len(),
            4,
            "flash should produce 4 border lines with flash color"
        );
    }

    #[test]
    fn build_flash_overlay_on_selected_cells() {
        let mut state = make_state();
        state.apply(GridCommand::SelectCell(CellCoord {
            row: 0,
            col: 0,
        }));
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let flash = FlashHint { alpha_factor: 1.0 };
        let frame = b.build(&state, None, Some(&flash), None);

        // Should have a flash fill rect on the selected cell.
        let flash_fill = Color::rgba(
            t.flash_fill.r,
            t.flash_fill.g,
            t.flash_fill.b,
            t.flash_fill.a,
        );
        let flash_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| r.fill == flash_fill)
            .collect();
        assert!(
            !flash_rects.is_empty(),
            "flash should produce overlay rects on selected cells"
        );
    }

    // ── zero row height early return ─────────────────────────

    #[test]
    fn build_zero_row_height_returns_early() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = vec![];
        let model = GridModel::new(cols, rows, 0.0, 40.0);
        let state = GridState::new(model, 800.0, 600.0);
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        // Only the background rect should be present.
        assert_eq!(
            frame.primitive_count(),
            1,
            "zero row_height → only background"
        );
    }

    // ── empty grid ───────────────────────────────────────────

    #[test]
    fn build_empty_grid_no_data_rows() {
        let cols = vec![
            ColumnDef::new("x", "X", 100.0),
        ];
        let rows: Vec<RowRecord> = vec![];
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let state = GridState::new(model, 800.0, 600.0);
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);
        // Should still have background + header, but no cell text.
        let texts = text_primitives(&frame);
        let data_texts: Vec<_> = texts
            .iter()
            .filter(|t| t.text != "X")
            .collect();
        assert!(
            data_texts.is_empty(),
            "empty grid should have no data cell text"
        );
    }

    // ── column drag ──────────────────────────────────────────

    #[test]
    fn build_column_drag_shows_ghost() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let drag = ColumnDragHint {
            source_col: 0,
            insert_before: 2,
            cursor_vx: 300.0,
            cursor_vy: 20.0,
            animated_offsets: vec![],
        };
        let frame = b.build(&state, Some(&drag), None, None);

        // Ghost badge rect (drag_ghost_bg fill + rounded corners).
        let ghost_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.fill == t.drag_ghost_bg
                    && r.corner_radius > 0.0
            })
            .collect();
        assert_eq!(
            ghost_rects.len(),
            1,
            "drag should show one ghost badge"
        );

        // Ghost label text.
        let texts = text_primitives(&frame);
        let ghost_labels: Vec<_> = texts
            .iter()
            .filter(|t| t.color == Theme::light().drag_ghost_text)
            .collect();
        assert!(
            !ghost_labels.is_empty(),
            "ghost badge should have a label"
        );
    }

    #[test]
    fn build_column_drag_overlay() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let drag = ColumnDragHint {
            source_col: 1,
            insert_before: 0,
            cursor_vx: 100.0,
            cursor_vy: 20.0,
            animated_offsets: vec![],
        };
        let frame = b.build(&state, Some(&drag), None, None);

        // Dim overlay on source column.
        let overlay_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.fill == t.drag_overlay
                    && r.y == 0.0
                    && (r.height - 600.0).abs() < 0.01
            })
            .collect();
        assert!(
            !overlay_rects.is_empty(),
            "drag should show dim overlay on source column"
        );
    }

    #[test]
    fn build_column_drag_out_of_bounds_no_panic() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let drag = ColumnDragHint {
            source_col: 999,
            insert_before: 0,
            cursor_vx: 100.0,
            cursor_vy: 20.0,
            animated_offsets: vec![],
        };
        // Should not panic.
        let _ = b.build(&state, Some(&drag), None, None);
    }

    #[test]
    fn build_column_drag_with_animated_offsets() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        let drag = ColumnDragHint {
            source_col: 0,
            insert_before: 2,
            cursor_vx: 200.0,
            cursor_vy: 20.0,
            animated_offsets: vec![150.0, 0.0, 250.0],
        };
        let frame = b.build(&state, Some(&drag), None, None);
        // Should not panic and produce valid output.
        assert!(frame.primitive_count() > 0);
    }

    // ── scrolled viewport ────────────────────────────────────

    #[test]
    fn build_scrolled_viewport_still_has_header() {
        let mut state = make_state();
        state.apply(GridCommand::ScrollTo {
            x: 0.0,
            y: 200.0,
        });
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        // Header is sticky — should always be present.
        let header_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.y == 0.0
                    && (r.height - 40.0).abs() < 0.01
                    && r.fill == t.header_bg
            })
            .collect();
        assert!(
            !header_rects.is_empty(),
            "scrolled viewport should still show header"
        );
    }

    // ── row number gutter ────────────────────────────────────

    #[test]
    fn build_with_row_numbers() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
        ];
        let rows: Vec<RowRecord> = (0..5)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("v{i}"));
                r
            })
            .collect();
        let mut model = GridModel::new(cols, rows, 30.0, 40.0);
        model.row_number_width = 50.0;
        let state = GridState::new(model, 800.0, 600.0);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Gutter background rect.
        let gutter_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.x == 0.0
                    && (r.width - 50.0).abs() < 0.01
                    && r.fill == t.gutter_bg
            })
            .collect();
        assert!(
            !gutter_rects.is_empty(),
            "should have gutter background"
        );

        // Row numbers as text (1-indexed).
        let texts = text_primitives(&frame);
        assert!(
            texts.iter().any(|t| t.text == "1"),
            "should show row number 1"
        );
        assert!(
            texts.iter().any(|t| t.text == "5"),
            "should show row number 5"
        );
    }

    #[test]
    fn build_without_row_numbers_no_gutter() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let mut model = GridModel::new(cols, vec![], 30.0, 40.0);
        model.row_number_width = 0.0;
        let state = GridState::new(model, 800.0, 600.0);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        // Gutter border line at x ≈ rnw-0.5 should not exist.
        let lines = line_primitives(&frame);
        let gutter_border = lines.iter().any(|l| {
            l.color == t.gutter_border
                && (l.x1 - l.x2).abs() < 0.01
                && l.y1 == 0.0
        });
        assert!(
            !gutter_border,
            "no row numbers → no gutter border"
        );
    }

    // ── pinned columns ───────────────────────────────────────

    #[test]
    fn build_pinned_columns_solid_background() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
            ColumnDef::new("c", "C", 200.0),
        ];
        let rows: Vec<RowRecord> = (0..5)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r.set("c", format!("c{i}"));
                r
            })
            .collect();
        let mut model = GridModel::new(cols, rows, 30.0, 40.0);
        model.pinned_count = 1;
        let rnw = model.row_number_width;
        let state = GridState::new(model, 800.0, 600.0);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Pinned data overlay background at x=rnw,
        // y=header_height, width=100 (first col width).
        let pinned_bg: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                (r.x - rnw).abs() < 0.01
                    && (r.y - 40.0).abs() < 0.01
                    && (r.width - 100.0).abs() < 0.01
                    && r.fill == t.bg
            })
            .collect();
        assert!(
            !pinned_bg.is_empty(),
            "pinned columns should have solid background overlay"
        );
    }

    #[test]
    fn build_pinned_columns_separator_line() {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 150.0),
        ];
        let rows: Vec<RowRecord> = (0..3)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("a{i}"));
                r.set("b", format!("b{i}"));
                r
            })
            .collect();
        let mut model = GridModel::new(cols, rows, 30.0, 40.0);
        model.pinned_count = 1;
        let rnw = model.row_number_width;
        let state = GridState::new(model, 800.0, 600.0);
        let b = SceneBuilder::new(1.0);
        let frame = b.build(&state, None, None, None);

        // Separator line at x ≈ rnw + pinned_width - 0.5.
        let sep_x = rnw + 100.0 - 0.5;
        let lines = line_primitives(&frame);
        let sep = lines.iter().any(|l| {
            (l.x1 - sep_x).abs() < 0.01
                && (l.x2 - sep_x).abs() < 0.01
                && l.y1 >= 39.0
        });
        assert!(sep, "should have pinned column separator line");
    }

    // ── scrollbars ───────────────────────────────────────────

    #[test]
    fn build_scrollbar_primitives_present_when_content_overflows()
    {
        // Make content taller than viewport.
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows: Vec<RowRecord> = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("v{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        // Viewport height 600 < total height = 40 + 100*30 = 3040.
        let state = GridState::new(model, 800.0, 600.0);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Scrollbar thumb (a rect with scrollbar_thumb fill and
        // scrollbar_radius corner_radius).
        let thumb_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.fill == t.scrollbar_thumb
                    && (r.corner_radius - t.scrollbar_radius).abs()
                        < 0.01
            })
            .collect();
        assert!(
            !thumb_rects.is_empty(),
            "overflowing content should show scrollbar thumb"
        );
    }

    #[test]
    fn build_scrollbar_arrows_as_polygons() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows: Vec<RowRecord> = (0..100)
            .map(|i| {
                let mut r = RowRecord::new(i);
                r.set("a", format!("v{i}"));
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let state = GridState::new(model, 800.0, 600.0);
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Scrollbar arrows are polygons with scrollbar_thumb fill.
        let arrow_polys: Vec<_> = polygon_primitives(&frame)
            .into_iter()
            .filter(|p| p.fill == t.scrollbar_thumb)
            .collect();
        // At least 2 arrows (up + down) for vertical scrollbar.
        assert!(
            arrow_polys.len() >= 2,
            "should have scrollbar arrow polygons, got {}",
            arrow_polys.len()
        );
    }

    // ── dark theme produces different colors ─────────────────

    #[test]
    fn build_dark_theme_different_background() {
        let state = make_state();
        let b_light = SceneBuilder::new(1.0);
        let b_dark =
            SceneBuilder::with_theme(1.0, Theme::dark());
        let f_light = b_light.build(&state, None, None, None);
        let f_dark = b_dark.build(&state, None, None, None);

        // Background rects should differ.
        let bg_light = match &f_light.primitives[0] {
            ScenePrimitive::Rect(r) => r.fill,
            _ => panic!("expected Rect"),
        };
        let bg_dark = match &f_dark.primitives[0] {
            ScenePrimitive::Rect(r) => r.fill,
            _ => panic!("expected Rect"),
        };
        assert_ne!(bg_light, bg_dark);
    }

    // ── high DPR ─────────────────────────────────────────────

    #[test]
    fn build_high_dpr_frame_stores_dpr() {
        let state = make_state();
        let b = SceneBuilder::new(3.0);
        let frame = b.build(&state, None, None, None);
        assert_eq!(frame.dpr, 3.0);
    }

    // ── ColumnDragHint / FlashHint constructors ──────────────

    #[test]
    fn column_drag_hint_debug() {
        let h = ColumnDragHint {
            source_col: 0,
            insert_before: 1,
            cursor_vx: 50.0,
            cursor_vy: 25.0,
            animated_offsets: vec![],
        };
        let s = format!("{:?}", h);
        assert!(s.contains("ColumnDragHint"));
    }

    #[test]
    fn flash_hint_debug_and_copy() {
        let f = FlashHint { alpha_factor: 0.75 };
        let f2 = f; // Copy
        assert_eq!(f.alpha_factor, f2.alpha_factor);
        let s = format!("{:?}", f);
        assert!(s.contains("FlashHint"));
    }

    // ── drag same position is no-op offset ───────────────────

    #[test]
    fn build_drag_same_position_no_preview_offset() {
        let state = make_state();
        let b = SceneBuilder::new(1.0);
        // source_col == insert_before → no reorder needed.
        let drag = ColumnDragHint {
            source_col: 1,
            insert_before: 1,
            cursor_vx: 100.0,
            cursor_vy: 20.0,
            animated_offsets: vec![],
        };
        let frame = b.build(&state, Some(&drag), None, None);
        // Should still render without panic.
        assert!(frame.primitive_count() > 0);
    }

    // ── search highlights ────────────────────────────────────

    #[test]
    fn build_search_highlights_matching_cells() {
        let mut state = make_state();
        state.apply(GridCommand::Search {
            query: "a0".into(),
        });
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);

        // Should have search highlight rects.
        let highlight_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.fill == t.search_highlight
                    || r.fill == t.search_current
            })
            .collect();
        assert!(
            !highlight_rects.is_empty(),
            "search should highlight matching cells"
        );
    }

    #[test]
    fn build_no_search_no_highlights() {
        let state = make_state();
        let t = Theme::light();
        let b = SceneBuilder::with_theme(1.0, t.clone());
        let frame = b.build(&state, None, None, None);
        let highlight_rects: Vec<_> = rect_primitives(&frame)
            .into_iter()
            .filter(|r| {
                r.fill == t.search_highlight
                    || r.fill == t.search_current
            })
            .collect();
        assert!(
            highlight_rects.is_empty(),
            "no search → no highlights"
        );
    }
}
