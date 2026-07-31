// ghostty's half of the `snapshot` comparison; see workloads_ghostty.rs for
// why ghostty benches live in their own binary.
//
// Run with `cargo bench --bench snapshot_ghostty`.

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use libghostty_vt::fmt::{Format, Formatter, FormatterOptions};
use rio_vt_benchmark::{corpus, ghostty};

fn bench_process(c: &mut Criterion) {
    let data = corpus::mixed();
    let mut group = c.benchmark_group("process");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("ghostty", |b| {
        b.iter_batched(
            || ghostty::new(0),
            |mut term| term.vt_write(&data),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let data = corpus::mixed();

    let mut gt = ghostty::new(0);
    gt.vt_write(&data);
    let mut gt_ansi =
        Formatter::new(&gt, FormatterOptions::new().with_format(Format::Vt))
            .expect("ghostty formatter");
    let mut gt_plain =
        Formatter::new(&gt, FormatterOptions::new().with_format(Format::Plain))
            .expect("ghostty formatter");

    let mut ansi = c.benchmark_group("contents_formatted");
    ansi.bench_function("ghostty", |b| {
        b.iter(|| black_box(gt_ansi.format_alloc(None).expect("format")))
    });
    ansi.finish();

    let mut plain = c.benchmark_group("contents_plain");
    plain.bench_function("ghostty", |b| {
        b.iter(|| black_box(gt_plain.format_alloc(None).expect("format")))
    });
    plain.finish();
}

criterion_group!(benches, bench_process, bench_serialize);
criterion_main!(benches);
