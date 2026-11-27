//! Footer bar and pagination helpers.
//!
//! The footer is a bar reserved at the bottom of the grid. It shows the
//! total / selected row counts and the rows-per-page selector on the left,
//! and (when pagination is enabled) the page navigation on the right.
//!
//! This module only holds renderer-agnostic, pure logic: the user-facing
//! label templates ([`FooterLabels`]) and the page-number windowing
//! ([`page_items`]). Pagination state lives on
//! [`GridModel`](crate::model::GridModel) and is mutated through
//! [`GridCommand`](crate::commands::GridCommand).

/// Default rows-per-page choices offered by the footer selector.
pub const DEFAULT_PAGE_SIZE_OPTIONS: [u64; 4] = [10, 25, 50, 100];

/// Default number of rows per page when pagination is enabled.
pub const DEFAULT_PAGE_SIZE: u64 = 25;

/// Default footer height in logical pixels.
pub const DEFAULT_FOOTER_HEIGHT: f64 = 48.0;

/// Number of pages needed to show `row_count` rows `page_size` at a time.
///
/// Always at least `1` (an empty grid still has one — empty — page) and
/// `page_size` is treated as at least `1`.
pub fn page_count(row_count: u64, page_size: u64) -> u64 {
    let size = page_size.max(1);
    row_count.div_ceil(size).max(1)
}

/// One entry of the page-number strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageItem {
    /// A page button, zero-based.
    Page(u64),
    /// A gap standing for two or more hidden pages.
    Ellipsis,
}

/// Page-number strip for the footer, e.g. `1 … 4 5 6 … 50`.
///
/// `page` is the zero-based current page and `pages` the total page count.
/// The first page, the last page and the current page with its two
/// neighbours are always present. A gap of exactly one page is filled with
/// that page (an ellipsis would hide nothing more than the number itself).
/// Up to 7 pages are listed in full.
pub fn page_items(page: u64, pages: u64) -> Vec<PageItem> {
    let pages = pages.max(1);
    let page = page.min(pages - 1);
    if pages <= 7 {
        return (0..pages).map(PageItem::Page).collect();
    }
    let mut keep: Vec<u64> = vec![0, pages - 1, page];
    if page > 0 {
        keep.push(page - 1);
    }
    if page + 1 < pages {
        keep.push(page + 1);
    }
    keep.sort_unstable();
    keep.dedup();

    let mut out = Vec::with_capacity(keep.len() + 2);
    let mut prev: Option<u64> = None;
    for p in keep {
        if let Some(q) = prev {
            match p - q {
                1 => {}
                2 => out.push(PageItem::Page(q + 1)),
                _ => out.push(PageItem::Ellipsis),
            }
        }
        out.push(PageItem::Page(p));
        prev = Some(p);
    }
    out
}

/// Format `n` with a no-break space as grouping separator (`1250` →
/// `1 250`). A plain U+00A0, not the narrow U+202F: several UI fonts
/// render the narrow one with no visible width on a canvas.
pub fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('\u{a0}');
        }
        out.push(ch);
    }
    out
}

/// User-facing footer text templates.
///
/// The scene crate has no locale access, so the host (`rs-grid-web`)
/// pushes localized templates through
/// [`GridCommand::SetFooterLabels`](crate::commands::GridCommand::SetFooterLabels).
/// Placeholders: `{n}` (a count), `{from}`, `{to}` and `{total}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterLabels {
    /// Total row count, e.g. `"{n} rows"`.
    pub rows: String,
    /// Selected row count, e.g. `"{n} selected"`.
    pub selected: String,
    /// Label in front of the rows-per-page selector.
    pub rows_per_page: String,
    /// Visible range, e.g. `"{from}–{to} of {total}"`.
    pub range: String,
}

impl Default for FooterLabels {
    fn default() -> Self {
        Self {
            rows: "{n} rows".to_owned(),
            selected: "{n} selected".to_owned(),
            rows_per_page: "Rows per page:".to_owned(),
            range: "{from}\u{2013}{to} of {total}".to_owned(),
        }
    }
}

impl FooterLabels {
    /// `"1 250 rows"`.
    pub fn rows_text(&self, n: u64) -> String {
        self.rows.replace("{n}", &group_thousands(n))
    }

    /// `"3 selected"`.
    pub fn selected_text(&self, n: u64) -> String {
        self.selected.replace("{n}", &group_thousands(n))
    }

    /// `"1–25 of 1 250"` (one-based, inclusive).
    pub fn range_text(&self, from: u64, to: u64, total: u64) -> String {
        self.range
            .replace("{from}", &group_thousands(from))
            .replace("{to}", &group_thousands(to))
            .replace("{total}", &group_thousands(total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages_of(items: &[PageItem]) -> Vec<String> {
        items
            .iter()
            .map(|i| match i {
                PageItem::Page(p) => (p + 1).to_string(),
                PageItem::Ellipsis => "…".to_owned(),
            })
            .collect()
    }

    #[test]
    fn page_count_rounds_up_and_is_at_least_one() {
        assert_eq!(page_count(0, 25), 1);
        assert_eq!(page_count(1, 25), 1);
        assert_eq!(page_count(25, 25), 1);
        assert_eq!(page_count(26, 25), 2);
        assert_eq!(page_count(1_000, 0), 1_000);
    }

    #[test]
    fn small_page_counts_are_listed_in_full() {
        assert_eq!(pages_of(&page_items(0, 1)), ["1"]);
        assert_eq!(
            pages_of(&page_items(3, 7)),
            ["1", "2", "3", "4", "5", "6", "7"]
        );
    }

    #[test]
    fn first_page_shows_leading_pages_then_gap_then_last() {
        assert_eq!(pages_of(&page_items(0, 50)), ["1", "2", "…", "50"]);
    }

    #[test]
    fn middle_page_shows_neighbours_between_two_gaps() {
        assert_eq!(
            pages_of(&page_items(24, 50)),
            ["1", "…", "24", "25", "26", "…", "50"]
        );
    }

    #[test]
    fn single_hidden_page_is_filled_instead_of_an_ellipsis() {
        // current = page 3 (idx 2): neighbours 2,3,4; gap to last is large,
        // gap between 1 and 2 is none.
        assert_eq!(
            pages_of(&page_items(2, 50)),
            ["1", "2", "3", "4", "…", "50"]
        );
        // current = page 4 (idx 3): page 2 is the single hidden page.
        assert_eq!(
            pages_of(&page_items(3, 50)),
            ["1", "2", "3", "4", "5", "…", "50"]
        );
    }

    #[test]
    fn last_page_shows_trailing_pages() {
        assert_eq!(pages_of(&page_items(49, 50)), ["1", "…", "49", "50"]);
    }

    #[test]
    fn out_of_range_page_is_clamped() {
        assert_eq!(page_items(999, 10), page_items(9, 10));
    }

    #[test]
    fn thousands_are_grouped() {
        assert_eq!(group_thousands(0), "0");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(group_thousands(1_000), "1\u{a0}000");
        assert_eq!(group_thousands(1_234_567), "1\u{a0}234\u{a0}567");
    }

    #[test]
    fn labels_fill_placeholders() {
        let l = FooterLabels::default();
        assert_eq!(l.rows_text(1_250), "1\u{a0}250 rows");
        assert_eq!(l.selected_text(3), "3 selected");
        assert_eq!(l.range_text(1, 25, 1_250), "1\u{2013}25 of 1\u{a0}250");
    }
}
