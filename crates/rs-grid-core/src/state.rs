use crate::{
    commands::GridCommand,
    hit_test,
    model::GridModel,
    selection::{CellCoord, SelectionState},
    viewport::ViewportState,
};

/// The complete mutable state of a grid instance.
#[derive(Debug)]
pub struct GridState {
    pub model: GridModel,
    pub viewport: ViewportState,
    pub selection: SelectionState,
}

impl GridState {
    pub fn new(model: GridModel, viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            model,
            viewport: ViewportState::new(viewport_width, viewport_height),
            selection: SelectionState::default(),
        }
    }

    /// Apply a command, mutating state in place.
    pub fn apply(&mut self, cmd: GridCommand) {
        match cmd {
            GridCommand::SelectCell(coord) => {
                self.selection.select_cell(coord.row, coord.col);
            }
            GridCommand::ExtendSelection(coord) => {
                self.selection.extend_to(coord.row, coord.col);
            }
            GridCommand::ScrollTo { x, y } => {
                let max_x = (self.model.total_width() - self.viewport.width).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
            }
            GridCommand::ScrollBy { dx, dy } => {
                let x = self.viewport.scroll_x + dx;
                let y = self.viewport.scroll_y + dy;
                let max_x = (self.model.total_width() - self.viewport.width).max(0.0);
                let max_y = (self.model.total_height() - self.viewport.height).max(0.0);
                self.viewport.scroll_x = x.clamp(0.0, max_x);
                self.viewport.scroll_y = y.clamp(0.0, max_y);
            }
            GridCommand::Resize { width, height } => {
                self.viewport.width = width;
                self.viewport.height = height;
            }
            GridCommand::ClearSelection => {
                self.selection.clear();
            }
        }
    }

    /// Hit-test a viewport-space pointer position against the grid.
    pub fn hit_test(&self, vx: f64, vy: f64) -> Option<CellCoord> {
        hit_test::hit_test(
            vx,
            vy,
            &self.model,
            self.viewport.scroll_x,
            self.viewport.scroll_y,
        )
    }
}
