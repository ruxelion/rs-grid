use crate::primitives::Color;

/// Visual properties for rendering a grid.
///
/// All color, typography, and spacing values live here.
/// `dpr` (device pixel ratio) is intentionally absent — it is a
/// hardware property, not a theme property, and stays on `SceneBuilder`.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    // ── palette ──────────────────────────────────────────────────────────────
    pub bg: Color,
    pub header_bg: Color,
    pub header_text: Color,
    pub cell_text: Color,
    pub grid_line: Color,
    pub header_border: Color,
    pub selection_fill: Color,
    pub selection_border: Color,
    pub scrollbar_track: Color,
    pub scrollbar_thumb: Color,

    /// Subtle background for odd data rows (0 = same as `bg`).
    pub row_alt_bg: Color,
    /// Background overlay for the row under the cursor (transparent = disabled).
    pub row_hover_bg: Color,

    // ── typography ───────────────────────────────────────────────────────────
    pub font_size: f64,
    pub header_font_size: f64,
    /// Render column header labels with font-weight 600.
    pub header_font_bold: bool,

    // ── search ──────────────────────────────────────────────────────────────
    /// Background highlight for cells matching the active search query.
    pub search_highlight: Color,
    /// Background highlight for the current (focused) search match.
    pub search_current: Color,

    // ── spacing ──────────────────────────────────────────────────────────────
    pub cell_padding: f64,

    // ── scrollbar ─────────────────────────────────────────────────────────────
    /// Track + thumb total width in logical pixels.
    pub scrollbar_width: f64,
    /// Corner radius of the thumb in logical pixels.
    pub scrollbar_radius: f64,
}

impl Theme {
    /// Light theme — AG Grid-inspired palette.
    pub fn light() -> Self {
        Self {
            bg: Color::rgb(255, 255, 255),
            header_bg: Color::rgb(248, 249, 250),
            header_text: Color::rgb(24, 29, 31),
            cell_text: Color::rgb(24, 29, 31),
            grid_line: Color::rgb(224, 224, 224),
            header_border: Color::rgb(186, 191, 199),
            selection_fill: Color::rgba(31, 119, 220, 46),
            selection_border: Color::rgba(31, 119, 220, 210),
            scrollbar_track: Color::rgb(241, 241, 241),
            scrollbar_thumb: Color::rgba(100, 100, 110, 160),
            row_alt_bg: Color::rgb(252, 252, 253),
            row_hover_bg: Color::rgba(0, 0, 0, 10),
            search_highlight: Color::rgba(255, 213, 0, 80),
            search_current: Color::rgba(255, 165, 0, 140),
            font_size: 14.0,
            header_font_size: 12.0,
            header_font_bold: true,
            cell_padding: 10.0,
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
        }
    }

    /// Dark theme — Tokyo Night palette.
    pub fn dark() -> Self {
        Self {
            bg: Color::rgb(26, 27, 38),
            header_bg: Color::rgb(36, 40, 59),
            header_text: Color::rgb(169, 177, 214),
            cell_text: Color::rgb(192, 202, 245),
            grid_line: Color::rgb(42, 47, 69),
            header_border: Color::rgb(61, 68, 102),
            selection_fill: Color::rgba(122, 162, 255, 51),
            selection_border: Color::rgba(122, 162, 255, 204),
            scrollbar_track: Color::rgb(31, 35, 53),
            scrollbar_thumb: Color::rgba(169, 177, 214, 102),
            row_alt_bg: Color::rgb(30, 32, 48),
            row_hover_bg: Color::rgba(255, 255, 255, 10),
            search_highlight: Color::rgba(255, 213, 0, 80),
            search_current: Color::rgba(255, 165, 0, 140),
            font_size: 14.0,
            header_font_size: 13.0,
            header_font_bold: true,
            cell_padding: 10.0,
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
