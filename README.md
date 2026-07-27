# rio-vt-benchmark

A benchmark comparing [rio-vt](https://crates.io/crates/rio-vt) against other terminal
engines: [vt100](https://crates.io/crates/vt100) and
[alacritty_terminal](https://crates.io/crates/alacritty_terminal).

Each parses a terminal byte stream into an in-memory screen. The benchmark measures
parsing a stream, serializing a filled screen (rio-vt and vt100 only, which expose a
screen-to-ANSI dump), and resizing a filled screen. Everything runs at a fixed 80x24.

## Running

```
cargo bench                    # everything
cargo bench --bench workloads  # parse across several input shapes
cargo bench --bench resize     # resize a filled screen
```

Criterion writes an HTML report under `target/criterion/`.

## Results

Criterion medians on an Apple Silicon Mac: rio-vt 0.5.0-alpha.3, vt100 0.15,
alacritty_terminal 0.26. Numbers depend on the CPU and the input, so run it yourself.

### Parsing (`process`)

Feeding a byte stream through the parser into the screen. Higher throughput is better.

| Workload            | rio-vt      | vt100      | alacritty   | winner       |
| ------------------- | ----------- | ---------- | ----------- | ------------ |
| mixed               | 302 MiB/s   | 221 MiB/s  | 254 MiB/s   | rio-vt       |
| ascii_plain         | 835 MiB/s   | 196 MiB/s  | 279 MiB/s   | rio-vt 3.0×  |
| sgr_churn           | 235 MiB/s   | 349 MiB/s  | 332 MiB/s   | vt100        |
| scroll_storm        | 274 MiB/s   | 101 MiB/s  | 266 MiB/s   | rio-vt       |
| alt_screen_redraw   | 588 MiB/s   | 231 MiB/s  | 282 MiB/s   | rio-vt 2.1×  |
| unicode_wide        | 248 MiB/s   | 203 MiB/s  | 337 MiB/s   | alacritty    |

### Serializing a filled screen

Reading the visible 80x24 screen back out. Lower time is better. Only rio-vt and vt100
expose a screen-to-ANSI/text dump, so alacritty is not shown here.

| Operation                    | rio-vt   | vt100    | winner      |
| ---------------------------- | -------- | -------- | ----------- |
| contents_formatted (ANSI)    | 4.3 µs   | 18.6 µs  | rio-vt 4.3× |
| contents_plain (text)        | 3.8 µs   | 13.9 µs  | rio-vt 3.7× |

### Resizing a filled screen

Fill the screen, then resize 80x24 to 100x40 and back. Lower time is better.

| Operation | rio-vt  | vt100   | alacritty | winner      |
| --------- | ------- | ------- | --------- | ----------- |
| resize    | 5.0 µs  | 7.5 µs  | 227 µs    | rio-vt      |

## Reading the results

rio-vt parses faster on most input shapes, and by a wide margin when the work is plain
glyphs (`ascii_plain`) or full-screen repaints (`alt_screen_redraw`). It also serializes
a screen several times faster, and resizes far faster than either alacritty or vt100
while still reflowing wrapped lines (vt100 skips reflow, so it clips content on shrink;
alacritty reflows but is two orders of magnitude slower here).

alacritty is competitive on parsing, landing between rio-vt and vt100 on most shapes and
taking `unicode_wide` outright. vt100 still parses `sgr_churn` fastest, where rio-vt's
per-cell style interning costs more than a plain per-cell style. On resize, alacritty is
the outlier: it reflows its grid far more expensively than either rio-vt or vt100.

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
