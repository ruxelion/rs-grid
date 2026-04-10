// AUTO-GENERATED — do not edit by hand.
// Regenerate: just gen-class-map  (or: cd examples/basic-leptos && npm run gen)
// Source: daisyui/theme/object.js  light theme  (daisyui v5.5.19)
// Generated: 2026-04-10

use crate::primitives::Color;

// ── Badge geometry ─────────────────────────────────────────────────────────
// Derived from DaisyUI v5 CSS variables:
//   --radius-selector = 0.5rem  →  8px
//   --border          = 1px           →  1px
//   --size-selector   = 0.25rem  →  4px
//
// Badge height per size: xs=16 sm=20 md=24 lg=28 xl=32 (px)

pub const BADGE_RADIUS: f64 = 8.0;
pub const BADGE_BORDER: f64 = 1.0;

/// Per-size badge geometry: horizontal padding, vertical padding,
/// font-size delta relative to the 14px base font.
pub struct BadgeSz {
    pub px: f64,  // padding-inline (each side)
    pub py: f64,  // padding-block  (each side)
    pub fd: f64,  // font-size delta vs 14px base
}

pub const SZ_XS: BadgeSz = BadgeSz { px: 7.0, py: 3.0, fd: -4.0 };
pub const SZ_SM: BadgeSz = BadgeSz { px: 9.0, py: 4.0, fd: -2.0 };
pub const SZ_MD: BadgeSz = BadgeSz { px: 11.0, py: 5.0, fd: 0.0 };
pub const SZ_LG: BadgeSz = BadgeSz { px: 13.0, py: 6.0, fd: 2.0 };
pub const SZ_XL: BadgeSz = BadgeSz { px: 15.0, py: 7.0, fd: 4.0 };

// ── Badge colors — oklch → sRGB ────────────────────────────────────────────
// Each variant: _BG = badge fill, _FG = text/content colour.

pub const PRIMARY_BG: Color = Color::rgb(66, 42, 213);
pub const PRIMARY_FG: Color = Color::rgb(224, 231, 255);
pub const SECONDARY_BG: Color = Color::rgb(244, 48, 152);
pub const SECONDARY_FG: Color = Color::rgb(249, 228, 240);
pub const ACCENT_BG: Color = Color::rgb(0, 211, 187);
pub const ACCENT_FG: Color = Color::rgb(8, 77, 73);
pub const SUCCESS_BG: Color = Color::rgb(0, 211, 144);
pub const SUCCESS_FG: Color = Color::rgb(0, 76, 57);
pub const ERROR_BG: Color = Color::rgb(255, 98, 125);
pub const ERROR_FG: Color = Color::rgb(77, 2, 24);
pub const WARNING_BG: Color = Color::rgb(252, 183, 0);
pub const WARNING_FG: Color = Color::rgb(121, 50, 5);
pub const INFO_BG: Color = Color::rgb(0, 186, 254);
pub const INFO_FG: Color = Color::rgb(4, 46, 73);
pub const NEUTRAL_BG: Color = Color::rgb(9, 9, 11);
pub const NEUTRAL_FG: Color = Color::rgb(228, 228, 231);

// base-200 — used by badge-ghost (DaisyUI: bg-base-200 border-base-200)
pub const BASE_200: Color = Color::rgb(248, 248, 248);
