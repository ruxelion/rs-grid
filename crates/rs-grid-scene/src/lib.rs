//! Renderer-agnostic scene graph: converts a `GridState`
//! into an ordered list of drawing primitives.

/// Scene builder — turns `GridState` + `Theme` into a frame.
pub mod builder;
/// Immutable per-frame snapshot of drawing primitives.
pub mod frame;
/// Renderable primitive types (rect, text, line, polygon, image).
pub mod primitives;
/// Visual theme: colors, typography, and spacing.
pub mod theme;

pub use theme::Theme;
