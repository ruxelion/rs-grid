//! Generate `themes/light.css`, `themes/dark.css`, and `themes/dimmed.css`
//! from `Theme::light()`, `Theme::dark()`, and `Theme::dimmed()`.
//!
//! Run: `cargo run -p rs-grid-scene --bin generate-theme`
//!
//! The generated files are the single source of truth for all CSS
//! custom properties understood by `rs-grid-web::theme_from_css_vars`.
//! Do not edit them by hand — edit this file instead.

use std::path::PathBuf;

use rs_grid_scene::{primitives::Color, Theme};

// ── formatting helpers ────────────────────────────────────────────────────────

/// Color → CSS value: `#rrggbb` when opaque, `rgba(r, g, b, a)` otherwise.
fn c(color: Color) -> String {
    if color.a == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    } else {
        let a = color.a as f64 / 255.0;
        format!(
            "rgba({}, {}, {}, {:.2})",
            color.r, color.g, color.b, a
        )
    }
}

/// `f64` → `Npx` (no decimal for whole numbers, one decimal otherwise).
fn px(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}px", v as i64)
    } else {
        format!("{:.1}px", v)
    }
}

/// `f64` → bare number (no unit; used for ratios / alpha values).
fn num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

fn b(v: bool) -> &'static str {
    if v { "1" } else { "0" }
}

// ── Theme → CSS variable list ─────────────────────────────────────────────────

