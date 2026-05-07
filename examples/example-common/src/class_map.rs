//! DaisyUI / Tailwind class → canvas style resolver.
//!
//! Translates space-separated class strings into a
//! [`CellElementStyle`] that the scene builder uses to
//! emit styled cell elements.
//!
//! Plug this into your grid instance via
//! `GridCanvas::set_class_resolver`:
//!
//! ```ignore
//! on_mount: Box::new(|gc| {
//!     gc.set_class_resolver(Rc::new(resolve_classes));
//! })
//! ```
//!
//! # Supported components (DaisyUI v5)
//!
//! ## badge
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
//! ## btn
//!
//! **Base:** `btn`
//!
//! **Colour variants:** `btn-primary`, `btn-secondary`,
//! `btn-accent`, `btn-success`, `btn-error`,
//! `btn-warning`, `btn-info`, `btn-neutral`
//!
//! **Style modifiers:** `btn-outline`, `btn-soft`,
//! `btn-dash`, `btn-ghost`
//!
//! **Sizes:** `btn-xs`, `btn-sm`, `btn-md`,
//! `btn-lg`, `btn-xl`
//!
//! ## Tailwind utilities (canvas-meaningful)
//!
//! `font-bold`, `rounded-full`, `rounded-md`, `rounded`,
//! `text-xs`, `text-sm`
//!
//! ## Geometry (matches DaisyUI v5 exactly)
//!
//! Values come from [`crate::class_map_data`], generated
//! from DaisyUI's installed `node_modules`.
//! Regenerate with `just gen-class-map`.

use rs_grid_scene::{class_map::CellElementStyle, primitives::Color};

use crate::class_map_data::{
    badge, btn, ACCENT_BG, ACCENT_FG, BASE_200, ERROR_BG, ERROR_FG, INFO_BG,
    INFO_FG, NEUTRAL_BG, NEUTRAL_FG, PRIMARY_BG, PRIMARY_FG, SECONDARY_BG,
    SECONDARY_FG, SUCCESS_BG, SUCCESS_FG, WARNING_BG, WARNING_FG,
};

