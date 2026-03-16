use crate::primitives::ScenePrimitive;

/// A complete, immutable snapshot of what should be drawn for one frame.
#[derive(Debug, Clone, Default)]
pub struct SceneFrame {
    pub primitives: Vec<ScenePrimitive>,
    pub viewport_width: f64,
    pub viewport_height: f64,
    /// Device pixel ratio at the time the frame was built.
    pub dpr: f64,
}

impl SceneFrame {
    pub fn new(viewport_width: f64, viewport_height: f64, dpr: f64) -> Self {
        Self {
            primitives: Vec::new(),
            viewport_width,
            viewport_height,
            dpr,
        }
    }

    #[inline]
    pub fn push(&mut self, p: ScenePrimitive) {
        self.primitives.push(p);
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}
