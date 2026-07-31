# rio-vt-benchmark

A benchmark comparing [rio-vt](https://crates.io/crates/rio-vt) against other terminal
engines: [vt100](https://crates.io/crates/vt100),
[alacritty_terminal](https://crates.io/crates/alacritty_terminal) and
[libghostty-vt](https://crates.io/crates/libghostty-vt) (Rust bindings to Ghostty's
terminal core; building it compiles Ghostty from source and requires
[zig](https://ziglang.org) 0.16 on PATH). libghostty-vt is pinned to a git rev because
the crates.io release pins a Ghostty commit that only builds with zig 0.15.

Each parses a terminal byte stream into an in-memory screen. The benchmark measures
parsing a stream, serializing a filled screen (rio-vt, vt100 and libghostty-vt, which
expose a screen-to-ANSI dump), and resizing a filled screen. Everything runs at a
fixed 80x24.

## Running

```
cargo bench                     # rio-vt, vt100 and alacritty
cargo bench --features ghostty --bench workloads_ghostty \
    --bench snapshot_ghostty --bench resize_ghostty
cargo bench --bench workloads   # parse across several input shapes
cargo bench --bench resize      # resize a filled screen
```

The ghostty benches live in separate binaries behind the `ghostty` feature:
rio-vt and libghostty-vt both bundle simdutf, and linking the two C++ builds
into one binary lets the linker mix their objects, which crashes. Criterion
merges the groups by name, so the HTML report under `target/criterion/`
still compares all four engines.

## Results

Criterion medians on an Apple Silicon Mac: rio-vt 0.5.0-alpha.4, vt100 0.15,
alacritty_terminal 0.26, libghostty-vt git 72ac98f. Numbers depend on the CPU and the
input, so run it yourself.

### Parsing (`process`)

Feeding a byte stream through the parser into the screen. Higher throughput is better.

| Workload            | rio-vt      | vt100      | alacritty   | ghostty     | winner        |
| ------------------- | ----------- | ---------- | ----------- | ----------- | ------------- |
| mixed               | 294 MiB/s   | 212 MiB/s  | 247 MiB/s   | 338 MiB/s   | ghostty       |
| ascii_plain         | 865 MiB/s   | 191 MiB/s  | 301 MiB/s   | 1562 MiB/s  | ghostty 1.8×  |
| sgr_churn           | 233 MiB/s   | 331 MiB/s  | 332 MiB/s   | 238 MiB/s   | alacritty     |
| scroll_storm        | 266 MiB/s   | 97 MiB/s   | 277 MiB/s   | 490 MiB/s   | ghostty 1.8×  |
| alt_screen_redraw   | 567 MiB/s   | 221 MiB/s  | 294 MiB/s   | 572 MiB/s   | tie           |
| unicode_wide        | 243 MiB/s   | 198 MiB/s  | 349 MiB/s   | 685 MiB/s   | ghostty 2.0×  |

### Serializing a filled screen

Reading the visible 80x24 screen back out. Lower time is better. alacritty exposes no
screen-to-ANSI/text dump, so it is not shown here.

| Operation                    | rio-vt   | vt100    | ghostty  | winner      |
| ---------------------------- | -------- | -------- | -------- | ----------- |
| contents_formatted (ANSI)    | 4.4 µs   | 19.4 µs  | 12.3 µs  | rio-vt 2.8× |
| contents_plain (text)        | 4.1 µs   | 14.4 µs  | 9.1 µs   | rio-vt 2.2× |

### Resizing a filled screen

Fill the screen, then resize 80x24 to 100x40 and back. Lower time is better.

| Operation | rio-vt  | vt100   | alacritty | ghostty | winner      |
| --------- | ------- | ------- | --------- | ------- | ----------- |
| resize    | 5.3 µs  | 8.1 µs  | 237 µs    | 71 µs   | rio-vt      |

## Reading the results

ghostty parses fastest on most input shapes: plain glyphs (`ascii_plain`), scrolling
(`scroll_storm`), and wide characters (`unicode_wide`) by wide margins, plus the mixed
corpus. rio-vt is second on those shapes and ties ghostty on full-screen repaints
(`alt_screen_redraw`). `sgr_churn` goes to alacritty and vt100, where rio-vt's
and ghostty's style interning costs more than a plain per-cell style.

rio-vt serializes a screen 2-3× faster than ghostty and ~4× faster than vt100, and
resizes far faster than everything else while still reflowing wrapped lines (vt100
skips reflow, so it clips content on shrink; ghostty reflows at 71 µs; alacritty
reflows but is two orders of magnitude slower than rio-vt here).

## What the workloads are

The inputs live in `src/lib.rs` (`corpus`):

* `mixed`: colored directory listings, plain lines, a full-screen redraw now and then.
* `ascii_plain`: plain ASCII lines, no escape sequences.
* `sgr_churn`: a color change before nearly every character.
* `scroll_storm`: short lines and lots of newlines, mostly scrolling.
* `alt_screen_redraw`: repeated clear, home, and full repaint, the shape a TUI produces.
* `unicode_wide`: CJK text and emoji, so multi-byte decoding and wide characters.

## License

MIT
