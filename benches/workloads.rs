// The same `process` operation (parse a byte stream into the screen) across
// several input shapes, so the comparison is not tied to one corpus:
//
//   ascii_plain        plain text, no escape sequences
//   sgr_churn          a color change before almost every character
//   scroll_storm       short lines, mostly newlines
//   alt_screen_redraw  repeated full-screen repaints (TUI style)
//   unicode_wide       CJK text and emoji
//
// Run with `cargo bench --bench workloads`.

use criterion::{
    criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use rio_vt_benchmark::{alacritty, corpus, new_rio, new_vt100};

fn parse_workload(c: &mut Criterion, name: &str, data: Vec<u8>) {
    let mut group = c.benchmark_group(name);
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

fn benches(c: &mut Criterion) {
    parse_workload(c, "ascii_plain", corpus::ascii_plain());
    parse_workload(c, "sgr_churn", corpus::sgr_churn());
    parse_workload(c, "scroll_storm", corpus::scroll_storm());
    parse_workload(c, "alt_screen_redraw", corpus::alt_screen_redraw());
    parse_workload(c, "unicode_wide", corpus::unicode_wide());
}

criterion_group!(workloads, benches);
criterion_main!(workloads);
