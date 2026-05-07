use rs_grid_core::{
    model::GridModel,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    viewport::ViewportState,
};

use crate::{
    frame::SceneFrame,
    primitives::{Color, PolygonPrimitive, RectPrimitive, ScenePrimitive},
    theme::Theme,
};

/// Emit the vertical and horizontal scrollbar primitives.
///
/// Computes `ScrollbarGeom` once and reuses it for both the
/// vertical render and the horizontal scrollbar's vsb_w
/// calculation, avoiding the double-compute present in the
/// original inlined code.
pub(super) fn emit_scrollbars(
    frame: &mut SceneFrame,
    vp: &ViewportState,
    model: &GridModel,
    rnw: f64,
    t: &Theme,
) {
    // ── vertical scrollbar ───────────────────────────────────
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
                clip: None,
            }));
        }

        // Arrow icons
        let cx = sb.track_x + sb.track_w * 0.5;
        let arrow_size = (sb.track_w * 0.45).max(3.0);

        // Up arrow ▲
        let mid_up = sb.up_btn_y + sb.arrow_h * 0.5;
        emit_scrollbar_arrow(
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
        emit_scrollbar_arrow(
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
            clip: None,
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
            clip: None,
        }));
    }

    // ── horizontal scrollbar ─────────────────────────────────
    // Reserve space for the vertical scrollbar width if visible.
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
                clip: None,
            }));
        }

        // Arrow icons
        let cy = hsb.track_y + hsb.track_h * 0.5;
        let arrow_size = (hsb.track_h * 0.45).max(3.0);

        // Left arrow ◀
        let mid_left = hsb.left_btn_x + hsb.arrow_w * 0.5;
        emit_scrollbar_arrow(
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
        emit_scrollbar_arrow(
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
            clip: None,
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
            clip: None,
        }));
    }
}

/// Emit a single scrollbar arrow as a `PolygonPrimitive`.
///
/// - `cross`    — center coordinate on the perpendicular axis.
/// - `mid`      — center of the button along its scroll axis.
/// - `size`     — half-size of the arrow triangle.
/// - `dir`      — `-1.0` = up/left, `+1.0` = down/right.
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

#[cfg(test)]
mod tests {
    use rs_grid_core::{
        column::ColumnDef, model::GridModel, row::RowRecord,
        viewport::ViewportState,
    };

    use crate::{frame::SceneFrame, primitives::ScenePrimitive, theme::Theme};

    use super::emit_scrollbars;

    // ── helpers ──────────────────────────────────────────────

    /// Tall grid: 100 rows × 40 px = 4000 px total height.
    /// Viewport height 300 px → vertical scrollbar appears.
    fn make_tall_model() -> GridModel {
        let cols = vec![ColumnDef::new("a", "A", 200.0)];
        let rows: Vec<RowRecord> = (0..100).map(RowRecord::new).collect();
        GridModel::new(cols, rows, 40.0, 48.0)
    }

    /// Wide grid: 5 columns × 300 px = 1500 px total width.
    /// Viewport width 400 px → horizontal scrollbar appears.
    fn make_wide_model() -> GridModel {
        let cols = vec![
            ColumnDef::new("a", "A", 300.0),
            ColumnDef::new("b", "B", 300.0),
            ColumnDef::new("c", "C", 300.0),
            ColumnDef::new("d", "D", 300.0),
            ColumnDef::new("e", "E", 300.0),
        ];
        let rows: Vec<RowRecord> = (0..5).map(RowRecord::new).collect();
        GridModel::new(cols, rows, 40.0, 48.0)
    }

    // ── vertical scrollbar ───────────────────────────────────

    #[test]
    fn emit_scrollbars_vertical_emits_rects_and_polygons() {
        let mut frame = SceneFrame::new(400.0, 300.0, 1.0);
        let vp = ViewportState::new(400.0, 300.0);
        let model = make_tall_model();
        let t = Theme::light();

        emit_scrollbars(&mut frame, &vp, &model, 0.0, &t);

        // Vertical scrollbar: 2 btn bg + 2 arrows + 1 track
        // + 1 thumb = 6 primitives minimum
        assert!(
            frame.primitive_count() >= 6,
            "expected ≥6 primitives, got {}",
            frame.primitive_count()
        );
        let has_polygon = frame
            .primitives
            .iter()
            .any(|p| matches!(p, ScenePrimitive::Polygon(_)));
        assert!(has_polygon, "expected arrow polygons");
    }

    // ── horizontal scrollbar ─────────────────────────────────

    #[test]
    fn emit_scrollbars_horizontal_emits_primitives() {
        let mut frame = SceneFrame::new(400.0, 300.0, 1.0);
        let vp = ViewportState::new(400.0, 300.0);
        let model = make_wide_model();
        let t = Theme::light();

        emit_scrollbars(&mut frame, &vp, &model, 0.0, &t);

        // Horizontal scrollbar must emit rects + polygons
        let rect_count = frame
            .primitives
            .iter()
            .filter(|p| matches!(p, ScenePrimitive::Rect(_)))
            .count();
        let poly_count = frame
            .primitives
            .iter()
            .filter(|p| matches!(p, ScenePrimitive::Polygon(_)))
            .count();
        // At least 3 rects (track + thumb + btn bg) for hsb
        assert!(rect_count >= 3, "expected ≥3 rects, got {rect_count}");
        // At least 2 polygons (left + right arrows) for hsb
        assert!(poly_count >= 2, "expected ≥2 polygons, got {poly_count}");
    }

    #[test]
    fn emit_scrollbars_horizontal_uses_horizontal_arrows() {
        let mut frame = SceneFrame::new(400.0, 300.0, 1.0);
        let vp = ViewportState::new(400.0, 300.0);
        // Wide but not tall: only horizontal scrollbar
        let cols = vec![
            ColumnDef::new("a", "A", 300.0),
            ColumnDef::new("b", "B", 300.0),
        ];
        let rows = vec![RowRecord::new(0)];
        // row_height 40, 1 row = 40 px total, fits in 300 →
        // no vertical scrollbar
        let model = GridModel::new(cols, rows, 40.0, 48.0);
        let t = Theme::light();

        emit_scrollbars(&mut frame, &vp, &model, 0.0, &t);

        // With no vertical scrollbar, all polygons are
        // horizontal arrows (points differ from vertical).
        let polygons: Vec<_> = frame
            .primitives
            .iter()
            .filter_map(|p| match p {
                ScenePrimitive::Polygon(poly) => Some(poly),
                _ => None,
            })
            .collect();
        // 2 horizontal arrows expected
        assert_eq!(polygons.len(), 2);
    }

    // ── no scrollbar when content fits ───────────────────────

    #[test]
    fn emit_scrollbars_no_scrollbar_when_content_fits() {
        let mut frame = SceneFrame::new(800.0, 600.0, 1.0);
        let vp = ViewportState::new(800.0, 600.0);
        // 1 column, 1 row — easily fits the viewport
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let rows = vec![RowRecord::new(0)];
        let model = GridModel::new(cols, rows, 40.0, 48.0);
        let t = Theme::light();

        emit_scrollbars(&mut frame, &vp, &model, 0.0, &t);

        assert_eq!(
            frame.primitive_count(),
            0,
            "no scrollbar expected when content fits"
        );
    }
}
