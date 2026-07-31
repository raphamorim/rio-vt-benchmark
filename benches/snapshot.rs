// Parsing and screen serialization on the mixed corpus:
//
//   process              feed the byte stream through the parser
//   contents_formatted   serialize the visible screen back to ANSI
//   contents_plain       extract the visible screen as plain text
//
// Run with `cargo bench`.

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use rio_vt::crosswords::formatter::FormatOptions;
use rio_vt_benchmark::{alacritty, corpus, new_rio, new_vt100};

fn bench_process(c: &mut Criterion) {
    let data = corpus::mixed();
    let mut group = c.benchmark_group("process");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("rio-vt", |b| {
        b.iter_batched(
            new_rio,
            |(mut term, mut parser)| parser.advance(&mut term, &data),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("vt100", |b| {
        b.iter_batched(
            new_vt100,
            |mut parser| parser.process(&data),
            BatchSize::SmallInput,
        )
    });
    group.bench_function("alacritty", |b| {
        b.iter_batched(
            || alacritty::new(0),
            |(mut term, mut parser)| parser.advance(&mut term, &data),
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let data = corpus::mixed();

    // Pre-fill the screens once; serialization is read-only.
    let (mut rio_term, mut rio_parser) = new_rio();
    rio_parser.advance(&mut rio_term, &data);
    let mut vt = new_vt100();
    vt.process(&data);

    let mut ansi = c.benchmark_group("contents_formatted");
    ansi.bench_function("rio-vt", |b| {
        b.iter(|| black_box(rio_term.contents_formatted()))
    });
    ansi.bench_function("vt100", |b| {
        b.iter(|| black_box(vt.screen().contents_formatted()))
    });
    ansi.finish();

    let mut plain = c.benchmark_group("contents_plain");
    plain.bench_function("rio-vt", |b| {
        b.iter(|| black_box(rio_term.format(FormatOptions::plain())))
    });
    plain.bench_function("vt100", |b| {
        b.iter(|| black_box(vt.screen().contents()))
    });
    plain.finish();
}

criterion_group!(benches, bench_process, bench_serialize);
criterion_main!(benches);
