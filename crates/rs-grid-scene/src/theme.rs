use crate::primitives::Color;

/// Visual properties for rendering a grid.
///
/// All color, typography, and spacing values live here.
/// `dpr` (device pixel ratio) is intentionally absent — it is a
/// hardware property, not a theme property, and stays on `SceneBuilder`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
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
    /// Vertical inset applied to each end of the column separator
    /// line in the header. 0 = full height; 6 = shorter divider.
    pub header_separator_inset: f64,
    /// Width of the column separator line in the header, in logical
    /// pixels.
    pub header_separator_width: f64,
    /// Fill color for the selection rectangle.
    pub selection_fill: Color,
    /// Border color for the selection rectangle.
    pub selection_border: Color,
    /// Fill color for column headers when the column is in
    /// the selection range. Defaults to `selection_fill`.
    pub header_selection_fill: Color,
    /// Fill color for the row-number gutter when the row is
    /// in the selection range. Defaults to `selection_fill`.
    pub gutter_selection_fill: Color,
    /// Scrollbar track background.
    pub scrollbar_track: Color,
    /// Scrollbar thumb color.
    pub scrollbar_thumb: Color,

    /// Subtle background for odd data rows (0 = same as `bg`).
    pub row_alt_bg: Color,
    /// Background overlay for the row under the cursor (transparent = disabled).
    pub row_hover_bg: Color,

    // ── row / header dimensions ──────────────────────────────────────────────
    /// Height of the sticky header row in logical pixels.
    pub header_height: f64,
    /// Height of each data row in logical pixels.
    pub row_height: f64,

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

    // ── column header menu icon ───────────────────────────────────────────────
    /// Color of the three-dot (⋮) menu icon in each column header.
    pub header_menu_icon: Color,
    /// Background color of the menu icon button on hover.
    pub header_menu_icon_hover_bg: Color,
    /// Corner radius of the menu icon hover background, in logical pixels.
    pub header_menu_icon_radius: f64,
    /// Gap between the button right edge and the column right edge,
    /// in logical pixels. Increase to shift the icon leftward.
    pub header_menu_icon_margin_r: f64,
    /// Width of the menu icon button in logical pixels.
    pub header_menu_icon_btn_w: f64,
    /// Height of the menu icon button in logical pixels.
    /// Use 0.0 to auto-size to `header_height − 12`.
    pub header_menu_icon_btn_h: f64,
    /// Radius of each dot in the three-dot icon, in logical pixels.
    pub header_menu_icon_dot_r: f64,

    // ── pinned columns ────────────────────────────────────────────────────────
    /// Background of the pinned-column data band.
    /// Defaults to `bg`.
    pub pinned_bg: Color,
    /// Background of the pinned-column header band.
    /// Defaults to `header_bg`.
    pub pinned_header_bg: Color,
    /// Color of the vertical separator at the right edge of the
    /// pinned band. Defaults to `header_border`.
    pub pinned_separator_color: Color,
    /// Width of the pinned-column separator line in logical pixels.
    pub pinned_separator_width: f64,

    // ── row-number gutter ─────────────────────────────────────────────────────
    /// Background of the row-number gutter column.
    pub gutter_bg: Color,
    /// Text color of row numbers in the gutter.
    pub gutter_text: Color,
    /// Font size of row numbers in logical pixels.
    pub gutter_font_size: f64,
    /// Render row numbers with font-weight 600.
    pub gutter_font_bold: bool,
    /// Right border color of the gutter column.
    pub gutter_border: Color,
}

