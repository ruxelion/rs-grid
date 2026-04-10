//! DaisyUI / Tailwind class → canvas style resolver.
//!
//! Translates space-separated class strings into a
//! [`CellElementStyle`] that the scene builder can use to
//! emit [`ScenePrimitive`]s.
//!
//! Only a canvas-meaningful subset of classes is
//! recognised. Unknown names are silently ignored.
//!
//! # Supported classes — badge component (DaisyUI v5)
//!
//! **Base:** `badge`
//!
//! **Colour variants:** `badge-primary`, `badge-secondary`,
//! `badge-accent`, `badge-success`, `badge-error`,
//! `badge-warning`, `badge-info`, `badge-neutral`
//!
//! **Style modifiers:** `badge-outline`, `badge-soft`,
//! `badge-dash`, `badge-ghost`
//!
//! **Sizes:** `badge-xs`, `badge-sm`, `badge-md`,
//! `badge-lg`, `badge-xl`
//!
//! **Tailwind utilities:** `font-bold`, `rounded-full`,
//! `rounded-md`, `rounded`, `text-xs`, `text-sm`
//!
//! ## Geometry (matches DaisyUI v5 exactly)
//!
//! Values come from [`crate::class_map_data`], generated
//! from DaisyUI's installed `node_modules`.
//! Regenerate with `just gen-class-map`.

use crate::class_map_data::{
    ACCENT_BG, ACCENT_FG, BADGE_BORDER, BADGE_RADIUS, BASE_200,
    ERROR_BG, ERROR_FG, INFO_BG, INFO_FG, NEUTRAL_BG, NEUTRAL_FG,
    PRIMARY_BG, PRIMARY_FG, SECONDARY_BG, SECONDARY_FG,
    SUCCESS_BG, SUCCESS_FG, SZ_LG, SZ_MD, SZ_SM, SZ_XL, SZ_XS,
    WARNING_BG, WARNING_FG,
};
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
    /// Badge background. `None` → transparent.
    pub background: Option<Color>,
    /// Render text in bold weight.
    pub bold: bool,
    /// Corner radius of the badge rectangle (logical px).
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
            border_radius: 0.0,
            padding_x: 0.0,
            padding_y: 0.0,
            border_color: None,
            border_width: BADGE_BORDER,
            font_size_delta: 0.0,
        }
    }
}

// ── resolve_classes ──────────────────────────────────────

