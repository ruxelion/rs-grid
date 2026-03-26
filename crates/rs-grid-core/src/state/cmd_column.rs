use crate::{
    commands::{CommandOutput, GridCommand},
    format::{format_cell, CellFormat},
    undo::UndoEntry,
};

use super::GridState;

impl GridState {
    pub(super) fn cmd_column(&mut self, cmd: GridCommand) -> CommandOutput {
        match cmd {
            GridCommand::ResizeColumn { col_idx, new_width } => {
                /// Minimum column width in logical pixels.
                const MIN_COL_WIDTH: f64 = 20.0;
                if col_idx < self.model.columns.len() {
                    self.model.columns[col_idx].width =
                        new_width.max(MIN_COL_WIDTH);
                    self.model.rebuild_offsets();
                }
                CommandOutput::None
            }
            GridCommand::SetPinnedColumnCount { count } => {
                self.model.pinned_count = count.min(self.model.columns.len());
                CommandOutput::None
            }
            GridCommand::MoveColumn { from_idx, to_idx } => {
                let len = self.model.columns.len();
                if from_idx < len && to_idx < len && from_idx != to_idx {
                    let col = self.model.columns.remove(from_idx);
                    self.model.columns.insert(to_idx, col);
                    self.model.rebuild_offsets();
                    self.history.push(UndoEntry::MoveColumn {
                        from_idx: to_idx,
                        to_idx: from_idx,
                    });
                }
                CommandOutput::None
            }
            GridCommand::AutoFitColumn {
                col_idx,
                char_width,
                header_char_width,
                cell_padding,
            } => {
                // Minimum column width in logical pixels.
                const MIN_COL_WIDTH: f64 = 20.0;
                // Max rows sampled for auto-fit width.
                const MAX_SAMPLE_ROWS: u64 = 1_000;
                if col_idx < self.model.columns.len() {
                    let old_width = self.model.columns[col_idx].width;
                    let col_key = self.model.columns[col_idx].key.clone();
                    let label = &self.model.columns[col_idx].label;
                    let header_w = label.len() as f64 * header_char_width
                        + cell_padding * 2.0;
                    let col_format = self.model.columns[col_idx].format.clone();
                    let row_count =
                        self.model.display_row_count().min(MAX_SAMPLE_ROWS);
                    let mut max_w = header_w;
                    for r in 0..row_count {
                        if let Some(val) = self.model.get_cell(r, &col_key) {
                            let w = match &col_format {
                                Some(CellFormat::Image { .. }) => {
                                    self.model.row_height + cell_padding * 2.0
                                }
                                Some(CellFormat::ImageText {
                                    image_size,
                                    gap,
                                    ..
                                }) => {
                                    let label_len = val
                                        .find(' ')
                                        .map(|i| val[i + 1..].len())
                                        .unwrap_or(val.len());
                                    image_size
                                        + gap
                                        + label_len as f64 * char_width
                                        + cell_padding * 2.0
                                }
                                Some(fmt) => {
                                    let formatted = format_cell(&val, fmt);
                                    formatted.text.len() as f64 * char_width
                                        + cell_padding * 2.0
                                }
                                None => {
                                    val.len() as f64 * char_width
                                        + cell_padding * 2.0
                                }
                            };
                            if w > max_w {
                                max_w = w;
                            }
                        }
                    }
                    self.model.columns[col_idx].width =
                        max_w.max(MIN_COL_WIDTH);
                    self.model.rebuild_offsets();
                    self.history
                        .push(UndoEntry::ResizeColumn { col_idx, old_width });
                }
                CommandOutput::None
            }
            GridCommand::AutoFitAllColumns {
                char_width,
                header_char_width,
                cell_padding,
            } => {
                let n = self.model.columns.len();
                for col_idx in 0..n {
                    self.apply(GridCommand::AutoFitColumn {
                        col_idx,
                        char_width,
                        header_char_width,
                        cell_padding,
                    });
                }
                CommandOutput::None
            }
            _ => unreachable!(),
        }
    }
}
