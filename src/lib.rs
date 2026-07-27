//! Shared setup for the benchmarks in `benches/`.
//!
//! Both engines are driven at a fixed 80x24 with no scrollback. `new_rio`
//! and `new_vt100` build a fresh screen; the `corpus` module builds the byte
//! streams each benchmark feeds in.

use rio_vt::ansi::CursorShape;
use rio_vt::crosswords::{Crosswords, CrosswordsSize};
use rio_vt::event::{VoidListener, WindowId};
use rio_vt::performer::handler::Processor;

pub const ROWS: u16 = 24;
pub const COLS: u16 = 80;

/// A rio-vt screen sized `cols` x `rows`, with cell pixel metrics filled in
/// (rio-vt tracks pixel size; the values only matter for pixel-space queries,
/// not for parsing).
pub fn rio_dims(cols: usize, rows: usize) -> CrosswordsSize {
    CrosswordsSize::new_with_dimensions(
        cols,
        rows,
        cols as u32 * 8,
        rows as u32 * 16,
        8,
        16,
    )
}

/// A fresh rio-vt screen plus its parser (rio-vt keeps state and parser
/// separate).
pub fn new_rio() -> (Crosswords<VoidListener>, Processor) {
    let term = Crosswords::new(
        rio_dims(COLS as usize, ROWS as usize),
        CursorShape::Block,
        VoidListener,
        WindowId::from(0),
        0,
        0,
    );
    (term, Processor::default())
}

/// A fresh rio-vt screen with a scrollback history limit, plus its parser.
pub fn new_rio_scrollback(
    scrollback: usize,
) -> (Crosswords<VoidListener>, Processor) {
    let term = Crosswords::new(
        rio_dims(COLS as usize, ROWS as usize),
        CursorShape::Block,
        VoidListener,
        WindowId::from(0),
        0,
        scrollback,
    );
    (term, Processor::default())
}

/// A fresh vt100 parser (it bundles state and parsing together).
pub fn new_vt100() -> vt100::Parser {
    vt100::Parser::new(ROWS, COLS, 0)
}

/// alacritty_terminal helpers: a `Term` plus the `vte` ANSI parser it feeds.
pub mod alacritty {
    use super::{COLS, ROWS};
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::term::test::TermSize;
    use alacritty_terminal::term::{Config, Term};
    use alacritty_terminal::vte::ansi::Processor;

    pub type Screen = Term<VoidListener>;

    #[inline]
    pub fn size(cols: usize, rows: usize) -> TermSize {
        TermSize::new(cols, rows)
    }

    /// A fresh 80x24 terminal with `scrollback` history, plus its parser
    /// (alacritty keeps state and parser separate, like rio-vt).
    pub fn new(scrollback: usize) -> (Screen, Processor) {
        let cfg = Config {
            scrolling_history: scrollback,
            ..Default::default()
        };
        let term = Term::new(cfg, &size(COLS as usize, ROWS as usize), VoidListener);
        (term, Processor::new())
    }
}

/// Byte streams fed to the parsers. Each returns roughly a few hundred KB so a
/// single iteration is long enough to time cleanly.
pub mod corpus {
    /// A representative mix: colored directory listings, plain lines, and a
    /// full-screen redraw every so often.
    pub fn mixed() -> Vec<u8> {
        let mut out = Vec::new();
        for i in 0..3000u32 {
            out.extend_from_slice(b"\x1b[1;34mdir\x1b[0m  \x1b[32mfile");
            out.extend_from_slice(i.to_string().as_bytes());
            out.extend_from_slice(
                b".rs\x1b[0m  \x1b[90msome regular output text here\x1b[0m\r\n",
            );
            if i % 40 == 0 {
                out.extend_from_slice(
                    b"\x1b[2J\x1b[H\x1b[1;33m== section header ==\x1b[0m\r\n",
                );
            }
        }
        out
    }

    /// Plain ASCII lines, no escape sequences. Isolates the printable-run
    /// path with nothing else in the way.
    pub fn ascii_plain() -> Vec<u8> {
        let mut out = Vec::new();
        let line: &[u8] =
            b"the quick brown fox jumps over the lazy dog 0123456789 while typing";
        for _ in 0..3500u32 {
            out.extend_from_slice(line);
            out.extend_from_slice(b"\r\n");
        }
        out
    }

    /// A foreground color change before nearly every character: heavy SGR and
    /// style traffic rather than raw glyph volume.
    pub fn sgr_churn() -> Vec<u8> {
        let mut out = Vec::new();
        let text: &[u8] = b"the quick brown fox jumps";
        for i in 0..3000u32 {
            for (j, &b) in text.iter().enumerate() {
                let color = ((i + j as u32) % 256) as u8;
                out.extend_from_slice(b"\x1b[38;5;");
                out.extend_from_slice(color.to_string().as_bytes());
                out.push(b'm');
                out.push(b);
            }
            out.extend_from_slice(b"\x1b[0m\r\n");
        }
        out
    }

    /// Long lines with no interior newline, so the terminal soft-wraps each
    /// across several rows. Resizing this forces the reflow path (wrapped
    /// lines rejoin on grow and re-split on shrink), which the short-line
    /// corpora never exercise.
    pub fn wrapped() -> Vec<u8> {
        let mut out = Vec::new();
        for i in 0..2000u32 {
            out.extend_from_slice(b"line");
            out.extend_from_slice(i.to_string().as_bytes());
            out.push(b' ');
            // ~240 columns of content, so at 80 cols each line wraps 3 rows.
            for _ in 0..20 {
                out.extend_from_slice(b"lorem ipsum ");
            }
            out.extend_from_slice(b"\r\n");
        }
        out
    }

    /// Short lines and many newlines: mostly scrolling, little cell writing.
    pub fn scroll_storm() -> Vec<u8> {
        let mut out = Vec::new();
        for i in 0..24000u32 {
            out.extend_from_slice(b"line ");
            out.extend_from_slice(i.to_string().as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        out
    }

    /// Repeated full-screen repaints on the alternate screen, the shape a TUI
    /// like vim or htop produces: clear, home, then paint every row.
    pub fn alt_screen_redraw() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\x1b[?1049h");
        for frame in 0..450u32 {
            out.extend_from_slice(b"\x1b[2J\x1b[H");
            for row in 1..=24u32 {
                out.extend_from_slice(b"\x1b[");
                out.extend_from_slice(row.to_string().as_bytes());
                out.extend_from_slice(b";1H\x1b[7m row ");
                out.extend_from_slice(frame.to_string().as_bytes());
                out.extend_from_slice(
                    b" content filler filler filler filler filler\x1b[0m",
                );
            }
        }
        out.extend_from_slice(b"\x1b[?1049l");
        out
    }

    /// CJK text and emoji: multi-byte decoding plus wide-character width
    /// handling, the part of the pipeline the plain-ASCII path skips.
    pub fn unicode_wide() -> Vec<u8> {
        let mut out = Vec::new();
        let sample = "こんにちは世界 日本語のテキスト 絵文字 😀🌍🚀 ";
        for _ in 0..3000u32 {
            out.extend_from_slice(sample.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
        out
    }
}
