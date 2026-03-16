use rs_grid_core::{scrollbar::ScrollbarGeom, state::GridState};

use crate::{
    frame::SceneFrame,
    primitives::{Color, LinePrimitive, RectPrimitive, ScenePrimitive, TextPrimitive},
};

// ── palette ──────────────────────────────────────────────────────────────────

const BG: Color = Color::rgb(255, 255, 255);
const HEADER_BG: Color = Color::rgb(242, 242, 247);
const HEADER_TEXT: Color = Color::rgb(90, 90, 100);
const CELL_TEXT: Color = Color::rgb(20, 20, 20);
const GRID_LINE: Color = Color::rgb(210, 210, 215);
const HEADER_BORDER: Color = Color::rgb(180, 180, 190);
const SELECTION_FILL: Color = Color::rgba(59, 130, 246, 50);
const SELECTION_BORDER: Color = Color::rgba(59, 130, 246, 200);

// ── builder ───────────────────────────────────────────────────────────────────

/// Transforms a `GridState` snapshot into a `SceneFrame`.
///
/// Instantiate once and reuse; all state is read from `GridState` on each
/// `build()` call.
pub struct SceneBuilder {
    pub dpr: f64,
    pub cell_padding: f64,
    pub font_size: f64,
    pub header_font_size: f64,
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self {
            dpr: 1.0,
            cell_padding: 8.0,
            font_size: 13.0,
            header_font_size: 12.0,
        }
    }
}

impl SceneBuilder {
    pub fn new(dpr: f64) -> Self {
        Self {
            dpr,
            ..Default::default()
        }
    }

    /// Build a complete `SceneFrame` from the current `GridState`.
    pub fn build(&self, state: &GridState) -> SceneFrame {
        let vp = &state.viewport;
        let model = &state.model;
        let sel = &state.selection;

        let mut frame = SceneFrame::new(vp.width, vp.height, self.dpr);

        // ── background ───────────────────────────────────────────────────────
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: vp.width,
            height: vp.height,
            fill: BG,
            stroke: None,
            stroke_width: 0.0,
        }));

        let col_widths: Vec<f64> = model.columns.iter().map(|c| c.width).collect();
        let (col_start, col_end) =
            vp.visible_columns(&model.column_offsets, &col_widths);
        let (row_start, row_end) =
            vp.visible_rows(model.data.row_count(), model.row_height, model.header_height);

        let sx = vp.scroll_x;
        let sy = vp.scroll_y;

        // ── header background ────────────────────────────────────────────────
        frame.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: vp.width,
            height: model.header_height,
            fill: HEADER_BG,
            stroke: None,
            stroke_width: 0.0,
        }));

        // ── column headers ───────────────────────────────────────────────────
        for ci in col_start..col_end {
            let col = &model.columns[ci];
            let cx = model.column_offsets.offsets[ci] - sx;
            let mid_y = model.header_height * 0.5 + self.header_font_size * 0.35;

            // Header label
            frame.push(ScenePrimitive::Text(TextPrimitive {
                x: cx + self.cell_padding,
                y: mid_y,
                text: col.label.clone(),
                color: HEADER_TEXT,
                font_size: self.header_font_size,
                clip: Some([cx, 0.0, col.width, model.header_height]),
            }));

            // Column separator in header
            let sep_x = cx + col.width - 0.5;
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: sep_x,
                y1: 0.0,
                x2: sep_x,
                y2: model.header_height,
                color: HEADER_BORDER,
                width: 1.0,
            }));
        }

        // Header bottom border
        frame.push(ScenePrimitive::Line(LinePrimitive {
            x1: 0.0,
            y1: model.header_height - 0.5,
            x2: vp.width,
            y2: model.header_height - 0.5,
            color: HEADER_BORDER,
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

            let mid_y = ry + model.row_height * 0.5 + self.font_size * 0.35;

            for ci in col_start..col_end {
                let col = &model.columns[ci];
                let cx = model.column_offsets.offsets[ci] - sx;

                // Selection fill
                if sel.is_selected(ri, ci) {
                    frame.push(ScenePrimitive::Rect(RectPrimitive {
                        x: cx,
                        y: ry,
                        width: col.width,
                        height: model.row_height,
                        fill: SELECTION_FILL,
                        stroke: Some(SELECTION_BORDER),
                        stroke_width: 1.0,
                    }));
                }

                // Cell text
                if let Some(text) = model.get_cell(ri, &col.key) {
                    if !text.is_empty() {
                        frame.push(ScenePrimitive::Text(TextPrimitive {
                            x: cx + self.cell_padding,
                            y: mid_y,
                            text,
                            color: CELL_TEXT,
                            font_size: self.font_size,
                            clip: Some([cx, ry, col.width, model.row_height]),
                        }));
                    }
                }

                // Vertical grid line
                let vline_x = cx + col.width - 0.5;
                frame.push(ScenePrimitive::Line(LinePrimitive {
                    x1: vline_x,
                    y1: ry,
                    x2: vline_x,
                    y2: ry + model.row_height,
                    color: GRID_LINE,
                    width: 1.0,
                }));
            }

            // Horizontal grid line
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: 0.0,
                y1: ry + model.row_height - 0.5,
                x2: vp.width,
                y2: ry + model.row_height - 0.5,
                color: GRID_LINE,
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
        ) {
            // Track
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: sb.track_x,
                y: sb.track_y,
                width: sb.track_w,
                height: sb.track_h,
                fill: Color::rgba(0, 0, 0, 18),
                stroke: None,
                stroke_width: 0.0,
            }));

            // Thumb (inset 2px on each side)
            const INSET: f64 = 2.0;
            frame.push(ScenePrimitive::Rect(RectPrimitive {
                x: sb.track_x + INSET,
                y: sb.thumb_y + INSET,
                width: (sb.track_w - INSET * 2.0).max(2.0),
                height: (sb.thumb_h - INSET * 2.0).max(4.0),
                fill: Color::rgba(90, 90, 100, 170),
                stroke: None,
                stroke_width: 0.0,
            }));
        }

        frame
    }
}
