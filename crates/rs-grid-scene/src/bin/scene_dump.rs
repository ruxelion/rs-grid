//! `scene-dump` — serialize a `SceneFrame` to JSON for inspection.
//!
//! Gives an AI agent (or a human) a way to see the *rendered* scene — every
//! primitive's position, color, clip, and text — without a browser. Because
//! the scene layer is renderer-agnostic, the JSON is a faithful, inspectable
//! representation of what the canvas would draw.
//!
//! Build with the `serde` feature (enforced via `required-features`):
//!
//! ```sh
//! cargo run -p rs-grid-scene --features serde --bin scene-dump -- <scenario>
//! ```
//!
//! Usage:
//! - `scene-dump` — list the available scenarios on stderr.
//! - `scene-dump <name>` — print that scenario's `SceneFrame` as pretty JSON.

use rs_grid_scene::{
    builder::SceneBuilder,
    sample_scenes::{self, SCENARIOS},
};

fn print_scenarios() {
    eprintln!("usage: scene-dump <scenario>");
    eprintln!("scenarios:");
    for (name, _) in SCENARIOS {
        eprintln!("  {name}");
    }
}

fn main() {
    let Some(name) = std::env::args().nth(1) else {
        print_scenarios();
        std::process::exit(2);
    };
    let Some(state) = sample_scenes::build(&name) else {
        eprintln!("unknown scenario: {name}");
        print_scenarios();
        std::process::exit(2);
    };

    let frame = SceneBuilder::new(1.0).build(&state, None, None, None, None);
    let json =
        serde_json::to_string_pretty(&frame).expect("SceneFrame serializes");
    println!("{json}");
}
