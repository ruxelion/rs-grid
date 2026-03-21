use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use crate::datasource::{CellStatus, DataSource};
use crate::row::RowRecord;

/// A page-cache data source for async/server-side data.
///
/// Cells are served from an in-memory page cache.
/// Pages are inserted by an external fetch coordinator.
/// Cloning shares the same underlying cache (`Rc`).
#[derive(Debug, Clone)]
pub struct PageCacheDataSource {
    inner: Rc<RefCell<PageCacheInner>>,
}

#[derive(Debug)]
struct PageCacheInner {
    /// Total row count as reported by the server.
    total_rows: u64,
    /// Number of rows per page.
    page_size: u64,
    /// Cached pages: page_number → rows.
    pages: HashMap<u64, Vec<RowRecord>>,
    /// Pages currently being fetched (avoid duplicate requests).
    pending: HashSet<u64>,
    /// Maximum number of pages to keep in cache (LRU eviction).
    max_cached_pages: usize,
    /// Access order for LRU eviction (front = oldest).
    access_order: VecDeque<u64>,
}

impl PageCacheDataSource {
    pub fn new(total_rows: u64, page_size: u64) -> Self {
        assert!(page_size > 0, "page_size must be > 0");
        Self {
            inner: Rc::new(RefCell::new(PageCacheInner {
                total_rows,
                page_size,
                pages: HashMap::new(),
                pending: HashSet::new(),
                max_cached_pages: 50,
                access_order: VecDeque::new(),
            })),
        }
    }

    pub fn page_size(&self) -> u64 {
        self.inner.borrow().page_size
    }

    pub fn total_rows(&self) -> u64 {
        self.inner.borrow().total_rows
    }

    pub fn set_total_rows(&self, n: u64) {
        self.inner.borrow_mut().total_rows = n;
    }

    pub fn set_max_cached_pages(&self, n: usize) {
        self.inner.borrow_mut().max_cached_pages = n;
    }

    /// Insert a fetched page into the cache.
    pub fn insert_page(
        &self,
        page_num: u64,
        rows: Vec<RowRecord>,
    ) {
        let mut inner = self.inner.borrow_mut();
        inner.pending.remove(&page_num);

        // LRU: remove old entry from access order if present
        inner
            .access_order
            .retain(|&p| p != page_num);
        inner.access_order.push_back(page_num);

        inner.pages.insert(page_num, rows);

        // Evict oldest pages if over limit
        while inner.pages.len() > inner.max_cached_pages {
            if let Some(old) = inner.access_order.pop_front()
            {
                inner.pages.remove(&old);
            } else {
                break;
            }
        }
    }

    /// Clear all cached pages (e.g. after sort/filter change).
    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.pages.clear();
        inner.pending.clear();
        inner.access_order.clear();
    }

    pub fn is_page_loaded(&self, page_num: u64) -> bool {
        self.inner.borrow().pages.contains_key(&page_num)
    }

    pub fn is_page_pending(&self, page_num: u64) -> bool {
        self.inner.borrow().pending.contains(&page_num)
    }

    pub fn mark_pending(&self, page_num: u64) {
        self.inner.borrow_mut().pending.insert(page_num);
    }

    pub fn unmark_pending(&self, page_num: u64) {
        self.inner.borrow_mut().pending.remove(&page_num);
    }

    /// Return page numbers in the given row range that are
    /// neither loaded nor pending.
    pub fn needed_pages(
        &self,
        first_row: u64,
        last_row: u64,
    ) -> Vec<u64> {
        let inner = self.inner.borrow();
        let ps = inner.page_size;
        let first_page = first_row / ps;
        let last_page = last_row.saturating_sub(1) / ps;
        let mut result = Vec::new();
        for p in first_page..=last_page {
            if !inner.pages.contains_key(&p)
                && !inner.pending.contains(&p)
            {
                result.push(p);
            }
        }
        result
    }

    /// Page number for a given row.
    fn page_for_row(
        page_size: u64,
        row: u64,
    ) -> (u64, usize) {
        let page = row / page_size;
        let offset = (row % page_size) as usize;
        (page, offset)
    }
}

impl DataSource for PageCacheDataSource {
    fn row_count(&self) -> u64 {
        self.inner.borrow().total_rows
    }

    fn get_cell(
        &self,
        row: u64,
        col_key: &str,
    ) -> Option<String> {
        let inner = self.inner.borrow();
        let (page, offset) =
            Self::page_for_row(inner.page_size, row);
        let page_data = inner.pages.get(&page)?;
        let record = page_data.get(offset)?;
        record.get(col_key).map(str::to_owned)
    }

