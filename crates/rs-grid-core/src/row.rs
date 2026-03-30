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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_row_has_no_cells() {
        let r = RowRecord::new(42);
        assert_eq!(r.id, 42);
        assert!(r.cells.is_empty());
    }

    #[test]
    fn set_and_get() {
        let mut r = RowRecord::new(0);
        r.set("name", "Alice");
        assert_eq!(r.get("name"), Some("Alice"));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let r = RowRecord::new(0);
        assert_eq!(r.get("missing"), None);
    }

    #[test]
    fn set_overwrites_existing_value() {
        let mut r = RowRecord::new(0);
        r.set("k", "old");
        r.set("k", "new");
        assert_eq!(r.get("k"), Some("new"));
    }

    #[test]
    fn set_chaining() {
        let mut r = RowRecord::new(0);
        r.set("a", "1").set("b", "2").set("c", "3");
        assert_eq!(r.get("a"), Some("1"));
        assert_eq!(r.get("b"), Some("2"));
        assert_eq!(r.get("c"), Some("3"));
    }

    #[test]
    fn cell_value_default_is_empty() {
        let cv = CellValue::default();
        assert!(cv.0.is_empty());
    }

    #[test]
    fn cell_value_clone() {
        let cv = CellValue("hello".to_string());
        let cv2 = cv.clone();
        assert_eq!(cv.0, cv2.0);
    }
}
