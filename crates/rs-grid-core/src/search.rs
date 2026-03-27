use crate::{
    model::{DataSourceMode, GridModel},
    selection::CellCoord,
};

/// Active search state.
///
/// Tracks the current query, all matching cell coordinates,
/// and the index of the currently focused match.
///
/// # Scanning limits
///
/// For performance, the search engine scans at most **100 000 rows** and
/// records at most **10 000 matches** per query. If the dataset exceeds
/// these thresholds the scan stops early and `matches` only reflects the
/// first portion of the data. This limit applies to client-side data
/// sources; server-side (paginated) sources are not scanned.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search text (empty = search inactive).
    pub query: String,
    /// Cell coordinates matching the query.
    pub matches: Vec<CellCoord>,
    /// Index into `matches` for the currently focused result.
    pub current: usize,
}

impl SearchState {
    /// Scan the model for cells containing `query` (case-
    /// insensitive) and return a new [`SearchState`].
    ///
    /// Returns an empty state when the query is empty or the
    /// data source is server-side (too much data to scan
    /// locally).
    ///
    /// Caps results at 10 000 matches and scans at most
    /// 100 000 rows to avoid stalling the main thread.
    pub(crate) fn run(model: &GridModel, query: &str) -> Self {
        let mut state = SearchState {
            query: query.to_string(),
            matches: Vec::new(),
            current: 0,
        };
        if query.is_empty() {
            return state;
        }
        // Server-side mode: too much data to search locally.
        if model.mode == DataSourceMode::ServerSide {
            return state;
        }
        // Max matches recorded per query.
        const MAX_MATCHES: usize = 10_000;
        // Max rows scanned per query.
        const MAX_ROWS: u64 = 100_000;
        let query_lower = query.to_ascii_lowercase();
        let row_count = model.display_row_count().min(MAX_ROWS);
        let col_count = model.columns.len();
        for r in 0..row_count {
            for ci in 0..col_count {
                let key = &model.columns[ci].key;
                if let Some(val) = model.get_cell(r, key) {
                    if val.to_ascii_lowercase().contains(&query_lower) {
                        state.matches.push(CellCoord { row: r, col: ci });
                        if state.matches.len() >= MAX_MATCHES {
                            return state;
                        }
                    }
                }
            }
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        column::ColumnDef,
        datasource::{FnDataSource, VecDataSource},
        model::GridModelBuilder,
        row::RowRecord,
    };

    fn one_row_model(value: &str) -> GridModel {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let mut r = RowRecord::new(0);
        r.set("a", value);
        GridModel::new(cols, vec![r], 30.0, 40.0)
    }

    #[test]
    fn run_empty_query_returns_empty() {
        let model = one_row_model("hello");
        let s = SearchState::run(&model, "");
        assert!(s.matches.is_empty());
    }

    #[test]
    fn run_server_side_skips_scan() {
        let cols = vec![ColumnDef::new("a", "A", 100.0)];
        let mut r = RowRecord::new(0);
        r.set("a", "hello");
        let model = GridModelBuilder::new(
            cols,
            Box::new(VecDataSource::new(vec![r])),
        )
        .mode(DataSourceMode::ServerSide)
        .build();
        let s = SearchState::run(&model, "hello");
        assert!(s.matches.is_empty());
    }

    #[test]
    fn run_caps_at_max_matches() {
        // 200 rows × 51 columns = 10 200 cells, all matching.
        let cols: Vec<ColumnDef> = (0..51)
            .map(|i| {
                ColumnDef::new(
                    &format!("c{i}"),
                    &format!("C{i}"),
                    80.0,
                )
            })
            .collect();
        let keys: Vec<String> =
            (0..51).map(|i| format!("c{i}")).collect();
        let ds = FnDataSource::new(200, move |_row, col| {
            if keys.contains(&col.to_string()) {
                Some("match".into())
            } else {
                None
            }
        });
        let model =
            GridModelBuilder::new(cols, Box::new(ds)).build();
        let s = SearchState::run(&model, "match");
        assert_eq!(s.matches.len(), 10_000);
    }

    #[test]
    fn run_ascii_lowercase_only() {
        let model = one_row_model("Éléphant");
        // ASCII lowercase: 'É' stays 'É', doesn't match 'é'.
        let s = SearchState::run(&model, "éléphant");
        assert!(
            s.matches.is_empty(),
            "to_ascii_lowercase does not fold accented chars"
        );
        // But exact case matches.
        let s2 = SearchState::run(&model, "Éléphant");
        assert_eq!(s2.matches.len(), 1);
    }
}
