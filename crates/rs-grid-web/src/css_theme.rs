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
/// | `--rs-grid-header-selection-fill` | `Theme::header_selection_fill` |
/// | `--rs-grid-gutter-selection-fill` | `Theme::gutter_selection_fill` |
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
/// | `--rs-grid-pinned-bg`              | `Theme::pinned_bg`               |
/// | `--rs-grid-pinned-header-bg`       | `Theme::pinned_header_bg`        |
/// | `--rs-grid-pinned-separator-color` | `Theme::pinned_separator_color`  |
/// | `--rs-grid-pinned-separator-width` | `Theme::pinned_separator_width`  |
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

    let mut t = fallback;
    t.bg = color("--rs-grid-bg", t.bg);
    t.header_bg = color("--rs-grid-header-bg", t.header_bg);
    t.header_text = color("--rs-grid-header-text", t.header_text);
    t.cell_text = color("--rs-grid-cell-text", t.cell_text);
    t.grid_line = color("--rs-grid-grid-line", t.grid_line);
    t.header_border = color("--rs-grid-header-border", t.header_border);
    t.header_separator_inset =
        px("--rs-grid-header-separator-inset", t.header_separator_inset);
    t.header_separator_width =
        px("--rs-grid-header-separator-width", t.header_separator_width);
    t.selection_fill = color("--rs-grid-selection-fill", t.selection_fill);
    t.selection_border =
        color("--rs-grid-selection-border", t.selection_border);
    t.header_selection_fill =
        color("--rs-grid-header-selection-fill", t.header_selection_fill);
    t.gutter_selection_fill =
        color("--rs-grid-gutter-selection-fill", t.gutter_selection_fill);
    t.scrollbar_track = color("--rs-grid-scrollbar-track", t.scrollbar_track);
    t.scrollbar_thumb = color("--rs-grid-scrollbar-thumb", t.scrollbar_thumb);
    t.row_alt_bg = color("--rs-grid-row-alt-bg", t.row_alt_bg);
    t.row_hover_bg = color("--rs-grid-row-hover-bg", t.row_hover_bg);
    t.scrollbar_width = px("--rs-grid-scrollbar-width", t.scrollbar_width);
    t.scrollbar_radius = px("--rs-grid-scrollbar-radius", t.scrollbar_radius);
    t.scrollbar_inset = px("--rs-grid-scrollbar-inset", t.scrollbar_inset);
    t.font_size = px("--rs-grid-font-size", t.font_size);
    t.header_font_size = px("--rs-grid-header-font-size", t.header_font_size);
    t.header_font_bold =
        bool_var("--rs-grid-header-font-bold", t.header_font_bold);
    t.cell_padding = px("--rs-grid-cell-padding", t.cell_padding);
    t.flash_fill = color("--rs-grid-flash-fill", t.flash_fill);
    t.flash_border = color("--rs-grid-flash-border", t.flash_border);
    t.search_highlight =
        color("--rs-grid-search-highlight", t.search_highlight);
    t.search_current = color("--rs-grid-search-current", t.search_current);
    t.skeleton_fg = color("--rs-grid-skeleton-fg", t.skeleton_fg);
    t.drag_overlay = color("--rs-grid-drag-overlay", t.drag_overlay);
    t.drag_ghost_bg = color("--rs-grid-drag-ghost-bg", t.drag_ghost_bg);
    t.drag_ghost_text = color("--rs-grid-drag-ghost-text", t.drag_ghost_text);
    t.drag_insert_line_width =
        px("--rs-grid-drag-insert-line-width", t.drag_insert_line_width);
    t.drag_ghost_radius =
        px("--rs-grid-drag-ghost-radius", t.drag_ghost_radius);
    t.drag_ghost_border_width = px(
        "--rs-grid-drag-ghost-border-width",
        t.drag_ghost_border_width,
    );
    t.drag_anim_alpha = px("--rs-grid-drag-anim-alpha", t.drag_anim_alpha);
    t.sort_arrow_width = px("--rs-grid-sort-arrow-width", t.sort_arrow_width);
    t.sort_arrow_height =
        px("--rs-grid-sort-arrow-height", t.sort_arrow_height);
    t.header_height = px("--rs-grid-header-height", t.header_height);
    t.row_height = px("--rs-grid-row-height", t.row_height);
    t.header_menu_icon =
        color("--rs-grid-header-menu-icon", t.header_menu_icon);
    t.header_menu_icon_hover_bg = color(
        "--rs-grid-header-menu-icon-hover-bg",
        t.header_menu_icon_hover_bg,
    );
    t.header_menu_icon_radius = px(
        "--rs-grid-header-menu-icon-radius",
        t.header_menu_icon_radius,
    );
    t.header_menu_icon_margin_r = px(
        "--rs-grid-header-menu-icon-margin-r",
        t.header_menu_icon_margin_r,
    );
    t.header_menu_icon_btn_w =
        px("--rs-grid-header-menu-icon-btn-w", t.header_menu_icon_btn_w);
    t.header_menu_icon_btn_h =
        px("--rs-grid-header-menu-icon-btn-h", t.header_menu_icon_btn_h);
    t.header_menu_icon_dot_r =
        px("--rs-grid-header-menu-icon-dot-r", t.header_menu_icon_dot_r);
    t.pinned_bg = color("--rs-grid-pinned-bg", t.pinned_bg);
    t.pinned_header_bg =
        color("--rs-grid-pinned-header-bg", t.pinned_header_bg);
    t.pinned_separator_color =
        color("--rs-grid-pinned-separator-color", t.pinned_separator_color);
    t.pinned_separator_width =
        px("--rs-grid-pinned-separator-width", t.pinned_separator_width);
    t.gutter_bg = color("--rs-grid-gutter-bg", t.gutter_bg);
    t.gutter_text = color("--rs-grid-gutter-text", t.gutter_text);
    t.gutter_font_size = px("--rs-grid-gutter-font-size", t.gutter_font_size);
    t.gutter_font_bold =
        bool_var("--rs-grid-gutter-font-bold", t.gutter_font_bold);
    t.gutter_border = color("--rs-grid-gutter-border", t.gutter_border);
    // cell buttons
    t.cell_btn_primary_bg =
        color("--rs-grid-cell-btn-primary-bg", t.cell_btn_primary_bg);
    t.cell_btn_primary_text =
        color("--rs-grid-cell-btn-primary-text", t.cell_btn_primary_text);
    t.cell_btn_secondary_bg =
        color("--rs-grid-cell-btn-secondary-bg", t.cell_btn_secondary_bg);
    t.cell_btn_secondary_text = color(
        "--rs-grid-cell-btn-secondary-text",
        t.cell_btn_secondary_text,
    );
    t.cell_btn_danger_bg =
        color("--rs-grid-cell-btn-danger-bg", t.cell_btn_danger_bg);
    t.cell_btn_danger_text =
        color("--rs-grid-cell-btn-danger-text", t.cell_btn_danger_text);
    t.cell_btn_ghost_color =
        color("--rs-grid-cell-btn-ghost-color", t.cell_btn_ghost_color);
    t.cell_btn_radius = px("--rs-grid-cell-btn-radius", t.cell_btn_radius);
    t.cell_btn_padding_y =
        px("--rs-grid-cell-btn-padding-y", t.cell_btn_padding_y);
    t.cell_btn_padding_x =
        px("--rs-grid-cell-btn-padding-x", t.cell_btn_padding_x);
    t.cell_btn_gap = px("--rs-grid-cell-btn-gap", t.cell_btn_gap);
    t.cell_btn_margin_r =
        px("--rs-grid-cell-btn-margin-r", t.cell_btn_margin_r);
    t
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
