/// Cell currently being edited inline.
#[derive(Debug, Clone)]
pub struct EditCell {
    /// Logical row index of the cell being edited.
    pub row: u64,
    /// Column key of the cell being edited.
    pub col_key: String,
    /// Column index at the time editing started.
    /// Stored so renderers can position the editor without
    /// re-resolving by key (which fails when keys are not unique).
    pub col_idx: usize,
    /// Cell value at the moment editing started.
    pub initial_value: String,
}
