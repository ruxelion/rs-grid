use rs_grid_scene::{primitives::Color, Theme};

/// Build a `Theme` by reading `--rs-grid-*` CSS custom properties from the
/// document root element (`:root`).
///
/// Each variable falls back to the corresponding `Theme::light()` value when
/// the variable is absent or cannot be parsed.
///
/// ## Supported variables
///
/// | Variable                        | Maps to                      |
/// |--------------------------------|------------------------------|
/// | `--rs-grid-bg`                 | `Theme::bg`                  |
/// | `--rs-grid-header-bg`          | `Theme::header_bg`           |
/// | `--rs-grid-header-text`        | `Theme::header_text`         |
/// | `--rs-grid-cell-text`          | `Theme::cell_text`           |
/// | `--rs-grid-grid-line`          | `Theme::grid_line`           |
/// | `--rs-grid-header-border`           | `Theme::header_border`            |
/// | `--rs-grid-header-separator-inset` | `Theme::header_separator_inset`   |
/// | `--rs-grid-header-separator-width` | `Theme::header_separator_width`   |
/// | `--rs-grid-selection-fill`     | `Theme::selection_fill`      |
/// | `--rs-grid-selection-border`   | `Theme::selection_border`    |
/// | `--rs-grid-scrollbar-track`    | `Theme::scrollbar_track`     |
/// | `--rs-grid-scrollbar-thumb`    | `Theme::scrollbar_thumb`     |
/// | `--rs-grid-scrollbar-width`    | `Theme::scrollbar_width`     |
/// | `--rs-grid-scrollbar-radius`   | `Theme::scrollbar_radius`    |
/// | `--rs-grid-scrollbar-inset`    | `Theme::scrollbar_inset`     |
/// | `--rs-grid-row-alt-bg`         | `Theme::row_alt_bg`          |
/// | `--rs-grid-font-size`          | `Theme::font_size`           |
/// | `--rs-grid-header-font-size`   | `Theme::header_font_size`    |
/// | `--rs-grid-header-font-bold`   | `Theme::header_font_bold`    |
/// | `--rs-grid-cell-padding`       | `Theme::cell_padding`        |
/// | `--rs-grid-drag-overlay`            | `Theme::drag_overlay`            |
/// | `--rs-grid-drag-ghost-bg`           | `Theme::drag_ghost_bg`           |
/// | `--rs-grid-drag-ghost-text`         | `Theme::drag_ghost_text`         |
/// | `--rs-grid-drag-insert-line-width`  | `Theme::drag_insert_line_width`  |
/// | `--rs-grid-drag-ghost-radius`       | `Theme::drag_ghost_radius`       |
/// | `--rs-grid-drag-ghost-border-width` | `Theme::drag_ghost_border_width` |
/// | `--rs-grid-drag-anim-alpha`         | `Theme::drag_anim_alpha`         |
/// | `--rs-grid-sort-arrow-width`        | `Theme::sort_arrow_width`        |
/// | `--rs-grid-sort-arrow-height`       | `Theme::sort_arrow_height`       |
/// | `--rs-grid-header-height`           | `Theme::header_height`           |
/// | `--rs-grid-row-height`              | `Theme::row_height`              |
/// | `--rs-grid-header-menu-icon`        | `Theme::header_menu_icon`        |
/// | `--rs-grid-gutter-bg`              | `Theme::gutter_bg`               |
/// | `--rs-grid-gutter-text`            | `Theme::gutter_text`             |
/// | `--rs-grid-gutter-font-size`       | `Theme::gutter_font_size`        |
/// | `--rs-grid-gutter-font-bold`       | `Theme::gutter_font_bold`        |
/// | `--rs-grid-gutter-border`          | `Theme::gutter_border`           |
pub fn theme_from_css_vars() -> Theme {
    let fallback = Theme::light();

    let Some(style) = root_computed_style() else {
        return fallback;
    };

    let color = |name: &str, fb: Color| -> Color {
        let raw = get_var(&style, name);
        parse_color(&raw).unwrap_or(fb)
    };

    let px = |name: &str, fb: f64| -> f64 {
        let raw = get_var(&style, name);
        parse_px(&raw).unwrap_or(fb)
    };

    let bool_var = |name: &str, fb: bool| -> bool {
        match get_var(&style, name).trim() {
            "0" | "false" => false,
            "1" | "true" => true,
            _ => fb,
        }
    };

    Theme {
        bg: color("--rs-grid-bg", fallback.bg),
        header_bg: color("--rs-grid-header-bg", fallback.header_bg),
        header_text: color("--rs-grid-header-text", fallback.header_text),
        cell_text: color("--rs-grid-cell-text", fallback.cell_text),
        grid_line: color("--rs-grid-grid-line", fallback.grid_line),
        header_border: color(
            "--rs-grid-header-border",
            fallback.header_border,
        ),
        header_separator_inset: px(
            "--rs-grid-header-separator-inset",
            fallback.header_separator_inset,
        ),
        header_separator_width: px(
            "--rs-grid-header-separator-width",
            fallback.header_separator_width,
        ),
        selection_fill: color(
            "--rs-grid-selection-fill",
            fallback.selection_fill,
        ),
        selection_border: color(
            "--rs-grid-selection-border",
            fallback.selection_border,
        ),
        scrollbar_track: color(
            "--rs-grid-scrollbar-track",
            fallback.scrollbar_track,
        ),
        scrollbar_thumb: color(
            "--rs-grid-scrollbar-thumb",
            fallback.scrollbar_thumb,
        ),
        row_alt_bg: color("--rs-grid-row-alt-bg", fallback.row_alt_bg),
        row_hover_bg: color("--rs-grid-row-hover-bg", fallback.row_hover_bg),
        scrollbar_width: px(
            "--rs-grid-scrollbar-width",
            fallback.scrollbar_width,
        ),
        scrollbar_radius: px(
            "--rs-grid-scrollbar-radius",
            fallback.scrollbar_radius,
        ),
        scrollbar_inset: px(
            "--rs-grid-scrollbar-inset",
            fallback.scrollbar_inset,
        ),
        font_size: px("--rs-grid-font-size", fallback.font_size),
        header_font_size: px(
            "--rs-grid-header-font-size",
            fallback.header_font_size,
        ),
        header_font_bold: bool_var(
            "--rs-grid-header-font-bold",
            fallback.header_font_bold,
        ),
        cell_padding: px("--rs-grid-cell-padding", fallback.cell_padding),
        flash_fill: color("--rs-grid-flash-fill", fallback.flash_fill),
        flash_border: color(
            "--rs-grid-flash-border",
            fallback.flash_border,
        ),
        search_highlight: color(
            "--rs-grid-search-highlight",
            fallback.search_highlight,
        ),
        search_current: color(
            "--rs-grid-search-current",
            fallback.search_current,
        ),
        skeleton_fg: color("--rs-grid-skeleton-fg", fallback.skeleton_fg),
        drag_overlay: color(
            "--rs-grid-drag-overlay",
            fallback.drag_overlay,
        ),
        drag_ghost_bg: color(
            "--rs-grid-drag-ghost-bg",
            fallback.drag_ghost_bg,
        ),
        drag_ghost_text: color(
            "--rs-grid-drag-ghost-text",
            fallback.drag_ghost_text,
        ),
        drag_insert_line_width: px(
            "--rs-grid-drag-insert-line-width",
            fallback.drag_insert_line_width,
        ),
        drag_ghost_radius: px(
            "--rs-grid-drag-ghost-radius",
            fallback.drag_ghost_radius,
        ),
        drag_ghost_border_width: px(
            "--rs-grid-drag-ghost-border-width",
            fallback.drag_ghost_border_width,
        ),
        drag_anim_alpha: px(
            "--rs-grid-drag-anim-alpha",
            fallback.drag_anim_alpha,
        ),
        sort_arrow_width: px(
            "--rs-grid-sort-arrow-width",
            fallback.sort_arrow_width,
        ),
        sort_arrow_height: px(
            "--rs-grid-sort-arrow-height",
            fallback.sort_arrow_height,
        ),
        header_height: px(
            "--rs-grid-header-height",
            fallback.header_height,
        ),
        row_height: px("--rs-grid-row-height", fallback.row_height),
        header_menu_icon: color(
            "--rs-grid-header-menu-icon",
            fallback.header_menu_icon,
        ),
        header_menu_icon_hover_bg: color(
            "--rs-grid-header-menu-icon-hover-bg",
            fallback.header_menu_icon_hover_bg,
        ),
        header_menu_icon_radius: px(
            "--rs-grid-header-menu-icon-radius",
            fallback.header_menu_icon_radius,
        ),
        header_menu_icon_margin_r: px(
            "--rs-grid-header-menu-icon-margin-r",
            fallback.header_menu_icon_margin_r,
        ),
        header_menu_icon_btn_w: px(
            "--rs-grid-header-menu-icon-btn-w",
            fallback.header_menu_icon_btn_w,
        ),
        header_menu_icon_btn_h: px(
            "--rs-grid-header-menu-icon-btn-h",
            fallback.header_menu_icon_btn_h,
        ),
        header_menu_icon_dot_r: px(
            "--rs-grid-header-menu-icon-dot-r",
            fallback.header_menu_icon_dot_r,
        ),
        gutter_bg: color("--rs-grid-gutter-bg", fallback.gutter_bg),
        gutter_text: color(
            "--rs-grid-gutter-text",
            fallback.gutter_text,
        ),
        gutter_font_size: px(
            "--rs-grid-gutter-font-size",
            fallback.gutter_font_size,
        ),
        gutter_font_bold: bool_var(
            "--rs-grid-gutter-font-bold",
            fallback.gutter_font_bold,
        ),
        gutter_border: color(
            "--rs-grid-gutter-border",
            fallback.gutter_border,
        ),
    }
}

