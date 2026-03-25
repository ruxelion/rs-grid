use crate::commands::{CommandOutput, GridCommand};

use super::GridState;

impl GridState {
    pub(super) fn cmd_meta(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::SetHoveredRow(row) => {
                self.hovered_row = row;
                CommandOutput::None
            }
            GridCommand::SetHeaderHeight(h) => {
                if h > 0.0 {
                    self.model.header_height = h;
                }
                CommandOutput::None
            }
            GridCommand::SetRowHeight(h) => {
                if h > 0.0 {
                    self.model.row_height = h;
                }
                CommandOutput::None
            }
            GridCommand::NotifyPageLoaded => {
                // No-op — triggers a re-render via dispatch.
                CommandOutput::None
            }
            GridCommand::SetTotalRowCount(n) => {
                // Update the underlying data source row count.
                // For PageCacheDataSource this is done
                // externally; here we just trigger re-render.
                let _ = n;
                CommandOutput::None
            }
            _ => unreachable!(),
        }
    }
}