fn theme_vars(t: &Theme) -> Vec<(&'static str, String)> {
    vec![
        // palette
        ("--rs-grid-bg",                        c(t.bg)),
        ("--rs-grid-header-bg",                 c(t.header_bg)),
        ("--rs-grid-header-text",               c(t.header_text)),
        ("--rs-grid-cell-text",                 c(t.cell_text)),
        ("--rs-grid-grid-line",                 c(t.grid_line)),
        ("--rs-grid-header-border",             c(t.header_border)),
        ("--rs-grid-header-separator-inset",    px(t.header_separator_inset)),
        ("--rs-grid-header-separator-width",    px(t.header_separator_width)),
        ("--rs-grid-selection-fill",            c(t.selection_fill)),
        ("--rs-grid-selection-border",          c(t.selection_border)),
        ("--rs-grid-header-selection-fill",     c(t.header_selection_fill)),
        ("--rs-grid-gutter-selection-fill",     c(t.gutter_selection_fill)),
        ("--rs-grid-scrollbar-track",           c(t.scrollbar_track)),
        ("--rs-grid-scrollbar-thumb",           c(t.scrollbar_thumb)),
        ("--rs-grid-row-alt-bg",                c(t.row_alt_bg)),
        ("--rs-grid-row-hover-bg",              c(t.row_hover_bg)),
        // dimensions
        ("--rs-grid-header-height",             px(t.header_height)),
        ("--rs-grid-row-height",                px(t.row_height)),
        // typography
        ("--rs-grid-font-size",                 px(t.font_size)),
        ("--rs-grid-header-font-size",          px(t.header_font_size)),
        (
            "--rs-grid-header-font-bold",
            b(t.header_font_bold).to_string(),
        ),
        // flash
        ("--rs-grid-flash-fill",                c(t.flash_fill)),
        ("--rs-grid-flash-border",              c(t.flash_border)),
        // search
        ("--rs-grid-search-highlight",          c(t.search_highlight)),
        ("--rs-grid-search-current",            c(t.search_current)),
        // skeleton
        ("--rs-grid-skeleton-fg",               c(t.skeleton_fg)),
        // spacing
        ("--rs-grid-cell-padding",              px(t.cell_padding)),
        // scrollbar
        ("--rs-grid-scrollbar-width",           px(t.scrollbar_width)),
        ("--rs-grid-scrollbar-radius",          px(t.scrollbar_radius)),
        ("--rs-grid-scrollbar-inset",           px(t.scrollbar_inset)),
        // column drag
        ("--rs-grid-drag-overlay",              c(t.drag_overlay)),
        ("--rs-grid-drag-ghost-bg",             c(t.drag_ghost_bg)),
        ("--rs-grid-drag-ghost-text",           c(t.drag_ghost_text)),
        (
            "--rs-grid-drag-insert-line-width",
            px(t.drag_insert_line_width),
        ),
        ("--rs-grid-drag-ghost-radius",         px(t.drag_ghost_radius)),
        (
            "--rs-grid-drag-ghost-border-width",
            px(t.drag_ghost_border_width),
        ),
        ("--rs-grid-drag-anim-alpha",           num(t.drag_anim_alpha)),
        // sort indicator
        ("--rs-grid-sort-arrow-width",          px(t.sort_arrow_width)),
        ("--rs-grid-sort-arrow-height",         px(t.sort_arrow_height)),
        // header menu icon
        ("--rs-grid-header-menu-icon",          c(t.header_menu_icon)),
        (
            "--rs-grid-header-menu-icon-hover-bg",
            c(t.header_menu_icon_hover_bg),
        ),
        (
            "--rs-grid-header-menu-icon-radius",
            px(t.header_menu_icon_radius),
        ),
        (
            "--rs-grid-header-menu-icon-margin-r",
            px(t.header_menu_icon_margin_r),
        ),
        (
            "--rs-grid-header-menu-icon-btn-w",
            px(t.header_menu_icon_btn_w),
        ),
        (
            "--rs-grid-header-menu-icon-btn-h",
            px(t.header_menu_icon_btn_h),
        ),
        (
            "--rs-grid-header-menu-icon-dot-r",
            px(t.header_menu_icon_dot_r),
        ),
        // pinned columns
        ("--rs-grid-pinned-bg",                 c(t.pinned_bg)),
        ("--rs-grid-pinned-header-bg",          c(t.pinned_header_bg)),
        (
            "--rs-grid-pinned-separator-color",
            c(t.pinned_separator_color),
        ),
        (
            "--rs-grid-pinned-separator-width",
            px(t.pinned_separator_width),
        ),
        // row-number gutter
        ("--rs-grid-gutter-bg",                 c(t.gutter_bg)),
        ("--rs-grid-gutter-text",               c(t.gutter_text)),
        ("--rs-grid-gutter-font-size",          px(t.gutter_font_size)),
        (
            "--rs-grid-gutter-font-bold",
            b(t.gutter_font_bold).to_string(),
        ),
        ("--rs-grid-gutter-border",             c(t.gutter_border)),
        // cell buttons
        (
            "--rs-grid-cell-btn-primary-bg",
            c(t.cell_btn_primary_bg),
        ),
        (
            "--rs-grid-cell-btn-primary-text",
            c(t.cell_btn_primary_text),
        ),
        (
            "--rs-grid-cell-btn-secondary-bg",
            c(t.cell_btn_secondary_bg),
        ),
        (
            "--rs-grid-cell-btn-secondary-text",
            c(t.cell_btn_secondary_text),
        ),
        (
            "--rs-grid-cell-btn-danger-bg",
            c(t.cell_btn_danger_bg),
        ),
        (
            "--rs-grid-cell-btn-danger-text",
            c(t.cell_btn_danger_text),
        ),
        (
            "--rs-grid-cell-btn-ghost-color",
            c(t.cell_btn_ghost_color),
        ),
        ("--rs-grid-cell-btn-radius",           px(t.cell_btn_radius)),
        (
            "--rs-grid-cell-btn-padding-y",
            px(t.cell_btn_padding_y),
        ),
        (
            "--rs-grid-cell-btn-padding-x",
            px(t.cell_btn_padding_x),
        ),
        ("--rs-grid-cell-btn-gap",              px(t.cell_btn_gap)),
        (
            "--rs-grid-cell-btn-margin-r",
            px(t.cell_btn_margin_r),
        ),
    ]
}

// ── context menu vars (CSS-only, not in Theme) ────────────────────────────────

const CTX_LIGHT: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg",            "#ffffff"),
    ("--rs-grid-ctx-border",        "#dde2eb"),
    (
        "--rs-grid-ctx-shadow",
        "0 4px 16px rgba(0, 0, 0, 0.12)",
    ),
    ("--rs-grid-ctx-text",          "#181d1f"),
    ("--rs-grid-ctx-text-disabled", "#9ca3af"),
    ("--rs-grid-ctx-hover-bg",      "#f8f9fb"),
    ("--rs-grid-ctx-separator",     "#e2e8f0"),
];

