//! Footer bar and pagination rendering.

use rs_grid_core::{
    column::ColumnDef, commands::GridCommand, model::GridModel, row::RowRecord,
    selection::CellCoord, state::GridState,
};
use rs_grid_scene::{
    Theme,
    builder::SceneBuilder,
    frame::{FooterAction, SceneFrame},
    primitives::ScenePrimitive,
};

fn state(rows: u64) -> GridState {
    let cols = vec![ColumnDef::new("a", "Alpha", 100.0)];
    let data: Vec<RowRecord> = (0..rows)
        .map(|i| {
            let mut r = RowRecord::new(i);
            r.set("a", format!("a{i}"));
            r
        })
        .collect();
    GridState::new(GridModel::new(cols, data, 30.0, 40.0), 900.0, 600.0)
}

fn paged(rows: u64, size: u64) -> GridState {
    let mut s = state(rows);
    s.apply(GridCommand::SetShowFooter(true));
    s.apply(GridCommand::SetPaginationEnabled(true));
    s.apply(GridCommand::SetPageSize(size));
    s
}

fn build(s: &GridState) -> SceneFrame {
    SceneBuilder::with_theme(1.0, Theme::light())
        .build(s, None, None, None, None)
}

fn texts(f: &SceneFrame) -> Vec<String> {
    f.primitives
        .iter()
        .filter_map(|p| match p {
            ScenePrimitive::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn no_footer_means_no_zones_and_body_sized_frame() {
    let f = build(&state(10));
    assert!(f.footer_zones.is_empty());
    assert_eq!(f.viewport_height, 600.0);
}

#[test]
fn footer_grows_the_frame_by_its_height() {
    let mut s = state(10);
    s.apply(GridCommand::SetShowFooter(true));
    let f = build(&s);
    assert_eq!(f.viewport_height, 600.0);
    assert_eq!(s.viewport.height, 600.0 - s.model.footer_height);
}

#[test]
fn footer_shows_total_and_selected_counts() {
    let mut s = state(1_250);
    s.apply(GridCommand::SetShowFooter(true));
    s.apply(GridCommand::SelectCell(CellCoord { row: 3, col: 0 }));
    s.apply(GridCommand::ExtendSelection(CellCoord { row: 5, col: 0 }));
    let t = texts(&build(&s));
    assert!(t.iter().any(|x| x == "1\u{a0}250 rows"), "{t:?}");
    assert!(t.iter().any(|x| x == "3 selected"), "{t:?}");
}

#[test]
fn counts_only_without_pagination_has_no_controls() {
    let mut s = state(100);
    s.apply(GridCommand::SetShowFooter(true));
    assert!(build(&s).footer_zones.is_empty());
}

#[test]
fn first_page_disables_prev_and_first() {
    let f = build(&paged(100, 25));
    let acts: Vec<FooterAction> =
        f.footer_zones.iter().map(|z| z.action).collect();
    assert!(acts.contains(&FooterAction::PageSizeMenu));
    assert!(acts.contains(&FooterAction::NextPage));
    assert!(acts.contains(&FooterAction::LastPage));
    assert!(acts.contains(&FooterAction::GoToPage(1)));
    assert!(!acts.contains(&FooterAction::PrevPage));
    assert!(!acts.contains(&FooterAction::FirstPage));
    // The current page is not a click target.
    assert!(!acts.contains(&FooterAction::GoToPage(0)));
}

#[test]
fn last_page_disables_next_and_last() {
    let mut s = paged(100, 25);
    s.apply(GridCommand::SetPage(3));
    let f = build(&s);
    let acts: Vec<FooterAction> =
        f.footer_zones.iter().map(|z| z.action).collect();
    assert!(acts.contains(&FooterAction::PrevPage));
    assert!(acts.contains(&FooterAction::FirstPage));
    assert!(!acts.contains(&FooterAction::NextPage));
    assert!(!acts.contains(&FooterAction::LastPage));
}

#[test]
fn footer_zones_lie_inside_the_footer_band() {
    let s = paged(100, 25);
    let f = build(&s);
    let top = s.viewport.height;
    for z in &f.footer_zones {
        assert!(z.y >= top, "{z:?}");
        assert!(z.y + z.height <= top + s.model.footer_height, "{z:?}");
        assert!(z.x >= 0.0 && z.x + z.width <= 900.0, "{z:?}");
    }
    let hit = f
        .footer_zones
        .iter()
        .find(|z| z.action == FooterAction::NextPage);
    let z = hit.expect("next zone");
    let found = f.hit_footer(z.x + 1.0, z.y + 1.0).map(|z| z.action);
    assert_eq!(found, Some(FooterAction::NextPage));
    // A point in the grid body hits nothing.
    assert!(f.hit_footer(z.x + 1.0, 10.0).is_none());
}

#[test]
fn range_label_reflects_the_current_page() {
    let mut s = paged(60, 25);
    s.apply(GridCommand::SetPage(2));
    let t = texts(&build(&s));
    assert!(t.iter().any(|x| x == "51\u{2013}60 of 60"), "{t:?}");
}

#[test]
fn paginated_body_draws_only_the_current_page_rows() {
    let mut s = paged(100, 25);
    s.apply(GridCommand::SetPage(1));
    let t = texts(&build(&s));
    // Row 25 (first of page 2) is drawn, row 24 (last of page 1) isn't.
    assert!(t.iter().any(|x| x == "a25"), "{t:?}");
    assert!(!t.iter().any(|x| x == "a24"), "{t:?}");
    // Row numbers stay global.
    assert!(t.iter().any(|x| x == "26"), "{t:?}");
}

#[test]
fn page_strip_uses_ellipsis_on_many_pages() {
    let mut s = paged(1_000, 10); // 100 pages
    s.apply(GridCommand::SetPage(49));
    let t = texts(&build(&s));
    assert!(t.iter().any(|x| x == "\u{2026}"), "{t:?}");
    assert!(t.iter().any(|x| x == "50"), "{t:?}");
    assert!(t.iter().any(|x| x == "100"), "{t:?}");
}
