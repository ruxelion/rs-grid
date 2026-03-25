use crate::primitives::Color;

/// Visual properties for rendering a grid.
///
/// All color, typography, and spacing values live here.
/// `dpr` (device pixel ratio) is intentionally absent — it is a
/// hardware property, not a theme property, and stays on `SceneBuilder`.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    // ── palette ──────────────────────────────────────────────────────────────
    /// Default cell background.
    pub bg: Color,
    /// Column header background.
    pub header_bg: Color,
    /// Column header text color.
    pub header_text: Color,
    /// Default cell text color.
    pub cell_text: Color,
    /// Grid line (cell border) color.
    pub grid_line: Color,
    /// Bottom border below the header row.
    pub header_border: Color,
    /// Fill color for the selection rectangle.
    pub selection_fill: Color,
    /// Border color for the selection rectangle.
    pub selection_border: Color,
    /// Scrollbar track background.
    pub scrollbar_track: Color,
    /// Scrollbar thumb color.
    pub scrollbar_thumb: Color,

    /// Subtle background for odd data rows (0 = same as `bg`).
    pub row_alt_bg: Color,
    /// Background overlay for the row under the cursor (transparent = disabled).
    pub row_hover_bg: Color,

    // ── typography ───────────────────────────────────────────────────────────
    /// Cell text font size in logical pixels.
    pub font_size: f64,
    /// Header text font size in logical pixels.
    pub header_font_size: f64,
    /// Render column header labels with font-weight 600.
    pub header_font_bold: bool,

    // ── flash (paste feedback) ───────────────────────────────────────────────
    /// Fill colour for the paste-flash animation (fades from full to transparent).
    pub flash_fill: Color,
    /// Border colour for the paste-flash animation.
    pub flash_border: Color,

    // ── search ──────────────────────────────────────────────────────────────
    /// Background highlight for cells matching the active search query.
    pub search_highlight: Color,
    /// Background highlight for the current (focused) search match.
    pub search_current: Color,

    // ── skeleton (async loading) ────────────────────────────────────────
    /// Foreground colour of skeleton loading bars.
    pub skeleton_fg: Color,

    // ── spacing ──────────────────────────────────────────────────────────────
    /// Horizontal padding inside each cell in logical pixels.
    pub cell_padding: f64,

    // ── scrollbar ─────────────────────────────────────────────────────────────
    /// Track + thumb total width in logical pixels.
    pub scrollbar_width: f64,
    /// Corner radius of the thumb in logical pixels.
    pub scrollbar_radius: f64,
    /// Gap between the track edge and the thumb on each side, in logical
    /// pixels.
    pub scrollbar_inset: f64,

    // ── column drag ───────────────────────────────────────────────────────────
    /// Dim overlay drawn over the source column header during a column drag.
    pub drag_overlay: Color,
    /// Background of the ghost header that follows the cursor during a column
    /// drag (semi-transparent version of `header_bg`).
    pub drag_ghost_bg: Color,
    /// Text color of the ghost header label during a column drag
    /// (semi-transparent version of `header_text`).
    pub drag_ghost_text: Color,
    /// Width of the column insertion indicator line during a drag, in logical
    /// pixels.
    pub drag_insert_line_width: f64,
    /// Corner radius of the ghost badge in logical pixels.
    /// Set to 0.0 to keep the rectangle shape.
    pub drag_ghost_radius: f64,
    /// Border width of the ghost badge in logical pixels.
    pub drag_ghost_border_width: f64,
    /// Exponential-smoothing factor for the column-drag animation
    /// (0–1). Higher = faster. Default 0.30 ≈ 200 ms at 60 fps.
    pub drag_anim_alpha: f64,

    // ── sort indicator ────────────────────────────────────────────────────────
    /// Half-width of the sort arrow triangle, in logical pixels.
    pub sort_arrow_width: f64,
    /// Half-height of the sort arrow triangle, in logical pixels.
    pub sort_arrow_height: f64,
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
            flash_fill: Color::rgba(255, 193, 7, 180),
            flash_border: Color::rgba(255, 193, 7, 210),
            search_highlight: Color::rgba(255, 213, 0, 80),
            search_current: Color::rgba(255, 165, 0, 140),
            skeleton_fg: Color::rgba(200, 200, 200, 100),
            font_size: 14.0,
            header_font_size: 12.0,
            header_font_bold: true,
            cell_padding: 10.0,
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
            scrollbar_inset: 2.0,
            drag_overlay: Color::rgba(128, 128, 128, 0),
            drag_ghost_bg: Color::rgba(248, 249, 250, 180),
            drag_ghost_text: Color::rgba(24, 29, 31, 200),
            drag_insert_line_width: 3.0,
            drag_ghost_radius: 4.0,
            drag_ghost_border_width: 1.0,
            drag_anim_alpha: 0.30,
            sort_arrow_width: 4.0,
            sort_arrow_height: 3.5,
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
            flash_fill: Color::rgba(255, 193, 7, 180),
            flash_border: Color::rgba(255, 193, 7, 210),
            search_highlight: Color::rgba(255, 213, 0, 80),
            search_current: Color::rgba(255, 165, 0, 140),
            skeleton_fg: Color::rgba(60, 65, 90, 100),
            font_size: 14.0,
            header_font_size: 13.0,
            header_font_bold: true,
            cell_padding: 10.0,
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
            scrollbar_inset: 2.0,
            drag_overlay: Color::rgba(128, 128, 128, 0),
            drag_ghost_bg: Color::rgba(36, 40, 59, 180),
            drag_ghost_text: Color::rgba(169, 177, 214, 200),
            drag_insert_line_width: 3.0,
            drag_ghost_radius: 4.0,
            drag_ghost_border_width: 1.0,
            drag_anim_alpha: 0.30,
            sort_arrow_width: 4.0,
            sort_arrow_height: 3.5,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
