/// Cell currently being edited inline.
#[derive(Debug, Clone)]
pub struct EditCell {
    /// Logical row index of the cell being edited.
    pub row: u64,
    /// Column key of the cell being edited.
    pub col_key: String,
    /// Cell value at the moment editing started.
    pub initial_value: String,
}
