//! Browser integration for rs-grid: DOM events, rAF loop,
//! CSS theme, and clipboard handling.

mod canvas;
mod css_theme;

pub use canvas::context_menu_config::{
    BuiltinAction, ContextMenuConfig, ContextMenuItem,
};
pub use canvas::fetcher::{FetchConfig, PageFetchRequest, PageFetchResponse};
pub use canvas::GridCanvas;
pub use css_theme::theme_from_css_vars;