    fn cell_status(
        &self,
        row: u64,
        col_key: &str,
    ) -> CellStatus {
        let inner = self.inner.borrow();
        let (page, offset) =
            Self::page_for_row(inner.page_size, row);
        match inner.pages.get(&page) {
            None => CellStatus::Loading,
            Some(page_data) => {
                match page_data.get(offset) {
                    None => CellStatus::Absent,
                    Some(record) => {
                        match record.get(col_key) {
                            Some(v) => {
                                CellStatus::Ready(
                                    v.to_owned(),
                                )
                            }
                            None => CellStatus::Absent,
                        }
                    }
                }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn DataSource> {
        Box::new(self.clone())
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row::RowRecord;

    fn make_row(id: u64) -> RowRecord {
        let mut r = RowRecord::new(id);
        r.set("name", format!("user_{id}"));
        r.set("email", format!("u{id}@test.com"));
        r
    }

    fn make_page(start: u64, count: u64) -> Vec<RowRecord> {
        (start..start + count).map(make_row).collect()
    }

    #[test]
    fn empty_cache_returns_loading() {
        let ds = PageCacheDataSource::new(1000, 100);
        assert_eq!(
            ds.cell_status(0, "name"),
            CellStatus::Loading
        );
        assert_eq!(
            ds.cell_status(50, "name"),
            CellStatus::Loading
        );
        assert_eq!(ds.get_cell(0, "name"), None);
    }

    #[test]
    fn inserted_page_returns_ready() {
        let ds = PageCacheDataSource::new(1000, 100);
        ds.insert_page(0, make_page(0, 100));
        assert_eq!(
            ds.cell_status(0, "name"),
            CellStatus::Ready("user_0".into())
        );
        assert_eq!(
            ds.get_cell(50, "name"),
            Some("user_50".into())
        );
        // Page 1 still loading
        assert_eq!(
            ds.cell_status(100, "name"),
            CellStatus::Loading
        );
    }

    #[test]
    fn absent_for_missing_column() {
        let ds = PageCacheDataSource::new(1000, 100);
        ds.insert_page(0, make_page(0, 100));
        assert_eq!(
            ds.cell_status(0, "nonexistent"),
            CellStatus::Absent
        );
    }

    #[test]
    fn needed_pages_empty_cache() {
        let ds = PageCacheDataSource::new(1000, 100);
        assert_eq!(ds.needed_pages(0, 250), vec![0, 1, 2]);
    }

    #[test]
    fn needed_pages_partial_cache() {
        let ds = PageCacheDataSource::new(1000, 100);
        ds.insert_page(0, make_page(0, 100));
        assert_eq!(ds.needed_pages(0, 250), vec![1, 2]);
    }

    #[test]
    fn needed_pages_excludes_pending() {
        let ds = PageCacheDataSource::new(1000, 100);
        ds.mark_pending(1);
        assert_eq!(ds.needed_pages(0, 250), vec![0, 2]);
    }

    #[test]
    fn clear_invalidates_all() {
        let ds = PageCacheDataSource::new(1000, 100);
        ds.insert_page(0, make_page(0, 100));
        ds.mark_pending(1);
        ds.clear();
        assert_eq!(
            ds.cell_status(0, "name"),
            CellStatus::Loading
        );
        assert!(!ds.is_page_pending(1));
    }

    #[test]
    fn clone_shares_cache() {
        let ds = PageCacheDataSource::new(1000, 100);
        let ds2 = ds.clone();
        ds2.insert_page(0, make_page(0, 100));
        // Original sees the data
        assert_eq!(
            ds.get_cell(0, "name"),
            Some("user_0".into())
        );
    }

    #[test]
    fn row_count_and_set_total_rows() {
        let ds = PageCacheDataSource::new(500, 50);
        assert_eq!(ds.row_count(), 500);
        ds.set_total_rows(2000);
        assert_eq!(ds.row_count(), 2000);
    }

    #[test]
    fn lru_eviction() {
        let ds = PageCacheDataSource::new(10000, 100);
        ds.set_max_cached_pages(3);
        ds.insert_page(0, make_page(0, 100));
        ds.insert_page(1, make_page(100, 100));
        ds.insert_page(2, make_page(200, 100));
        // All three loaded
        assert!(ds.is_page_loaded(0));
        assert!(ds.is_page_loaded(1));
        assert!(ds.is_page_loaded(2));
        // Insert a 4th — page 0 (oldest) should be evicted
        ds.insert_page(3, make_page(300, 100));
        assert!(!ds.is_page_loaded(0));
        assert!(ds.is_page_loaded(1));
        assert!(ds.is_page_loaded(3));
    }

    #[test]
    fn pending_lifecycle() {
        let ds = PageCacheDataSource::new(1000, 100);
        assert!(!ds.is_page_pending(0));
        ds.mark_pending(0);
        assert!(ds.is_page_pending(0));
        ds.unmark_pending(0);
        assert!(!ds.is_page_pending(0));
    }
}
