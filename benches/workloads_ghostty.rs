// ghostty's half of the `workloads` comparison, in its own bench binary:
// rio-vt and libghostty-vt both bundle simdutf (different versions, different
// compilers), and linking them into one binary lets the linker mix objects
// from the two copies, which crashes. Criterion merges the groups by name,
// so the report still compares all engines.
//
// Run with `cargo bench --bench workloads_ghostty`.

use criterion::{
    criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use rio_vt_benchmark::{corpus, ghostty};

fn parse_workload(c: &mut Criterion, name: &str, data: Vec<u8>) {
    let mut group = c.benchmark_group(name);
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

fn benches(c: &mut Criterion) {
    parse_workload(c, "ascii_plain", corpus::ascii_plain());
    parse_workload(c, "sgr_churn", corpus::sgr_churn());
    parse_workload(c, "scroll_storm", corpus::scroll_storm());
    parse_workload(c, "alt_screen_redraw", corpus::alt_screen_redraw());
    parse_workload(c, "unicode_wide", corpus::unicode_wide());
}

criterion_group!(workloads_ghostty, benches);
criterion_main!(workloads_ghostty);
