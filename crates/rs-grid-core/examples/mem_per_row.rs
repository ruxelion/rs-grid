//! Mesure l'empreinte mémoire réelle par ligne, par type de datasource.
//!
//! Utilise un allocateur global de suivi pour compter les octets alloués
//! sur le tas avant et après chaque construction.
//!
//! ```sh
//! cargo run -p rs-grid-core --example mem_per_row --release
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

// ── Allocateur de suivi ───────────────────────────────────────────────────────

static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);

struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            LIVE_BYTES
                .fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        LIVE_BYTES
            .fetch_sub(layout.size() as isize, Ordering::Relaxed);
    }
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            let delta = new_size as isize - layout.size() as isize;
            LIVE_BYTES.fetch_add(delta, Ordering::Relaxed);
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOC: TrackingAllocator = TrackingAllocator;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn live() -> isize {
    LIVE_BYTES.load(Ordering::Relaxed)
}

use rs_grid_core::{
    column::ColumnDef,
    datasource::FnDataSource,
    model::GridModel,
    row::RowRecord,
    state::GridState,
};

fn make_cols(n: usize) -> Vec<ColumnDef> {
    (0..n)
        .map(|i| ColumnDef::new(
            &format!("col_{i:02}"),
            &format!("Col {i}"),
            100.0,
        ))
        .collect()
}

fn section(title: &str) {
    println!("\n── {title} ──");
}

fn report(label: &str, bytes: isize, n: usize) {
    let per_row = bytes as f64 / n as f64;
    println!("  {label}");
    println!("    total : {:>10} bytes ({:.1} KB)", bytes, bytes as f64 / 1024.0);
    println!("    / row : {:>10.0} bytes", per_row);
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== rs-grid — empreinte mémoire par ligne ===");
    println!("    (mesures en mémoire live après construction)\n");

    const N: usize = 10_000;
    const N_COLS: usize = 10;

    // ── 1. FnDataSource — aucune allocation par ligne ─────────────────────────
    section("FnDataSource (virtuel / server-side)");
    {
        let before = live();
        let cols = make_cols(N_COLS);
        let data = Box::new(FnDataSource::new(N as u64, |_, _| None));
        let model =
            GridModel::with_data_source(cols, data, 30.0, 40.0);
        let state = GridState::new(model, 1_200.0, 800.0);
        let total = live() - before;
        println!("  GridState ({N} lignes, {N_COLS} cols, FnDataSource)");
        println!("    total : {:>10} bytes ({:.1} KB)", total, total as f64 / 1024.0);
        println!("    / row :          ~0 bytes  (données générées à la demande)");
        let _ = state; // keep alive until after measurement
    }

    // ── 2. VecDataSource — lignes vides (pas de valeurs de cellule) ───────────
    section("VecDataSource — lignes vides (RowRecord sans valeurs)");
    {
        let before = live();
        let rows: Vec<RowRecord> =
            (0..N as u64).map(RowRecord::new).collect();
        let delta = live() - before;
        report(&format!("{N} RowRecord::new(i)"), delta, N);
        drop(rows);
    }

    // ── 3. VecDataSource — 10 cols, valeurs courtes (~8 chars) ───────────────
    section("VecDataSource — 10 cols, valeurs ~8 chars (\"val_00001\")");
    {
        let before = live();
        let rows: Vec<RowRecord> = (0..N as u64)
            .map(|i| {
                let mut r = RowRecord::new(i);
                for j in 0..N_COLS {
                    r.set(format!("col_{j:02}"), format!("val_{i:05}"));
                }
                r
            })
            .collect();
        let delta = live() - before;
        report(&format!("{N} lignes × {N_COLS} cols"), delta, N);
        drop(rows);
    }

    // ── 4. VecDataSource — 10 cols, valeurs longues (~40 chars) ──────────────
    section("VecDataSource — 10 cols, valeurs ~40 chars");
    {
        let before = live();
        let rows: Vec<RowRecord> = (0..N as u64)
            .map(|i| {
                let mut r = RowRecord::new(i);
                for j in 0..N_COLS {
                    // 40-char value — triggers String heap allocation
                    r.set(
                        format!("col_{j:02}"),
                        format!("value_row_{i:06}_col_{j:02}_padding_____"),
                    );
                }
                r
            })
            .collect();
        let delta = live() - before;
        report(&format!("{N} lignes × {N_COLS} cols"), delta, N);
        drop(rows);
    }

    // ── 5. GridState complet — 10 cols, valeurs ~8 chars ─────────────────────
    section("GridState complet (modèle + état) — 10 cols, valeurs ~8 chars");
    {
        let before = live();
        let cols = make_cols(N_COLS);
        let rows: Vec<RowRecord> = (0..N as u64)
            .map(|i| {
                let mut r = RowRecord::new(i);
                for j in 0..N_COLS {
                    r.set(format!("col_{j:02}"), format!("val_{i:05}"));
                }
                r
            })
            .collect();
        let model = GridModel::new(cols, rows, 30.0, 40.0);
        let state = GridState::new(model, 1_200.0, 800.0);
        let total = live() - before;
        report(
            &format!("GridState ({N} lignes × {N_COLS} cols)"),
            total,
            N,
        );
        let _ = state;
    }

    // ── 6. Tailles stack des types clés ───────────────────────────────────────
    section("std::mem::size_of (taille sur la pile)");
    use std::mem::size_of;
    for (name, size) in [
        ("RowRecord ", size_of::<RowRecord>()),
        ("GridModel ", size_of::<GridModel>()),
        ("GridState ", size_of::<GridState>()),
        ("ColumnDef ", size_of::<ColumnDef>()),
    ] {
        println!("  {name}: {size} bytes");
    }
}
