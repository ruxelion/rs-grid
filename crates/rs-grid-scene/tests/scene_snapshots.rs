//! Golden snapshots of the entire `SceneFrame` for each sample scenario.
//!
//! The per-module unit tests check individual primitives; these guard the
//! *whole* rendered frame (every primitive's geometry, color, clip, draw
//! order) against accidental regressions — the gap the plan calls out. Uses
//! `Debug`, so no `serde` feature is required and they run in the default
//! `cargo nextest` gate.
//!
//! After an intentional rendering change, review and update with:
//!   cargo insta review          (needs cargo-insta)
//!   # or: INSTA_UPDATE=always cargo test -p rs-grid-scene --test
//! scene_snapshots

use rs_grid_scene::{builder::SceneBuilder, sample_scenes::SCENARIOS};

#[test]
fn scene_frames_match_snapshots() {
    for (name, build) in SCENARIOS {
        let state = build();
        let frame = SceneBuilder::new(1.0).build(&state, None, None, None);
        insta::assert_debug_snapshot!(*name, frame);
    }
}
