use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rs_grid_core::{
    column::ColumnDef,
    commands::GridCommand,
    datasource::FnDataSource,
    model::GridModel,
    state::GridState,
};
use rs_grid_scene::builder::SceneBuilder;

struct Config {
    label: &'static str,
    n_cols: usize,
    n_rows: u64,
    col_width: f64,
}

// Each row count tests the same visible area (~11 cols × ~25 rows).
// Expected result: frame time is flat across row counts — proves the
// renderer is O(visible_cells), not O(total_rows).
const CONFIGS: &[Config] = &[
    Config {
        label: "20cols_10k_rows",
        n_cols: 20,
        n_rows: 10_000,
        col_width: 120.0,
    },
    Config {
        label: "50cols_1M_rows",
        n_cols: 50,
        n_rows: 1_000_000,
        col_width: 100.0,
    },
    Config {
        label: "100cols_10M_rows",
        n_cols: 100,
        n_rows: 10_000_000,
        col_width: 80.0,
    },
    Config {
        label: "1000cols_1B_rows",
        n_cols: 1_000,
        n_rows: 1_000_000_000,
        col_width: 80.0,
    },
    Config {
        label: "50cols_1Q_rows",
        n_cols: 50,
        n_rows: 1_000_000_000_000_000, // 1 quadrillion
        col_width: 100.0,
    },
];

fn make_state(cfg: &Config, vp_width: f64, vp_height: f64) -> GridState {
    let cols: Vec<ColumnDef> = (0..cfg.n_cols)
        .map(|i| {
            ColumnDef::new(
                &format!("c{i}"),
                &format!("Col {i}"),
                cfg.col_width,
            )
        })
        .collect();
    // FnDataSource: data generated on demand — row count has no memory cost.
    let data = Box::new(FnDataSource::new(cfg.n_rows, |row, _col| {
        Some(format!("r{row}"))
    }));
    let model = GridModel::with_data_source(cols, data, 30.0, 40.0);
    GridState::new(model, vp_width, vp_height)
}

// ── scroll_frame ─────────────────────────────────────────────────────────────
//
// Simulates one animation frame: apply a scroll step then rebuild the scene.
// Measures the complete per-frame pipeline cost:
//   ScrollBy → viewport recalc → SceneBuilder::build()
//
// State accumulates scroll across iterations (scroll_y grows by 3px each
// call). The dataset is large enough that we never reach the bottom, so
// every iteration measures a fresh render in the scrollable zone.

fn bench_scroll_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll_frame");
    for cfg in CONFIGS {
        let mut state = make_state(cfg, 1_200.0, 800.0);
        let builder = SceneBuilder::new(1.0);
        let scroll_step = GridCommand::ScrollBy { dx: 0.0, dy: 3.0 };
        group.bench_function(BenchmarkId::from_parameter(cfg.label), |b| {
            b.iter(|| {
                state.apply(scroll_step.clone());
                black_box(builder.build(&state, None, None, None))
            })
        });
    }
    group.finish();
}

// ── scroll_frame/dpr2 ────────────────────────────────────────────────────────
//
// Same as above at DPR=2 (Retina / HiDPI). The DPR is stored in the frame
// for the renderer to consume — SceneBuilder cost should be identical.

fn bench_scroll_frame_dpr2(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll_frame/dpr2");
    for cfg in CONFIGS {
        let mut state = make_state(cfg, 1_200.0, 800.0);
        let builder = SceneBuilder::new(2.0);
        let scroll_step = GridCommand::ScrollBy { dx: 0.0, dy: 3.0 };
        group.bench_function(BenchmarkId::from_parameter(cfg.label), |b| {
            b.iter(|| {
                state.apply(scroll_step.clone());
                black_box(builder.build(&state, None, None, None))
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scroll_frame, bench_scroll_frame_dpr2);
criterion_main!(benches);
