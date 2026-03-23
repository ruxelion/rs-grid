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
