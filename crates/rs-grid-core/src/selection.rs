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
                CellCoord { row: a.row.min(f.row), col: a.col.min(f.col) },
                CellCoord { row: a.row.max(f.row), col: a.col.max(f.col) },
            )),
            _ => None,
        }
    }

    /// Sérialise la sélection en TSV (format RFC 4180 — compatible Excel/Sheets).
    pub fn to_tsv(
        &self,
        columns: &[crate::column::ColumnDef],
        data: &dyn crate::datasource::DataSource,
    ) -> Result<String, CopyError> {
        let (tl, br) = self.range().ok_or(CopyError::NoSelection)?;
        let row_count = br.row - tl.row + 1;
        if row_count > MAX_COPY_ROWS {
            return Err(CopyError::TooManyRows { actual: row_count, max: MAX_COPY_ROWS });
        }
        let mut out = String::new();
        for r in tl.row..=br.row {
            for ci in tl.col..=br.col {
                if ci > tl.col { out.push('\t'); }
                let cell = data.get_cell(r, &columns[ci].key).unwrap_or_default();
                // RFC 4180 : guillemets si la cellule contient tab, newline ou guillemet
                if cell.contains(['\t', '\n', '\r', '"']) {
                    out.push('"');
                    for ch in cell.chars() {
                        if ch == '"' { out.push('"'); }
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
        if chars.peek().is_none() { break; }
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
                Some('\t') => { chars.next(); } // more fields on same row
                Some('\r') => {
                    chars.next();
                    if chars.peek() == Some(&'\n') { chars.next(); }
                    break;
                }
                Some('\n') => { chars.next(); break; }
                _ => break, // EOF
            }
        }
        rows.push(row);
    }

    // Drop a trailing empty row (produced by a final newline)
    if rows.last().map_or(false, |r| r.len() == 1 && r[0].is_empty()) {
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
