/// A (row, col) address of a cell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellCoord {
    pub row: u64,
    pub col: usize,
}

/// Rectangular selection defined by an anchor and a focus cell.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub anchor: Option<CellCoord>,
    pub focus: Option<CellCoord>,
}

impl SelectionState {
    /// Start a new single-cell selection.
    pub fn select_cell(&mut self, row: u64, col: usize) {
        let coord = CellCoord { row, col };
        self.anchor = Some(coord.clone());
        self.focus = Some(coord);
    }

    /// Extend the selection to cover a new focus cell (shift-click / shift-arrow).
    pub fn extend_to(&mut self, row: u64, col: usize) {
        self.focus = Some(CellCoord { row, col });
    }

    /// Returns `true` if the given cell falls inside the selected rectangle.
    pub fn is_selected(&self, row: u64, col: usize) -> bool {
        match (&self.anchor, &self.focus) {
            (Some(a), Some(f)) => {
                let r_min = a.row.min(f.row);
                let r_max = a.row.max(f.row);
                let c_min = a.col.min(f.col);
                let c_max = a.col.max(f.col);
                row >= r_min && row <= r_max && col >= c_min && col <= c_max
            }
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.focus = None;
    }

    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }

    /// Bounding rectangle normalisé (min/max).
    pub fn range(&self) -> Option<(CellCoord, CellCoord)> {
        match (&self.anchor, &self.focus) {
            (Some(a), Some(f)) => Some((
                CellCoord {
                    row: a.row.min(f.row),
                    col: a.col.min(f.col),
                },
                CellCoord {
                    row: a.row.max(f.row),
                    col: a.col.max(f.col),
                },
            )),
            _ => None,
        }
    }

    /// Sérialise la sélection en TSV (format RFC 4180 — compatible Excel/Sheets).
    pub fn to_tsv(
        &self,
        model: &crate::model::GridModel,
    ) -> Result<String, CopyError> {
        let (tl, br) = self.range().ok_or(CopyError::NoSelection)?;
        let row_count = br.row - tl.row + 1;
        if row_count > MAX_COPY_ROWS {
            return Err(CopyError::TooManyRows {
                actual: row_count,
                max: MAX_COPY_ROWS,
            });
        }
        let mut out = String::new();
        for r in tl.row..=br.row {
            for ci in tl.col..=br.col {
                if ci > tl.col {
                    out.push('\t');
                }
                let cell = model
                    .get_cell(r, &model.columns[ci].key)
                    .unwrap_or_default();
                // RFC 4180 : guillemets si la cellule contient tab, newline ou guillemet
                if cell.contains(['\t', '\n', '\r', '"']) {
                    out.push('"');
                    for ch in cell.chars() {
                        if ch == '"' {
                            out.push('"');
                        }
                        out.push(ch);
                    }
                    out.push('"');
                } else {
                    out.push_str(&cell);
                }
            }
            out.push('\n');
        }
        Ok(out)
    }
}

/// Parse a TSV/CSV string (RFC 4180, tab-separated) into a 2-D array of strings.
/// Trailing empty row produced by a final newline is dropped.
pub fn parse_tsv(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut chars = text.chars().peekable();

    loop {
        if chars.peek().is_none() {
            break;
        }
        let mut row: Vec<String> = Vec::new();
        loop {
            // Parse one field
            let field = if chars.peek() == Some(&'"') {
                // Quoted field
                chars.next(); // consume opening '"'
                let mut s = String::new();
                loop {
                    match chars.next() {
                        None => break,
                        Some('"') => {
                            if chars.peek() == Some(&'"') {
                                chars.next(); // escaped quote
                                s.push('"');
                            } else {
                                break; // closing quote
                            }
                        }
                        Some(c) => s.push(c),
                    }
                }
                s
            } else {
                // Unquoted field — read until tab or newline
                let mut s = String::new();
                loop {
                    match chars.peek() {
                        None | Some('\t') | Some('\n') => break,
                        Some('\r') => break,
                        _ => s.push(chars.next().unwrap()),
                    }
                }
                s
            };
            row.push(field);
            match chars.peek() {
                Some('\t') => {
                    chars.next();
                } // more fields on same row
                Some('\r') => {
                    chars.next();
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    break;
                }
                Some('\n') => {
                    chars.next();
                    break;
                }
                _ => break, // EOF
            }
        }
        rows.push(row);
    }

    // Drop a trailing empty row (produced by a final newline)
    if rows
        .last()
        .map_or(false, |r| r.len() == 1 && r[0].is_empty())
    {
        rows.pop();
    }
    rows
}

