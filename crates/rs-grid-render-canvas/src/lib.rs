//! Canvas2D rendering backend for rs-grid.
//!
//! Consumes a [`SceneFrame`](rs_grid_scene::frame::SceneFrame)
//! and draws onto a `CanvasRenderingContext2d` via
//! wasm-bindgen. Sits in the middle of the dependency chain
//! (`core → scene → **render-canvas** → web → leptos`).

/// Renders a `SceneFrame` onto a `CanvasRenderingContext2d`.
pub mod renderer;
