//! CSS custom-property (de)serialization for [`Theme`].
//!
//! This module is the **single source of truth** for the `--rs-grid-*`
//! variables:
//!
//! - [`theme_to_css_vars`] (writer) — used by the `generate-theme` binary to
//!   emit `themes/{light,dark,dimmed}.css`.
//! - [`theme_from_css_vars_with`] (reader) — wrapped by
//!   `rs-grid-web::theme_from_css_vars`, which supplies a DOM-backed getter.
//!
//! The two are exact inverses. The `round_trips` test enforces that every
//! [`Theme`] field is wired into **both** directions — the executable form of
//! the AGENTS.md rule "any themeable value must be exposed as a CSS variable".
//! Forget either side (or add a `Theme` field with no variable) and the test
//! fails; the prose rule alone could not.

use crate::{Theme, primitives::Color};

// ── writer: Theme → CSS variables ────────────────────────────────────────────

/// Color → CSS value: `#rrggbb` when opaque, `rgba(r, g, b, a)` otherwise.
fn fmt_color(color: Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        let a = color.a as f64 / 255.0;
        format!("rgba({}, {}, {}, {:.2})", color.r, color.g, color.b, a)
    }
}

/// `f64` → `Npx` (no decimal for whole numbers, one decimal otherwise).
fn fmt_px(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}px", v as i64)
    } else {
        format!("{:.1}px", v)
    }
}

/// `f64` → bare number (no unit; used for ratios / alpha values).
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

/// `bool` → `"1"` / `"0"`.
fn fmt_bool(v: bool) -> &'static str {
    if v { "1" } else { "0" }
}

