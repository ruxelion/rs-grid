//! Persistable column-layout snapshot for the examples.
//!
//! Captures user-adjusted column widths, order, and pinned count so a demo
//! can restore them across reloads. The serialised JSON shape is a 3-tuple
//! `[[[key, width], ...], [key, ...], pinned]`, matching the format that was
//! previously inlined in the Leptos example — existing stored layouts stay
//! compatible.
//!
//! This module is intentionally web-agnostic (no `web-sys`): it only knows how
//! to (de)serialise a snapshot and apply it to a [`GridModel`]. Reading and
//! writing browser `localStorage` lives in `rs_grid_web::storage`.

use std::collections::HashMap;

use rs_grid_core::{column::ColumnOffsets, model::GridModel};
use serde::{Deserialize, Serialize};

/// A snapshot of column layout: `(widths_by_key, order_by_key, pinned_count)`.
///
/// Serialises as a JSON array of three elements to stay compatible with
/// layouts persisted by earlier demo versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSnapshot(
    /// Per-column widths, keyed by `ColumnDef.key`.
    pub Vec<(String, f64)>,
    /// Column order, as a list of `ColumnDef.key`.
    pub Vec<String>,
    /// Number of pinned (frozen) leading columns.
    pub usize,
);

impl LayoutSnapshot {
    /// Build a snapshot from raw layout parts — typically the values returned
    /// by `GridCanvas::column_widths()` / `column_order()` / `pinned_count()`.
    pub fn new(
        widths: Vec<(String, f64)>,
        order: Vec<String>,
        pinned: usize,
    ) -> Self {
        Self(widths, order, pinned)
    }

    /// Deserialise a snapshot from a JSON string. Returns `None` if the
    /// payload is missing or malformed.
    pub fn from_json(raw: &str) -> Option<Self> {
        serde_json::from_str(raw).ok()
    }

    /// Serialise the snapshot to a JSON string. Returns `None` if
    /// serialisation fails.
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    /// Apply this snapshot to a freshly built [`GridModel`]: restore column
    /// widths and order, clamp the pinned count, and recompute the column
    /// offsets so hit-testing stays in sync.
    pub fn apply(&self, model: &mut GridModel) {
        let LayoutSnapshot(widths, order, pinned) = self;

        let width_map: HashMap<&str, f64> =
            widths.iter().map(|(k, w)| (k.as_str(), *w)).collect();
        for col in model.columns.iter_mut() {
            if let Some(w) = width_map.get(col.key.as_str()) {
                col.width = *w;
            }
        }

        let order_idx: HashMap<&str, usize> = order
            .iter()
            .enumerate()
            .map(|(i, k)| (k.as_str(), i))
            .collect();
        model.columns.sort_by_key(|c| {
            order_idx.get(c.key.as_str()).copied().unwrap_or(usize::MAX)
        });

        model.pinned_count = (*pinned).min(model.columns.len());

        // Hit-testing reads `column_offsets`; keep it in sync after mutating
        // widths and reordering.
        model.column_offsets = ColumnOffsets::compute(&model.columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_model;

    #[test]
    fn json_round_trip_preserves_snapshot() {
        let snap = LayoutSnapshot::new(
            vec![("name".to_string(), 250.0)],
            vec!["email".to_string(), "name".to_string()],
            2,
        );
        let json = snap.to_json().expect("serialise");
        let back = LayoutSnapshot::from_json(&json).expect("deserialise");
        assert_eq!(snap, back);
    }

    #[test]
    fn json_shape_is_a_three_element_array() {
        // Backward-compat guard: the wire format must stay a 3-tuple so
        // layouts persisted by earlier demo versions still load.
        let snap = LayoutSnapshot::new(vec![], vec![], 0);
        assert_eq!(snap.to_json().unwrap(), "[[],[],0]");
    }

    #[test]
    fn from_json_rejects_garbage() {
        assert!(LayoutSnapshot::from_json("not json").is_none());
    }

    #[test]
    fn apply_restores_width_order_and_pinned() {
        let mut model = build_model(100, 5);
        let snap = LayoutSnapshot::new(
            vec![("email".to_string(), 999.0)],
            vec!["email".to_string()],
            1,
        );
        snap.apply(&mut model);

        // `email` moved to the front and took the persisted width.
        assert_eq!(model.columns[0].key, "email");
        assert_eq!(model.columns[0].width, 999.0);
        assert_eq!(model.pinned_count, 1);
    }

    #[test]
    fn apply_clamps_pinned_to_column_count() {
        let mut model = build_model(100, 3);
        let snap = LayoutSnapshot::new(vec![], vec![], 999);
        snap.apply(&mut model);
        assert_eq!(model.pinned_count, model.columns.len());
    }
}
