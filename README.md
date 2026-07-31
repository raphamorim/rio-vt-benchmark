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

Criterion medians on an Apple Silicon Mac: rio-vt 0.5.3, vt100 0.15,
alacritty_terminal 0.26, libghostty-vt git 72ac98f. Numbers depend on the CPU and the
input, so run it yourself.

### Parsing (`process`)

Feeding a byte stream through the parser into the screen. Higher throughput is better.

| Workload            | rio-vt      | vt100      | alacritty   | ghostty     | winner        |
| ------------------- | ----------- | ---------- | ----------- | ----------- | ------------- |
| mixed               | 436 MiB/s   | 212 MiB/s  | 247 MiB/s   | 338 MiB/s   | rio-vt 1.3×   |
| ascii_plain         | 2340 MiB/s  | 191 MiB/s  | 301 MiB/s   | 1562 MiB/s  | rio-vt 1.5×   |
| sgr_churn           | 377 MiB/s   | 331 MiB/s  | 332 MiB/s   | 238 MiB/s   | rio-vt        |
| scroll_storm        | 772 MiB/s   | 97 MiB/s   | 277 MiB/s   | 490 MiB/s   | rio-vt 1.6×   |
| alt_screen_redraw   | 891 MiB/s   | 221 MiB/s  | 294 MiB/s   | 572 MiB/s   | rio-vt 1.6×   |
| unicode_wide        | 726 MiB/s   | 198 MiB/s  | 349 MiB/s   | 685 MiB/s   | rio-vt        |

### Serializing a filled screen

Reading the visible 80x24 screen back out. Lower time is better. alacritty exposes no
screen-to-ANSI/text dump, so it is not shown here.

| Operation                    | rio-vt   | vt100    | ghostty  | winner      |
| ---------------------------- | -------- | -------- | -------- | ----------- |
| contents_formatted (ANSI)    | 4.5 µs   | 19.4 µs  | 12.3 µs  | rio-vt 2.7× |
| contents_plain (text)        | 4.0 µs   | 14.4 µs  | 9.1 µs   | rio-vt 2.3× |

### Resizing a filled screen

Fill the screen, then resize 80x24 to 100x40 and back. Lower time is better.

| Operation | rio-vt  | vt100   | alacritty | ghostty | winner      |
| --------- | ------- | ------- | --------- | ------- | ----------- |
| resize    | 5.0 µs  | 8.1 µs  | 237 µs    | 71 µs   | rio-vt      |

## Reading the results

rio-vt 0.5.3 parses fastest on every input shape: plain glyphs, scrolling, full-screen
repaints, wide characters, and the mixed corpus, with ghostty second on most of them.
`sgr_churn` is the tightest race; rio-vt edges out alacritty and vt100, whose plain
per-cell style storage avoids the interning cost rio-vt and ghostty pay per SGR
change (ghostty trails the group there).

rio-vt also serializes a screen 2-3× faster than ghostty and ~4× faster than vt100,
and resizes far faster than everything else while still reflowing wrapped lines
(vt100 skips reflow, so it clips content on shrink; ghostty reflows at 71 µs;
alacritty reflows but is two orders of magnitude slower than rio-vt here).

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
