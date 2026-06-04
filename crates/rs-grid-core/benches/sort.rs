use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId,
    Criterion,
};
use rs_grid_core::{
    column::ColumnDef, model::GridModel, row::RowRecord, sort::SortDir,
};

fn make_numeric_model(n: usize) -> GridModel {
    let cols = vec![ColumnDef::new("v", "Value", 100.0)];
    let rows: Vec<RowRecord> = (0..n as u64)
        .map(|i| {
            let mut r = RowRecord::new(i);
            r.set("v", (n as u64 - i).to_string());
            r
        })
        .collect();
    GridModel::new(cols, rows, 30.0, 40.0)
}

fn make_string_model(n: usize) -> GridModel {
    let cols = vec![ColumnDef::new("s", "Name", 150.0)];
    let rows: Vec<RowRecord> = (0..n as u64)
        .map(|i| {
            let mut r = RowRecord::new(i);
            // Zero-padded so the lexicographic order is not
            // the same as insertion order.
            r.set("s", format!("value_{:010}", n as u64 - i));
            r
        })
        .collect();
    GridModel::new(cols, rows, 30.0, 40.0)
}

fn make_filter_model(n: usize) -> GridModel {
    let cols = vec![ColumnDef::new("v", "Value", 100.0)];
    let rows: Vec<RowRecord> = (0..n as u64)
        .map(|i| {
            let mut r = RowRecord::new(i);
            r.set("v", format!("item_{i}"));
            r
        })
        .collect();
    GridModel::new(cols, rows, 30.0, 40.0)
}

// ── Sort: numeric (radix sort, cache miss) ───────────────────────────────────

fn bench_sort_numeric_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort/numeric/cold");
    for n_rows in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter_batched(
                    || make_numeric_model(n),
                    |mut m| {
                        m.apply_sort("v", &SortDir::Asc);
                        black_box(m.sort_order.len())
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }
    group.finish();
}

// ── Sort: numeric (radix sort, cache hit — direction toggle) ─────────────────

fn bench_sort_numeric_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort/numeric/cached");
    for n_rows in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter_batched(
                    || {
                        let mut m = make_numeric_model(n);
                        // Warm the cache with an initial sort.
                        m.apply_sort("v", &SortDir::Asc);
                        m
                    },
                    |mut m| {
                        // Toggle direction: hits the pre-computed key cache.
                        m.apply_sort("v", &SortDir::Desc);
                        black_box(m.sort_order.len())
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }
    group.finish();
}

// ── Sort: string (comparison sort, cache miss) ───────────────────────────────

fn bench_sort_string_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort/string/cold");
    for n_rows in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter_batched(
                    || make_string_model(n),
                    |mut m| {
                        m.apply_sort("s", &SortDir::Asc);
                        black_box(m.sort_order.len())
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }
    group.finish();
}

// ── Sort: string (comparison sort, cache hit) ────────────────────────────────

fn bench_sort_string_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort/string/cached");
    for n_rows in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter_batched(
                    || {
                        let mut m = make_string_model(n);
                        m.apply_sort("s", &SortDir::Asc);
                        m
                    },
                    |mut m| {
                        m.apply_sort("s", &SortDir::Desc);
                        black_box(m.sort_order.len())
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }
    group.finish();
}

// ── Filter ───────────────────────────────────────────────────────────────────

fn bench_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");
    for n_rows in [1_000usize, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(n_rows),
            &n_rows,
            |b, &n| {
                b.iter_batched(
                    || {
                        let mut m = make_filter_model(n);
                        // "item_5" matches item_5, item_50..59, item_500..599…
                        m.filters.insert("v".into(), "item_5".into());
                        m
                    },
                    |mut m| {
                        m.apply_filter();
                        black_box(m.filtered_indices.len())
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sort_numeric_cold,
    bench_sort_numeric_cached,
    bench_sort_string_cold,
    bench_sort_string_cached,
    bench_filter,
);
criterion_main!(benches);
