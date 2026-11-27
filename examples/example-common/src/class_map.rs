//! DaisyUI / Tailwind class → canvas style resolver.
//!
//! Translates space-separated class strings into a
//! [`CellElementStyle`] that the scene builder uses to
//! emit styled cell elements.
//!
//! Plug this into your grid instance via
//! `GridCanvas::set_class_resolver`, re-registering whenever the
//! active theme changes so colours stay in sync:
//!
//! ```ignore
//! on_mount: Box::new(move |gc| {
//!     gc.set_class_resolver(Rc::new(move |raw| {
//!         resolve_classes(raw, &theme)
//!     }));
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
//! ## progress
//!
//! **Base:** `progress`
//!
//! **Colour variants:** `progress-primary`,
//! `progress-secondary`, `progress-accent`,
//! `progress-success`, `progress-error`, `progress-warning`,
//! `progress-info`, `progress-neutral`
//!
//! Used by `CellFormat::ProgressBar`: the resolved
//! `background` is the bar fill colour.
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

use rs_grid_scene::{class_map::CellElementStyle, primitives::Color, Theme};

use crate::class_map_data::{badge, btn, progress, BASE_200};

/// Resolve space-separated DaisyUI / Tailwind class names
/// into a [`CellElementStyle`].
///
/// Classes are applied left-to-right. Post-processing
/// modifiers (`badge-soft`, `badge-dash`) are resolved
/// after the full pass so they always see the final colour.
///
/// Colour variants (`badge-primary`, `btn-success`, …) are read from
/// `theme.cell_btn_*` rather than the light-only `class_map_data`
/// constants, so badges/buttons repaint correctly when the demo
/// switches to the dark or dimmed theme — the same colours the
/// `ButtonStyle` cell buttons already use. Geometry (radius, padding,
/// sizes) is theme-invariant in DaisyUI, so it still comes from
/// `class_map_data`.
///
/// Register this as the class resolver on `GridCanvas`, re-registering
/// whenever the active theme changes:
///
/// ```ignore
/// gc.set_class_resolver(Rc::new(move |raw| resolve_classes(raw, &theme)));
/// ```
pub fn resolve_classes(classes: &str, theme: &Theme) -> CellElementStyle {
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
                s.background = Some(theme.cell_btn_primary_bg);
                s.color = Some(theme.cell_btn_primary_text);
            }
            "badge-secondary" => {
                s.background = Some(theme.cell_btn_secondary_bg);
                s.color = Some(theme.cell_btn_secondary_text);
            }
            "badge-accent" => {
                s.background = Some(theme.cell_btn_accent_bg);
                s.color = Some(theme.cell_btn_accent_text);
            }
            "badge-success" => {
                s.background = Some(theme.cell_btn_success_bg);
                s.color = Some(theme.cell_btn_success_text);
            }
            "badge-error" => {
                s.background = Some(theme.cell_btn_danger_bg);
                s.color = Some(theme.cell_btn_danger_text);
            }
            "badge-warning" => {
                s.background = Some(theme.cell_btn_warning_bg);
                s.color = Some(theme.cell_btn_warning_text);
            }
            "badge-info" => {
                s.background = Some(theme.cell_btn_info_bg);
                s.color = Some(theme.cell_btn_info_text);
            }
            "badge-neutral" => {
                s.background = Some(theme.cell_btn_neutral_bg);
                s.color = Some(theme.cell_btn_neutral_text);
            }

            // ── badge style modifiers ─────────────────────

            // Outline: stroke only, no fill.
            "badge-outline" => {
                let accent = s.background.unwrap_or(theme.cell_btn_neutral_bg);
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
                s.background = Some(theme.cell_btn_primary_bg);
                s.color = Some(theme.cell_btn_primary_text);
            }
            "btn-secondary" => {
                s.background = Some(theme.cell_btn_secondary_bg);
                s.color = Some(theme.cell_btn_secondary_text);
            }
            "btn-accent" => {
                s.background = Some(theme.cell_btn_accent_bg);
                s.color = Some(theme.cell_btn_accent_text);
            }
            "btn-success" => {
                s.background = Some(theme.cell_btn_success_bg);
                s.color = Some(theme.cell_btn_success_text);
            }
            "btn-error" => {
                s.background = Some(theme.cell_btn_danger_bg);
                s.color = Some(theme.cell_btn_danger_text);
            }
            "btn-warning" => {
                s.background = Some(theme.cell_btn_warning_bg);
                s.color = Some(theme.cell_btn_warning_text);
            }
            "btn-info" => {
                s.background = Some(theme.cell_btn_info_bg);
                s.color = Some(theme.cell_btn_info_text);
            }
            "btn-neutral" => {
                s.background = Some(theme.cell_btn_neutral_bg);
                s.color = Some(theme.cell_btn_neutral_text);
            }

            // ── btn style modifiers ───────────────────────

            // Outline: stroke only, no fill.
            "btn-outline" => {
                let accent = s.background.unwrap_or(theme.cell_btn_neutral_bg);
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

            // ── progress base ─────────────────────────────
            "progress" => {
                s.border_radius = progress::RADIUS;
            }

            // ── progress colour variants ──────────────────
            // The scene builder uses `background` as the bar
            // fill colour; `color` is carried for label use.
            "progress-primary" => {
                s.background = Some(progress::PRIMARY_BG);
                s.color = Some(progress::PRIMARY_FG);
            }
            "progress-secondary" => {
                s.background = Some(progress::SECONDARY_BG);
                s.color = Some(progress::SECONDARY_FG);
            }
            "progress-accent" => {
                s.background = Some(progress::ACCENT_BG);
                s.color = Some(progress::ACCENT_FG);
            }
            "progress-success" => {
                s.background = Some(progress::SUCCESS_BG);
                s.color = Some(progress::SUCCESS_FG);
            }
            "progress-error" => {
                s.background = Some(progress::ERROR_BG);
                s.color = Some(progress::ERROR_FG);
            }
            "progress-warning" => {
                s.background = Some(progress::WARNING_BG);
                s.color = Some(progress::WARNING_FG);
            }
            "progress-info" => {
                s.background = Some(progress::INFO_BG);
                s.color = Some(progress::INFO_FG);
            }
            "progress-neutral" => {
                s.background = Some(progress::NEUTRAL_BG);
                s.color = Some(progress::NEUTRAL_FG);
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
        let stroke = s
            .background
            .or(s.color)
            .unwrap_or(theme.cell_btn_neutral_bg);
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
        badge, btn, progress, ERROR_BG, INFO_BG, PRIMARY_BG, SUCCESS_BG,
        SUCCESS_FG,
    };

    // ── badge ─────────────────────────────────────────────

    #[test]
    fn badge_base_sets_padding_and_radius() {
        let s = resolve_classes("badge", &Theme::light());
        assert_eq!(s.border_radius, badge::RADIUS);
        assert_eq!(s.padding_x, badge::MD.px);
        assert_eq!(s.padding_y, badge::MD.py);
        assert!(s.background.is_none());
    }

    #[test]
    fn badge_success_sets_bg_and_fg() {
        let s = resolve_classes("badge badge-success", &Theme::light());
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
        let s =
            resolve_classes("badge badge-error badge-outline", &Theme::light());
        assert!(s.background.is_none());
        let bc = s.border_color.unwrap();
        assert_eq!((bc.r, bc.g, bc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
        let tc = s.color.unwrap();
        assert_eq!((tc.r, tc.g, tc.b), (ERROR_BG.r, ERROR_BG.g, ERROR_BG.b));
    }

    #[test]
    fn badge_soft_translucent_bg_and_colored_text() {
        let s =
            resolve_classes("badge badge-success badge-soft", &Theme::light());
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
        let s = resolve_classes("badge badge-info badge-dash", &Theme::light());
        assert!(s.background.is_none());
        let bc = s.border_color.expect("border");
        assert_eq!((bc.r, bc.g, bc.b), (INFO_BG.r, INFO_BG.g, INFO_BG.b));
    }

    #[test]
    fn badge_ghost_uses_base_200() {
        let s = resolve_classes("badge badge-ghost", &Theme::light());
        let bg = s.background.expect("ghost should have bg-base-200");
        assert_eq!((bg.r, bg.g, bg.b), (BASE_200.r, BASE_200.g, BASE_200.b));
        assert!(s.border_color.is_some());
    }

    #[test]
    fn badge_xl_larger_padding() {
        let s =
            resolve_classes("badge badge-success badge-xl", &Theme::light());
        assert_eq!(s.padding_x, badge::XL.px);
        assert_eq!(s.padding_y, badge::XL.py);
        assert_eq!(s.font_size_delta, badge::XL.fd);
    }

    #[test]
    fn badge_sm_reduces_padding_and_font() {
        let s =
            resolve_classes("badge badge-success badge-sm", &Theme::light());
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
            let s = resolve_classes(&format!("badge {v}"), &Theme::light());
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
            let s = resolve_classes(
                &format!("badge {v} badge-soft"),
                &Theme::light(),
            );
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
            let s = resolve_classes(
                &format!("badge {v} badge-dash"),
                &Theme::light(),
            );
            assert!(s.background.is_none(), "{v} badge-dash should have no bg");
            assert!(
                s.border_color.is_some(),
                "{v} badge-dash should have border"
            );
        }
    }

    #[test]
    fn font_bold_sets_bold() {
        let s =
            resolve_classes("badge badge-primary font-bold", &Theme::light());
        assert!(s.bold);
    }

    #[test]
    fn rounded_full_overrides_radius() {
        let s =
            resolve_classes("badge badge-info rounded-full", &Theme::light());
        assert_eq!(s.border_radius, 9999.0);
    }

    #[test]
    fn empty_class_returns_default() {
        let s = resolve_classes("", &Theme::light());
        assert!(s.background.is_none());
        assert_eq!(s.border_radius, 0.0);
    }

    #[test]
    fn unknown_classes_are_ignored() {
        let s = resolve_classes(
            "flex h-full w-full badge badge-info",
            &Theme::light(),
        );
        assert!(s.background.is_some());
        assert_eq!(s.border_radius, badge::RADIUS);
    }

    // ── btn ───────────────────────────────────────────────

    #[test]
    fn btn_base_sets_padding_and_radius() {
        let s = resolve_classes("btn", &Theme::light());
        assert_eq!(s.border_radius, btn::RADIUS);
        assert_eq!(s.padding_x, btn::MD.px);
        assert_eq!(s.padding_y, btn::MD.py);
        assert!(s.background.is_none());
    }

    #[test]
    fn btn_primary_sets_bg_and_fg() {
        let s = resolve_classes("btn btn-primary", &Theme::light());
        let bg = s.background.expect("background");
        assert_eq!(
            (bg.r, bg.g, bg.b),
            (PRIMARY_BG.r, PRIMARY_BG.g, PRIMARY_BG.b)
        );
        assert!(s.color.is_some());
    }

    #[test]
    fn btn_outline_clears_background() {
        let s = resolve_classes("btn btn-success btn-outline", &Theme::light());
        assert!(s.background.is_none());
        let bc = s.border_color.expect("border");
        assert_eq!(
            (bc.r, bc.g, bc.b),
            (SUCCESS_BG.r, SUCCESS_BG.g, SUCCESS_BG.b)
        );
    }

    #[test]
    fn btn_soft_translucent_bg() {
        let s = resolve_classes("btn btn-primary btn-soft", &Theme::light());
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
        let s = resolve_classes("btn btn-primary btn-ghost", &Theme::light());
        assert!(s.background.is_none());
        assert!(s.border_color.is_none());
    }

    #[test]
    fn btn_xs_smaller_padding() {
        let s = resolve_classes("btn btn-primary btn-xs", &Theme::light());
        assert_eq!(s.padding_x, btn::XS.px);
        assert_eq!(s.padding_y, btn::XS.py);
        assert!(s.font_size_delta < 0.0);
    }

    #[test]
    fn btn_xl_larger_padding_than_md() {
        let md = resolve_classes("btn btn-primary", &Theme::light());
        let xl = resolve_classes("btn btn-primary btn-xl", &Theme::light());
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
            let s = resolve_classes(&format!("btn {v}"), &Theme::light());
            assert!(s.background.is_some(), "{v} should have a background");
            assert!(s.color.is_some(), "{v} should have a text colour");
        }
    }

    // ── progress ──────────────────────────────────────────

    #[test]
    fn progress_base_sets_radius() {
        let s = resolve_classes("progress", &Theme::light());
        assert_eq!(s.border_radius, progress::RADIUS);
        assert!(s.background.is_none());
    }

    #[test]
    fn progress_success_sets_fill() {
        let s = resolve_classes("progress progress-success", &Theme::light());
        let bg = s.background.expect("fill colour");
        assert_eq!(
            (bg.r, bg.g, bg.b),
            (
                progress::SUCCESS_BG.r,
                progress::SUCCESS_BG.g,
                progress::SUCCESS_BG.b
            )
        );
    }

    #[test]
    fn progress_all_colour_variants_have_bg() {
        let variants = [
            "progress-primary",
            "progress-secondary",
            "progress-accent",
            "progress-success",
            "progress-error",
            "progress-warning",
            "progress-info",
            "progress-neutral",
        ];
        for v in variants {
            let s = resolve_classes(&format!("progress {v}"), &Theme::light());
            assert!(s.background.is_some(), "{v} should have a fill colour");
        }
    }
}
