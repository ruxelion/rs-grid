//! Generate `themes/light.css`, `themes/dark.css`, and `themes/dimmed.css`
//! from `Theme::light()`, `Theme::dark()`, and `Theme::dimmed()`.
//!
//! Run: `cargo run -p rs-grid-scene --bin generate-theme`
//!
//! The generated files are the single source of truth for all CSS
//! custom properties understood by `rs-grid-web::theme_from_css_vars`.
//! Do not edit them by hand — edit this file instead.

use std::path::PathBuf;

use rs_grid_scene::{Theme, css_vars::theme_to_css_vars};

// ── context menu vars (CSS-only, not in Theme)
// ────────────────────────────────

const CTX_LIGHT: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg", "#ffffff"),
    ("--rs-grid-ctx-border", "#dde2eb"),
    ("--rs-grid-ctx-shadow", "0 4px 16px rgba(0, 0, 0, 0.12)"),
    ("--rs-grid-ctx-text", "#181d1f"),
    ("--rs-grid-ctx-text-disabled", "#9ca3af"),
    ("--rs-grid-ctx-hover-bg", "#f8f9fb"),
    ("--rs-grid-ctx-separator", "#e2e8f0"),
];

const CTX_DARK: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg", "#252527"),
    ("--rs-grid-ctx-border", "#3a3a3c"),
    ("--rs-grid-ctx-shadow", "0 4px 16px rgba(0, 0, 0, 0.5)"),
    ("--rs-grid-ctx-text", "#d0d0d0"),
    ("--rs-grid-ctx-text-disabled", "#666668"),
    ("--rs-grid-ctx-hover-bg", "#2c2c2e"),
    ("--rs-grid-ctx-separator", "#333335"),
];

const CTX_DIMMED: &[(&str, &str)] = &[
    ("--rs-grid-ctx-bg", "#2d333b"),
    ("--rs-grid-ctx-border", "#444c56"),
    ("--rs-grid-ctx-shadow", "0 4px 16px rgba(0, 0, 0, 0.4)"),
    ("--rs-grid-ctx-text", "#adbac7"),
    ("--rs-grid-ctx-text-disabled", "#636e7b"),
    ("--rs-grid-ctx-hover-bg", "#373e47"),
    ("--rs-grid-ctx-separator", "#373e47"),
];

// ── CSS rendering
// ─────────────────────────────────────────────────────────────

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
    let light_vars = theme_to_css_vars(&light);
    let dark_vars = theme_to_css_vars(&dark);
    let dimmed_vars = theme_to_css_vars(&dimmed);

    let light_css = render_light(&light_vars);
    let dark_css = render_overlay("dark", &light_vars, &dark_vars, CTX_DARK);
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
