use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use rs_grid_core::{
    column::ColumnDef, datasource::FnDataSource, hit_test, model::GridModel,
    row::RowRecord,
};

fn make_model(n_cols: usize, n_rows: usize) -> GridModel {
    let cols: Vec<ColumnDef> = (0..n_cols)
        .map(|i| ColumnDef::new(format!("c{i}"), format!("C{i}"), 100.0))
        .collect();
    let rows: Vec<RowRecord> = (0..n_rows as u64).map(RowRecord::new).collect();
    GridModel::new(cols, rows, 30.0, 40.0)
}

fn make_model_fn(n_cols: usize, n_rows: u64) -> GridModel {
    let cols: Vec<ColumnDef> = (0..n_cols)
        .map(|i| ColumnDef::new(format!("c{i}"), format!("C{i}"), 100.0))
        .collect();
    let data = Box::new(FnDataSource::new(n_rows, |_, _| None));
    GridModel::with_data_source(cols, data, 30.0, 40.0)
}

fn bench_hit_test_cell(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_test/cell");
    for n_cols in [10usize, 100, 1000] {
        let model = make_model(n_cols, 1000);
        let rnw = model.effective_row_number_width();
        // Click in the middle of the visible data area.
        let vx = rnw + model.total_width() / 2.0;
        let vy = model.effective_header_height() + 15.0;
        group.bench_with_input(
            BenchmarkId::from_parameter(n_cols),
            &n_cols,
            |b, _| {
                b.iter(|| {
                    black_box(hit_test::hit_test(
                        black_box(vx),
                        black_box(vy),
                        &model,
                        0.0,
                        0.0,
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_hit_test_col_header(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_test/col_header");
    for n_cols in [10usize, 100, 1000] {
        let model = make_model(n_cols, 1000);
        let rnw = model.effective_row_number_width();
        let vx = rnw + model.total_width() / 2.0;
        group.bench_with_input(
            BenchmarkId::from_parameter(n_cols),
            &n_cols,
            |b, _| {
                b.iter(|| {
                    black_box(hit_test::hit_test_col_header(
                        black_box(vx),
                        20.0,
                        &model,
                        0.0,
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_hit_test_with_scroll(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_test/with_scroll");
    for n_cols in [10usize, 100, 1000] {
        let model = make_model(n_cols, 1000);
        let rnw = model.effective_row_number_width();
        let vx = rnw + 400.0;
        let vy = model.effective_header_height() + 150.0;
        // Simulate mid-scroll position.
        let scroll_x = model.total_width() / 4.0;
        let scroll_y = model.total_height() / 4.0;
        group.bench_with_input(
            BenchmarkId::from_parameter(n_cols),
            &n_cols,
            |b, _| {
                b.iter(|| {
                    black_box(hit_test::hit_test(
                        black_box(vx),
                        black_box(vy),
                        &model,
                        black_box(scroll_x),
                        black_box(scroll_y),
                    ))
                })
            },
        );
    }
    group.finish();
}

// ── hit_test/extreme ─────────────────────────────────────────────────────────
//
// Fixed 1 000 cols, row counts from 1 k to 1 quadrillion, mid-scroll position.
// Expected: flat — hit-test is O(log n_cols), not O(n_rows).
// Also exercises the precision-preserving row arithmetic at extreme scroll_y.

fn bench_hit_test_extreme(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_test/extreme");
    for n_rows in [
        1_000u64,
        1_000_000_000,         // 1 milliard
        1_000_000_000_000_000, // 1 quadrillion
    ] {
        let model = make_model_fn(1_000, n_rows);
        let rnw = model.effective_row_number_width();
        let vx = rnw + 400.0; // visible viewport position
        let vy = model.effective_header_height() + 15.0;
        // Mid-scroll: exercises both horizontal binary search and the
        // precision-preserving vertical decomposition at large scroll_y.
        let scroll_x = model.total_width() / 2.0;
        let scroll_y = (n_rows / 2) as f64 * 30.0;
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, _| {
                b.iter(|| {
                    black_box(hit_test::hit_test(
                        black_box(vx),
                        black_box(vy),
                        &model,
                        black_box(scroll_x),
                        black_box(scroll_y),
                    ))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_hit_test_cell,
    bench_hit_test_col_header,
    bench_hit_test_with_scroll,
    bench_hit_test_extreme,
);
criterion_main!(benches);
