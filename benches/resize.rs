// Resizing a filled screen. The engines reflow their grid on a size change;
// this fills an 80x24 screen with the mixed corpus, then resizes out to
// 100x40 and back to 80x24 per iteration.
//
// Run with `cargo bench --bench resize`.

use criterion::{
    criterion_group, criterion_main, BatchSize, Criterion,
};
use rio_vt_benchmark::{
    alacritty, corpus, new_rio, new_rio_scrollback, new_vt100, rio_dims, COLS, ROWS,
};

fn bench_resize(c: &mut Criterion) {
    let data = corpus::mixed();
    let mut group = c.benchmark_group("resize");

    group.bench_function("rio-vt", |b| {
        b.iter_batched(
            || {
                let (mut term, mut parser) = new_rio();
                parser.advance(&mut term, &data);
                term
            },
            |mut term| {
                term.resize(rio_dims(100, 40));
                term.resize(rio_dims(80, 24));
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("vt100", |b| {
        b.iter_batched(
            || {
                let mut parser = new_vt100();
                parser.process(&data);
                parser
            },
            |mut parser| {
                parser.set_size(40, 100);
                parser.set_size(24, 80);
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("alacritty", |b| {
        b.iter_batched(
            || {
                let (mut term, mut parser) = alacritty::new(0);
                parser.advance(&mut term, &data);
                term
            },
            |mut term| {
                term.resize(alacritty::size(100, 40));
                term.resize(alacritty::size(80, 24));
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

// Resize with soft-wrapped scrollback, which forces the reflow path: wrapped
// lines rejoin on grow and re-split on shrink across the whole history.
fn bench_resize_reflow(c: &mut Criterion) {
    let data = corpus::wrapped();
    let scrollback = 10_000;
    let mut group = c.benchmark_group("resize_reflow");

    group.bench_function("rio-vt", |b| {
        b.iter_batched(
            || {
                let (mut term, mut parser) = new_rio_scrollback(scrollback);
                parser.advance(&mut term, &data);
                term
            },
            |mut term| {
                term.resize(rio_dims(100, 40));
                term.resize(rio_dims(80, 24));
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("vt100", |b| {
        b.iter_batched(
            || {
                let mut parser = vt100::Parser::new(ROWS, COLS, scrollback);
                parser.process(&data);
                parser
            },
            |mut parser| {
                parser.set_size(40, 100);
                parser.set_size(24, 80);
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("alacritty", |b| {
        b.iter_batched(
            || {
                let (mut term, mut parser) = alacritty::new(scrollback);
                parser.advance(&mut term, &data);
                term
            },
            |mut term| {
                term.resize(alacritty::size(100, 40));
                term.resize(alacritty::size(80, 24));
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_resize, bench_resize_reflow);
criterion_main!(benches);
