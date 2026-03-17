use crate::primitives::Color;

/// Visual properties for rendering a grid.
///
/// All color, typography, and spacing values live here.
/// `dpr` (device pixel ratio) is intentionally absent — it is a
/// hardware property, not a theme property, and stays on `SceneBuilder`.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    // ── palette ──────────────────────────────────────────────────────────────
    pub bg:               Color,
    pub header_bg:        Color,
    pub header_text:      Color,
    pub cell_text:        Color,
    pub grid_line:        Color,
    pub header_border:    Color,
    pub selection_fill:   Color,
    pub selection_border: Color,
    pub scrollbar_track:  Color,
    pub scrollbar_thumb:  Color,

    // ── typography ───────────────────────────────────────────────────────────
    pub font_size:        f64,
    pub header_font_size: f64,

    // ── spacing ──────────────────────────────────────────────────────────────
    pub cell_padding:     f64,

    // ── scrollbar ─────────────────────────────────────────────────────────────
    /// Track + thumb total width in logical pixels.
    pub scrollbar_width:  f64,
    /// Corner radius of the thumb in logical pixels.
    pub scrollbar_radius: f64,
}

impl Theme {
    /// Light theme — reproduces the original hardcoded colours exactly.
    pub fn light() -> Self {
        Self {
            bg:               Color::rgb(255, 255, 255),
            header_bg:        Color::rgb(242, 242, 247),
            header_text:      Color::rgb(90,  90,  100),
            cell_text:        Color::rgb(20,  20,  20),
            grid_line:        Color::rgb(210, 210, 215),
            header_border:    Color::rgb(180, 180, 190),
            selection_fill:   Color::rgba(59,  130, 246, 50),
            selection_border: Color::rgba(59,  130, 246, 200),
            scrollbar_track:  Color::rgba(0,   0,   0,   18),
            scrollbar_thumb:  Color::rgba(90,  90,  100, 170),
            font_size:        13.0,
            header_font_size: 12.0,
            cell_padding:     8.0,
            scrollbar_width:  8.0,
            scrollbar_radius: 4.0,
        }
    }

    /// Dark theme — blue-grey palette, bright accent selection.
    pub fn dark() -> Self {
        Self {
            bg:               Color::rgb(18,  18,  24),
            header_bg:        Color::rgb(28,  28,  38),
            header_text:      Color::rgb(160, 160, 180),
            cell_text:        Color::rgb(220, 220, 230),
            grid_line:        Color::rgb(45,  45,  58),
            header_border:    Color::rgb(55,  55,  70),
            selection_fill:   Color::rgba(99,  160, 255, 55),
            selection_border: Color::rgba(99,  160, 255, 210),
            scrollbar_track:  Color::rgba(255, 255, 255, 15),
            scrollbar_thumb:  Color::rgba(180, 180, 200, 140),
            font_size:        13.0,
            header_font_size: 12.0,
            cell_padding:     8.0,
            scrollbar_width:  8.0,
            scrollbar_radius: 4.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
