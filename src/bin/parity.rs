//! Render-parity check for the rio-vt <- vt100 reconnect-snapshot migration.
//!
//! For each scenario it feeds an identical VT stream to both engines, takes
//! each engine's `contents_formatted()` snapshot, then renders BOTH snapshots
//! through the same reference client (a fresh vt100 parser) and diffs the
//! resulting grids cell-by-cell (text + fg/bg + bold/italic/underline/inverse)
//! plus the cursor. Byte-level snapshots differ by design (rio-vt uses \r\n,
//! vt100 uses positioning); what must match is what a client renders.
//!
//! Run: `cargo run --release --bin parity`

use rio_vt_benchmark::{new_rio_scrollback, rio_dims};

const COLS: u16 = 80;
const ROWS: u16 = 24;
// The reconnect-snapshot Screen is created with no scrollback, so match that:
// it changes how a row-grow resize sources new rows.
const SCROLLBACK: usize = 0;

#[derive(PartialEq, Eq)]
struct CellSig {
    text: String,
    fg: String,
    bg: String,
    attrs: u8,
}

struct Rendered {
    cells: Vec<CellSig>,
    cursor: (u16, u16),
    cols: u16,
}

/// Render a snapshot by parsing it into a fresh vt100 client and reading the grid.
fn render(snapshot: &[u8], cols: u16, rows: u16) -> Rendered {
    let mut p = vt100::Parser::new(rows, cols, 0);
    p.process(snapshot);
    let screen = p.screen();
    let mut cells = Vec::with_capacity((rows * cols) as usize);
    for r in 0..rows {
        for c in 0..cols {
            let sig = match screen.cell(r, c) {
                Some(cell) => {
                    let fg = format!("{:?}", cell.fgcolor());
                    let bg = format!("{:?}", cell.bgcolor());
                    let attrs = (cell.bold() as u8)
                        | ((cell.italic() as u8) << 1)
                        | ((cell.underline() as u8) << 2)
                        | ((cell.inverse() as u8) << 3);
                    let mut text = cell.contents();
                    // A whitespace cell with no colours/attrs is visually blank;
                    // canonicalize it so rio-vt trimming trailing blanks (which
                    // vt100 keeps as spaces) does not read as a difference.
                    if attrs == 0 && fg == "Default" && bg == "Default" && text.trim().is_empty() {
                        text = String::new();
                    }
                    CellSig { text, fg, bg, attrs }
                }
                None => CellSig {
                    text: String::new(),
                    fg: "Default".into(),
                    bg: "Default".into(),
                    attrs: 0,
                },
            };
            cells.push(sig);
        }
    }
    Rendered {
        cells,
        cursor: screen.cursor_position(),
        cols,
    }
}

fn vt_snapshot(stream: &[u8], resize_to: Option<(u16, u16)>) -> Vec<u8> {
    let mut p = vt100::Parser::new(ROWS, COLS, SCROLLBACK);
    p.process(stream);
    if let Some((cols, rows)) = resize_to {
        p.set_size(rows, cols);
    }
    p.screen().contents_formatted()
}

fn rio_snapshot(stream: &[u8], resize_to: Option<(u16, u16)>) -> Vec<u8> {
    let (mut term, mut parser) = new_rio_scrollback(SCROLLBACK);
    parser.advance(&mut term, stream);
    if let Some((cols, rows)) = resize_to {
        term.resize(rio_dims(cols as usize, rows as usize));
    }
    term.contents_formatted()
}

struct Scenario {
    name: &'static str,
    stream: Vec<u8>,
    resize_to: Option<(u16, u16)>,
}

/// Compare two rendered grids; return a list of human-readable differences.
fn diff(a: &Rendered, b: &Rendered) -> Vec<String> {
    let mut out = Vec::new();
    if a.cursor != b.cursor {
        out.push(format!("cursor: vt100={:?} rio-vt={:?}", a.cursor, b.cursor));
    }
    if a.cells.len() != b.cells.len() {
        out.push(format!(
            "cell count: vt100={} rio-vt={}",
            a.cells.len(),
            b.cells.len()
        ));
        return out;
    }
    for (i, (ca, cb)) in a.cells.iter().zip(&b.cells).enumerate() {
        if ca != cb {
            let (row, col) = (i as u16 / a.cols, i as u16 % a.cols);
            out.push(format!(
                "({row},{col}) vt100={:?}/{}/{}/{:04b} rio-vt={:?}/{}/{}/{:04b}",
                ca.text, ca.fg, ca.bg, ca.attrs, cb.text, cb.fg, cb.bg, cb.attrs
            ));
        }
    }
    out
}