const CTX_DARK: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg",            "#252527"),
    ("--rs-grid-ctx-border",        "#3a3a3c"),
    (
        "--rs-grid-ctx-shadow",
        "0 4px 16px rgba(0, 0, 0, 0.5)",
    ),
    ("--rs-grid-ctx-text",          "#d0d0d0"),
    ("--rs-grid-ctx-text-disabled", "#666668"),
    ("--rs-grid-ctx-hover-bg",      "#2c2c2e"),
    ("--rs-grid-ctx-separator",     "#333335"),
];

const CTX_DIMMED: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg",            "#2d333b"),
    ("--rs-grid-ctx-border",        "#444c56"),
    (
        "--rs-grid-ctx-shadow",
        "0 4px 16px rgba(0, 0, 0, 0.4)",
    ),
    ("--rs-grid-ctx-text",          "#adbac7"),
    ("--rs-grid-ctx-text-disabled", "#636e7b"),
    ("--rs-grid-ctx-hover-bg",      "#373e47"),
    ("--rs-grid-ctx-separator",     "#373e47"),
];

// ── CSS rendering ─────────────────────────────────────────────────────────────

const HEADER: &str = concat!(
    "/* AUTO-GENERATED — do not edit.\n",
    " * Source of truth: crates/rs-grid-scene/src/theme.rs\n",
    " * Regenerate:",
    " cargo run -p rs-grid-scene --bin generate-theme\n",
    " */\n",
);

fn render_light(vars: &[(&str, String)]) -> String {
    let mut s = String::from(HEADER);
    s.push_str("\n:root {\n");
    for (name, val) in vars {
        s.push_str(&format!("  {}: {};\n", name, val));
    }
    s.push_str("\n  /* context menu */\n");
    for (name, val) in CTX_LIGHT {
        s.push_str(&format!("  {}: {};\n", name, val));
    }
    s.push_str("}\n");
    s
}

/// Emit only the vars that differ from light, under `:root.<selector>`.
fn render_overlay(
    selector: &str,
    light_vars: &[(&str, String)],
    theme_vars: &[(&str, String)],
    ctx: &[(&str, &str)],
) -> String {
    let mut s = String::from(HEADER);
    s.push_str(&format!("\n:root.{} {{\n", selector));
    for ((ln, lv), (tn, tv)) in light_vars.iter().zip(theme_vars.iter()) {
        assert_eq!(ln, tn, "var order mismatch in theme_vars");
        if lv != tv {
            s.push_str(&format!("  {}: {};\n", tn, tv));
        }
    }
    s.push_str("\n  /* context menu */\n");
    for (name, val) in ctx {
        s.push_str(&format!("  {}: {};\n", name, val));
    }
    s.push_str("}\n");
    s
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    // CARGO_MANIFEST_DIR = crates/rs-grid-scene
    // CSS files live in examples/example-common/themes/
    let themes = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/example-common/themes");

    let light = Theme::light();
    let dark = Theme::dark();
    let dimmed = Theme::dimmed();
    let light_vars = theme_vars(&light);
    let dark_vars = theme_vars(&dark);
    let dimmed_vars = theme_vars(&dimmed);

    let light_css = render_light(&light_vars);
    let dark_css =
        render_overlay("dark", &light_vars, &dark_vars, CTX_DARK);
    let dimmed_css =
        render_overlay("dimmed", &light_vars, &dimmed_vars, CTX_DIMMED);

    let light_path = themes.join("light.css");
    let dark_path = themes.join("dark.css");
    let dimmed_path = themes.join("dimmed.css");

    std::fs::write(&light_path, &light_css).expect("write light.css");
    std::fs::write(&dark_path, &dark_css).expect("write dark.css");
    std::fs::write(&dimmed_path, &dimmed_css).expect("write dimmed.css");

    println!("Generated:");
    println!("  {}", light_path.display());
    println!("  {}", dark_path.display());
    println!("  {}", dimmed_path.display());
}
