use rs_grid_core::selection::CellCoord;

use super::{
    GridCanvas,
    dom_helpers::{document, make_el, set_styles},
};

impl GridCanvas {
    /// Validation error for a cell's *current at-rest value*, independent
    /// of any active edit session — `None` if the cell is valid or
    /// `row`/`col_key` don't resolve. Mirrors [`GridCanvas::validation_error`]
    /// (which only covers an in-progress edit). Useful to build a fully
    /// custom indicator without the built-in hover tooltip.
    pub fn cell_validation_error(
        &self,
        row: u64,
        col_key: &str,
    ) -> Option<String> {
        let state = self.0.state.borrow();
        let col = state.model.columns.iter().find(|c| c.key == col_key)?;
        let raw = state.model.get_cell(row, col_key)?;
        col.validate_value(&raw).err()
    }

    fn cell_validation_message(
        &self,
        coord: CellCoord,
    ) -> Option<(String, String)> {
        let state = self.0.state.borrow();
        let col = state.model.columns.get(coord.col)?;
        let raw = state.model.get_cell(coord.row, &col.key)?;
        col.validate_value(&raw)
            .err()
            .map(|msg| (col.key.clone(), msg))
    }

    fn ensure_tooltip_el(&self) -> web_sys::HtmlElement {
        if let Some(el) = self.0.tooltip_el.borrow().as_ref() {
            return el.clone();
        }
        let doc = document();
        let el = make_el(&doc, "div");
        set_styles(
            &el,
            &[
                ("position", "fixed"),
                // Never intercepts real mouse events — the mouse stays over
                // the canvas, this element only ever shows/hides visually.
                ("pointer-events", "none"),
                ("z-index", "10000"),
                ("display", "none"),
            ],
        );
        doc.body()
            .expect("no body")
            .append_child(&el)
            .expect("append tooltip");
        *self.0.tooltip_el.borrow_mut() = Some(el.clone());
        el
    }

    fn show_validation_tooltip(&self, row: u64, col_key: &str, message: &str) {
        let Some((left, top, w, h)) = self.cell_client_rect(row, col_key)
        else {
            return;
        };
        let el = self.ensure_tooltip_el();
        let _ = el.set_attribute("data-tip", message);
        let class = self.0.tooltip_class.borrow();
        let _ = el.set_attribute("class", class.as_deref().unwrap_or(""));
        set_styles(
            &el,
            &[
                ("left", &format!("{left}px")),
                ("top", &format!("{top}px")),
                ("width", &format!("{w}px")),
                ("height", &format!("{h}px")),
                ("display", "block"),
            ],
        );
    }

    /// Hides the at-rest validation tooltip, if currently shown. The
    /// backing DOM element is kept around (not removed) for reuse on the
    /// next hover — so this stays O(1) regardless of how many cells in
    /// the grid are invalid.
    pub(super) fn hide_validation_tooltip(&self) {
        if let Some(el) = self.0.tooltip_el.borrow().as_ref() {
            let _ = el.style().set_property("display", "none");
        }
        *self.0.tooltip_cell.borrow_mut() = None;
    }

    /// Tears down the tooltip DOM element entirely. Called from
    /// `detach()`.
    pub(super) fn remove_validation_tooltip(&self) {
        if let Some(el) = self.0.tooltip_el.borrow_mut().take() {
            el.remove();
        }
        *self.0.tooltip_cell.borrow_mut() = None;
    }

    /// Sets the CSS class applied to the at-rest validation tooltip's
    /// wrapper element — e.g. daisyUI's `"tooltip tooltip-open
    /// tooltip-error"`. rs-grid renders no visual of its own: without a
    /// class, the tooltip anchor is positioned but invisible. `data-tip`
    /// is always set to the validation message (read by daisyUI's
    /// `content: attr(data-tip)`). The class must force an always-open
    /// state (e.g. `tooltip-open`) since the wrapper never receives a
    /// real `:hover` — the mouse stays over the canvas, not this element.
    pub fn set_validation_tooltip_class(&self, class: Option<String>) {
        *self.0.tooltip_class.borrow_mut() = class;
        // If a tooltip is currently shown, re-apply immediately so a
        // live class change takes effect without requiring a new hover.
        if let Some((row, col)) = *self.0.tooltip_cell.borrow() {
            let key = self
                .0
                .state
                .borrow()
                .model
                .columns
                .get(col)
                .map(|c| c.key.clone());
            if let Some((key, msg)) = key.and_then(|k| {
                self.cell_validation_error(row, &k).map(|m| (k, m))
            }) {
                self.show_validation_tooltip(row, &key, &msg);
            }
        }
    }

    /// Re-evaluates the at-rest validation tooltip for the cell under
    /// `(vx, vy)` (viewport coordinates). No-op if the hovered cell is
    /// unchanged from the last call — avoids DOM writes on every
    /// `mousemove`, only on an actual cell change.
    pub(super) fn refresh_validation_tooltip(&self, vx: f64, vy: f64) {
        let coord = self.0.state.borrow().hit_test(vx, vy);
        let found = coord.and_then(|c| {
            let (row, col) = (c.row, c.col);
            self.cell_validation_message(c)
                .map(|(key, msg)| (row, col, key, msg))
        });
        let cell_key = found.as_ref().map(|(row, col, ..)| (*row, *col));
        if *self.0.tooltip_cell.borrow() == cell_key {
            return;
        }
        *self.0.tooltip_cell.borrow_mut() = cell_key;
        match found {
            Some((row, _, col_key, msg)) => {
                self.show_validation_tooltip(row, &col_key, &msg);
            }
            None => self.hide_validation_tooltip(),
        }
    }
}
