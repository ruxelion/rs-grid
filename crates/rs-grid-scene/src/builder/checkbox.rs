use rs_grid_core::row_check::CheckboxTriState;

use crate::{
    frame::SceneFrame,
    primitives::{Color, LinePrimitive, RectPrimitive, ScenePrimitive},
    theme::Theme,
};

/// Emit a checkbox (row cell or header) centered inside the given band.
///
/// `Checked`/`Indeterminate` fill the box with `Theme::checkbox_checked_bg`;
/// `Checked` draws a check mark, `Indeterminate` a dash, both as simple
/// `Line` segments — no new primitive type needed, mirroring how the
/// sort-arrow reuses `Polygon` rather than adding a dedicated shape.
pub(super) fn emit_checkbox(
    frame: &mut SceneFrame,
    band_x: f64,
    band_y: f64,
    band_width: f64,
    band_height: f64,
    t: &Theme,
    tri: CheckboxTriState,
) {
    let s = t.checkbox_size;
    let bx = band_x + (band_width - s) / 2.0;
    let by = band_y + (band_height - s) / 2.0;

    let fill = match tri {
        CheckboxTriState::Unchecked => Color::rgba(0, 0, 0, 0),
        _ => t.checkbox_checked_bg,
    };
    frame.push(ScenePrimitive::Rect(RectPrimitive {
        x: bx,
        y: by,
        width: s,
        height: s,
        fill,
        stroke: Some(t.checkbox_border),
        stroke_width: t.checkbox_border_width,
        corner_radius: t.checkbox_radius,
        clip: None,
    }));

    let mark_width = (t.checkbox_border_width + 0.5).max(2.0);
    match tri {
        CheckboxTriState::Checked => {
            // Check mark as two line segments (short stroke, then long).
            let (x1, y1) = (bx + s * 0.22, by + s * 0.55);
            let (x2, y2) = (bx + s * 0.42, by + s * 0.75);
            let (x3, y3) = (bx + s * 0.80, by + s * 0.28);
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1,
                y1,
                x2,
                y2,
                color: t.checkbox_mark_color,
                width: mark_width,
            }));
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: x2,
                y1: y2,
                x2: x3,
                y2: y3,
                color: t.checkbox_mark_color,
                width: mark_width,
            }));
        }
        CheckboxTriState::Indeterminate => {
            let y_mid = by + s * 0.5;
            frame.push(ScenePrimitive::Line(LinePrimitive {
                x1: bx + s * 0.2,
                y1: y_mid,
                x2: bx + s * 0.8,
                y2: y_mid,
                color: t.checkbox_mark_color,
                width: mark_width,
            }));
        }
        CheckboxTriState::Unchecked => {}
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use rs_grid_core::row_check::CheckboxTriState;

    use super::emit_checkbox;
    use crate::{frame::SceneFrame, primitives::ScenePrimitive, theme::Theme};

    fn make_frame() -> SceneFrame {
        SceneFrame::new(800.0, 600.0, 1.0)
    }

    #[test]
    fn unchecked_draws_box_only_no_mark() {
        let mut frame = make_frame();
        let t = Theme::light();
        emit_checkbox(
            &mut frame,
            0.0,
            0.0,
            42.0,
            42.0,
            &t,
            CheckboxTriState::Unchecked,
        );
        assert_eq!(frame.primitive_count(), 1);
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill.a, 0, "unchecked box must be unfilled");
                assert_eq!(r.stroke, Some(t.checkbox_border));
            }
            _ => panic!("expected Rect"),
        }
    }

    #[test]
    fn checked_fills_box_and_draws_two_line_check_mark() {
        let mut frame = make_frame();
        let t = Theme::light();
        emit_checkbox(
            &mut frame,
            0.0,
            0.0,
            42.0,
            42.0,
            &t,
            CheckboxTriState::Checked,
        );
        assert_eq!(frame.primitive_count(), 3, "1 box + 2 mark segments");
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.checkbox_checked_bg)
            }
            _ => panic!("expected Rect"),
        }
        let line_count = frame
            .primitives
            .iter()
            .filter(|p| matches!(p, ScenePrimitive::Line(_)))
            .count();
        assert_eq!(line_count, 2, "check mark is two connected segments");
    }

    #[test]
    fn indeterminate_fills_box_and_draws_one_dash() {
        let mut frame = make_frame();
        let t = Theme::light();
        emit_checkbox(
            &mut frame,
            0.0,
            0.0,
            42.0,
            42.0,
            &t,
            CheckboxTriState::Indeterminate,
        );
        assert_eq!(frame.primitive_count(), 2, "1 box + 1 dash segment");
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.fill, t.checkbox_checked_bg)
            }
            _ => panic!("expected Rect"),
        }
        match &frame.primitives[1] {
            ScenePrimitive::Line(l) => {
                assert_eq!(l.y1, l.y2, "indeterminate dash is horizontal");
            }
            _ => panic!("expected Line"),
        }
    }

    #[test]
    fn box_is_centered_within_the_band() {
        let mut frame = make_frame();
        let t = Theme::light();
        // Band wider/taller than the checkbox itself — the box must sit
        // centered, not flush with the band's top-left corner.
        emit_checkbox(
            &mut frame,
            10.0,
            20.0,
            60.0,
            60.0,
            &t,
            CheckboxTriState::Unchecked,
        );
        match &frame.primitives[0] {
            ScenePrimitive::Rect(r) => {
                assert_eq!(r.x, 10.0 + (60.0 - t.checkbox_size) / 2.0);
                assert_eq!(r.y, 20.0 + (60.0 - t.checkbox_size) / 2.0);
            }
            _ => panic!("expected Rect"),
        }
    }
}
