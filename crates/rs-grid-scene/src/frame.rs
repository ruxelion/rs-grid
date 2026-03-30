use crate::primitives::ScenePrimitive;

/// A complete, immutable snapshot of what should be drawn for one frame.
#[derive(Debug, Clone, Default)]
pub struct SceneFrame {
    /// Ordered drawing primitives (back-to-front).
    pub primitives: Vec<ScenePrimitive>,
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
