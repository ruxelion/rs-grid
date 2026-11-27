//! Footer bar interaction: page navigation clicks and the rows-per-page
//! menu.
//!
//! The footer itself is drawn on the canvas by `rs-grid-scene`
//! (`builder/footer.rs`), which also registers a `FooterZone` per enabled
//! control. This module only routes clicks on those zones — it never
//! recomputes the footer's layout.
//!
//! The rows-per-page menu reuses the context menu's DOM shell (same
//! backdrop, same `rs-grid-ctx-*` ids and theme tokens), so it matches
//! the other popups and closes the same way (outside click, `Escape`).
//! Closures use `Closure::forget()` for the same reason as there: removing
//! the shell root reclaims every attached listener.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use rs_grid_core::{commands::GridCommand, pagination::group_thousands};
use rs_grid_scene::{
    builder::TextMeasurer,
    frame::{FooterAction, FooterZone},
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::MouseEvent;

use super::{
    GridCanvas,
    context_menu::{
        create_menu_shell, make_menu_item, read_ctx_colors, remove_ctx_menu,
    },
    dom_helpers::document,
};

/// Feather "check" glyph marking the current page size.
const ICON_CHECK: &str = concat!(
    r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" "#,
    r#"stroke="currentColor" stroke-width="2.5" "#,
    r#"stroke-linecap="round" stroke-linejoin="round">"#,
    r#"<polyline points="20 6 9 17 4 12"/></svg>"#
);

/// A [`TextMeasurer`] backed by an offscreen Canvas2D context, using
/// the same font string as `rs-grid-render-canvas`'s `draw_text` so the
/// widths match what is drawn. Results are memoized: the footer measures
/// the same handful of strings on every frame.
///
/// `None` when no 2D context can be created (the scene then falls back
/// to its per-character estimate).
pub(super) fn text_measurer() -> Option<Rc<TextMeasurer>> {
    /// Past this many distinct strings the memo is simply reset.
    const MAX_ENTRIES: usize = 512;

    let ctx = document()
        .create_element("canvas")
        .ok()?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()?
        .get_context("2d")
        .ok()??
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .ok()?;
    let memo: RefCell<HashMap<(String, u32, bool), f64>> =
        RefCell::new(HashMap::new());

    Some(Rc::new(move |text: &str, font_size: f64, bold: bool| {
        let size = font_size.round() as u32;
        let key = (text.to_owned(), size, bold);
        if let Some(w) = memo.borrow().get(&key) {
            return *w;
        }
        let weight = if bold { "600" } else { "400" };
        ctx.set_font(&format!(
            "normal {weight} {size}px system-ui, sans-serif"
        ));
        let w = ctx.measure_text(text).map(|m| m.width()).unwrap_or(0.0);
        let mut memo = memo.borrow_mut();
        if memo.len() >= MAX_ENTRIES {
            memo.clear();
        }
        memo.insert(key, w);
        w
    }))
}

impl GridCanvas {
    /// Record which footer control is under the pointer (drawn with the
    /// hover fill) and redraw when it changed.
    pub(super) fn set_hovered_footer(&self, hovered: Option<FooterAction>) {
        if self.0.builder.borrow().hovered_footer == hovered {
            return;
        }
        self.0.builder.borrow_mut().hovered_footer = hovered;
        self.render();
    }

    /// `true` when viewport-y `vy` falls in the footer band — below the
    /// scrollable body — and the footer is shown.
    pub(super) fn in_footer_band(&self, vy: f64) -> bool {
        let s = self.0.state.borrow();
        s.model.effective_footer_height() > 0.0 && vy >= s.viewport.height
    }

    /// The enabled footer control under `(vx, vy)`, from the last frame.
    pub(super) fn hit_footer(&self, vx: f64, vy: f64) -> Option<FooterZone> {
        self.0
            .last_frame
            .borrow()
            .as_ref()
            .and_then(|f| f.hit_footer(vx, vy).copied())
    }

    /// Run the action of a clicked footer control.
    pub(super) fn handle_footer_click(&self, zone: FooterZone) {
        let (current, pages) = {
            let s = self.0.state.borrow();
            (s.model.current_page(), s.model.page_count())
        };
        match zone.action {
            FooterAction::FirstPage => self.dispatch(GridCommand::SetPage(0)),
            FooterAction::PrevPage => {
                self.dispatch(GridCommand::SetPage(current.saturating_sub(1)));
            }
            FooterAction::NextPage => {
                self.dispatch(GridCommand::SetPage(current + 1));
            }
            FooterAction::LastPage => {
                self.dispatch(GridCommand::SetPage(pages.saturating_sub(1)));
            }
            FooterAction::GoToPage(p) => {
                self.dispatch(GridCommand::SetPage(p));
            }
            FooterAction::PageSizeMenu => self.show_page_size_menu(zone),
        }
    }

    /// Open the rows-per-page menu above the selector `zone`.
    fn show_page_size_menu(&self, zone: FooterZone) {
        remove_ctx_menu();
        let colors = read_ctx_colors();
        let footer_bg = self.0.builder.borrow().theme.footer_bg.to_css();
        let rect = self.0.canvas.get_bounding_client_rect();
        let x = (rect.left() + zone.x) as i32;
        let zone_top = rect.top() + zone.y;
        let (_, menu) = create_menu_shell(
            x,
            zone_top as i32,
            &colors,
            &footer_bg,
            &self.0.canvas,
        );

        // Offer the configured sizes plus the current one, so the active
        // size is always listed (and checked) even if it isn't an option.
        let (current, mut sizes) = {
            let s = self.0.state.borrow();
            (s.model.page_size, s.model.page_size_options.clone())
        };
        if !sizes.contains(&current) {
            sizes.push(current);
        }
        sizes.sort_unstable();
        sizes.dedup();

        let doc = document();
        for n in sizes {
            let icon = if n == current { ICON_CHECK } else { "" };
            let item = make_menu_item(
                &doc,
                icon,
                &group_thousands(n),
                "",
                true,
                &colors,
            );
            let gc = self.clone();
            let cb = Closure::<dyn FnMut(_)>::new(move |_: MouseEvent| {
                remove_ctx_menu();
                gc.dispatch(GridCommand::SetPageSize(n));
            });
            item.add_event_listener_with_callback(
                "click",
                cb.as_ref().unchecked_ref(),
            )
            .expect("add page-size click listener");
            // Intentional forget — see the module comment.
            cb.forget();
            menu.append_child(&item).expect("append page-size item");
        }

        // Match the selector's width, then sit right above it (the shell
        // only knows how to clamp inside the canvas, which would cover
        // the selector).
        let _ = menu
            .style()
            .set_property("min-width", &format!("{}px", zone.width));
        let mh = menu.offset_height() as f64;
        let top = (zone_top - mh - 4.0).max(rect.top());
        let _ = menu
            .style()
            .set_property("top", &format!("{}px", top as i32));
    }
}
