# rio-vt-benchmark

A small benchmark that compares [rio-vt](https://crates.io/crates/rio-vt) with the
[vt100](https://crates.io/crates/vt100) crate.

Both crates parse a terminal byte stream into an in-memory screen and can turn that
screen back into text or ANSI. The benchmark measures three things: parsing a stream
into the screen, serializing a filled screen, and resizing a filled screen. Each is run
on rio-vt and vt100 at a fixed 80x24 with no scrollback.

## Running

```
cargo bench                       # everything
cargo bench --bench snapshot      # parse + serialize on the mixed corpus
cargo bench --bench workloads     # parse across several input shapes
cargo bench --bench resize        # resize a filled screen
```

Criterion writes an HTML report under `target/criterion/`.

## Results

Criterion medians on an Apple Silicon Mac, rio-vt 0.5.0-alpha.2 and vt100 0.15. Numbers
depend on the CPU and the input, so run it yourself.

### Parsing (`process`)

Feeding a byte stream through the parser into the screen. Higher throughput is better.

| Workload            | rio-vt      | vt100      | winner      |
| ------------------- | ----------- | ---------- | ----------- |
| mixed               | 305 MiB/s   | 216 MiB/s  | rio-vt 1.4× |
| ascii_plain         | 893 MiB/s   | 195 MiB/s  | rio-vt 4.6× |
| sgr_churn           | 233 MiB/s   | 317 MiB/s  | vt100 1.4×  |
| scroll_storm        | 277 MiB/s   | 100 MiB/s  | rio-vt 2.8× |
| alt_screen_redraw   | 589 MiB/s   | 223 MiB/s  | rio-vt 2.6× |
| unicode_wide        | 248 MiB/s   | 197 MiB/s  | rio-vt 1.3× |

### Serializing a filled screen

Reading the visible 80x24 screen back out. Lower time is better.

| Operation                    | rio-vt   | vt100    |
| ---------------------------- | -------- | -------- |
| contents_formatted (ANSI)    | 4.4 µs   | 18.6 µs  |
| contents_plain (text)        | 3.8 µs   | 14.0 µs  |

### Resizing a filled screen

Fill the screen, then resize 80x24 to 100x40 and back. Lower time is better.

| Operation | rio-vt  | vt100   |
| --------- | ------- | ------- |
| resize    | 51 µs   | 7.4 µs  |

## Reading the results

rio-vt parses faster on most input shapes, and by a wide margin when the work is plain
glyphs (`ascii_plain`), scrolling (`scroll_storm`), or full-screen repaints
(`alt_screen_redraw`). It also serializes a screen several times faster, whether to ANSI
or plain text.

vt100 wins in two places. It parses `sgr_churn` faster: that input changes the
foreground color before almost every character, and rio-vt's per-cell style interning
costs more than vt100's per-cell style. And it resizes faster, because rio-vt's grid
reflow does more bookkeeping than vt100's. Resize happens rarely (on a real window
drag), so it matters far less than steady parsing, but it is a real difference.

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
