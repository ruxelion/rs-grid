//! Generic visual-style types for `CellFormat::Styled` rendering.
//!
//! [`CellElementStyle`] describes how the scene builder should
//! render a single styled element (background rect + text).
//!
//! [`ClassResolver`] is the type of a class-name resolver function.
//! Provide your own implementation — e.g. a DaisyUI resolver from
//! `example-common` — and register it on [`SceneBuilder`] via
//! [`SceneBuilder::set_class_resolver`]. Without a resolver,
//! styled elements render as plain text (no background, no padding).
//!
//! [`SceneBuilder`]: crate::builder::SceneBuilder

use crate::primitives::Color;

// ── CellElementStyle ─────────────────────────────────────

/// Visual style resolved from a set of class names.
///
/// Consumed by the scene builder to emit
/// `RectPrimitive` + `TextPrimitive` pairs.
#[derive(Debug, Clone)]
pub struct CellElementStyle {
    /// Text colour override. `None` → use `Theme::cell_text`.
    pub color: Option<Color>,
    /// Element background. `None` → transparent.
    pub background: Option<Color>,
    /// Render text in bold weight.
    pub bold: bool,
    /// Render text in italic style.
    pub italic: bool,
    /// Corner radius (logical px).
    pub border_radius: f64,
    /// Horizontal inner padding (logical px each side).
    pub padding_x: f64,
    /// Vertical inner padding (logical px each side).
    pub padding_y: f64,
    /// Stroke colour. `None` → no stroke.
    pub border_color: Option<Color>,
    /// Stroke width (logical px).
    pub border_width: f64,
    /// Font-size delta relative to `Theme::font_size`.
    pub font_size_delta: f64,
}

impl Default for CellElementStyle {
    fn default() -> Self {
        Self {
            color: None,
            background: None,
            bold: false,
            italic: false,
            border_radius: 0.0,
            padding_x: 0.0,
            padding_y: 0.0,
            border_color: None,
            border_width: 1.0,
            font_size_delta: 0.0,
        }
    }
}

// ── ClassResolver ─────────────────────────────────────────

/// Function type that maps a space-separated class string
/// to a [`CellElementStyle`].
///
/// Implement this to support a specific CSS framework (e.g.
/// DaisyUI). Register the resolver on `SceneBuilder` via
/// `SceneBuilder::set_class_resolver`.
pub type ClassResolver = dyn Fn(&str) -> CellElementStyle;
