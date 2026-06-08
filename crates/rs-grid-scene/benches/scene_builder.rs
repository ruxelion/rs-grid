use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rs_grid_core::{
    column::ColumnDef, model::GridModel, row::RowRecord, state::GridState,
};
use rs_grid_scene::builder::SceneBuilder;

struct Config {
    label: &'static str,
    n_cols: usize,
    n_rows: usize,
    col_width: f64,
    vp_width: f64,
    vp_height: f64,
}

// Each config is sized so the number of visible cells differs meaningfully.
// visible_cols ≈ (vp_width - rnw - 14) / col_width
// visible_rows ≈ (vp_height - 40) / 30
const CONFIGS: &[Config] = &[
    Config {
        label: "small",
        n_cols: 20,
        n_rows: 200,
        col_width: 120.0,
        vp_width: 800.0,
        vp_height: 400.0,
        // ~6 cols × ~12 rows = ~72 visible cells
    },
    Config {
        label: "medium",
        n_cols: 50,
        n_rows: 1_000,
        col_width: 100.0,
        vp_width: 1_200.0,
        vp_height: 800.0,
        // ~11 cols × ~25 rows = ~275 visible cells
    },
    Config {
        label: "dense",
        n_cols: 100,
        n_rows: 2_000,
        col_width: 60.0,
        vp_width: 1_920.0,
        vp_height: 1_080.0,
        // ~31 cols × ~34 rows = ~1054 visible cells
    },
];

fn make_state(cfg: &Config) -> GridState {
    let cols: Vec<ColumnDef> = (0..cfg.n_cols)
        .map(|i| {
            ColumnDef::new(format!("c{i}"), format!("Col {i}"), cfg.col_width)
        })
        .collect();
    let rows: Vec<RowRecord> = (0..cfg.n_rows as u64)
        .map(|i| {
            let mut r = RowRecord::new(i);
            for j in 0..cfg.n_cols {
                r.set(format!("c{j}"), format!("r{i}c{j}"));
            }
            r
        })
        .collect();
    let model = GridModel::new(cols, rows, 30.0, 40.0);
    GridState::new(model, cfg.vp_width, cfg.vp_height)
}

fn bench_scene_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene/build");
    for cfg in CONFIGS {
        let state = make_state(cfg);
        let builder = SceneBuilder::new(1.0);
        group.bench_with_input(
            BenchmarkId::from_parameter(cfg.label),
            cfg.label,
            |b, _| {
                b.iter(|| {
                    black_box(builder.build(
                        black_box(&state),
                        None,
                        None,
                        None,
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_scene_build_dpr2(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene/build/dpr2");
    for cfg in CONFIGS {
        let state = make_state(cfg);
        let builder = SceneBuilder::new(2.0);
        group.bench_with_input(
            BenchmarkId::from_parameter(cfg.label),
            cfg.label,
            |b, _| {
                b.iter(|| {
                    black_box(builder.build(
                        black_box(&state),
                        None,
                        None,
                        None,
                    ))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_scene_build, bench_scene_build_dpr2);
criterion_main!(benches);
