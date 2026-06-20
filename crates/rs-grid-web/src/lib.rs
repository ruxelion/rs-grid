//! Browser integration for rs-grid: DOM events, rAF loop,
//! CSS theme, localisation, and clipboard handling.
//!
//! Sits near the top of the dependency chain (`core → scene
//! → render-canvas → **web** → leptos`).
//!
//! Key types: [`GridCanvas`], [`Locale`],
//! [`ContextMenuConfig`], [`theme_from_css_vars`].

mod canvas;
mod css_theme;
mod locale;
pub mod storage;

pub use canvas::{
    GridCanvas,
    context_menu_config::{BuiltinAction, ContextMenuConfig, ContextMenuItem},
    fetcher::{FetchConfig, PageFetchRequest, PageFetchResponse},
};
pub use css_theme::theme_from_css_vars;
pub use locale::Locale;