pub const MAX_COPY_ROWS: u64 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyError {
    NoSelection,
    TooManyRows { actual: u64, max: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{column::ColumnDef, model::GridModel, row::RowRecord};

    fn sel(r: u64, c: usize) -> SelectionState {
        let mut s = SelectionState::default();
        s.select_cell(r, c);
        s
    }

    // ── SelectionState ────────────────────────────────────────────────────────

    #[test]
    fn select_cell_sets_anchor_and_focus() {
        let s = sel(3, 2);
        assert_eq!(s.anchor, Some(CellCoord { row: 3, col: 2 }));
        assert_eq!(s.focus, Some(CellCoord { row: 3, col: 2 }));
    }

    #[test]
    fn extend_to_moves_focus_only() {
        let mut s = sel(0, 0);
        s.extend_to(4, 2);
        assert_eq!(s.anchor, Some(CellCoord { row: 0, col: 0 }));
        assert_eq!(s.focus, Some(CellCoord { row: 4, col: 2 }));
    }

    #[test]
    fn is_selected_single_cell() {
        let s = sel(2, 1);
        assert!(s.is_selected(2, 1));
        assert!(!s.is_selected(2, 2));
        assert!(!s.is_selected(3, 1));
    }

    #[test]
    fn is_selected_rectangle() {
        let mut s = sel(1, 1);
        s.extend_to(3, 3);
        // corners
        assert!(s.is_selected(1, 1));
        assert!(s.is_selected(3, 3));
        assert!(s.is_selected(2, 2));
        // outside
        assert!(!s.is_selected(0, 1));
        assert!(!s.is_selected(1, 0));
        assert!(!s.is_selected(4, 3));
    }

    #[test]
    fn is_selected_inverted_anchor_focus() {
        // anchor > focus — rectangle is still normalised
        let mut s = sel(4, 3);
        s.extend_to(1, 1);
        assert!(s.is_selected(1, 1));
        assert!(s.is_selected(4, 3));
        assert!(s.is_selected(2, 2));
    }

    #[test]
    fn clear_resets_state() {
        let mut s = sel(0, 0);
        s.clear();
        assert!(!s.has_selection());
        assert!(!s.is_selected(0, 0));
    }

    #[test]
    fn range_normalises_coords() {
        let mut s = sel(5, 3);
        s.extend_to(2, 1);
        let (tl, br) = s.range().unwrap();
        assert_eq!(tl, CellCoord { row: 2, col: 1 });
        assert_eq!(br, CellCoord { row: 5, col: 3 });
    }

    #[test]
    fn range_none_when_empty() {
        assert!(SelectionState::default().range().is_none());
    }

    // ── parse_tsv ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_simple() {
        let r = parse_tsv("a\tb\nc\td\n");
        assert_eq!(r, vec![vec!["a", "b"], vec!["c", "d"]]);
    }

    #[test]
    fn parse_no_trailing_newline() {
        let r = parse_tsv("a\tb");
        assert_eq!(r, vec![vec!["a", "b"]]);
    }

    #[test]
    fn parse_quoted_field_with_tab() {
        let r = parse_tsv("\"a\tb\"\tc");
        assert_eq!(r, vec![vec!["a\tb", "c"]]);
    }

    #[test]
    fn parse_escaped_quote() {
        let r = parse_tsv("\"say \"\"hello\"\"\"");
        assert_eq!(r, vec![vec!["say \"hello\""]]);
    }

    #[test]
    fn parse_crlf_line_endings() {
        let r = parse_tsv("a\tb\r\nc\td\r\n");
        assert_eq!(r, vec![vec!["a", "b"], vec!["c", "d"]]);
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse_tsv("").is_empty());
    }

    // ── to_tsv ────────────────────────────────────────────────────────────────

    fn make_model() -> GridModel {
        let cols = vec![
            ColumnDef::new("a", "A", 100.0),
            ColumnDef::new("b", "B", 100.0),
        ];
        let rows = vec![
            {
                let mut r = RowRecord::new(0);
                r.set("a", "hello");
                r.set("b", "world");
                r
            },
            {
                let mut r = RowRecord::new(1);
                r.set("a", "foo");
                r.set("b", "bar");
                r
            },
        ];
        GridModel::new(cols, rows, 30.0, 40.0)
    }

    #[test]
    fn to_tsv_single_cell() {
        let mut s = SelectionState::default();
        s.select_cell(0, 0);
        let model = make_model();
        assert_eq!(s.to_tsv(&model).unwrap(), "hello\n");
    }

    #[test]
    fn to_tsv_full_range() {
        let mut s = SelectionState::default();
        s.select_cell(0, 0);
        s.extend_to(1, 1);
        let model = make_model();
        assert_eq!(s.to_tsv(&model).unwrap(), "hello\tworld\nfoo\tbar\n");
    }

    #[test]
    fn to_tsv_no_selection_error() {
        let model = make_model();
        assert_eq!(
            SelectionState::default().to_tsv(&model),
            Err(CopyError::NoSelection)
        );
    }
}
