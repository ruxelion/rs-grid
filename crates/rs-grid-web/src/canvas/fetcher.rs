use std::collections::HashMap;

use rs_grid_core::{commands::GridCommand, row::RowRecord, sort::SortState};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Server response for a single page of data.
pub struct PageFetchResponse {
    /// Rows returned for this page.
    pub rows: Vec<RowRecord>,
    /// Server-reported total row count.
    pub total_rows: u64,
}

/// Parameters sent to the server for a page request.
pub struct PageFetchRequest {
    /// Zero-based page number to fetch.
    pub page_num: u64,
    /// Number of rows per page.
    pub page_size: u64,
    /// Active sort state (`None` = natural order).
    pub sort: Option<SortState>,
    /// Active column filters (col_key to search text).
    pub filters: HashMap<String, String>,
}

/// Configuration for async page-based data fetching.
///
/// The caller provides two closures:
/// - `build_url` turns a `PageFetchRequest` into a URL
/// - `parse_response` converts a JSON `JsValue` into rows
pub struct FetchConfig {
    /// Build a URL from a page request.
    pub build_url: Box<dyn Fn(&PageFetchRequest) -> String>,
    /// Parse a JSON response into rows and total count.
    pub parse_response:
        Box<dyn Fn(wasm_bindgen::JsValue) -> Result<PageFetchResponse, String>>,
    /// Extra pages to prefetch around the visible range.
    pub buffer_pages: u64,
}

impl std::fmt::Debug for FetchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchConfig")
            .field("buffer_pages", &self.buffer_pages)
            .finish()
    }
}

/// Fetch a URL via `window.fetch()` and return parsed JSON.
async fn fetch_json(
    url: &str,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    let window = web_sys::window().expect("no global window");
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    let resp = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    Ok(json)
}

use super::GridCanvas;

impl GridCanvas {
    /// Check which pages are needed for the current
    /// viewport and spawn async fetches for missing ones.
    pub(super) fn maybe_fetch_pages(&self) {
        let config_ref = self.0.fetch_config.borrow();
        let Some(config) = config_ref.as_ref() else {
            return;
        };
        let cache_ref = self.0.page_cache.borrow();
        let Some(cache) = cache_ref.as_ref() else {
            return;
        };

        let state = self.0.state.borrow();
        let (row_start, row_end) = state.viewport.visible_rows(
            state.model.display_row_count(),
            state.model.row_height,
            state.model.header_height,
        );
        let sort = state.sort.clone();
        let filters = state.model.filters.clone();
        drop(state);

        let page_size = cache.page_size();
        let buf = config.buffer_pages * page_size;
        let fetch_start = row_start.saturating_sub(buf);
        let fetch_end = row_end.saturating_add(buf);

        let needed = cache.needed_pages(fetch_start, fetch_end);

        for page_num in needed {
            cache.mark_pending(page_num);
            let cache_clone = cache.clone();
            let req = PageFetchRequest {
                page_num,
                page_size,
                sort: sort.clone(),
                filters: filters.clone(),
            };
            let url = (config.build_url)(&req);

            // Clone the GridCanvas (Rc) so the async block
            // can re-borrow fetch_config from Inner.
            let gc = self.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match fetch_json(&url).await {
                    Ok(json) => {
                        // Re-borrow parse_response
                        // from the Rc<Inner>.
                        let cfg = gc.0.fetch_config.borrow();
                        let parse =
                            &cfg.as_ref().expect("config").parse_response;
                        match parse(json) {
                            Ok(resp) => {
                                cache_clone.set_total_rows(resp.total_rows);
                                cache_clone.insert_page(page_num, resp.rows);
                                drop(cfg);
                                gc.dispatch(GridCommand::NotifyPageLoaded);
                            }
                            Err(e) => {
                                cache_clone.unmark_pending(page_num);
                                drop(cfg);
                                web_sys::console::warn_1(
                                    &format!("parse error: {e}").into(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        cache_clone.unmark_pending(page_num);
                        web_sys::console::warn_1(&e);
                    }
                }
            });
        }
    }
}
