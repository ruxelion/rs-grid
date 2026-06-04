use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rs_grid_core::{
    column::ColumnDef,
    datasource::FnDataSource,
    model::GridModel,
    state::GridState,
};

fn make_cols(n: usize) -> Vec<ColumnDef> {
    (0..n)
        .map(|i| ColumnDef::new(&format!("c{i}"), &format!("Col {i}"), 120.0))
        .collect()
}

// ── init/rows ────────────────────────────────────────────────────────────────
//
// Varies the row count while keeping the column count fixed (20 cols).
// Uses FnDataSource so row count has zero allocation cost.
//
// Expected result: nearly flat — GridState::new is O(n_cols), not O(n_rows).
// This is the "10 M rows initialises in the same time as 1 k rows" story.

fn bench_init_by_rows(c: &mut Criterion) {
    let mut group = c.benchmark_group("init/rows");
    for n_rows in [
        1_000u64,
        100_000,
        1_000_000,
        100_000_000,
        1_000_000_000,           // 1 milliard
        1_000_000_000_000_000,   // 1 quadrillion
    ] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter(|| {
                    let cols = make_cols(20);
                    let data =
                        Box::new(FnDataSource::new(n, |_, _| None));
                    let model = GridModel::with_data_source(
                        cols, data, 30.0, 40.0,
                    );
                    black_box(GridState::new(model, 1_200.0, 800.0))
                })
            },
        );
    }
    group.finish();
}

// ── init/cols ────────────────────────────────────────────────────────────────
//
// Varies the column count while keeping row count at 1 M.
// Reveals the real cost driver: offset precomputation is O(n_cols).

fn bench_init_by_cols(c: &mut Criterion) {
    let mut group = c.benchmark_group("init/cols");
    for n_cols in [5usize, 20, 50, 100, 1_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_cols),
            &n_cols,
            |b, &n| {
                b.iter(|| {
                    let cols = make_cols(n);
                    let data = Box::new(FnDataSource::new(
                        1_000_000,
                        |_, _| None,
                    ));
                    let model = GridModel::with_data_source(
                        cols, data, 30.0, 40.0,
                    );
                    black_box(GridState::new(model, 1_200.0, 800.0))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_init_by_rows, bench_init_by_cols);
criterion_main!(benches);