impl Theme {
    /// Light theme — matches rs-grid-theme.css `:root` block.
    pub fn light() -> Self {
        Self {
            // palette
            bg: Color::rgb(255, 255, 255),
            header_bg: Color::rgb(249, 249, 249),
            header_text: Color::rgb(0, 0, 0),
            cell_text: Color::rgb(0, 0, 0),
            grid_line: Color::rgb(224, 224, 224),
            header_border: Color::rgb(224, 224, 224),
            header_separator_inset: 15.0,
            header_separator_width: 2.0,
            selection_fill: Color::rgba(50, 190, 250, 51),
            selection_border: Color::rgba(50, 190, 250, 204),
            header_selection_fill: Color::rgba(0, 0, 0, 18),
            gutter_selection_fill: Color::rgba(0, 0, 0, 18),
            scrollbar_track: Color::rgb(241, 241, 241),
            scrollbar_thumb: Color::rgba(100, 100, 110, 166),
            row_alt_bg: Color::rgb(255, 255, 255),
            row_hover_bg: Color::rgba(50, 190, 250, 20),
            // row / header dimensions
            header_height: 50.0,
            row_height: 40.0,
            // typography
            font_size: 14.0,
            header_font_size: 14.0,
            header_font_bold: true,
            // flash
            flash_fill: Color::rgba(255, 220, 0, 255),
            flash_border: Color::rgba(255, 220, 0, 255),
            // search
            search_highlight: Color::rgba(255, 213, 0, 77),
            search_current: Color::rgba(255, 165, 0, 140),
            // skeleton
            skeleton_fg: Color::rgba(200, 200, 200, 77),
            // spacing
            cell_padding: 10.0,
            // scrollbar
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
            scrollbar_inset: 2.0,
            // column drag
            drag_overlay: Color::rgba(0, 0, 0, 8),
            drag_ghost_bg: Color::rgb(249, 249, 249),
            drag_ghost_text: Color::rgb(0, 0, 0),
            drag_insert_line_width: 1.0,
            drag_ghost_radius: 4.0,
            drag_ghost_border_width: 1.0,
            drag_anim_alpha: 0.5,
            // sort indicator
            sort_arrow_width: 4.0,
            sort_arrow_height: 3.5,
            // column header menu icon
            header_menu_icon: Color::rgba(0, 0, 0, 255),
            header_menu_icon_hover_bg: Color::rgba(0, 0, 0, 15),
            header_menu_icon_radius: 3.0,
            header_menu_icon_margin_r: 10.0,
            header_menu_icon_btn_w: 22.0,
            header_menu_icon_btn_h: 22.0,
            header_menu_icon_dot_r: 1.2,
            // pinned columns
            pinned_bg: Color::rgb(255, 255, 255),
            pinned_header_bg: Color::rgb(249, 249, 249),
            pinned_separator_color: Color::rgb(224, 224, 224),
            pinned_separator_width: 1.0,
            // row-number gutter
            gutter_bg: Color::rgb(249, 249, 249),
            gutter_text: Color::rgba(0, 0, 0, 255),
            gutter_font_size: 14.0,
            gutter_font_bold: true,
            gutter_border: Color::rgb(220, 220, 220),
        }
    }