fn scenarios() -> Vec<Scenario> {
    let mut v = Vec::new();

    // colored `ls`: bold-blue dirs, green files, dim trailing text.
    let mut s = Vec::new();
    for i in 0..30u32 {
        s.extend_from_slice(b"\x1b[1;34msrc\x1b[0m  \x1b[32mmain");
        s.extend_from_slice(i.to_string().as_bytes());
        s.extend_from_slice(b".rs\x1b[0m  \x1b[90m(edited)\x1b[0m\r\n");
    }
    v.push(Scenario { name: "colored_ls", stream: s, resize_to: None });

    // wrapped paragraph: a long line with no interior newline.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    s.extend_from_slice("the quick brown fox jumps over the lazy dog ".repeat(6).as_bytes());
    v.push(Scenario { name: "wrapped_paragraph", stream: s, resize_to: None });

    // vim-like: clear, text, a reverse-video status bar on the last row.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    for i in 0..20u32 {
        s.extend_from_slice(b"line ");
        s.extend_from_slice(i.to_string().as_bytes());
        s.extend_from_slice(b" some text\r\n");
    }
    s.extend_from_slice(b"\x1b[24;1H\x1b[7m-- INSERT --  file.rs  10,1\x1b[0m");
    v.push(Scenario { name: "vim_like", stream: s, resize_to: None });

    // htop-like: colored bars using background colors + inverse.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    for cpu in 0..8u32 {
        s.extend_from_slice(b"CPU");
        s.extend_from_slice(cpu.to_string().as_bytes());
        s.extend_from_slice(b" [\x1b[42m    \x1b[41m   \x1b[0m       ]\r\n");
    }
    s.extend_from_slice(b"\x1b[7m  PID USER      CPU% MEM%  Command\x1b[0m\r\n");
    v.push(Scenario { name: "htop_like", stream: s, resize_to: None });

    // alt-screen: enter alt, draw a full-screen UI, stay in alt.
    let mut s = Vec::new();
    s.extend_from_slice(b"regular scrollback line\r\n");
    s.extend_from_slice(b"\x1b[?1049h\x1b[2J\x1b[H");
    s.extend_from_slice(b"\x1b[1;33mALT SCREEN APP\x1b[0m\r\n");
    for i in 0..10u32 {
        s.extend_from_slice(b"  item ");
        s.extend_from_slice(i.to_string().as_bytes());
        s.extend_from_slice(b"\r\n");
    }
    v.push(Scenario { name: "alt_screen", stream: s, resize_to: None });

    // SGR combinations: each cell a different attribute mix.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    let sgrs: &[&[u8]] = &[
        b"1", b"2", b"3", b"4", b"7", b"1;4", b"31", b"1;32", b"38;5;208",
        b"48;5;27", b"38;2;255;100;0", b"90", b"1;3;4;31",
    ];
    for (i, sgr) in sgrs.iter().enumerate() {
        s.extend_from_slice(b"\x1b[");
        s.extend_from_slice(sgr);
        s.extend_from_slice(b"mX\x1b[0m ");
        if i % 4 == 3 {
            s.extend_from_slice(b"\r\n");
        }
    }
    v.push(Scenario { name: "sgr_combos", stream: s, resize_to: None });

    // unicode + wide characters.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    s.extend_from_slice("こんにちは 世界  \x1b[32m日本語\x1b[0m  😀🌍 ok\r\n".as_bytes());
    s.extend_from_slice("mixed ascii and 全角 width\r\n".as_bytes());
    v.push(Scenario { name: "unicode_wide", stream: s, resize_to: None });

    // resize-then-snapshot: reconnect at a narrower width forces reflow.
    let mut s = Vec::new();
    s.extend_from_slice(b"\x1b[2J\x1b[H");
    s.extend_from_slice("a long wrapped line of content that spans well past eighty columns and keeps going".repeat(3).as_bytes());
    v.push(Scenario { name: "resize_100x40", stream: s.clone(), resize_to: Some((100, 40)) });
    v.push(Scenario { name: "resize_60x24", stream: s, resize_to: Some((60, 24)) });

    v
}

fn main() {
    let scenarios = scenarios();
    let mut failures = 0;
    println!("render-parity: rio-vt vs vt100 reconnect snapshots\n");

    for sc in &scenarios {
        let (cols, rows) = sc.resize_to.unwrap_or((COLS, ROWS));
        let r_vt = render(&vt_snapshot(&sc.stream, sc.resize_to), cols, rows);
        let r_rio = render(&rio_snapshot(&sc.stream, sc.resize_to), cols, rows);
        let diffs = diff(&r_vt, &r_rio);
        if diffs.is_empty() {
            println!("  PASS  {}", sc.name);
        } else {
            failures += 1;
            println!("  FAIL  {}  ({} differing cells/fields)", sc.name, diffs.len());
            for d in diffs.iter().take(6) {
                println!("          {d}");
            }
            if diffs.len() > 6 {
                println!("          ... and {} more", diffs.len() - 6);
            }
        }
    }

    println!(
        "\n{} / {} scenarios render-identical",
        scenarios.len() - failures,
        scenarios.len()
    );
    if failures > 0 {
        std::process::exit(1);
    }
}
