//! DaisyUI / Tailwind class → canvas style resolver.
//!
//! Translates space-separated class strings into a
//! [`CellElementStyle`] that the scene builder can use to
//! emit [`ScenePrimitive`]s.
//!
//! Only a canvas-meaningful subset of classes is
//! recognised. Unknown names are silently ignored.
//!
//! # Supported classes (v1 — badge component)
//!
//! **Base:** `badge`
//!
//! **Colour variants:** `badge-primary`, `badge-secondary`,
//! `badge-accent`, `badge-success`, `badge-error`,
//! `badge-warning`, `badge-info`, `badge-neutral`
//!
//! **Modifiers:** `badge-outline`, `badge-xs`, `badge-sm`,
//! `badge-lg`
//!
//! **Tailwind utilities:** `font-bold`, `rounded-full`,
//! `rounded-md`, `rounded`, `text-xs`, `text-sm`

use crate::primitives::Color;

// ── CellElementStyle ─────────────────────────────────────

/// Visual style resolved from a set of class names.
///
/// Consumed by `emit_styled` in the scene builder to emit
/// `RectPrimitive` + `TextPrimitive` pairs.
#[derive(Debug, Clone)]
pub struct CellElementStyle {
    /// Text colour override. `None` → use `Theme::cell_text`.
    pub color: Option<Color>,
    /// Badge background. `None` → transparent (outline
    /// mode uses `border_color` only).
    pub background: Option<Color>,
    /// Render text in bold weight.
    pub bold: bool,
    /// Corner radius of the badge rectangle (logical px).
    pub border_radius: f64,
    /// Horizontal inner padding (logical px each side).
    pub padding_x: f64,
    /// Vertical inner padding (logical px each side).
    pub padding_y: f64,
    /// Stroke colour for outline badges. `None` → no stroke.
    pub border_color: Option<Color>,
    /// Stroke width (logical px). Used when `border_color`
    /// is `Some`.
    pub border_width: f64,
    /// Font-size adjustment relative to `Theme::font_size`.
    /// Negative = smaller (badge-sm / text-xs), positive =
    /// larger (badge-lg).
    pub font_size_delta: f64,
}

impl Default for CellElementStyle {
    fn default() -> Self {
        Self {
            color: None,
            background: None,
            bold: false,
            border_radius: 0.0,
            padding_x: 0.0,
            padding_y: 0.0,
            border_color: None,
            border_width: 1.0,
            font_size_delta: 0.0,
        }
    }
}

// ── resolve_classes ──────────────────────────────────────