/// All `--rs-grid-*` custom properties for a theme, in declaration order.
///
/// The order defines the order of lines in the generated CSS files; keep it
/// stable to avoid churn in `themes/*.css`.
pub fn theme_to_css_vars(t: &Theme) -> Vec<(&'static str, String)> {
    vec![
        // palette
        ("--rs-grid-bg", fmt_color(t.bg)),
        ("--rs-grid-header-bg", fmt_color(t.header_bg)),
        ("--rs-grid-header-text", fmt_color(t.header_text)),
        ("--rs-grid-cell-text", fmt_color(t.cell_text)),
        ("--rs-grid-grid-line", fmt_color(t.grid_line)),
        ("--rs-grid-header-border", fmt_color(t.header_border)),
        (
            "--rs-grid-header-separator-inset",
            fmt_px(t.header_separator_inset),
        ),
        (
            "--rs-grid-header-separator-width",
            fmt_px(t.header_separator_width),
        ),
        ("--rs-grid-selection-fill", fmt_color(t.selection_fill)),
        ("--rs-grid-selection-border", fmt_color(t.selection_border)),
        (
            "--rs-grid-header-selection-fill",
            fmt_color(t.header_selection_fill),
        ),
        (
            "--rs-grid-gutter-selection-fill",
            fmt_color(t.gutter_selection_fill),
        ),
        ("--rs-grid-scrollbar-track", fmt_color(t.scrollbar_track)),
        ("--rs-grid-scrollbar-thumb", fmt_color(t.scrollbar_thumb)),
        ("--rs-grid-row-alt-bg", fmt_color(t.row_alt_bg)),
        ("--rs-grid-row-hover-bg", fmt_color(t.row_hover_bg)),
        ("--rs-grid-locked-cell-bg", fmt_color(t.locked_cell_bg)),
        ("--rs-grid-locked-cell-text", fmt_color(t.locked_cell_text)),
        (
            "--rs-grid-invalid-cell-border",
            fmt_color(t.invalid_cell_border),
        ),
        (
            "--rs-grid-invalid-cell-border-width",
            fmt_px(t.invalid_cell_border_width),
        ),
        (
            "--rs-grid-decoration-border-width",
            fmt_px(t.decoration_border_width),
        ),
        // dimensions
        ("--rs-grid-header-height", fmt_px(t.header_height)),
        ("--rs-grid-row-height", fmt_px(t.row_height)),
        // typography
        ("--rs-grid-font-size", fmt_px(t.font_size)),
        ("--rs-grid-header-font-size", fmt_px(t.header_font_size)),
        (
            "--rs-grid-header-font-bold",
            fmt_bool(t.header_font_bold).to_string(),
        ),
        (
            "--rs-grid-header-font-italic",
            fmt_bool(t.header_font_italic).to_string(),
        ),
        // flash
        ("--rs-grid-flash-fill", fmt_color(t.flash_fill)),
        ("--rs-grid-flash-border", fmt_color(t.flash_border)),
        // search
        ("--rs-grid-search-highlight", fmt_color(t.search_highlight)),
        ("--rs-grid-search-current", fmt_color(t.search_current)),
        // skeleton
        ("--rs-grid-skeleton-fg", fmt_color(t.skeleton_fg)),
        // progress bar
        ("--rs-grid-progress-track", fmt_color(t.progress_track)),
        ("--rs-grid-progress-fill", fmt_color(t.progress_fill)),
        ("--rs-grid-progress-height", fmt_px(t.progress_height)),
        ("--rs-grid-progress-radius", fmt_px(t.progress_radius)),
        // spacing
        ("--rs-grid-cell-padding", fmt_px(t.cell_padding)),
        // scrollbar
        ("--rs-grid-scrollbar-width", fmt_px(t.scrollbar_width)),
        ("--rs-grid-scrollbar-radius", fmt_px(t.scrollbar_radius)),
        ("--rs-grid-scrollbar-inset", fmt_px(t.scrollbar_inset)),
        // column drag
        ("--rs-grid-drag-overlay", fmt_color(t.drag_overlay)),
        ("--rs-grid-drag-ghost-bg", fmt_color(t.drag_ghost_bg)),
        ("--rs-grid-drag-ghost-text", fmt_color(t.drag_ghost_text)),
        (
            "--rs-grid-drag-insert-line-width",
            fmt_px(t.drag_insert_line_width),
        ),
        ("--rs-grid-drag-ghost-radius", fmt_px(t.drag_ghost_radius)),
        (
            "--rs-grid-drag-ghost-border-width",
            fmt_px(t.drag_ghost_border_width),
        ),
        ("--rs-grid-drag-anim-alpha", fmt_num(t.drag_anim_alpha)),
        // sort indicator
        ("--rs-grid-sort-arrow-width", fmt_px(t.sort_arrow_width)),
        ("--rs-grid-sort-arrow-height", fmt_px(t.sort_arrow_height)),
        // header menu icon
        ("--rs-grid-header-menu-icon", fmt_color(t.header_menu_icon)),
        (
            "--rs-grid-header-menu-icon-hover-bg",
            fmt_color(t.header_menu_icon_hover_bg),
        ),
        (
            "--rs-grid-header-menu-icon-radius",
            fmt_px(t.header_menu_icon_radius),
        ),
        (
            "--rs-grid-header-menu-icon-margin-r",
            fmt_px(t.header_menu_icon_margin_r),
        ),
        (
            "--rs-grid-header-menu-icon-btn-w",
            fmt_px(t.header_menu_icon_btn_w),
        ),
        (
            "--rs-grid-header-menu-icon-btn-h",
            fmt_px(t.header_menu_icon_btn_h),
        ),
        (
            "--rs-grid-header-menu-icon-dot-r",
            fmt_px(t.header_menu_icon_dot_r),
        ),
        // pinned columns
        ("--rs-grid-pinned-bg", fmt_color(t.pinned_bg)),
        ("--rs-grid-pinned-header-bg", fmt_color(t.pinned_header_bg)),
        (
            "--rs-grid-pinned-separator-color",
            fmt_color(t.pinned_separator_color),
        ),
        (
            "--rs-grid-pinned-separator-width",
            fmt_px(t.pinned_separator_width),
        ),
        // row-number gutter
        ("--rs-grid-gutter-bg", fmt_color(t.gutter_bg)),
        ("--rs-grid-gutter-text", fmt_color(t.gutter_text)),
        ("--rs-grid-gutter-font-size", fmt_px(t.gutter_font_size)),
        (
            "--rs-grid-gutter-font-bold",
            fmt_bool(t.gutter_font_bold).to_string(),
        ),
        (
            "--rs-grid-gutter-font-italic",
            fmt_bool(t.gutter_font_italic).to_string(),
        ),
        ("--rs-grid-gutter-border", fmt_color(t.gutter_border)),
        // cell buttons
        (
            "--rs-grid-cell-btn-primary-bg",
            fmt_color(t.cell_btn_primary_bg),
        ),
        (
            "--rs-grid-cell-btn-primary-text",
            fmt_color(t.cell_btn_primary_text),
        ),
        (
            "--rs-grid-cell-btn-secondary-bg",
            fmt_color(t.cell_btn_secondary_bg),
        ),
        (
            "--rs-grid-cell-btn-secondary-text",
            fmt_color(t.cell_btn_secondary_text),
        ),
        (
            "--rs-grid-cell-btn-danger-bg",
            fmt_color(t.cell_btn_danger_bg),
        ),
        (
            "--rs-grid-cell-btn-danger-text",
            fmt_color(t.cell_btn_danger_text),
        ),
        (
            "--rs-grid-cell-btn-ghost-color",
            fmt_color(t.cell_btn_ghost_color),
        ),
        ("--rs-grid-cell-btn-radius", fmt_px(t.cell_btn_radius)),
        ("--rs-grid-cell-btn-padding-y", fmt_px(t.cell_btn_padding_y)),
        ("--rs-grid-cell-btn-padding-x", fmt_px(t.cell_btn_padding_x)),
        ("--rs-grid-cell-btn-gap", fmt_px(t.cell_btn_gap)),
        ("--rs-grid-cell-btn-margin-r", fmt_px(t.cell_btn_margin_r)),
    ]
}

