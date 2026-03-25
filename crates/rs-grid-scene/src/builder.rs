use std::collections::HashSet;

use rs_grid_core::{
    column::{format_cell, CellAlign, CellFormat, ColumnDef},
    datasource::CellStatus,
    model::GridModel,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    selection::SelectionState,
    sort::SortDir,
    state::GridState,
    viewport::ViewportState,
};

use crate::{
    frame::SceneFrame,
    primitives::{
        Color, ImagePrimitive, LinePrimitive, PolygonPrimitive, RectPrimitive,
        ScenePrimitive, TextAlign, TextPrimitive,
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
        let preview_offsets: Option<Vec<f64>> =
            col_drag.and_then(|hint| {
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
                Self::emit_cell(
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
                    Self::emit_cell(
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
                let a = (t.flash_border.a as f64 * f.alpha_factor).round()
                    as u8;
                Color::rgba(t.flash_border.r, t.flash_border.g, t.flash_border.b, a)
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

                    frame.push(ScenePrimitive::Text(TextPrimitive {
                        x: cx + t.cell_padding,
                        y: mid_y,
                        text: col.label.clone(),
                        color: t.header_text,
                        font_size: t.header_font_size,
                        bold: t.header_font_bold,
                        clip: Some([cx, 0.0, col.width, model.header_height]),
                        align: TextAlign::Left,
                    }));

                    // Sort indicator ▲ / ▼
                    if let Some(s) = &state.sort {
                        if s.col_key == col.key {
                            let aw = t.sort_arrow_width;
                            let ah = t.sort_arrow_height;
                            let ax =
                                cx + col.width - t.cell_padding - aw;
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
                        y2: model.header_height
                            - t.header_separator_inset,
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
                fill: t.header_bg,
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

                if is_selected {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: 0.0,
                        y: ry,
                        width: rnw,
                        height: model.row_height,
                        fill: t.selection_fill,
                        stroke: None,
                        stroke_width: 0.0,
                        corner_radius: 0.0,
                    }));
                }

                let mid_y = ry + model.row_height * 0.5 + t.font_size * 0.35;
                frame.push(ScenePrimitive::Text(TextPrimitive {
                    x: rnw - t.cell_padding,
                    y: mid_y,
                    text: (ri + 1).to_string(),
                    color: t.header_text,
                    font_size: t.font_size,
                    bold: false,
                    clip: Some([0.0, ry, rnw, model.row_height]),
                    align: TextAlign::Right,
                }));

                // Horizontal grid line inside gutter
                frame.push(ScenePrimitive::Line(LinePrimitive {
                    x1: 0.0,
                    y1: ry + model.row_height - 0.5,
                    x2: rnw,
                    y2: ry + model.row_height - 0.5,
                    color: t.grid_line,
                    width: 1.0,
                }));
            }

            // Gutter right border (full height)
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: rnw - 0.5,
                y1: 0.0,
                x2: rnw - 0.5,
                y2: vp.height,
                color: t.header_border,
                width: 1.0,
            }));

            // Header bottom border re-drawn on top of gutter
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: 0.0,
                y1: model.header_height - 0.5,
                x2: rnw,
                y2: model.header_height - 0.5,
                color: t.header_border,
                width: 1.0,
            }));
        }

        // ── scrollbars ───────────────────────────────────────────────────────
        Self::emit_scrollbars(&mut frame, vp, model, rnw, t);

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
                let mid_y = ghost_y
                    + ghost_h * 0.5
                    + t.header_font_size * 0.35;
                frame.push(ScenePrimitive::Text(TextPrimitive {
                    x: ghost_x + t.cell_padding,
                    y: mid_y,
                    text: cols[hint.source_col].label.clone(),
                    color: t.drag_ghost_text,
                    font_size: t.header_font_size,
                    bold: t.header_font_bold,
                    clip: Some([ghost_x, ghost_y, ghost_w, ghost_h]),
                    align: TextAlign::Left,
                }));
            }
        }

        frame
    }

    /// Emit the vertical and horizontal scrollbar primitives.
    ///
    /// Computes `ScrollbarGeom` once and reuses it for both the
    /// vertical render and the horizontal scrollbar's vsb_w
    /// calculation, avoiding the double-compute present in the
    /// original inlined code.
    fn emit_scrollbars(
        frame: &mut SceneFrame,
        vp: &ViewportState,
        model: &GridModel,
        rnw: f64,
        t: &Theme,
    ) {
        // ── vertical scrollbar ───────────────────────────────────────────────
        let vsb = ScrollbarGeom::compute(
            vp.scroll_y,
            vp.width,
            vp.height,
            model.header_height,
            model.total_height(),
            t.scrollbar_width,
        );

        if let Some(sb) = &vsb {
            // Arrow button backgrounds
            for btn_y in [sb.up_btn_y, sb.down_btn_y] {
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: sb.track_x,
                    y: btn_y,
                    width: sb.track_w,
                    height: sb.arrow_h,
                    fill: t.scrollbar_track,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));
            }

            // Arrow icons
            let cx = sb.track_x + sb.track_w * 0.5;
            let arrow_size = (sb.track_w * 0.45).max(3.0);

            // Up arrow ▲
            let mid_up = sb.up_btn_y + sb.arrow_h * 0.5;
            Self::emit_scrollbar_arrow(
                frame,
                cx,
                mid_up,
                arrow_size,
                -1.0,
                true,
                t.scrollbar_thumb,
            );

            // Down arrow ▼
            let mid_dn = sb.down_btn_y + sb.arrow_h * 0.5;
            Self::emit_scrollbar_arrow(
                frame,
                cx,
                mid_dn,
                arrow_size,
                1.0,
                true,
                t.scrollbar_thumb,
            );

            // Track
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: sb.track_x,
                y: sb.track_y,
                width: sb.track_w,
                height: sb.track_h,
                fill: t.scrollbar_track,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));

            // Thumb (inset on each side)
            let inset = t.scrollbar_inset;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: sb.track_x + inset,
                y: sb.thumb_y + inset,
                width: (sb.track_w - inset * 2.0).max(2.0),
                height: (sb.thumb_h - inset * 2.0).max(4.0),
                fill: t.scrollbar_thumb,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: t.scrollbar_radius,
            }));
        }

        // ── horizontal scrollbar ─────────────────────────────────────────────
        // Reserve space for the vertical scrollbar width if it is visible.
        let vsb_w = if vsb.is_some() {
            t.scrollbar_width
        } else {
            0.0
        };

        if let Some(hsb) = HScrollbarGeom::compute(
            vp.scroll_x,
            vp.width,
            vp.height,
            rnw,
            model.total_width(),
            vsb_w,
            t.scrollbar_width,
        ) {
            // Arrow button backgrounds
            for btn_x in [hsb.left_btn_x, hsb.right_btn_x] {
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: btn_x,
                    y: hsb.track_y,
                    width: hsb.arrow_w,
                    height: hsb.track_h,
                    fill: t.scrollbar_track,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));
            }

            // Arrow icons
            let cy = hsb.track_y + hsb.track_h * 0.5;
            let arrow_size = (hsb.track_h * 0.45).max(3.0);

            // Left arrow ◀
            let mid_left = hsb.left_btn_x + hsb.arrow_w * 0.5;
            Self::emit_scrollbar_arrow(
                frame,
                cy,
                mid_left,
                arrow_size,
                -1.0,
                false,
                t.scrollbar_thumb,
            );

            // Right arrow ▶
            let mid_right = hsb.right_btn_x + hsb.arrow_w * 0.5;
            Self::emit_scrollbar_arrow(
                frame,
                cy,
                mid_right,
                arrow_size,
                1.0,
                false,
                t.scrollbar_thumb,
            );

            // Track
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: hsb.track_x,
                y: hsb.track_y,
                width: hsb.track_w,
                height: hsb.track_h,
                fill: t.scrollbar_track,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));

            // Thumb (inset on each side)
            let inset = t.scrollbar_inset;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: hsb.thumb_x + inset,
                y: hsb.track_y + inset,
                width: (hsb.thumb_w - inset * 2.0).max(4.0),
                height: (hsb.track_h - inset * 2.0).max(2.0),
                fill: t.scrollbar_thumb,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: t.scrollbar_radius,
            }));
        }
    }

    /// Emit a single scrollbar arrow as a `PolygonPrimitive`.
    ///
    /// - `cross` — center coordinate on the perpendicular axis
    ///   (x for vertical bars, y for horizontal bars).
    /// - `mid`   — center of the button along its scroll axis.
    /// - `size`  — half-size of the arrow triangle.
    /// - `dir`   — `-1.0` = up/left, `+1.0` = down/right.
    /// - `vertical` — `true` for vertical scrollbar arrows.
    fn emit_scrollbar_arrow(
        frame: &mut SceneFrame,
        cross: f64,
        mid: f64,
        size: f64,
        dir: f64,
        vertical: bool,
        fill: Color,
    ) {
        let points = if vertical {
            vec![
                [cross, mid + dir * size * 0.45],
                [cross + size, mid - dir * size],
                [cross - size, mid - dir * size],
            ]
        } else {
            vec![
                [mid + dir * size * 0.45, cross],
                [mid - dir * size, cross - size],
                [mid - dir * size, cross + size],
            ]
        };
        frame.push(ScenePrimitive::Polygon(PolygonPrimitive {
            points,
            fill,
            corner_radius: size * 0.25,
        }));
    }

    /// Emit selection fill, search highlight, and cell content
    /// (text, image, or skeleton) for a single cell.
    ///
    /// Shared by the scrollable-column and pinned-column render
    /// loops to avoid duplicating ~150 lines of logic.
    #[allow(clippy::too_many_arguments)]
    fn emit_cell(
        frame: &mut SceneFrame,
        col: &ColumnDef,
        ri: u64,
        ci: usize,
        cx: f64,
        ry: f64,
        mid_y: f64,
        row_height: f64,
        cell_status: CellStatus,
        sel: &SelectionState,
        search_set: &HashSet<(u64, usize)>,
        search_current: Option<(u64, usize)>,
        t: &Theme,
        flash: Option<&FlashHint>,
    ) {
        // Selection fill (no border — outer border drawn below)
        if sel.is_selected(ri, ci) {
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: cx,
                y: ry,
                width: col.width,
                height: row_height,
                fill: t.selection_fill,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));
            // Flash overlay — themed fade on paste
            if let Some(f) = flash {
                let a = (t.flash_fill.a as f64 * f.alpha_factor).round()
                    as u8;
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: cx,
                    y: ry,
                    width: col.width,
                    height: row_height,
                    fill: Color::rgba(t.flash_fill.r, t.flash_fill.g, t.flash_fill.b, a),
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: 0.0,
                }));
            }
        }

        // Search highlight
        if search_set.contains(&(ri, ci)) {
            let fill = if search_current == Some((ri, ci)) {
                t.search_current
            } else {
                t.search_highlight
            };
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: cx,
                y: ry,
                width: col.width,
                height: row_height,
                fill,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: 0.0,
            }));
        }

        // Cell text, image, or skeleton
        match cell_status {
            CellStatus::Ready(raw) if !raw.is_empty() => {
                if let Some(CellFormat::Image {
                    base_url,
                    border_radius,
                    padding,
                }) = &col.format
                {
                    let url = match base_url {
                        Some(base) => format!("{base}{raw}"),
                        None => raw,
                    };
                    let pad = *padding;
                    frame.push(ScenePrimitive::Image(ImagePrimitive {
                        url,
                        x: cx + pad,
                        y: ry + pad,
                        width: col.width - pad * 2.0,
                        height: row_height - pad * 2.0,
                        corner_radius: *border_radius,
                        clip: Some([cx, ry, col.width, row_height]),
                        placeholder_color: t.skeleton_fg,
                    }));
                } else if let Some(CellFormat::ImageText {
                    base_url,
                    suffix,
                    image_size,
                    border_radius,
                    gap,
                }) = &col.format
                {
                    Self::emit_image_text(
                        frame,
                        &raw,
                        cx,
                        ry,
                        col.width,
                        row_height,
                        mid_y,
                        t,
                        base_url,
                        suffix,
                        *image_size,
                        *border_radius,
                        *gap,
                    );
                } else {
                    let (txt, align, bold, color) =
                        if let Some(fmt) = &col.format {
                            let fc = format_cell(&raw, fmt);
                            let a = match fc.align.unwrap_or_default() {
                                CellAlign::Left => TextAlign::Left,
                                CellAlign::Right => TextAlign::Right,
                                CellAlign::Center => TextAlign::Center,
                            };
                            let c = fc
                                .color
                                .map(|c| Color::rgba(c[0], c[1], c[2], c[3]))
                                .unwrap_or(t.cell_text);
                            (fc.text, a, fc.bold, c)
                        } else {
                            (raw, TextAlign::Left, false, t.cell_text)
                        };
                    let x = match align {
                        TextAlign::Right => cx + col.width - t.cell_padding,
                        TextAlign::Center => cx + col.width / 2.0,
                        TextAlign::Left => cx + t.cell_padding,
                    };
                    frame.push(ScenePrimitive::Text(TextPrimitive {
                        x,
                        y: mid_y,
                        text: txt,
                        color,
                        font_size: t.font_size,
                        bold,
                        clip: Some([cx, ry, col.width, row_height]),
                        align,
                    }));
                }
            }
            CellStatus::Loading => {
                let bar_w = col.width * 0.6;
                let bar_h = t.font_size * 0.5;
                let bar_x = cx + t.cell_padding;
                let bar_y = ry + (row_height - bar_h) / 2.0;
                frame.push(ScenePrimitive::Rect(RectPrimitive {
                    x: bar_x,
                    y: bar_y,
                    width: bar_w,
                    height: bar_h,
                    fill: t.skeleton_fg,
                    stroke: None,
                    stroke_width: 0.0,
                    corner_radius: bar_h / 2.0,
                }));
            }
            _ => {}
        }
    }

    /// Emit an image + text pair for `CellFormat::ImageText`.
    ///
    /// Raw value = `"KEY Label"`. Image URL is built from
    /// `base_url + key.lowercase() + suffix`. The image is
    /// rendered on the left, text on the right.
    #[allow(clippy::too_many_arguments)]
    fn emit_image_text(
        frame: &mut SceneFrame,
        raw: &str,
        cx: f64,
        ry: f64,
        col_width: f64,
        row_height: f64,
        mid_y: f64,
        t: &Theme,
        base_url: &str,
        suffix: &str,
        image_size: f64,
        border_radius: f64,
        gap: f64,
    ) {
        let (key, label) = raw.split_once(' ').unwrap_or((raw, ""));

        // Image — vertically centered in the cell.
        let img_pad = (row_height - image_size) / 2.0;
        let img_x = cx + t.cell_padding;
        let img_y = ry + img_pad;
        let url = format!("{base_url}{key}{suffix}");
        frame.push(ScenePrimitive::Image(ImagePrimitive {
            url,
            x: img_x,
            y: img_y,
            width: image_size,
            height: image_size,
            corner_radius: border_radius,
            clip: Some([cx, ry, col_width, row_height]),
            placeholder_color: t.skeleton_fg,
        }));

        // Text — offset after the image.
        if !label.is_empty() {
            let text_x = img_x + image_size + gap;
            frame.push(ScenePrimitive::Text(TextPrimitive {
                x: text_x,
                y: mid_y,
                text: label.to_owned(),
                color: t.cell_text,
                font_size: t.font_size,
                bold: false,
                clip: Some([cx, ry, col_width, row_height]),
                align: TextAlign::Left,
            }));
        }
    }
}
