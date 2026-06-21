use rs_grid_scene::{Theme, css_vars::theme_from_css_vars_with};

/// Build a `Theme` by reading `--rs-grid-*` CSS custom properties from the
/// document root element (`:root`).
///
/// Each variable falls back to the corresponding `Theme::light()` value when
/// the variable is absent or cannot be parsed. This is a thin DOM wrapper: the
/// full field ↔ variable mapping (and its parsing) lives in
/// `rs_grid_scene::css_vars`, the single source of truth shared with the
/// `generate-theme` writer and enforced by a round-trip test there.
pub fn theme_from_css_vars() -> Theme {
    let Some(style) = root_computed_style() else {
        return Theme::light();
    };
    theme_from_css_vars_with(|name| {
        let v = get_var(&style, name);
        (!v.is_empty()).then_some(v)
    })
}

// ── DOM helpers
// ───────────────────────────────────────────────────────────────

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
