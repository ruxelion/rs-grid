use crate::commands::{CommandOutput, GridCommand};

use super::{clamp_scroll, GridState};

impl GridState {
    pub(super) fn cmd_scroll(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::ScrollTo { x, y } => {
                let (sx, sy) = clamp_scroll(x, y, &self.model, &self.viewport);
                self.viewport.scroll_x = sx;
                self.viewport.scroll_y = sy;
                CommandOutput::None
            }
            GridCommand::ScrollBy { dx, dy } => {
                let (sx, sy) = clamp_scroll(
                    self.viewport.scroll_x + dx,
                    self.viewport.scroll_y + dy,
                    &self.model,
                    &self.viewport,
                );
                self.viewport.scroll_x = sx;
                self.viewport.scroll_y = sy;
                CommandOutput::None
            }
            GridCommand::Resize { width, height } => {
                self.viewport.width = width;
                self.viewport.height = height;
                self.model.recalculate_flex_widths(width);
                self.model.rebuild_offsets();
                CommandOutput::None
            }
            _ => unreachable!(),
        }
    }
}