// ── reader: CSS variables → Theme ────────────────────────────────────────────

/// Build a [`Theme`] from a CSS-variable getter.
///
/// `get` returns the raw string value of a `--rs-grid-*` property, or `None`
/// when the variable is absent or empty. Absent / unparseable values fall back
/// to the corresponding [`Theme::light`] value. Exact inverse of
/// [`theme_to_css_vars`].
pub fn theme_from_css_vars_with(get: impl Fn(&str) -> Option<String>) -> Theme {
    let raw = |name: &str| get(name).unwrap_or_default();
    let color = |name: &str, fb: Color| parse_color(&raw(name)).unwrap_or(fb);
    let px = |name: &str, fb: f64| parse_px(&raw(name)).unwrap_or(fb);
    let boolean = |name: &str, fb: bool| match raw(name).trim() {
        "0" | "false" => false,
        "1" | "true" => true,
        _ => fb,
    };

    let mut t = Theme::light();
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
    t.locked_cell_bg = color("--rs-grid-locked-cell-bg", t.locked_cell_bg);
    t.locked_cell_text =
        color("--rs-grid-locked-cell-text", t.locked_cell_text);
    t.invalid_cell_border =
        color("--rs-grid-invalid-cell-border", t.invalid_cell_border);
    t.invalid_cell_border_width = px(
        "--rs-grid-invalid-cell-border-width",
        t.invalid_cell_border_width,
    );
    t.decoration_border_width = px(
        "--rs-grid-decoration-border-width",
        t.decoration_border_width,
    );
    t.header_height = px("--rs-grid-header-height", t.header_height);
    t.row_height = px("--rs-grid-row-height", t.row_height);
    t.font_size = px("--rs-grid-font-size", t.font_size);
    t.header_font_size = px("--rs-grid-header-font-size", t.header_font_size);
    t.header_font_bold =
        boolean("--rs-grid-header-font-bold", t.header_font_bold);
    t.header_font_italic =
        boolean("--rs-grid-header-font-italic", t.header_font_italic);
    t.flash_fill = color("--rs-grid-flash-fill", t.flash_fill);
    t.flash_border = color("--rs-grid-flash-border", t.flash_border);
    t.search_highlight =
        color("--rs-grid-search-highlight", t.search_highlight);
    t.search_current = color("--rs-grid-search-current", t.search_current);
    t.skeleton_fg = color("--rs-grid-skeleton-fg", t.skeleton_fg);
    t.progress_track = color("--rs-grid-progress-track", t.progress_track);
    t.progress_fill = color("--rs-grid-progress-fill", t.progress_fill);
    t.progress_height = px("--rs-grid-progress-height", t.progress_height);
    t.progress_radius = px("--rs-grid-progress-radius", t.progress_radius);
    t.cell_padding = px("--rs-grid-cell-padding", t.cell_padding);
    t.scrollbar_width = px("--rs-grid-scrollbar-width", t.scrollbar_width);
    t.scrollbar_radius = px("--rs-grid-scrollbar-radius", t.scrollbar_radius);
    t.scrollbar_inset = px("--rs-grid-scrollbar-inset", t.scrollbar_inset);
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
        boolean("--rs-grid-gutter-font-bold", t.gutter_font_bold);
    t.gutter_font_italic =
        boolean("--rs-grid-gutter-font-italic", t.gutter_font_italic);
    t.gutter_border = color("--rs-grid-gutter-border", t.gutter_border);
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

// ── parsers ──────────────────────────────────────────────────────────────────

/// Parse a CSS color string into a `Color`.
///
/// Supported formats:
/// - `#rrggbb` / `#rrggbbaa`
/// - `#rgb` / `#rgba`
/// - `rgb(r, g, b)`
/// - `rgba(r, g, b, a)` — `a` is a 0–1 float
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

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::{BTreeSet, HashMap},
    };

    use super::*;

    /// A `Theme` whose every field carries a distinct, round-trip-exact
    /// sentinel value. The struct literal (no `..`) makes the compiler reject
    /// this the moment a `Theme` field is added without giving it a sentinel —
    /// which forces the new field through the round-trip check below.
    ///
    /// `n` is the field's ordinal; colors are opaque (exact `#rrggbb`) and
    /// lengths are integers (exact `Npx`), so formatting never loses data.
    fn sentinel_theme() -> Theme {
        // Distinct opaque color per ordinal (r = n guarantees uniqueness).
        let c = |n: u8| Color::rgb(n, 255 - n, 128);
        Theme {
            bg: c(1),
            header_bg: c(2),
            header_text: c(3),
            cell_text: c(4),
            grid_line: c(5),
            header_border: c(6),
            header_separator_inset: 7.0,
            header_separator_width: 8.0,
            selection_fill: c(9),
            selection_border: c(10),
            header_selection_fill: c(11),
            gutter_selection_fill: c(12),
            scrollbar_track: c(13),
            scrollbar_thumb: c(14),
            row_alt_bg: c(15),
            row_hover_bg: c(16),
            header_height: 17.0,
            row_height: 18.0,
            font_size: 19.0,
            header_font_size: 20.0,
            header_font_bold: true,
            header_font_italic: false,
            flash_fill: c(23),
            flash_border: c(24),
            search_highlight: c(25),
            search_current: c(26),
            skeleton_fg: c(27),
            progress_track: c(28),
            progress_fill: c(29),
            progress_height: 30.0,
            progress_radius: 31.0,
            cell_padding: 32.0,
            scrollbar_width: 33.0,
            scrollbar_radius: 34.0,
            scrollbar_inset: 35.0,
            drag_overlay: c(36),
            drag_ghost_bg: c(37),
            drag_ghost_text: c(38),
            drag_insert_line_width: 39.0,
            drag_ghost_radius: 40.0,
            drag_ghost_border_width: 41.0,
            drag_anim_alpha: 42.0,
            sort_arrow_width: 43.0,
            sort_arrow_height: 44.0,
            header_menu_icon: c(45),
            header_menu_icon_hover_bg: c(46),
            header_menu_icon_radius: 47.0,
            header_menu_icon_margin_r: 48.0,
            header_menu_icon_btn_w: 49.0,
            header_menu_icon_btn_h: 50.0,
            header_menu_icon_dot_r: 51.0,
            pinned_bg: c(52),
            pinned_header_bg: c(53),
            pinned_separator_color: c(54),
            pinned_separator_width: 55.0,
            gutter_bg: c(56),
            gutter_text: c(57),
            gutter_font_size: 58.0,
            gutter_font_bold: false,
            gutter_font_italic: true,
            gutter_border: c(61),
            cell_btn_primary_bg: c(62),
            cell_btn_primary_text: c(63),
            cell_btn_secondary_bg: c(64),
            cell_btn_secondary_text: c(65),
            cell_btn_danger_bg: c(66),
            cell_btn_danger_text: c(67),
            cell_btn_ghost_color: c(68),
            cell_btn_radius: 69.0,
            cell_btn_padding_y: 70.0,
            cell_btn_padding_x: 71.0,
            cell_btn_gap: 72.0,
            cell_btn_margin_r: 73.0,
            locked_cell_bg: c(74),
            locked_cell_text: c(75),
            invalid_cell_border: c(76),
            invalid_cell_border_width: 77.0,
            decoration_border_width: 78.0,
        }
    }

    /// The core invariant: every `Theme` field is wired into both the writer
    /// and the reader. A field missing from either side (or with no variable
    /// at all) keeps its `Theme::light()` fallback after the round-trip, which
    /// differs from the sentinel → mismatch.
    #[test]
    fn round_trips_every_field() {
        let original = sentinel_theme();
        let map: HashMap<&str, String> =
            theme_to_css_vars(&original).into_iter().collect();
        let restored = theme_from_css_vars_with(|name| map.get(name).cloned());
        assert_eq!(
            restored, original,
            "a Theme field is not wired into both theme_to_css_vars and \
             theme_from_css_vars_with (or has no CSS variable)",
        );
    }

    /// The writer and reader must cover exactly the same set of variables.
    /// Clearer diagnostic than the struct round-trip when a single variable is
    /// forgotten on one side.
    fn writer_var_names() -> BTreeSet<String> {
        theme_to_css_vars(&Theme::light())
            .iter()
            .map(|(n, _)| n.to_string())
            .collect()
    }

    #[test]
    fn writer_and_reader_cover_the_same_variables() {
        let requested = RefCell::new(BTreeSet::new());
        // Returning None still drives the reader through every field.
        let _ = theme_from_css_vars_with(|name| {
            requested.borrow_mut().insert(name.to_string());
            None
        });
        assert_eq!(
            writer_var_names(),
            requested.into_inner(),
            "theme_to_css_vars and theme_from_css_vars_with read/write \
             different CSS variable sets",
        );
    }

    /// Real built-in values (including semi-transparent colors via the `rgba`
    /// path) must be *CSS-stable*: re-serializing the round-tripped theme
    /// yields byte-identical variables. (Direct struct equality is too strict —
    /// 2-decimal alpha is inherently ±1 lossy at the u8 level, but it
    /// re-formats to the same string, which is what actually ships.)
    #[test]
    fn builtin_themes_are_css_stable() {
        for theme in [Theme::light(), Theme::dark(), Theme::dimmed()] {
            let map: HashMap<&str, String> =
                theme_to_css_vars(&theme).into_iter().collect();
            let restored =
                theme_from_css_vars_with(|name| map.get(name).cloned());
            assert_eq!(
                theme_to_css_vars(&restored),
                theme_to_css_vars(&theme),
                "round-trip changed the generated CSS variables",
            );
        }
    }

    /// Absent variables fall back to `Theme::light()` (graceful default).
    #[test]
    fn absent_vars_fall_back_to_light() {
        let restored = theme_from_css_vars_with(|_| None);
        assert_eq!(restored, Theme::light());
    }
}
