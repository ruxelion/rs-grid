// AUTO-GENERATED — do not edit by hand.
// Regenerate: just gen-class-map  (or: cd examples/basic-leptos && npm run gen)
// Source: daisyui/theme/object.js + daisyui/components/*/object.js
//         Light theme — daisyui v5.5.19
// Generated: 2026-04-12

use rs_grid_scene::primitives::Color;

// ── Shared semantic colours (light theme, same across all components) ────────
// Derived from oklch() values in daisyui/theme/object.js
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

/// base-200 colour — used by ghost variants and skeleton
pub const BASE_200:     Color = Color::rgb(248, 248, 248);
/// base-content — dark text on light backgrounds
pub const BASE_CONTENT: Color = Color::rgb(24, 24, 27);
/// Global border width from --border
pub const BORDER_W:     f64   = 1.0;

// ── Per-component geometry modules ────────────────────────────────────────────
// Each module contains RADIUS, BORDER, FONT_SIZE, optional Sz size table,
// and colour constants for variants available on that component.
// Colours are redundant with the shared constants above but are provided
// for ergonomic access via e.g. badge::SUCCESS_BG.

/// Auto-generated from daisyui v5.5.19 `components/badge/object.js`
pub mod badge {
    use rs_grid_scene::primitives::Color;

    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 8.0;
    pub const BORDER: f64 = 1.0;
    pub const PADDING_X: f64 = 11.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Size system ───────────────────────────────────────────────────────
    /// Per-size geometry: padding-inline (px), padding-block (py),
    /// and font-size delta relative to the 14px base (fd).
    pub struct Sz { pub px: f64, pub py: f64, pub fd: f64 }

    /// h=16.0px  font=10.0px
    pub const XS: Sz = Sz { px: 7.0, py: 3.0, fd: -4.0 };
    /// h=20.0px  font=12.0px
    pub const SM: Sz = Sz { px: 9.0, py: 4.0, fd: -2.0 };
    /// h=24.0px  font=14.0px
    pub const MD: Sz = Sz { px: 11.0, py: 5.0, fd: 0.0 };
    /// h=28.0px  font=16.0px
    pub const LG: Sz = Sz { px: 13.0, py: 6.0, fd: 2.0 };
    /// h=32.0px  font=18.0px
    pub const XL: Sz = Sz { px: 15.0, py: 7.0, fd: 4.0 };

    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────
    // _BG = badge/component fill  |  _FG = text/content colour
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
    pub const GHOST_BG: Color = Color::rgb(248, 248, 248);
}

/// Auto-generated from daisyui v5.5.19 `components/button/object.js`
pub mod btn {
    use rs_grid_scene::primitives::Color;

    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 4.0;
    pub const BORDER: f64 = 1.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Size system ───────────────────────────────────────────────────────
    /// Per-size geometry: padding-inline (px), padding-block (py),
    /// and font-size delta relative to the 14px base (fd).
    pub struct Sz { pub px: f64, pub py: f64, pub fd: f64 }

    /// h=24.0px  font=10.0px
    pub const XS: Sz = Sz { px: 11.0, py: 7.0, fd: -4.0 };
    /// h=32.0px  font=12.0px
    pub const SM: Sz = Sz { px: 15.0, py: 10.0, fd: -2.0 };
    /// h=40.0px  font=14.0px
    pub const MD: Sz = Sz { px: 19.0, py: 13.0, fd: 0.0 };
    /// h=48.0px  font=16.0px
    pub const LG: Sz = Sz { px: 23.0, py: 16.0, fd: 2.0 };
    /// h=56.0px  font=18.0px
    pub const XL: Sz = Sz { px: 27.0, py: 19.0, fd: 4.0 };

    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────
    // _BG = badge/component fill  |  _FG = text/content colour
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
    pub const GHOST_BG: Color = Color::rgb(248, 248, 248);
}