// ── DOM helpers ───────────────────────────────────────────────────────────────

pub(crate) fn root_computed_style() -> Option<web_sys::CssStyleDeclaration> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let root = document.document_element()?;
    window.get_computed_style(&root).ok().flatten()
}

pub(crate) fn get_var(
    style: &web_sys::CssStyleDeclaration,
    name: &str,
) -> String {
    style
        .get_property_value(name)
        .unwrap_or_default()
        .trim()
        .to_string()
}

// ── parsers ───────────────────────────────────────────────────────────────────

/// Parse a CSS color string into a `Color`.
///
/// Supported formats:
/// - `#rrggbb` / `#rrggbbaa`
/// - `#rgb` / `#rgba`
/// - `rgb(r, g, b)`
/// - `rgba(r, g, b, a)`  — `a` is a 0–1 float
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('#') {
        parse_hex(s)
    } else if s.starts_with("rgba(") {
        parse_rgba_fn(s)
    } else if s.starts_with("rgb(") {
        parse_rgb_fn(s)
    } else {
        None
    }
}

fn parse_hex(s: &str) -> Option<Color> {
    let h = s.trim_start_matches('#');
    match h.len() {
        3 => {
            let r = u8::from_str_radix(&h[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&h[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&h[2..3].repeat(2), 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&h[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&h[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&h[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&h[3..4].repeat(2), 16).ok()?;
            Some(Color::rgba(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            let a = u8::from_str_radix(&h[6..8], 16).ok()?;
            Some(Color::rgba(r, g, b, a))
        }
        _ => None,
    }
}

fn parse_rgb_fn(s: &str) -> Option<Color> {
    let inner = s.trim_start_matches("rgb(").trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let r = parts[0].trim().parse::<u8>().ok()?;
    let g = parts[1].trim().parse::<u8>().ok()?;
    let b = parts[2].trim().parse::<u8>().ok()?;
    Some(Color::rgb(r, g, b))
}

fn parse_rgba_fn(s: &str) -> Option<Color> {
    let inner = s.trim_start_matches("rgba(").trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let r = parts[0].trim().parse::<u8>().ok()?;
    let g = parts[1].trim().parse::<u8>().ok()?;
    let b = parts[2].trim().parse::<u8>().ok()?;
    let a_f: f64 = parts[3].trim().parse().ok()?;
    let a = (a_f * 255.0).round() as u8;
    Some(Color::rgba(r, g, b, a))
}

/// Parse a CSS length with optional `px` suffix into `f64`.
fn parse_px(s: &str) -> Option<f64> {
    let s = s.trim().trim_end_matches("px");
    s.parse::<f64>().ok()
}
