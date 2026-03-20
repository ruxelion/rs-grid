use rs_grid_core::{
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    sort::SortDir,
    state::GridState,
};

use crate::{
    frame::SceneFrame,
    primitives::{
        LinePrimitive, PolygonPrimitive, RectPrimitive, ScenePrimitive,
        TextAlign, TextPrimitive,
    },
    theme::Theme,
};

// ── builder ───────────────────────────────────────────────────────────────────

/// Transforms a `GridState` snapshot into a `SceneFrame`.
///
/// Instantiate once and reuse; all state is read from `GridState` on each
/// `build()` call.
pub struct SceneBuilder {
    /// Device pixel ratio — hardware property, not a theme property.
    pub dpr: f64,
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
    pub fn new(dpr: f64) -> Self {
        Self {
            dpr,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(dpr: f64, theme: Theme) -> Self {
        Self { dpr, theme }
    }

    /// Build a complete `SceneFrame` from the current `GridState`.
    pub fn build(&self, state: &GridState) -> SceneFrame {
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

        // Helper: viewport x of the left edge of column `ci`.
        // Pinned columns are not shifted by scroll_x.
        let col_vx = |ci: usize| -> f64 {
            let off = model.column_offsets.offsets[ci];
            if ci < pinned_count { off + rnw } else { off - sx + rnw }
        };

        // ── data rows ────────────────────────────────────────────────────────
        for ri in row_start..row_end {
            let ry = model.row_top(ri) - sy;

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

            for ci in col_start..col_end {
                let col = &model.columns[ci];
                let cx = col_vx(ci);

                // Selection fill (no border — outer border drawn below)
                if sel.is_selected(ri, ci) {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: cx,
                        y: ry,
                        width: col.width,
                        height: model.row_height,
                        fill: t.selection_fill,
                        stroke: None,
                        stroke_width: 0.0,
                        corner_radius: 0.0,
                    }));
                }

                // Cell text
                if let Some(text) = model.get_cell(ri, &col.key) {
                    if !text.is_empty() {
                        frame.push(ScenePrimitive::Text(TextPrimitive {
                            x: cx + t.cell_padding,
                            y: mid_y,
                            text,
                            color: t.cell_text,
                            font_size: t.font_size,
                            bold: false,
                            clip: Some([cx, ry, col.width, model.row_height]),
                            align: TextAlign::Left,
                        }));
                    }
                }
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
                let ry = model.row_top(ri) - sy;
                if ry + model.row_height < model.header_height
                    || ry > vp.height
                {
                    continue;
                }
                let mid_y =
                    ry + model.row_height * 0.5 + t.font_size * 0.35;

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

                    if sel.is_selected(ri, ci) {
                        frame.push(ScenePrimitive::Rect(RectPrimitive {
                            x: cx,
                            y: ry,
                            width: col.width,
                            height: model.row_height,
                            fill: t.selection_fill,
                            stroke: None,
                            stroke_width: 0.0,
                            corner_radius: 0.0,
                        }));
                    }

                    if let Some(text) = model.get_cell(ri, &col.key) {
                        if !text.is_empty() {
                            frame.push(ScenePrimitive::Text(TextPrimitive {
                                x: cx + t.cell_padding,
                                y: mid_y,
                                text,
                                color: t.cell_text,
                                font_size: t.font_size,
                                bold: false,
                                clip: Some([
                                    cx,
                                    ry,
                                    col.width,
                                    model.row_height,
                                ]),
                                align: TextAlign::Left,
                            }));
                        }
                    }
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
            let y1 = model.row_top(tl.row) - sy;
            let x2 =
                col_vx(br.col) + model.columns[br.col].width;
            let y2 = model.row_top(br.row) - sy + model.row_height;

            // top
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1,
                y1: y1 + 0.5,
                x2,
                y2: y1 + 0.5,
                color: t.selection_border,
                width: 1.0,
            }));
            // bottom
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1,
                y1: y2 - 0.5,
                x2,
                y2: y2 - 0.5,
                color: t.selection_border,
                width: 1.0,
            }));
            // left
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x1 + 0.5,
                y1,
                x2: x1 + 0.5,
                y2,
                color: t.selection_border,
                width: 1.0,
            }));
            // right
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x2 - 0.5,
                y1,
                x2: x2 - 0.5,
                y2,
                color: t.selection_border,
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
                        .map_or(false, |(tl, br)| {
                            ci >= tl.col && ci <= br.col
                        });
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
                        clip: Some([
                            cx,
                            0.0,
                            col.width,
                            model.header_height,
                        ]),
                        align: TextAlign::Left,
                    }));

                    // Sort indicator ▲ / ▼
                    if let Some(s) = &state.sort {
                        if s.col_key == col.key {
                            const AW: f64 = 4.0;
                            const AH: f64 = 3.5;
                            let ax =
                                cx + col.width - t.cell_padding - AW;
                            let ay =
                                mid_y - t.header_font_size * 0.35;
                            let points = if s.dir == SortDir::Asc {
                                vec![
                                    [ax, ay - AH],
                                    [ax + AW, ay + AH * 0.6],
                                    [ax - AW, ay + AH * 0.6],
                                ]
                            } else {
                                vec![
                                    [ax, ay + AH],
                                    [ax + AW, ay - AH * 0.6],
                                    [ax - AW, ay - AH * 0.6],
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
                        y1: 0.0,
                        x2: sep_x,
                        y2: model.header_height,
                        color: t.header_border,
                        width: 1.0,
                    }));
                }
            };

        // Scrollable column headers
        render_col_headers(&mut frame, col_start..col_end);
        // Pinned column headers (on top)
        if pinned_count > 0 {
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
                let ry = model.row_top(ri) - sy;
                if ry + model.row_height < model.header_height || ry > vp.height
                {
                    continue;
                }

                let is_selected = sel
                    .range()
                    .map_or(false, |(tl, br)| ri >= tl.row && ri <= br.row);

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

        // ── vertical scrollbar ───────────────────────────────────────────────
        if let Some(sb) = ScrollbarGeom::compute(
            vp.scroll_y,
            vp.width,
            vp.height,
            model.header_height,
            model.total_height(),
            t.scrollbar_width,
        ) {
            // ── arrow buttons background ──────────────────────────────────────
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

            // ── arrow icons ───────────────────────────────────────────────────
            let cx = sb.track_x + sb.track_w * 0.5;
            let arrow_size = (sb.track_w * 0.45).max(3.0);

            // Up arrow ▲
            let mid_up = sb.up_btn_y + sb.arrow_h * 0.5;
            frame.push(ScenePrimitive::Polygon(PolygonPrimitive {
                points: vec![
                    [cx, mid_up - arrow_size * 0.45],
                    [cx + arrow_size, mid_up + arrow_size * 1.0],
                    [cx - arrow_size, mid_up + arrow_size * 1.0],
                ],
                fill: t.scrollbar_thumb,
                corner_radius: arrow_size * 0.25,
            }));

            // Down arrow ▼
            let mid_dn = sb.down_btn_y + sb.arrow_h * 0.5;
            frame.push(ScenePrimitive::Polygon(PolygonPrimitive {
                points: vec![
                    [cx, mid_dn + arrow_size * 0.45],
                    [cx + arrow_size, mid_dn - arrow_size * 1.0],
                    [cx - arrow_size, mid_dn - arrow_size * 1.0],
                ],
                fill: t.scrollbar_thumb,
                corner_radius: arrow_size * 0.25,
            }));

            // ── track ─────────────────────────────────────────────────────────
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

            // ── thumb (inset 2px on each side) ────────────────────────────────
            const INSET: f64 = 2.0;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: sb.track_x + INSET,
                y: sb.thumb_y + INSET,
                width: (sb.track_w - INSET * 2.0).max(2.0),
                height: (sb.thumb_h - INSET * 2.0).max(4.0),
                fill: t.scrollbar_thumb,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: t.scrollbar_radius,
            }));
        }

        // ── horizontal scrollbar ─────────────────────────────────────────────
        let vsb_w = if ScrollbarGeom::compute(
            vp.scroll_y, vp.width, vp.height,
            model.header_height, model.total_height(), t.scrollbar_width,
        ).is_some() { t.scrollbar_width } else { 0.0 };

        if let Some(hsb) = HScrollbarGeom::compute(
            vp.scroll_x,
            vp.width,
            vp.height,
            rnw,
            model.total_width(),
            vsb_w,
            t.scrollbar_width,
        ) {
            // ── arrow buttons background ──────────────────────────────────────
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

            // ── arrow icons ───────────────────────────────────────────────────
            let cy = hsb.track_y + hsb.track_h * 0.5;
            let arrow_size = (hsb.track_h * 0.45).max(3.0);

            // Left arrow ◀
            let mid_left = hsb.left_btn_x + hsb.arrow_w * 0.5;
            frame.push(ScenePrimitive::Polygon(PolygonPrimitive {
                points: vec![
                    [mid_left - arrow_size * 0.45, cy],
                    [mid_left + arrow_size * 1.0,  cy - arrow_size],
                    [mid_left + arrow_size * 1.0,  cy + arrow_size],
                ],
                fill: t.scrollbar_thumb,
                corner_radius: arrow_size * 0.25,
            }));

            // Right arrow ▶
            let mid_right = hsb.right_btn_x + hsb.arrow_w * 0.5;
            frame.push(ScenePrimitive::Polygon(PolygonPrimitive {
                points: vec![
                    [mid_right + arrow_size * 0.45, cy],
                    [mid_right - arrow_size * 1.0,  cy - arrow_size],
                    [mid_right - arrow_size * 1.0,  cy + arrow_size],
                ],
                fill: t.scrollbar_thumb,
                corner_radius: arrow_size * 0.25,
            }));

            // ── track ─────────────────────────────────────────────────────────
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

            // ── thumb (inset 2px on each side) ────────────────────────────────
            const INSET: f64 = 2.0;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: hsb.thumb_x + INSET,
                y: hsb.track_y + INSET,
                width: (hsb.thumb_w - INSET * 2.0).max(4.0),
                height: (hsb.track_h - INSET * 2.0).max(2.0),
                fill: t.scrollbar_thumb,
                stroke: None,
                stroke_width: 0.0,
                corner_radius: t.scrollbar_radius,
            }));
        }

        frame
    }
}