/// Auto-generated from daisyui v5.5.19 `components/alert/object.js`
pub mod alert {
    use rs_grid_scene::primitives::Color;

    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 8.0;
    pub const BORDER: f64 = 1.0;
    pub const PADDING_X: f64 = 16.0;
    pub const PADDING_Y: f64 = 12.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────
    // _BG = badge/component fill  |  _FG = text/content colour
    pub const SUCCESS_BG: Color = Color::rgb(0, 211, 144);
    pub const SUCCESS_FG: Color = Color::rgb(0, 76, 57);
    pub const ERROR_BG: Color = Color::rgb(255, 98, 125);
    pub const ERROR_FG: Color = Color::rgb(77, 2, 24);
    pub const WARNING_BG: Color = Color::rgb(252, 183, 0);
    pub const WARNING_FG: Color = Color::rgb(121, 50, 5);
    pub const INFO_BG: Color = Color::rgb(0, 186, 254);
    pub const INFO_FG: Color = Color::rgb(4, 46, 73);
    pub const GHOST_BG: Color = Color::rgb(248, 248, 248);
}

/// Auto-generated from daisyui v5.5.19 `components/kbd/object.js`
pub mod kbd {
    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 4.0;
    pub const BORDER: f64 = 1.0;
    pub const PADDING_X: f64 = 8.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Size system ───────────────────────────────────────────────────────
    /// Per-size geometry: padding-inline (px), padding-block (py),
    /// and font-size delta relative to the 14px base (fd).
    pub struct Sz { pub px: f64, pub py: f64, pub fd: f64 }

    /// h=16.0px  font=10.0px
    pub const XS: Sz = Sz { px: 7.0, py: 3.0, fd: -4.0 };
    /// h=20.0px  font=12.0px
    pub const SM: Sz = Sz { px: 9.0, py: 4.0, fd: -2.0 };
    /// h=24.0px  font=14.0px
    pub const MD: Sz = Sz { px: 11.0, py: 5.0, fd: 0.0 };
    /// h=28.0px  font=16.0px
    pub const LG: Sz = Sz { px: 13.0, py: 6.0, fd: 2.0 };
    /// h=32.0px  font=18.0px
    pub const XL: Sz = Sz { px: 15.0, py: 7.0, fd: 4.0 };
}

/// Auto-generated from daisyui v5.5.19 `components/status/object.js`
pub mod status {
    use rs_grid_scene::primitives::Color;

    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 8.0;
    pub const BORDER: f64 = 1.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Size system ───────────────────────────────────────────────────────
    /// Per-size geometry: padding-inline (px), padding-block (py),
    /// and font-size delta relative to the 14px base (fd).
    pub struct Sz { pub px: f64, pub py: f64, pub fd: f64 }

    /// h=16.0px  font=10.0px
    pub const XS: Sz = Sz { px: 7.0, py: 3.0, fd: -4.0 };
    /// h=20.0px  font=12.0px
    pub const SM: Sz = Sz { px: 9.0, py: 4.0, fd: -2.0 };
    /// h=24.0px  font=14.0px
    pub const MD: Sz = Sz { px: 11.0, py: 5.0, fd: 0.0 };
    /// h=28.0px  font=16.0px
    pub const LG: Sz = Sz { px: 13.0, py: 6.0, fd: 2.0 };
    /// h=32.0px  font=18.0px
    pub const XL: Sz = Sz { px: 15.0, py: 7.0, fd: 4.0 };

    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────
    // _BG = badge/component fill  |  _FG = text/content colour
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
    pub const GHOST_BG: Color = Color::rgb(248, 248, 248);
}

/// Auto-generated from daisyui v5.5.19 `components/progress/object.js`
pub mod progress {
    use rs_grid_scene::primitives::Color;

    // ── Geometry ──────────────────────────────────────────────────────────
    pub const RADIUS: f64 = 8.0;
    pub const BORDER: f64 = 1.0;
    pub const FONT_SIZE: f64 = 14.0;

    // ── Colour variants (oklch → sRGB, light theme) ──────────────────────
    // _BG = badge/component fill  |  _FG = text/content colour
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
    pub const GHOST_BG: Color = Color::rgb(248, 248, 248);
}