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
                if col_idx < self.model.columns.len() {
                    self.model.columns[col_idx].flex = None;
                    self.model.columns[col_idx].width =
                        self.model.columns[col_idx].clamp_width(new_width);
                    self.model.rebuild_offsets();
                }
                CommandOutput::None
            }
            GridCommand::CommitColumnResize {
                col_idx,
                old_width,
                old_flex,
            } => {
                if col_idx < self.model.columns.len() {
                    let cur = self.model.columns[col_idx].width;
                    let flex_changed = old_flex
                        != self.model.columns[col_idx].flex;
                    if (cur - old_width).abs() > f64::EPSILON
                        || flex_changed
                    {
                        self.history.push(UndoEntry::ResizeColumn {
                            col_idx,
                            old_width,
                            old_flex,
                        });
                    }
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
                header_right_reserve,
            } => {
                // Max rows sampled for auto-fit width.
                const MAX_SAMPLE_ROWS: u64 = 1_000;
                if col_idx < self.model.columns.len() {
                    let old_width = self.model.columns[col_idx].width;
                    let old_flex = self.model.columns[col_idx].flex;
                    let col_key = self.model.columns[col_idx].key.clone();
                    let label = &self.model.columns[col_idx].label;
                    let header_w = label.chars().count() as f64
                        * header_char_width
                        + cell_padding * 2.0
                        + header_right_reserve;
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
                                        .map(|i| val[i + 1..].chars().count())
                                        .unwrap_or_else(|| val.chars().count());
                                    image_size
                                        + gap
                                        + label_len as f64 * char_width
                                        + cell_padding * 2.0
                                }
                                Some(fmt) => {
                                    let formatted = format_cell(&val, fmt);
                                    formatted.text.chars().count() as f64
                                        * char_width
                                        + cell_padding * 2.0
                                }
                                None => {
                                    val.chars().count() as f64 * char_width
                                        + cell_padding * 2.0
                                }
                            };
                            if w > max_w {
                                max_w = w;
                            }
                        }
                    }
                    self.model.columns[col_idx].flex = None;
                    self.model.columns[col_idx].width =
                        self.model.columns[col_idx].clamp_width(max_w);
                    self.model.rebuild_offsets();
                    self.history.push(UndoEntry::ResizeColumn {
                        col_idx,
                        old_width,
                        old_flex,
                    });
                }
                CommandOutput::None
            }
            GridCommand::AutoFitAllColumns {
                char_width,
                header_char_width,
                cell_padding,
                header_right_reserve,
            } => {
                let n = self.model.columns.len();
                for col_idx in 0..n {
                    self.apply(GridCommand::AutoFitColumn {
                        col_idx,
                        char_width,
                        header_char_width,
                        cell_padding,
                        header_right_reserve,
                    });
                }
                CommandOutput::None
            }
            _ => unreachable!(),
        }
    }
}
