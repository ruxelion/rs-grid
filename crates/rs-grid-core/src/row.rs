use std::collections::HashMap;

/// A single cell value (currently a UTF-8 string).
#[derive(Debug, Clone, Default)]
pub struct CellValue(pub String);

/// One row of data keyed by column key.
#[derive(Debug, Clone)]
pub struct RowRecord {
    pub id: u64,
    pub cells: HashMap<String, CellValue>,
}

impl RowRecord {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            cells: HashMap::new(),
        }
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.cells.insert(key.into(), CellValue(value.into()));
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.cells.get(key).map(|v| v.0.as_str())
    }
}
