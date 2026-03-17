use rs_grid_core::{scrollbar::ScrollbarGeom, state::GridState};

use crate::{
    frame::SceneFrame,
    primitives::{LinePrimitive, RectPrimitive, ScenePrimitive, TextAlign, TextPrimitive},
    theme::Theme,
};

// ── builder ───────────────────────────────────────────────────────────────────

/// Transforms a `GridState` snapshot into a `SceneFrame`.
///
/// Instantiate once and reuse; all state is read from `GridState` on each
/// `build()` call.
pub struct SceneBuilder {
    /// Device pixel ratio — hardware property, not a theme property.
    pub dpr:   f64,
    pub theme: Theme,
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self {
            dpr:   1.0,
            theme: Theme::default(),
        }
    }
}

impl SceneBuilder {
    pub fn new(dpr: f64) -> Self {
        Self { dpr, theme: Theme::default() }
    }

    pub fn with_theme(dpr: f64, theme: Theme) -> Self {
        Self { dpr, theme }
    }

    /// Build a complete `SceneFrame` from the current `GridState`.
    pub fn build(&self, state: &GridState) -> SceneFrame {
        let vp  = &state.viewport;
        let model = &state.model;
        let sel = &state.selection;
        let t   = &self.theme;

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

        let col_widths: Vec<f64> = model.columns.iter().map(|c| c.width).collect();
        let (col_start, col_end) =
            vp.visible_columns(&model.column_offsets, &col_widths);
        let (row_start, row_end) =
            vp.visible_rows(model.data.row_count(), model.row_height, model.header_height);

        let sx  = vp.scroll_x;
        let sy  = vp.scroll_y;
        let rnw = model.row_number_width; // row-number gutter width

        // ── header background ────────────────────────────────────────────────
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

        // ── column headers ───────────────────────────────────────────────────
        for ci in col_start..col_end {
            let col = &model.columns[ci];
            let cx = model.column_offsets.offsets[ci] - sx + rnw;
            let mid_y = model.header_height * 0.5 + t.header_font_size * 0.35;

            // Header label
            frame.push(ScenePrimitive::Text(TextPrimitive {
                x: cx + t.cell_padding,
                y: mid_y,
                text: col.label.clone(),
                color: t.header_text,
                font_size: t.header_font_size,
                clip: Some([cx, 0.0, col.width, model.header_height]),
                align: TextAlign::Left,
            }));

            // Column separator in header
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

        // Header bottom border
        frame.push(ScenePrimitive::Line(LinePrimitive {
            x1: 0.0,
            y1: model.header_height - 0.5,
            x2: vp.width,
            y2: model.header_height - 0.5,
            color: t.header_border,
            width: 1.0,
        }));

        // ── data rows ────────────────────────────────────────────────────────
        for ri in row_start..row_end {
            let ry = model.row_top(ri) - sy;

            // Skip rows that are fully outside the clip zone (overscan may
            // produce rows above the header).
            if ry + model.row_height < model.header_height || ry > vp.height {
                continue;
            }

            let mid_y = ry + model.row_height * 0.5 + t.font_size * 0.35;

            for ci in col_start..col_end {
                let col = &model.columns[ci];
                let cx = model.column_offsets.offsets[ci] - sx + rnw;

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

        // ── selection outer border ───────────────────────────────────────────
        if let Some((tl, br)) = sel.range() {
            let x1 = model.column_offsets.offsets[tl.col] - sx + rnw;
            let y1 = model.row_top(tl.row) - sy;
            let x2 = model.column_offsets.offsets[br.col] - sx + rnw + model.columns[br.col].width;
            let y2 = model.row_top(br.row) - sy + model.row_height;

            // top
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1, y1: y1 + 0.5, x2, y2: y1 + 0.5,
                color: t.selection_border, width: 1.0,
            }));
            // bottom
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1, y1: y2 - 0.5, x2, y2: y2 - 0.5,
                color: t.selection_border, width: 1.0,
            }));
            // left
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x1 + 0.5, y1, x2: x1 + 0.5, y2,
                color: t.selection_border, width: 1.0,
            }));
            // right
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x2 - 0.5, y1, x2: x2 - 0.5, y2,
                color: t.selection_border, width: 1.0,
            }));
        }

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
                if ry + model.row_height < model.header_height || ry > vp.height {
                    continue;
                }

                let is_selected = sel.range()
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

            // Thumb (inset 2px on each side)
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

        frame
    }
}