    /// Dark theme — Tokyo Night palette.
    pub fn dark() -> Self {
        Self {
            // palette
            bg: Color::rgb(26, 27, 38),
            header_bg: Color::rgb(36, 40, 59),
            header_text: Color::rgb(169, 177, 214),
            cell_text: Color::rgb(192, 202, 245),
            grid_line: Color::rgb(42, 47, 69),
            header_border: Color::rgb(61, 68, 102),
            header_separator_inset: 15.0,
            header_separator_width: 2.0,
            selection_fill: Color::rgba(122, 162, 255, 51),
            selection_border: Color::rgba(122, 162, 255, 204),
            header_selection_fill: Color::rgba(0, 0, 0, 10),
            gutter_selection_fill: Color::rgba(0, 0, 0, 10),
            scrollbar_track: Color::rgb(31, 35, 53),
            scrollbar_thumb: Color::rgba(169, 177, 214, 102),
            row_alt_bg: Color::rgb(30, 32, 48),
            row_hover_bg: Color::rgba(255, 255, 255, 10),
            // row / header dimensions
            header_height: 50.0,
            row_height: 40.0,
            // typography
            font_size: 14.0,
            header_font_size: 13.0,
            header_font_bold: true,
            // flash
            flash_fill: Color::rgba(255, 193, 7, 180),
            flash_border: Color::rgba(255, 193, 7, 210),
            // search
            search_highlight: Color::rgba(255, 213, 0, 80),
            search_current: Color::rgba(255, 165, 0, 140),
            // skeleton
            skeleton_fg: Color::rgba(60, 65, 90, 100),
            // spacing
            cell_padding: 10.0,
            // scrollbar
            scrollbar_width: 14.0,
            scrollbar_radius: 4.0,
            scrollbar_inset: 2.0,
            // column drag
            drag_overlay: Color::rgba(128, 128, 128, 0),
            drag_ghost_bg: Color::rgba(36, 40, 59, 180),
            drag_ghost_text: Color::rgba(169, 177, 214, 200),
            drag_insert_line_width: 3.0,
            drag_ghost_radius: 4.0,
            drag_ghost_border_width: 1.0,
            drag_anim_alpha: 0.30,
            // sort indicator
            sort_arrow_width: 4.0,
            sort_arrow_height: 3.5,
            // column header menu icon
            header_menu_icon: Color::rgba(169, 177, 214, 100),
            header_menu_icon_hover_bg: Color::rgba(169, 177, 214, 30),
            header_menu_icon_radius: 4.0,
            header_menu_icon_margin_r: 2.0,
            header_menu_icon_btn_w: 16.0,
            header_menu_icon_btn_h: 0.0,
            header_menu_icon_dot_r: 1.0,
            // pinned columns
            pinned_bg: Color::rgb(26, 27, 38),
            pinned_header_bg: Color::rgb(36, 40, 59),
            pinned_separator_color: Color::rgb(61, 68, 102),
            pinned_separator_width: 1.0,
            // row-number gutter
            gutter_bg: Color::rgb(36, 40, 59),
            gutter_text: Color::rgba(169, 177, 214, 153),
            gutter_font_size: 12.0,
            gutter_font_bold: false,
            gutter_border: Color::rgb(61, 68, 102),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_light() {
        assert_eq!(Theme::default(), Theme::light());
    }

    #[test]
    fn light_and_dark_differ() {
        assert_ne!(Theme::light(), Theme::dark());
    }

    // ── light theme sanity checks ───────────────────────────

    #[test]
    fn light_bg_is_white() {
        let t = Theme::light();
        assert_eq!(t.bg, Color::rgb(255, 255, 255));
    }

    #[test]
    fn light_header_height_positive() {
        let t = Theme::light();
        assert!(t.header_height > 0.0);
    }

    #[test]
    fn light_row_height_positive() {
        let t = Theme::light();
        assert!(t.row_height > 0.0);
    }

    #[test]
    fn light_font_sizes_positive() {
        let t = Theme::light();
        assert!(t.font_size > 0.0);
        assert!(t.header_font_size > 0.0);
    }

    #[test]
    fn light_cell_padding_positive() {
        let t = Theme::light();
        assert!(t.cell_padding > 0.0);
    }

    #[test]
    fn light_scrollbar_dimensions_positive() {
        let t = Theme::light();
        assert!(t.scrollbar_width > 0.0);
        assert!(t.scrollbar_radius > 0.0);
        assert!(t.scrollbar_inset >= 0.0);
    }

    #[test]
    fn light_selection_fill_is_semi_transparent() {
        let t = Theme::light();
        assert!(t.selection_fill.a > 0);
        assert!(t.selection_fill.a < 255);
    }

    #[test]
    fn light_header_font_bold() {
        assert!(Theme::light().header_font_bold);
    }

    #[test]
    fn light_drag_anim_alpha_in_range() {
        let t = Theme::light();
        assert!(t.drag_anim_alpha > 0.0);
        assert!(t.drag_anim_alpha <= 1.0);
    }

    #[test]
    fn light_sort_arrow_dimensions_positive() {
        let t = Theme::light();
        assert!(t.sort_arrow_width > 0.0);
        assert!(t.sort_arrow_height > 0.0);
    }

    // ── dark theme sanity checks ────────────────────────────

    #[test]
    fn dark_bg_is_dark() {
        let t = Theme::dark();
        // All channels below 50 indicates a dark background.
        assert!(t.bg.r < 50 && t.bg.g < 50 && t.bg.b < 50);
    }

    #[test]
    fn dark_header_height_positive() {
        assert!(Theme::dark().header_height > 0.0);
    }

    #[test]
    fn dark_row_height_positive() {
        assert!(Theme::dark().row_height > 0.0);
    }

    #[test]
    fn dark_cell_text_is_light() {
        let t = Theme::dark();
        // At least one channel above 150 to be readable on dark bg.
        assert!(
            t.cell_text.r > 150 || t.cell_text.g > 150 || t.cell_text.b > 150
        );
    }

    #[test]
    fn dark_font_sizes_positive() {
        let t = Theme::dark();
        assert!(t.font_size > 0.0);
        assert!(t.header_font_size > 0.0);
    }

    // ── clone / equality ────────────────────────────────────

    #[test]
    fn theme_clone_equals_original() {
        let t = Theme::light();
        let t2 = t.clone();
        assert_eq!(t, t2);
    }

    #[test]
    fn theme_debug_does_not_panic() {
        let _ = format!("{:?}", Theme::light());
        let _ = format!("{:?}", Theme::dark());
    }
}
