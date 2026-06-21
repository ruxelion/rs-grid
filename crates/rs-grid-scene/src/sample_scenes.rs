//! Canonical sample `GridState`s used by the `scene-dump` binary and the
//! golden snapshot tests (and, later, the MCP scene tool). Keeping them in one
//! place means the JSON an agent inspects and the snapshot that guards against
//! regressions are built from the exact same scenarios.
//!
//! `#[doc(hidden)]` — this is demo/inspection scaffolding, not part of the
//! supported API surface.

use rs_grid_core::{
    column::ColumnDef, commands::GridCommand, model::GridModel, row::RowRecord,
    selection::CellCoord, state::GridState,
};

/// Viewport width used by every sample scene.
pub const VP_W: f64 = 800.0;
/// Viewport height used by every sample scene.
pub const VP_H: f64 = 400.0;

/// A small synthetic model: `n_cols` × `n_rows`, cells `"r{row}c{col}"`.
fn sample_model(n_cols: usize, n_rows: u64) -> GridModel {
    let cols: Vec<ColumnDef> = (0..n_cols)
        .map(|i| ColumnDef::new(format!("c{i}"), format!("Col {i}"), 120.0))
        .collect();
    let rows: Vec<RowRecord> = (0..n_rows)
        .map(|r| {
            let mut row = RowRecord::new(r);
            for c in 0..n_cols {
                row.set(format!("c{c}"), format!("r{r}c{c}"));
            }
            row
        })
        .collect();
    GridModel::new(cols, rows, 30.0, 40.0)
}

fn basic() -> GridState {
    GridState::new(sample_model(5, 20), VP_W, VP_H)
}

fn selection() -> GridState {
    let mut s = GridState::new(sample_model(5, 20), VP_W, VP_H);
    let _ = s.apply(GridCommand::SelectCell(CellCoord { row: 2, col: 1 }));
    s
}

fn pinned() -> GridState {
    let mut s = GridState::new(sample_model(6, 20), VP_W, VP_H);
    let _ = s.apply(GridCommand::SetPinnedColumnCount { count: 2 });
    s
}

fn scrolled() -> GridState {
    let mut s = GridState::new(sample_model(10, 200), VP_W, VP_H);
    let _ = s.apply(GridCommand::ScrollTo { x: 300.0, y: 600.0 });
    s
}

/// A named scenario: a label and a builder of its `GridState`.
pub type Scenario = (&'static str, fn() -> GridState);

/// All sample scenarios, in display order.
pub const SCENARIOS: &[Scenario] = &[
    ("basic", basic),
    ("selection", selection),
    ("pinned", pinned),
    ("scrolled", scrolled),
];

/// Build the named scenario's `GridState`, or `None` if the name is unknown.
pub fn build(name: &str) -> Option<GridState> {
    SCENARIOS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, build)| build())
}