/// Resolve space-separated DaisyUI / Tailwind class names
/// into a [`CellElementStyle`].
///
/// Classes are applied left-to-right. Post-processing
/// modifiers (`badge-soft`, `badge-dash`) are resolved
/// after the full pass so they always see the final colour.
///
/// Register this as the class resolver on `GridCanvas`:
///
/// ```ignore
/// gc.set_class_resolver(Rc::new(resolve_classes));
/// ```
pub fn resolve_classes(classes: &str) -> CellElementStyle {
    let mut s = CellElementStyle::default();
    let mut soft = false;
    let mut dash = false;

    for cls in classes.split_whitespace() {
        match cls {
            // ── badge base ────────────────────────────────
            "badge" => {
                s.border_radius = badge::RADIUS;
                s.padding_x = badge::MD.px;
                s.padding_y = badge::MD.py;
            }

            // ── badge colour variants ─────────────────────
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

            // ── badge style modifiers ─────────────────────

            // Outline: stroke only, no fill.
            "badge-outline" => {
                let accent = s.background.unwrap_or(NEUTRAL_BG);
                s.border_color = Some(accent);
                s.color = Some(accent);
                s.background = None;
                s.border_width = badge::BORDER;
            }

            // Soft: semi-transparent background.
            "badge-soft" => {
                soft = true;
            }

            // Dash: dashed outline approximation.
            "badge-dash" => {
                dash = true;
            }

            // Ghost: bg-base-200 + border-base-200.
            "badge-ghost" => {
                s.background = Some(BASE_200);
                s.border_color = Some(BASE_200);
                s.border_width = badge::BORDER;
                s.color = None;
            }

            // ── badge sizes ───────────────────────────────
            "badge-xs" => {
                s.padding_x = badge::XS.px;
                s.padding_y = badge::XS.py;
                s.font_size_delta = badge::XS.fd;
            }
            "badge-sm" => {
                s.padding_x = badge::SM.px;
                s.padding_y = badge::SM.py;
                s.font_size_delta = badge::SM.fd;
            }
            "badge-md" => {
                s.padding_x = badge::MD.px;
                s.padding_y = badge::MD.py;
                s.font_size_delta = badge::MD.fd;
            }
            "badge-lg" => {
                s.padding_x = badge::LG.px;
                s.padding_y = badge::LG.py;
                s.font_size_delta = badge::LG.fd;
            }
            "badge-xl" => {
                s.padding_x = badge::XL.px;
                s.padding_y = badge::XL.py;
                s.font_size_delta = badge::XL.fd;
            }

            // ── btn base ──────────────────────────────────
            "btn" => {
                s.border_radius = btn::RADIUS;
                s.padding_x = btn::MD.px;
                s.padding_y = btn::MD.py;
            }

            // ── btn colour variants ───────────────────────
            "btn-primary" => {
                s.background = Some(PRIMARY_BG);
                s.color = Some(PRIMARY_FG);
            }
            "btn-secondary" => {
                s.background = Some(SECONDARY_BG);
                s.color = Some(SECONDARY_FG);
            }
            "btn-accent" => {
                s.background = Some(ACCENT_BG);
                s.color = Some(ACCENT_FG);
            }
            "btn-success" => {
                s.background = Some(SUCCESS_BG);
                s.color = Some(SUCCESS_FG);
            }
            "btn-error" => {
                s.background = Some(ERROR_BG);
                s.color = Some(ERROR_FG);
            }
            "btn-warning" => {
                s.background = Some(WARNING_BG);
                s.color = Some(WARNING_FG);
            }
            "btn-info" => {
                s.background = Some(INFO_BG);
                s.color = Some(INFO_FG);
            }
            "btn-neutral" => {
                s.background = Some(NEUTRAL_BG);
                s.color = Some(NEUTRAL_FG);
            }

            // ── btn style modifiers ───────────────────────

            // Outline: stroke only, no fill.
            "btn-outline" => {
                let accent = s.background.unwrap_or(NEUTRAL_BG);
                s.border_color = Some(accent);
                s.color = Some(accent);
                s.background = None;
                s.border_width = btn::BORDER;
            }

            // Soft: semi-transparent background.
            "btn-soft" => {
                soft = true;
            }

            // Dash: dashed outline approximation.
            "btn-dash" => {
                dash = true;
            }

            // Ghost: no background, no border.
            "btn-ghost" => {
                s.background = None;
                s.border_color = None;
                s.color = None;
            }

            // ── btn sizes ─────────────────────────────────
            "btn-xs" => {
                s.padding_x = btn::XS.px;
                s.padding_y = btn::XS.py;
                s.font_size_delta = btn::XS.fd;
            }
            "btn-sm" => {
                s.padding_x = btn::SM.px;
                s.padding_y = btn::SM.py;
                s.font_size_delta = btn::SM.fd;
            }
            "btn-md" => {
                s.padding_x = btn::MD.px;
                s.padding_y = btn::MD.py;
                s.font_size_delta = btn::MD.fd;
            }
            "btn-lg" => {
                s.padding_x = btn::LG.px;
                s.padding_y = btn::LG.py;
                s.font_size_delta = btn::LG.fd;
            }
            "btn-xl" => {
                s.padding_x = btn::XL.px;
                s.padding_y = btn::XL.py;
                s.font_size_delta = btn::XL.fd;
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

    // soft: translucent background, saturated text.
    if soft {
        if let Some(bg) = s.background {
            // DaisyUI: color-mix(in oklab, color 8%, base-100).
            // Approximation: alpha ≈ 20 (8% of 255).
            s.background = Some(Color::rgba(bg.r, bg.g, bg.b, 20));
            // Border: color-mix(in oklab, color 10%) ≈ alpha 25.
            s.border_color = Some(Color::rgba(bg.r, bg.g, bg.b, 25));
            // Text uses the saturated colour.
            s.color = Some(bg);
        }
    }

    // dash: outline with no fill.
    if dash {
        let stroke = s.background.or(s.color).unwrap_or(NEUTRAL_BG);
        s.border_color = Some(stroke);
        s.background = None;
        s.border_width = badge::BORDER;
    }

    s
}

// ── tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::class_map_data::{
        badge, btn, ERROR_BG, INFO_BG, PRIMARY_BG, SUCCESS_BG, SUCCESS_FG,
    };

    // ── badge ─────────────────────────────────────────────

    #[test]
    fn badge_base_sets_padding_and_radius() {
        let s = resolve_classes("badge");
        assert_eq!(s.border_radius, badge::RADIUS);
        assert_eq!(s.padding_x, badge::MD.px);
        assert_eq!(s.padding_y, badge::MD.py);
        assert!(s.background.is_none());
    }

    #[test]
    fn badge_success_sets_bg_and_fg() {
        let s = resolve_classes("badge badge-success");
        let bg = s.background.expect("background");
        assert_eq!(
            (bg.r, bg.g, bg.b),
            (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b)
        );
        let fg = s.color.expect("text color");
        assert_eq!(
            (fg.r, fg.g, fg.b),
            (SUCCESS_FG.r, SUCCESS_FG.g, SUCCESS_FG.b)
        );
    }

    #[test]
    fn badge_outline_clears_background() {
        let s = resolve_classes("badge badge-error badge-outline");
        assert!(s.background.is_none());
        let bc = s.border_color.unwrap();
        assert_eq!((bc.r, bc.g, bc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
        let tc = s.color.unwrap();
        assert_eq!((tc.r, tc.g, tc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
    }

    #[test]
    fn badge_soft_translucent_bg_and_colored_text() {
        let s = resolve_classes("badge badge-success badge-soft");
        let bg = s.background.expect("soft background");
        assert_eq!(
            (bg.r, bg.g, bg.b),
            (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b)
        );
        assert!(bg.a <= 25, "soft bg should be ~8% translucent");
        let text = s.color.expect("text color");
        assert_eq!(
            (text.r, text.g, text.b),
            (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b)
        );
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
        let s = resolve_classes("badge badge-ghost");
        let bg = s.background.expect("ghost should have bg-base-200");
        assert_eq!((bg.r, bg.g, bg.b), (BASE_200.r, BASE_200.g, BASE_200.b));
        assert!(s.border_color.is_some());
    }

    #[test]
    fn badge_xl_larger_padding() {
        let s = resolve_classes("badge badge-success badge-xl");
        assert_eq!(s.padding_x, badge::XL.px);
        assert_eq!(s.padding_y, badge::XL.py);
        assert_eq!(s.font_size_delta, badge::XL.fd);
    }

    #[test]
    fn badge_sm_reduces_padding_and_font() {
        let s = resolve_classes("badge badge-success badge-sm");
        assert_eq!(s.padding_x, badge::SM.px);
        assert_eq!(s.padding_y, badge::SM.py);
        assert!(s.font_size_delta < 0.0);
    }

    #[test]
    fn badge_all_colour_variants_have_bg_and_fg() {
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
            assert!(s.background.is_some(), "{v} should have a background");
            assert!(s.color.is_some(), "{v} should have a text colour");
        }
    }

    #[test]
    fn badge_soft_modifier_on_all_variants() {
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
            let bg =
                s.background.unwrap_or_else(|| panic!("{v} soft has no bg"));
            assert!(
                bg.a <= 25,
                "{v} badge-soft bg should be ~8% translucent, got a={}",
                bg.a
            );
        }
    }

    #[test]
    fn badge_dash_modifier_on_variants() {
        for v in ["badge-primary", "badge-success", "badge-error"] {
            let s = resolve_classes(&format!("badge {v} badge-dash"));
            assert!(s.background.is_none(), "{v} badge-dash should have no bg");
            assert!(
                s.border_color.is_some(),
                "{v} badge-dash should have border"
            );
        }
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
    fn unknown_classes_are_ignored() {
        let s = resolve_classes("flex h-full w-full badge badge-info");
        assert!(s.background.is_some());
        assert_eq!(s.border_radius, badge::RADIUS);
    }

    // ── btn ───────────────────────────────────────────────

    #[test]
    fn btn_base_sets_padding_and_radius() {
        let s = resolve_classes("btn");
        assert_eq!(s.border_radius, btn::RADIUS);
        assert_eq!(s.padding_x, btn::MD.px);
        assert_eq!(s.padding_y, btn::MD.py);
        assert!(s.background.is_none());
    }

    #[test]
    fn btn_primary_sets_bg_and_fg() {
        let s = resolve_classes("btn btn-primary");
        let bg = s.background.expect("background");
        assert_eq!(
            (bg.r, bg.g, bg.b),
            (PRIMARY_BG.r, PRIMARY_BG.g, PRIMARY_BG.b)
        );
        assert!(s.color.is_some());
    }

    #[test]
    fn btn_outline_clears_background() {
        let s = resolve_classes("btn btn-success btn-outline");
        assert!(s.background.is_none());
        let bc = s.border_color.expect("border");
        assert_eq!(
            (bc.r, bc.g, bc.b),
            (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b)
        );
    }

    #[test]
    fn btn_soft_translucent_bg() {
        let s = resolve_classes("btn btn-primary btn-soft");
        let bg = s.background.expect("soft bg");
        assert!(
            bg.a <= 25,
            "btn-soft bg should be ~8% translucent, got a={}",
            bg.a
        );
        assert!(s.border_color.is_some());
    }

    #[test]
    fn btn_ghost_clears_bg_and_border() {
        let s = resolve_classes("btn btn-primary btn-ghost");
        assert!(s.background.is_none());
        assert!(s.border_color.is_none());
    }

    #[test]
    fn btn_xs_smaller_padding() {
        let s = resolve_classes("btn btn-primary btn-xs");
        assert_eq!(s.padding_x, btn::XS.px);
        assert_eq!(s.padding_y, btn::XS.py);
        assert!(s.font_size_delta < 0.0);
    }

    #[test]
    fn btn_xl_larger_padding_than_md() {
        let md = resolve_classes("btn btn-primary");
        let xl = resolve_classes("btn btn-primary btn-xl");
        assert!(xl.padding_x > md.padding_x);
        assert!(xl.padding_y > md.padding_y);
    }

    #[test]
    fn btn_all_colour_variants_have_bg_and_fg() {
        let variants = [
            "btn-primary",
            "btn-secondary",
            "btn-accent",
            "btn-success",
            "btn-error",
            "btn-warning",
            "btn-info",
            "btn-neutral",
        ];
        for v in variants {
            let s = resolve_classes(&format!("btn {v}"));
            assert!(s.background.is_some(), "{v} should have a background");
            assert!(s.color.is_some(), "{v} should have a text colour");
        }
    }
}
