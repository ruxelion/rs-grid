use rs_grid_scene::{primitives::Color, Theme};

/// Build a `Theme` by reading `--rs-grid-*` CSS custom properties from the
/// document root element (`:root`).
///
/// Each variable falls back to the corresponding `Theme::light()` value when
/// the variable is absent or cannot be parsed.
///
/// ## Supported variables
///
/// | Variable                        | Maps to                    |
/// |--------------------------------|----------------------------|
/// | `--rs-grid-bg`                 | `Theme::bg`                |
/// | `--rs-grid-header-bg`          | `Theme::header_bg`         |
/// | `--rs-grid-header-text`        | `Theme::header_text`       |
/// | `--rs-grid-cell-text`          | `Theme::cell_text`         |
/// | `--rs-grid-grid-line`          | `Theme::grid_line`         |
/// | `--rs-grid-header-border`      | `Theme::header_border`     |
/// | `--rs-grid-selection-fill`     | `Theme::selection_fill`    |
/// | `--rs-grid-selection-border`   | `Theme::selection_border`  |
/// | `--rs-grid-scrollbar-track`    | `Theme::scrollbar_track`   |
/// | `--rs-grid-scrollbar-thumb`    | `Theme::scrollbar_thumb`   |
/// | `--rs-grid-scrollbar-width`    | `Theme::scrollbar_width`   |
/// | `--rs-grid-scrollbar-radius`   | `Theme::scrollbar_radius`  |
/// | `--rs-grid-row-alt-bg`         | `Theme::row_alt_bg`        |
/// | `--rs-grid-font-size`          | `Theme::font_size`         |
/// | `--rs-grid-header-font-size`   | `Theme::header_font_size`  |
/// | `--rs-grid-header-font-bold`   | `Theme::header_font_bold`  |
/// | `--rs-grid-cell-padding`       | `Theme::cell_padding`      |
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
        header_border: color("--rs-grid-header-border", fallback.header_border),
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
    }
}

// ── DOM helpers ───────────────────────────────────────────────────────────────

fn root_computed_style() -> Option<web_sys::CssStyleDeclaration> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let root = document.document_element()?;
    window.get_computed_style(&root).ok().flatten()
}

fn get_var(style: &web_sys::CssStyleDeclaration, name: &str) -> String {
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