/// Resolve space-separated class names into a
/// [`CellElementStyle`].
///
/// Classes are applied left-to-right; later entries
/// override earlier ones for the same property.
///
/// ```
/// use rs_grid_scene::class_map::resolve_classes;
/// let s = resolve_classes("badge badge-success");
/// assert!(s.background.is_some());
/// ```
pub fn resolve_classes(classes: &str) -> CellElementStyle {
    let mut s = CellElementStyle::default();

    for cls in classes.split_whitespace() {
        match cls {
            // ── base ─────────────────────────────────────
            "badge" => {
                s.border_radius = 12.0;
                s.padding_x = 8.0;
                s.padding_y = 2.0;
            }

            // ── colour variants ───────────────────────────
            "badge-primary" => {
                s.background = Some(Color::rgb(99, 102, 241));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-secondary" => {
                s.background = Some(Color::rgb(168, 85, 247));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-accent" => {
                s.background = Some(Color::rgb(20, 184, 166));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-success" => {
                s.background = Some(Color::rgb(34, 197, 94));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-error" => {
                s.background = Some(Color::rgb(239, 68, 68));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-warning" => {
                s.background = Some(Color::rgb(245, 158, 11));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-info" => {
                s.background = Some(Color::rgb(59, 130, 246));
                s.color = Some(Color::rgb(255, 255, 255));
            }
            "badge-neutral" => {
                s.background = Some(Color::rgb(107, 114, 128));
                s.color = Some(Color::rgb(255, 255, 255));
            }

            // ── modifiers ─────────────────────────────────
            "badge-outline" => {
                // Reuse the text colour as border; remove
                // the fill so only the stroke is visible.
                s.border_color =
                    s.color.or(Some(Color::rgb(107, 114, 128)));
                s.background = None;
                s.border_width = 1.0;
            }
            "badge-xs" => {
                s.padding_x = 4.0;
                s.padding_y = 1.0;
                s.font_size_delta = -4.0;
            }
            "badge-sm" => {
                s.padding_x = 5.0;
                s.padding_y = 1.0;
                s.font_size_delta = -2.0;
            }
            "badge-lg" => {
                s.padding_x = 12.0;
                s.padding_y = 4.0;
                s.font_size_delta = 1.0;
            }

            // ── Tailwind utilities (canvas-meaningful) ────
            "font-bold" => {
                s.bold = true;
            }
            "rounded-full" => {
                s.border_radius = 9999.0;
            }
            "rounded-md" => {
                s.border_radius = 6.0;
            }
            "rounded" => {
                s.border_radius = 4.0;
            }
            "text-xs" => {
                s.font_size_delta = -3.0;
            }
            "text-sm" => {
                s.font_size_delta = -1.0;
            }

            // Unknown class — silently ignored.
            _ => {}
        }
    }

    s
}

// ── tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_base_sets_padding_and_radius() {
        let s = resolve_classes("badge");
        assert_eq!(s.border_radius, 12.0);
        assert_eq!(s.padding_x, 8.0);
        assert_eq!(s.padding_y, 2.0);
        assert!(s.background.is_none());
    }

    #[test]
    fn badge_success_sets_green_bg() {
        let s = resolve_classes("badge badge-success");
        let bg = s.background.expect("background");
        assert_eq!(bg.r, 34);
        assert_eq!(bg.g, 197);
        assert_eq!(bg.b, 94);
        assert!(s.color.is_some());
    }

    #[test]
    fn badge_outline_clears_background() {
        let s = resolve_classes("badge badge-error badge-outline");
        assert!(s.background.is_none());
        assert!(s.border_color.is_some());
        // Border should reuse the error text colour.
        let bc = s.border_color.unwrap();
        assert_eq!(bc.r, 255);
    }

    #[test]
    fn badge_sm_reduces_padding_and_font() {
        let s = resolve_classes("badge badge-success badge-sm");
        assert_eq!(s.padding_x, 5.0);
        assert!(s.font_size_delta < 0.0);
    }

    #[test]
    fn unknown_classes_are_ignored() {
        let s = resolve_classes("flex h-full w-full badge badge-info");
        assert!(s.background.is_some());
        assert_eq!(s.border_radius, 12.0);
    }

    #[test]
    fn font_bold_sets_bold() {
        let s = resolve_classes("badge badge-primary font-bold");
        assert!(s.bold);
    }

    #[test]
    fn rounded_full_overrides_radius() {
        let s = resolve_classes("badge badge-info rounded-full");
        assert_eq!(s.border_radius, 9999.0);
    }

    #[test]
    fn empty_class_returns_default() {
        let s = resolve_classes("");
        assert!(s.background.is_none());
        assert_eq!(s.border_radius, 0.0);
    }

    #[test]
    fn all_colour_variants_have_white_text() {
        let variants = [
            "badge-primary",
            "badge-secondary",
            "badge-accent",
            "badge-success",
            "badge-error",
            "badge-warning",
            "badge-info",
            "badge-neutral",
        ];
        for v in variants {
            let s = resolve_classes(&format!("badge {v}"));
            let c = s.color.unwrap_or_else(|| panic!("{v} has no color"));
            assert_eq!(
                (c.r, c.g, c.b),
                (255, 255, 255),
                "{v} text should be white"
            );
        }
    }
}
