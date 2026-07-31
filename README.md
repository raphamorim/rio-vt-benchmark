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
cargo bench                    # everything
cargo bench --bench workloads  # parse across several input shapes
cargo bench --bench resize     # resize a filled screen
```

Criterion writes an HTML report under `target/criterion/`.

## Results

Criterion medians on an Apple Silicon Mac: rio-vt 0.5.0-alpha.4, vt100 0.15,
alacritty_terminal 0.26, libghostty-vt git 72ac98f. Numbers depend on the CPU and the
input, so run it yourself.

### Parsing (`process`)

Feeding a byte stream through the parser into the screen. Higher throughput is better.

| Workload            | rio-vt      | vt100      | alacritty   | ghostty     | winner        |
| ------------------- | ----------- | ---------- | ----------- | ----------- | ------------- |
| mixed               | 287 MiB/s   | 214 MiB/s  | 248 MiB/s   | 342 MiB/s   | ghostty       |
| ascii_plain         | 829 MiB/s   | 193 MiB/s  | 293 MiB/s   | 1538 MiB/s  | ghostty 1.9×  |
| sgr_churn           | 230 MiB/s   | 325 MiB/s  | 331 MiB/s   | 233 MiB/s   | alacritty     |
| scroll_storm        | 267 MiB/s   | 99 MiB/s   | 272 MiB/s   | 477 MiB/s   | ghostty 1.8×  |
| alt_screen_redraw   | 565 MiB/s   | 222 MiB/s  | 293 MiB/s   | 529 MiB/s   | rio-vt        |
| unicode_wide        | 245 MiB/s   | 200 MiB/s  | 338 MiB/s   | 597 MiB/s   | ghostty 1.8×  |

### Serializing a filled screen

Reading the visible 80x24 screen back out. Lower time is better. alacritty exposes no
screen-to-ANSI/text dump, so it is not shown here.

| Operation                    | rio-vt   | vt100    | ghostty  | winner      |
| ---------------------------- | -------- | -------- | -------- | ----------- |
| contents_formatted (ANSI)    | 4.4 µs   | 20.3 µs  | 12.5 µs  | rio-vt 2.9× |
| contents_plain (text)        | 4.0 µs   | 14.4 µs  | 9.0 µs   | rio-vt 2.3× |

### Resizing a filled screen

Fill the screen, then resize 80x24 to 100x40 and back. Lower time is better.

| Operation | rio-vt  | vt100   | alacritty | ghostty | winner      |
| --------- | ------- | ------- | --------- | ------- | ----------- |
| resize    | 5.1 µs  | 7.8 µs  | 239 µs    | 70 µs   | rio-vt      |

## Reading the results

ghostty parses fastest on most input shapes: plain glyphs (`ascii_plain`), scrolling
(`scroll_storm`), and wide characters (`unicode_wide`) by wide margins, plus the mixed
corpus. rio-vt is second on those shapes and takes full-screen repaints
(`alt_screen_redraw`) outright. `sgr_churn` goes to alacritty and vt100, where rio-vt's
and ghostty's style interning costs more than a plain per-cell style.

rio-vt serializes a screen 2-3× faster than ghostty and ~5× faster than vt100, and
resizes far faster than everything else while still reflowing wrapped lines (vt100
skips reflow, so it clips content on shrink; ghostty reflows at 70 µs; alacritty
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
