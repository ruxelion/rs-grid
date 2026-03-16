use crate::row::RowRecord;

pub trait DataSource: std::fmt::Debug {
    fn row_count(&self) -> u64;
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String>;
    fn clone_box(&self) -> Box<dyn DataSource>;
}

impl Clone for Box<dyn DataSource> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ── VecDataSource ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VecDataSource {
    pub rows: Vec<RowRecord>,
}

impl VecDataSource {
    pub fn new(rows: Vec<RowRecord>) -> Self {
        Self { rows }
    }
}

impl DataSource for VecDataSource {
    fn row_count(&self) -> u64 {
        self.rows.len() as u64
    }
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String> {
        let row = usize::try_from(row).ok()?;
        self.rows.get(row)?.get(col_key).map(str::to_owned)
    }
    fn clone_box(&self) -> Box<dyn DataSource> {
        Box::new(self.clone())
    }
}

// ── FnDataSource ──────────────────────────────────────────────────────────────

pub struct FnDataSource<F: Fn(u64, &str) -> Option<String>> {
    count: u64,
    f: F,
}

impl<F: Fn(u64, &str) -> Option<String>> FnDataSource<F> {
    pub fn new(count: u64, f: F) -> Self {
        Self { count, f }
    }
}

impl<F: Fn(u64, &str) -> Option<String>> DataSource for FnDataSource<F> {
    fn row_count(&self) -> u64 {
        self.count
    }
    fn get_cell(&self, row: u64, col_key: &str) -> Option<String> {
        (self.f)(row, col_key)
    }
    /// FnDataSource is not cloneable — wrap GridModel in Rc if sharing is needed.
    fn clone_box(&self) -> Box<dyn DataSource> {
        panic!("FnDataSource is not cloneable; wrap GridModel in Rc if sharing is needed")
    }
}

// Debug manuel car les closures n'implémentent pas Debug
impl<F: Fn(u64, &str) -> Option<String>> std::fmt::Debug for FnDataSource<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FnDataSource")
            .field("count", &self.count)
            .finish()
    }
}
