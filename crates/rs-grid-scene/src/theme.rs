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

    /// Subtle background for odd data rows (0 = same as `bg`).
    pub row_alt_bg:       Color,

    // ── typography ───────────────────────────────────────────────────────────
    pub font_size:        f64,
    pub header_font_size: f64,
    /// Render column header labels with font-weight 600.
    pub header_font_bold: bool,

    // ── spacing ──────────────────────────────────────────────────────────────
    pub cell_padding:     f64,

    // ── scrollbar ─────────────────────────────────────────────────────────────
    /// Track + thumb total width in logical pixels.
    pub scrollbar_width:  f64,
    /// Corner radius of the thumb in logical pixels.
    pub scrollbar_radius: f64,
}

impl Theme {
    /// Light theme — AG Grid-inspired palette.
    pub fn light() -> Self {
        Self {
            bg:               Color::rgb(255, 255, 255),
            header_bg:        Color::rgb(248, 249, 250),
            header_text:      Color::rgb(24,  29,  31),
            cell_text:        Color::rgb(24,  29,  31),
            grid_line:        Color::rgb(224, 224, 224),
            header_border:    Color::rgb(186, 191, 199),
            selection_fill:   Color::rgba(31,  119, 220, 46),
            selection_border: Color::rgba(31,  119, 220, 210),
            scrollbar_track:  Color::rgb(241, 241, 241),
            scrollbar_thumb:  Color::rgba(100, 100, 110, 160),
            row_alt_bg:       Color::rgb(252, 252, 253),
            font_size:        14.0,
            header_font_size: 12.0,
            header_font_bold: true,
            cell_padding:     10.0,
            scrollbar_width:  14.0,
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
            row_alt_bg:       Color::rgb(22,  22,  30),
            font_size:        14.0,
            header_font_size: 12.0,
            header_font_bold: true,
            cell_padding:     10.0,
            scrollbar_width:  14.0,
            scrollbar_radius: 4.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
