//! Renderer-agnostic scene graph: converts a `GridState`
//! into an ordered list of drawing primitives.
//!
//! Sits between `rs-grid-core` and the rendering backends
//! in the dependency chain (`core → **scene** →
//! render-canvas → web → leptos`).
//!
//! Key types: [`SceneBuilder`](builder::SceneBuilder),
//! [`SceneFrame`](frame::SceneFrame),
//! [`ScenePrimitive`](primitives::ScenePrimitive),
//! [`Theme`].

/// Scene builder — turns `GridState` + `Theme` into a frame.
pub mod builder;
/// Generic visual-style types for `CellFormat::Styled` rendering.
pub mod class_map;
/// CSS custom-property (de)serialization for [`Theme`] (single source of
/// truth for the `--rs-grid-*` variables).
pub mod css_vars;
/// Immutable per-frame snapshot of drawing primitives.
pub mod frame;
/// Renderable primitive types (rect, text, line, polygon, image).
pub mod primitives;
/// Sample `GridState`s shared by `scene-dump` and the snapshot tests.
#[doc(hidden)]
pub mod sample_scenes;
/// Visual theme: colors, typography, and spacing.
pub mod theme;

pub use theme::Theme;
