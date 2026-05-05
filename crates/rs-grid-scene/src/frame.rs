use crate::primitives::ScenePrimitive;

/// A hit-testable zone for a rendered cell button.
///
/// Populated by `SceneBuilder::build()` alongside the drawing
/// primitives.  The web layer reads this list on mousedown to
/// detect sub-cell button clicks before dispatching cell
/// selection.
#[derive(Debug, Clone)]
pub struct ButtonZone {
    /// Data row index.
    pub row: u64,
    /// Column index (use to look up the column key in
    /// `GridState::model.columns`).
    pub col: usize,
    /// Button identifier from [`ButtonDef::id`].
    pub button_id: String,
    /// Left edge in viewport-relative logical pixels.
    pub x: f64,
    /// Top edge in viewport-relative logical pixels.
    pub y: f64,
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
}

impl ButtonZone {
    /// Returns `true` when `(vx, vy)` falls inside this zone.
    #[inline]
    pub fn contains(&self, vx: f64, vy: f64) -> bool {
        vx >= self.x
            && vx < self.x + self.width
            && vy >= self.y
            && vy < self.y + self.height
    }
}

/// A complete, immutable snapshot of what should be drawn for one frame.
#[derive(Debug, Clone, Default)]
pub struct SceneFrame {
    /// Ordered drawing primitives (back-to-front).
    pub primitives: Vec<ScenePrimitive>,
    /// Button hit-zones for the current frame.
    ///
    /// Populated by `SceneBuilder`; consumed by the web event
    /// layer for mousedown routing.
    pub button_zones: Vec<ButtonZone>,
    /// Viewport width in logical pixels.
    pub viewport_width: f64,
    /// Viewport height in logical pixels.
    pub viewport_height: f64,
    /// Device pixel ratio at the time the frame was built.
    pub dpr: f64,
}

impl SceneFrame {
    /// Create an empty frame with the given dimensions and DPR.
    pub fn new(viewport_width: f64, viewport_height: f64, dpr: f64) -> Self {
        Self {
            primitives: Vec::new(),
            button_zones: Vec::new(),
            viewport_width,
            viewport_height,
            dpr,
        }
    }

    /// Append a primitive to the frame.
    #[inline]
    pub fn push(&mut self, p: ScenePrimitive) {
        self.primitives.push(p);
    }

    /// Append a button zone to the frame.
    #[inline]
    pub fn push_button_zone(&mut self, z: ButtonZone) {
        self.button_zones.push(z);
    }

    /// Return the first [`ButtonZone`] that contains `(vx, vy)`.
    pub fn hit_button(
        &self,
        vx: f64,
        vy: f64,
    ) -> Option<&ButtonZone> {
        self.button_zones.iter().find(|z| z.contains(vx, vy))
    }

    /// Return the number of primitives in this frame.
    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{
        Color, LinePrimitive, RectPrimitive, TextPrimitive,
    };

    #[test]
    fn new_frame_is_empty() {
        let f = SceneFrame::new(800.0, 600.0, 2.0);
        assert_eq!(f.primitive_count(), 0);
        assert!(f.primitives.is_empty());
    }

    #[test]
    fn new_frame_stores_dimensions() {
        let f = SceneFrame::new(1024.0, 768.0, 1.5);
        assert_eq!(f.viewport_width, 1024.0);
        assert_eq!(f.viewport_height, 768.0);
        assert_eq!(f.dpr, 1.5);
    }

    #[test]
    fn push_increments_count() {
        let mut f = SceneFrame::new(100.0, 100.0, 1.0);
        f.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            fill: Color::rgb(0, 0, 0),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
        assert_eq!(f.primitive_count(), 1);

        f.push(ScenePrimitive::Line(LinePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            color: Color::rgb(0, 0, 0),
            width: 1.0,
        }));
        assert_eq!(f.primitive_count(), 2);
    }

    #[test]
    fn primitives_preserve_push_order() {
        let mut f = SceneFrame::new(100.0, 100.0, 1.0);
        f.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            fill: Color::rgb(255, 0, 0),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
        f.push(ScenePrimitive::Text(TextPrimitive {
            x: 0.0,
            y: 0.0,
            text: "hi".into(),
            color: Color::rgb(0, 0, 0),
            font_size: 12.0,
            bold: false,
            clip: None,
            align: crate::primitives::TextAlign::Left,
            max_width: None,
        }));

        assert!(matches!(f.primitives[0], ScenePrimitive::Rect(_)));
        assert!(matches!(f.primitives[1], ScenePrimitive::Text(_)));
    }

    #[test]
    fn default_frame_is_zero() {
        let f = SceneFrame::default();
        assert_eq!(f.viewport_width, 0.0);
        assert_eq!(f.viewport_height, 0.0);
        assert_eq!(f.dpr, 0.0);
        assert_eq!(f.primitive_count(), 0);
    }

    #[test]
    fn frame_clone() {
        let mut f = SceneFrame::new(800.0, 600.0, 1.0);
        f.push(ScenePrimitive::Rect(RectPrimitive {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            fill: Color::rgb(0, 0, 0),
            stroke: None,
            stroke_width: 0.0,
            corner_radius: 0.0,
            clip: None,
        }));
        let f2 = f.clone();
        assert_eq!(f2.primitive_count(), 1);
        assert_eq!(f2.viewport_width, 800.0);
    }

    #[test]
    fn frame_debug_does_not_panic() {
        let f = SceneFrame::new(800.0, 600.0, 1.0);
        let _ = format!("{:?}", f);
    }
}