/// Resolve space-separated DaisyUI / Tailwind class names
/// into a [`CellElementStyle`].
///
/// Classes are applied left-to-right. Post-processing
/// modifiers (`badge-soft`, `badge-dash`) are resolved
/// after the full pass so they always see the final colour.
///
/// Color and geometry constants come from
/// [`crate::class_map_data`] (generated from DaisyUI
/// sources). Regenerate with `just gen-class-map`.
///
/// ```
/// use rs_grid_scene::class_map::resolve_classes;
/// let s = resolve_classes("badge badge-success");
/// assert!(s.background.is_some());
/// ```
pub fn resolve_classes(classes: &str) -> CellElementStyle {
    let mut s = CellElementStyle::default();
    let mut soft = false;
    let mut dash = false;

    for cls in classes.split_whitespace() {
        match cls {
            // ── base ─────────────────────────────────────
            "badge" => {
                s.border_radius = BADGE_RADIUS;
                s.padding_x = SZ_MD.px;
                s.padding_y = SZ_MD.py;
            }

            // ── colour variants ───────────────────────────
            "badge-primary" => {
                s.background = Some(PRIMARY_BG);
                s.color = Some(PRIMARY_FG);
            }
            "badge-secondary" => {
                s.background = Some(SECONDARY_BG);
                s.color = Some(SECONDARY_FG);
            }
            "badge-accent" => {
                s.background = Some(ACCENT_BG);
                s.color = Some(ACCENT_FG);
            }
            "badge-success" => {
                s.background = Some(SUCCESS_BG);
                s.color = Some(SUCCESS_FG);
            }
            "badge-error" => {
                s.background = Some(ERROR_BG);
                s.color = Some(ERROR_FG);
            }
            "badge-warning" => {
                s.background = Some(WARNING_BG);
                s.color = Some(WARNING_FG);
            }
            "badge-info" => {
                s.background = Some(INFO_BG);
                s.color = Some(INFO_FG);
            }
            "badge-neutral" => {
                s.background = Some(NEUTRAL_BG);
                s.color = Some(NEUTRAL_FG);
            }

            // ── style modifiers ───────────────────────────

            // Outline: stroke only, no fill.
            // Text and border both use the variant colour;
            // background is cleared.
            "badge-outline" => {
                let accent = s.background.unwrap_or(NEUTRAL_BG);
                s.border_color = Some(accent);
                s.color = Some(accent);
                s.background = None;
                s.border_width = BADGE_BORDER;
            }

            // Soft: semi-transparent background, saturated
            // text. Resolved after all colours are set.
            "badge-soft" => {
                soft = true;
            }

            // Dash: dashed outline, no fill. Canvas2D does
            // not support dashed rounded rects yet, so this
            // renders as a regular outline for now.
            "badge-dash" => {
                dash = true;
            }

            // Ghost: DaisyUI bg-base-200 + border-base-200.
            "badge-ghost" => {
                s.background = Some(BASE_200);
                s.border_color = Some(BASE_200);
                s.border_width = BADGE_BORDER;
                s.color = None; // fall back to theme cell_text
            }

            // ── sizes ─────────────────────────────────────
            "badge-xs" => {
                s.padding_x = SZ_XS.px;
                s.padding_y = SZ_XS.py;
                s.font_size_delta = SZ_XS.fd;
            }
            "badge-sm" => {
                s.padding_x = SZ_SM.px;
                s.padding_y = SZ_SM.py;
                s.font_size_delta = SZ_SM.fd;
            }
            "badge-md" => {
                s.padding_x = SZ_MD.px;
                s.padding_y = SZ_MD.py;
                s.font_size_delta = SZ_MD.fd;
            }
            "badge-lg" => {
                s.padding_x = SZ_LG.px;
                s.padding_y = SZ_LG.py;
                s.font_size_delta = SZ_LG.fd;
            }
            "badge-xl" => {
                s.padding_x = SZ_XL.px;
                s.padding_y = SZ_XL.py;
                s.font_size_delta = SZ_XL.fd;
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

    // ── post-processing modifiers ─────────────────────────

    // badge-soft: translucent background, saturated text.
    // Applied after colour variants so the colour is known.
    if soft {
        if let Some(bg) = s.background {
            // DaisyUI: color-mix(in oklab, color 8%, base-100).
            // Approximation: alpha ≈ 20 (8% of 255).
            s.background = Some(Color::rgba(bg.r, bg.g, bg.b, 20));
            // Border: color-mix(in oklab, color 10%) ≈ alpha 25.
            s.border_color = Some(Color::rgba(bg.r, bg.g, bg.b, 25));
            // Text uses the saturated colour (readable on light bg).
            s.color = Some(bg);
        }
    }

    // badge-dash: outline with no fill (dashed approximation).
    // Applied after colour variants so the colour is known.
    if dash {
        let stroke = s
            .background
            .or(s.color)
            .unwrap_or(NEUTRAL_BG);
        s.border_color = Some(stroke);
        s.background = None;
        s.border_width = BADGE_BORDER;
    }

    s
}

// ── tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_map_data::{
        ERROR_BG, INFO_BG, SUCCESS_BG, SUCCESS_FG,
    };

    #[test]
    fn badge_base_sets_padding_and_radius() {
        let s = resolve_classes("badge");
        assert_eq!(s.border_radius, BADGE_RADIUS);
        assert_eq!(s.padding_x, SZ_MD.px);
        assert_eq!(s.padding_y, SZ_MD.py);
        assert!(s.background.is_none());
    }

    #[test]
    fn badge_success_sets_bg_and_fg() {
        let s = resolve_classes("badge badge-success");
        let bg = s.background.expect("background");
        assert_eq!((bg.r, bg.g, bg.b), (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b));
        let fg = s.color.expect("text color");
        assert_eq!((fg.r, fg.g, fg.b), (SUCCESS_FG.r, SUCCESS_FG.g, SUCCESS_FG.b));
    }

    #[test]
    fn badge_outline_clears_background() {
        let s = resolve_classes("badge badge-error badge-outline");
        assert!(s.background.is_none());
        // Border uses the error colour.
        let bc = s.border_color.unwrap();
        assert_eq!((bc.r, bc.g, bc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
        // Text also uses the error colour — visible on white bg.
        let tc = s.color.unwrap();
        assert_eq!((tc.r, tc.g, tc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
    }

    #[test]
    fn badge_soft_translucent_bg_and_colored_text() {
        let s = resolve_classes("badge badge-success badge-soft");
        let bg = s.background.expect("soft background");
        // Background uses the success colour at low alpha (~8%).
        assert_eq!((bg.r, bg.g, bg.b), (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b));
        assert!(bg.a <= 25, "soft bg should be ~8% translucent");
        // Text should be the saturated colour, not the content colour.
        let text = s.color.expect("text color");
        assert_eq!((text.r, text.g, text.b), (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b));
        // Soft badges also have a subtle border.
        assert!(s.border_color.is_some(), "soft should have border");
    }

    #[test]
    fn badge_dash_clears_background_keeps_border() {
        let s = resolve_classes("badge badge-info badge-dash");
        assert!(s.background.is_none());
        let bc = s.border_color.expect("border");
        assert_eq!((bc.r, bc.g, bc.b), (INFO_BG.r, INFO_BG.g, INFO_BG.b));
    }

    #[test]
    fn badge_ghost_uses_base_200() {
        // DaisyUI: bg-base-200 + border-base-200.
        let s = resolve_classes("badge badge-ghost");
        let bg = s.background.expect("ghost should have bg-base-200");
        assert_eq!((bg.r, bg.g, bg.b), (BASE_200.r, BASE_200.g, BASE_200.b));
        assert!(s.border_color.is_some());
    }

    #[test]
    fn badge_xl_larger_padding() {
        let s = resolve_classes("badge badge-success badge-xl");
        assert_eq!(s.padding_x, SZ_XL.px);
        assert_eq!(s.padding_y, SZ_XL.py);
        assert_eq!(s.font_size_delta, SZ_XL.fd);
    }

    #[test]
    fn badge_md_explicit_medium() {
        let s_default = resolve_classes("badge badge-success");
        let s_md = resolve_classes("badge badge-success badge-md");
        assert_eq!(s_default.padding_x, s_md.padding_x);
        assert_eq!(s_default.font_size_delta, s_md.font_size_delta);
    }

    #[test]
    fn badge_sm_reduces_padding_and_font() {
        let s = resolve_classes("badge badge-success badge-sm");
        assert_eq!(s.padding_x, SZ_SM.px);
        assert_eq!(s.padding_y, SZ_SM.py);
        assert!(s.font_size_delta < 0.0);
    }

    #[test]
    fn unknown_classes_are_ignored() {
        let s = resolve_classes("flex h-full w-full badge badge-info");
        assert!(s.background.is_some());
        assert_eq!(s.border_radius, BADGE_RADIUS);
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
    fn all_colour_variants_have_bg_and_fg() {
        // All variants set both a background colour and a
        // readable foreground colour (DaisyUI -content colours
        // — not necessarily white).
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
            assert!(
                s.background.is_some(),
                "{v} should have a background"
            );
            assert!(
                s.color.is_some(),
                "{v} should have a text colour"
            );
        }
    }

    #[test]
    fn soft_modifier_works_on_all_colour_variants() {
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
            let s = resolve_classes(&format!("badge {v} badge-soft"));
            let bg = s
                .background
                .unwrap_or_else(|| panic!("{v} soft has no bg"));
            assert!(
                bg.a <= 25,
                "{v} badge-soft bg should be ~8% translucent, got a={}",
                bg.a
            );
        }
    }

    #[test]
    fn dash_modifier_works_on_all_colour_variants() {
        let variants = [
            "badge-primary",
            "badge-success",
            "badge-error",
            "badge-neutral",
        ];
        for v in variants {
            let s = resolve_classes(&format!("badge {v} badge-dash"));
            assert!(
                s.background.is_none(),
                "{v} badge-dash should have no bg"
            );
            assert!(
                s.border_color.is_some(),
                "{v} badge-dash should have border"
            );
        }
    }
}
