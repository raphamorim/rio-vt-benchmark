# rio-vt-benchmark

A small benchmark that compares [rio-vt](https://crates.io/crates/rio-vt) with the
[vt100](https://crates.io/crates/vt100) crate.

Both crates parse a terminal byte stream into an in-memory screen and can turn that
screen back into ANSI. The benchmark measures the two halves separately on the same
80x24 screen:

* `process`: parse a stream of program output into the screen.
* `contents_formatted`: serialize the visible screen back to ANSI.

## Running

```
cargo bench
```

Criterion writes an HTML report under `target/criterion/`.

## Results

The input is about 225 KB of colored program output: directory listings, plain lines,
and a full-screen redraw every so often. Both parsers start from an empty 80x24 screen
and read the same bytes; the snapshot case serializes an already-filled screen so it
isolates formatting from parsing.

Measured with rio-vt 0.5.0-alpha.2 and vt100 0.15 on an Apple Silicon Mac (criterion
median):

| Operation                        | rio-vt            | vt100             |
| -------------------------------- | ----------------- | ----------------- |
| `process` (parse)                | 766 µs, 294 MiB/s | 1.04 ms, 217 MiB/s |
| `contents_formatted` (snapshot)  | 4.4 µs            | 18.8 µs           |

On this workload rio-vt parses about 1.35x faster and serializes about 4.2x faster.
Exact figures depend on the CPU and the input, so run it yourself.

## What the numbers cover

The corpus lives in `benches/snapshot.rs` (`corpus()`), a stand-in for everyday
output: colored `ls`-style lines, plain text, and a clear-screen redraw at intervals.
`process` is the hot path for anything that streams a lot of output; the snapshot cost
matters when you serialize a screen, for example to restore it elsewhere.

## License

MIT
