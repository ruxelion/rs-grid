use std::collections::HashMap;

/// A single cell value (currently a UTF-8 string).
#[derive(Debug, Clone, Default)]
pub struct CellValue(pub String);

/// One row of data keyed by column key.
#[derive(Debug, Clone)]
pub struct RowRecord {
    /// Unique row identifier.
    pub id: u64,
    /// Cell values keyed by column key.
    pub cells: HashMap<String, CellValue>,
}

impl RowRecord {
    /// Create an empty row with the given identifier.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            cells: HashMap::new(),
        }
    }

    /// Insert or update a cell value and return `self` for
    /// chaining.
    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.cells.insert(key.into(), CellValue(value.into()));
        self
    }

    /// Look up a cell value by column key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cells.get(key).map(|v| v.0.as_str())
    }
}
