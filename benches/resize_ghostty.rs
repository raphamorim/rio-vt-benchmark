// ghostty's half of the `resize` comparison; see workloads_ghostty.rs for
// why ghostty benches live in their own binary.
//
// Run with `cargo bench --bench resize_ghostty`.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rio_vt_benchmark::{corpus, ghostty};

fn bench_resize(c: &mut Criterion) {
    let data = corpus::mixed();
    let mut group = c.benchmark_group("resize");

    group.bench_function("ghostty", |b| {
        b.iter_batched(
            || {
                let mut term = ghostty::new(0);
                term.vt_write(&data);
                term
            },
            |mut term| {
                term.resize(100, 40, 8, 16).expect("resize");
                term.resize(80, 24, 8, 16).expect("resize");
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_resize_reflow(c: &mut Criterion) {
    let data = corpus::wrapped();
    let scrollback = 10_000;
    let mut group = c.benchmark_group("resize_reflow");

    group.bench_function("ghostty", |b| {
        b.iter_batched(
            || {
                let mut term = ghostty::new(scrollback);
                term.vt_write(&data);
                term
            },
            |mut term| {
                term.resize(100, 40, 8, 16).expect("resize");
                term.resize(80, 24, 8, 16).expect("resize");
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_resize, bench_resize_reflow);
criterion_main!(benches);
