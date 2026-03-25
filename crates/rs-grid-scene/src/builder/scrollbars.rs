use rs_grid_core::{
    model::GridModel,
    scrollbar::{HScrollbarGeom, ScrollbarGeom},
    viewport::ViewportState,
};

use crate::{
    frame::SceneFrame,
    primitives::{
        Color, PolygonPrimitive, RectPrimitive, ScenePrimitive,
    },
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
            }));
        }

        // Arrow icons
        let cx = sb.track_x + sb.track_w * 0.5;
        let arrow_size = (sb.track_w * 0.45).max(3.0);

        // Up arrow ▲
        let mid_up = sb.up_btn_y + sb.arrow_h * 0.5;
        emit_scrollbar_arrow(
            frame, cx, mid_up, arrow_size, -1.0, true,
            t.scrollbar_thumb,
        );

        // Down arrow ▼
        let mid_dn = sb.down_btn_y + sb.arrow_h * 0.5;
        emit_scrollbar_arrow(
            frame, cx, mid_dn, arrow_size, 1.0, true,
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

    // ── horizontal scrollbar ─────────────────────────────────
    // Reserve space for the vertical scrollbar width if visible.
    let vsb_w = if vsb.is_some() { t.scrollbar_width } else { 0.0 };

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
        emit_scrollbar_arrow(
            frame, cy, mid_left, arrow_size, -1.0, false,
            t.scrollbar_thumb,
        );

        // Right arrow ▶
        let mid_right = hsb.right_btn_x + hsb.arrow_w * 0.5;
        emit_scrollbar_arrow(
            frame, cy, mid_right, arrow_size, 1.0, false,
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
