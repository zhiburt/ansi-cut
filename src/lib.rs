//! # Ansi-cut
//!
//! A library for cutting a string while preserving colors.
//!
//! ## Example
//!
//! ```
//! use ansi_cut::AnsiCut;
//! use owo_colors::{colors::*, OwoColorize};
//!
//! let colored_text = "When the night has come"
//!     .fg::<Black>()
//!     .bg::<White>()
//!     .to_string();
//!
//! let cutted_text = colored_text.cut(5..);
//!
//! println!("{}", cutted_text);
//! ```

use ansi_parser::AnsiSequence;
use ansi_parser::{AnsiParser, Output};
use std::ops::{Bound, RangeBounds};

/// AnsiCut a trait to cut a string while keeping information
/// about its color defined as ANSI control sequences.
pub trait AnsiCut {
    /// Cut string from the beginning of the range to the end.
    /// Preserving its colors.
    ///
    /// Range is defined in terms of `byte`s of the string not containing ANSI
    /// control sequences.
    ///
    /// Exceeding an upper bound does not panic.
    ///
    /// # Panics
    ///
    /// Panics if a start or end indexes are not on a UTF-8 code point boundary.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use owo_colors::{OwoColorize, colors::*};
    /// use ansi_cut::AnsiCut;
    /// let colored_text = format!("{} {} {}", "A".fg::<Black>(), "Colored".fg::<Red>(), "Text".fg::<Blue>()).bg::<Yellow>().to_string();
    /// let cut_text = colored_text.cut(5..);
    /// println!("{}", cut_text);
    /// ```
    fn cut<R>(&self, range: R) -> String
    where
        R: RangeBounds<usize>;
}

impl AnsiCut for &str {
    fn cut<R>(&self, range: R) -> String
    where
        R: RangeBounds<usize>,
    {
        crate::cut(&self, range)
    }
}

impl AnsiCut for String {
    fn cut<R>(&self, range: R) -> String
    where
        R: RangeBounds<usize>,
    {
        crate::cut(&self, range)
    }
}

/// Returns an Vec over chunk_size elements of string, starting at the beginning of the slice.
/// It uses chars but not bytes!
///
/// The chunks are vectors and do not overlap.
/// If chunk_size does not divide the length of the slice, then the last chunk will not have length chunk_size.
///
/// # Panics
///
/// Panics if chunk_size is 0.
///
/// # Examples
///
/// ```rust
/// use owo_colors::{OwoColorize, colors::*};
/// let colored_text = format!("{} {} {}", "A".fg::<Black>(), "Colored".fg::<Red>(), "Text".fg::<Blue>()).bg::<Yellow>().to_string();
/// let chunks = ansi_cut::chunks(&colored_text, 3);
/// for chunk in &chunks {
///     println!("{}", chunk);
/// }
/// ```
pub fn chunks(s: &str, chunk_size: usize) -> Vec<String> {
    assert!(chunk_size > 0);

    let stripped = srip_ansi_sequences(s);
    let count_chars = stripped.chars().count();
    let mut chunks = Vec::new();
    let mut start_pos = 0;

    while start_pos < count_chars {
        let start = stripped
            .chars()
            .map(|c| c.len_utf8())
            .take(start_pos)
            .sum::<usize>();
        let end_pos = std::cmp::min(start_pos + chunk_size, count_chars);
        let end = stripped
            .chars()
            .map(|c| c.len_utf8())
            .take(end_pos)
            .sum::<usize>();
        let part = s.cut(start..end);
        start_pos = end_pos;

        if part.is_empty() {
            break;
        }

        chunks.push(part);
    }

    chunks
}

// Bounds are byte index
// It's not safe to go over grapheme boundres.
fn cut<S, R>(string: S, bounds: R) -> String
where
    S: AsRef<str>,
    R: RangeBounds<usize>,
{
    let string = string.as_ref();
    let (start, end) = bounds_to_usize(bounds.start_bound(), bounds.end_bound());

    cut_str(string, start, end)
}

fn cut_str(string: &str, lower_bound: usize, upper_bound: Option<usize>) -> String {
    let mut asci_state = AnsiState::default();
    let tokens = string.ansi_parse();
    let mut buf = String::new();
    let mut index = 0;

    '_tokens_loop: for token in tokens {
        match token {
            Output::TextBlock(text) => {
                let block_end_index = index + text.len();
                if lower_bound > block_end_index {
                    index += text.len();
                    continue;
                };

                let mut start = 0;
                if lower_bound > index {
                    start = lower_bound - index;
                }

                let mut end = text.len();
                let mut done = false;
                if let Some(upper_bound) = upper_bound {
                    if upper_bound > index && upper_bound < block_end_index {
                        end = upper_bound - index;
                        done = true;
                    }
                }

                index += text.len();

                match text.get(start..end) {
                    Some(text) => {
                        buf.push_str(text);
                        if done {
                            break '_tokens_loop;
                        }
                    }
                    None => {
                        panic!("One of indexes are not on a UTF-8 code point boundary");
                    }
                }
            }
            Output::Escape(seq) => {
                let seq_str = seq.to_string();
                buf.push_str(&seq_str);
                if let AnsiSequence::SetGraphicsMode(v) = seq {
                    update_ansi_state(&mut asci_state, v.as_ref());
                }
            }
        }
    }

    complete_ansi_sequences(&asci_state, &mut buf);

    buf
}

#[derive(Debug, Clone, Default)]
struct AnsiState {
    fg_color: Option<AnsiColor>,
    bg_color: Option<AnsiColor>,
    undr_color: Option<AnsiColor>,
    bold: bool,
    faint: bool,
    italic: bool,
    underline: bool,
    double_underline: bool,
    slow_blink: bool,
    rapid_blink: bool,
    inverse: bool,
    hide: bool,
    crossedout: bool,
    reset: bool,
    framed: bool,
    encircled: bool,
    font: Option<u8>,
    fraktur: bool,
    proportional_spacing: bool,
    overlined: bool,
    igrm_underline: bool,
    igrm_double_underline: bool,
    igrm_overline: bool,
    igrm_double_overline: bool,
    igrm_stress_marking: bool,
    superscript: bool,
    subscript: bool,
    unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AnsiColor {
    Bit4 { index: u8 },
    Bit8 { index: u8 },
    Bit24 { r: u8, g: u8, b: u8 },
}

fn update_ansi_state(state: &mut AnsiState, mode: &[u8]) {
    let mut ptr = mode;
    loop {
        if ptr.is_empty() {
            break;
        }

        let tag = ptr[0];

        match tag {
            0 => {
                *state = AnsiState::default();
                state.reset = true;
            }
            1 => state.bold = true,
            2 => state.faint = true,
            3 => state.italic = true,
            4 => state.underline = true,
            5 => state.slow_blink = true,
            6 => state.rapid_blink = true,
            7 => state.inverse = true,
            8 => state.hide = true,
            9 => state.crossedout = true,
            10 => state.font = None,
            n @ 11..=19 => state.font = Some(n),
            20 => state.fraktur = true,
            21 => state.double_underline = true,
            22 => {
                state.faint = false;
                state.bold = false;
            }
            23 => {
                state.italic = false;
            }
            24 => {
                state.underline = false;
                state.double_underline = false;
            }
            25 => {
                state.slow_blink = false;
                state.rapid_blink = false;
            }
            26 => {
                state.proportional_spacing = true;
            }
            28 => {
                state.inverse = false;
            }
            29 => {
                state.crossedout = false;
            }
            n @ 30..=37 | n @ 90..=97 => {
                state.fg_color = Some(AnsiColor::Bit4 { index: n });
            }
            38 => {
                if let Some((color, n)) = parse_ansi_color(ptr) {
                    state.fg_color = Some(color);
                    ptr = &ptr[n..];
                }
            }
            39 => {
                state.fg_color = None;
            }
            n @ 40..=47 | n @ 100..=107 => {
                state.bg_color = Some(AnsiColor::Bit4 { index: n });
            }
            48 => {
                if let Some((color, n)) = parse_ansi_color(ptr) {
                    state.bg_color = Some(color);
                    ptr = &ptr[n..];
                }
            }
            49 => {
                state.bg_color = None;
            }
            50 => {
                state.proportional_spacing = false;
            }
            51 => {
                state.framed = true;
            }
            52 => {
                state.encircled = true;
            }
            53 => {
                state.overlined = true;
            }
            54 => {
                state.encircled = false;
                state.framed = false;
            }
            55 => {
                state.overlined = false;
            }
            58 => {
                if let Some((color, n)) = parse_ansi_color(ptr) {
                    state.undr_color = Some(color);
                    ptr = &ptr[n..];
                }
            }
            59 => {
                state.undr_color = None;
            }
            60 => {
                state.igrm_underline = true;
            }
            61 => {
                state.igrm_double_underline = true;
            }
            62 => {
                state.igrm_overline = true;
            }
            63 => {
                state.igrm_double_overline = true;
            }
            64 => {
                state.igrm_stress_marking = true;
            }
            65 => {
                state.igrm_underline = false;
                state.igrm_double_underline = false;
                state.igrm_overline = false;
                state.igrm_double_overline = false;
                state.igrm_stress_marking = false;
            }
            73 => {
                state.superscript = true;
            }
            74 => {
                state.subscript = true;
            }
            75 => {
                state.subscript = false;
                state.superscript = false;
            }
            _ => {
                state.unknown = true;
            }
        }

        ptr = &ptr[1..];
    }
}

fn parse_ansi_color(buf: &[u8]) -> Option<(AnsiColor, usize)> {
    match buf {
        [b'2', b';', index, ..] => Some((AnsiColor::Bit8 { index: *index }, 3)),
        [b'5', b';', r, b';', g, b';', b, ..] => Some((
            AnsiColor::Bit24 {
                r: *r,
                g: *g,
                b: *b,
            },
            7,
        )),
        _ => None,
    }
}

fn complete_ansi_sequences(state: &AnsiState, buf: &mut String) {
    macro_rules! emit_static {
        ($s:expr) => {
            buf.push_str(concat!("\u{1b}[", $s, "m"))
        };
    }

    if state.unknown && state.reset {
        emit_static!("0");
    }

    if state.font.is_some() {
        emit_static!("10");
    }

    if state.bold || state.faint {
        emit_static!("22");
    }

    if state.italic {
        emit_static!("23");
    }

    if state.underline || state.double_underline {
        emit_static!("24");
    }

    if state.slow_blink || state.rapid_blink {
        emit_static!("25");
    }

    if state.inverse {
        emit_static!("28");
    }

    if state.crossedout {
        emit_static!("29");
    }

    if state.fg_color.is_some() {
        emit_static!("39");
    }

    if state.bg_color.is_some() {
        emit_static!("49");
    }

    if state.proportional_spacing {
        emit_static!("50");
    }

    if state.encircled || state.framed {
        emit_static!("54");
    }

    if state.overlined {
        emit_static!("55");
    }

    if state.igrm_underline
        || state.igrm_double_underline
        || state.igrm_overline
        || state.igrm_double_overline
        || state.igrm_stress_marking
    {
        emit_static!("65");
    }

    if state.undr_color.is_some() {
        emit_static!("59");
    }

    if state.subscript || state.superscript {
        emit_static!("75");
    }

    if state.unknown {
        emit_static!("0");
    }
}

fn bounds_to_usize(left: Bound<&usize>, right: Bound<&usize>) -> (usize, Option<usize>) {
    match (left, right) {
        (Bound::Included(x), Bound::Included(y)) => (*x, Some(y + 1)),
        (Bound::Included(x), Bound::Excluded(y)) => (*x, Some(*y)),
        (Bound::Included(x), Bound::Unbounded) => (*x, None),
        (Bound::Unbounded, Bound::Unbounded) => (0, None),
        (Bound::Unbounded, Bound::Included(y)) => (0, Some(y + 1)),
        (Bound::Unbounded, Bound::Excluded(y)) => (0, Some(*y)),
        (Bound::Excluded(_), Bound::Unbounded)
        | (Bound::Excluded(_), Bound::Included(_))
        | (Bound::Excluded(_), Bound::Excluded(_)) => {
            unreachable!("A start bound can't be excluded")
        }
    }
}

fn srip_ansi_sequences(string: &str) -> String {
    let tokens = string.ansi_parse();
    let mut buf = String::new();
    for token in tokens {
        match token {
            Output::TextBlock(text) => {
                buf.push_str(text);
            }
            Output::Escape(_) => {}
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ansi_color_test() {
        let tests: Vec<(&[u8], _)> = vec![
            (&[b'2', b';', 200], Some(AnsiColor::Bit8 { index: 200 })),
            (
                &[b'2', b';', 100, b';', 123, b';', 39],
                Some(AnsiColor::Bit8 { index: 100 }),
            ),
            (
                &[b'2', b';', 100, 1, 2, 3],
                Some(AnsiColor::Bit8 { index: 100 }),
            ),
            (&[b'2', b';'], None),
            (&[b'2', 1, 2, 3], None),
            (&[b'2'], None),
            (
                &[b'5', b';', 100, b';', 123, b';', 39],
                Some(AnsiColor::Bit24 {
                    r: 100,
                    g: 123,
                    b: 39,
                }),
            ),
            (
                &[b'5', b';', 100, b';', 123, b';', 39, 1, 2, 3],
                Some(AnsiColor::Bit24 {
                    r: 100,
                    g: 123,
                    b: 39,
                }),
            ),
            (
                &[b'5', b';', 100, b';', 123, b';', 39, 1, 2, 3],
                Some(AnsiColor::Bit24 {
                    r: 100,
                    g: 123,
                    b: 39,
                }),
            ),
            (&[b'5', b';', 100, b';', 123, b';'], None),
            (&[b'5', b';', 100, b';', 123], None),
            (&[b'5', b';', 100, b';'], None),
            (&[b'5', b';', 100], None),
            (&[b'5', b';'], None),
            (&[b'5'], None),
            (&[], None),
        ];

        for (i, (bytes, expected)) in tests.into_iter().enumerate() {
            assert_eq!(parse_ansi_color(bytes).map(|a| a.0), expected, "test={}", i);
        }
    }

    #[test]
    fn cut_colored_fg_test() {
        let colored_s = "\u{1b}[30mTEXT\u{1b}[39m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..4));
        assert_eq!("\u{1b}[30mEXT\u{1b}[39m", colored_s.cut(1..));
        assert_eq!("\u{1b}[30mTEX\u{1b}[39m", colored_s.cut(..3));
        assert_eq!("\u{1b}[30mEX\u{1b}[39m", colored_s.cut(1..3));

        assert_eq!("TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEX", srip_ansi_sequences(&colored_s.cut(..3)));
        assert_eq!("EX", srip_ansi_sequences(&colored_s.cut(1..3)));

        let colored_s = "\u{1b}[30mTEXT\u{1b}[39m \u{1b}[31mTEXT\u{1b}[39m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..9));
        assert_eq!(
            "\u{1b}[30mXT\u{1b}[39m \u{1b}[31mTEXT\u{1b}[39m",
            colored_s.cut(2..)
        );
        assert_eq!(
            "\u{1b}[30mTEXT\u{1b}[39m \u{1b}[31mT\u{1b}[39m",
            colored_s.cut(..6)
        );
        assert_eq!(
            "\u{1b}[30mXT\u{1b}[39m \u{1b}[31mT\u{1b}[39m",
            colored_s.cut(2..6)
        );

        assert_eq!("TEXT TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEXT T", srip_ansi_sequences(&colored_s.cut(..6)));
        assert_eq!("XT T", srip_ansi_sequences(&colored_s.cut(2..6)));

        assert_eq!("\u{1b}[30m\u{1b}[39m", cut("\u{1b}[30m\u{1b}[39m", ..));
    }

    #[test]
    fn cut_colored_bg_test() {
        let colored_s = "\u{1b}[40mTEXT\u{1b}[49m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..4));
        assert_eq!("\u{1b}[40mEXT\u{1b}[49m", colored_s.cut(1..));
        assert_eq!("\u{1b}[40mTEX\u{1b}[49m", colored_s.cut(..3));
        assert_eq!("\u{1b}[40mEX\u{1b}[49m", colored_s.cut(1..3));

        // todo: determine if this is the right behaviour
        assert_eq!("\u{1b}[40m\u{1b}[49m", colored_s.cut(3..3));

        assert_eq!("TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEX", srip_ansi_sequences(&colored_s.cut(..3)));
        assert_eq!("EX", srip_ansi_sequences(&colored_s.cut(1..3)));

        let colored_s = "\u{1b}[40mTEXT\u{1b}[49m \u{1b}[41mTEXT\u{1b}[49m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..9));
        assert_eq!(
            "\u{1b}[40mXT\u{1b}[49m \u{1b}[41mTEXT\u{1b}[49m",
            colored_s.cut(2..)
        );
        assert_eq!(
            "\u{1b}[40mTEXT\u{1b}[49m \u{1b}[41mT\u{1b}[49m",
            colored_s.cut(..6)
        );
        assert_eq!(
            "\u{1b}[40mXT\u{1b}[49m \u{1b}[41mT\u{1b}[49m",
            colored_s.cut(2..6)
        );

        assert_eq!("TEXT TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEXT T", srip_ansi_sequences(&colored_s.cut(..6)));
        assert_eq!("XT T", srip_ansi_sequences(&colored_s.cut(2..6)));

        assert_eq!("\u{1b}[40m\u{1b}[49m", cut("\u{1b}[40m\u{1b}[49m", ..));
    }

    #[test]
    fn cut_colored_bg_fg_test() {
        let colored_s = "\u{1b}[31;40mTEXT\u{1b}[0m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..4));
        assert_eq!("\u{1b}[31;40mEXT\u{1b}[0m", colored_s.cut(1..));
        assert_eq!("\u{1b}[31;40mTEX\u{1b}[39m\u{1b}[49m", colored_s.cut(..3));
        assert_eq!("\u{1b}[31;40mEX\u{1b}[39m\u{1b}[49m", colored_s.cut(1..3));

        assert_eq!("TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEX", srip_ansi_sequences(&colored_s.cut(..3)));
        assert_eq!("EX", srip_ansi_sequences(&colored_s.cut(1..3)));

        let colored_s = "\u{1b}[31;40mTEXT\u{1b}[0m \u{1b}[34;42mTEXT\u{1b}[0m";
        assert_eq!(colored_s, colored_s.cut(..));
        assert_eq!(colored_s, colored_s.cut(0..9));
        assert_eq!(
            "\u{1b}[31;40mXT\u{1b}[0m \u{1b}[34;42mTEXT\u{1b}[0m",
            colored_s.cut(2..)
        );
        assert_eq!(
            "\u{1b}[31;40mTEXT\u{1b}[0m \u{1b}[34;42mT\u{1b}[39m\u{1b}[49m",
            colored_s.cut(..6)
        );
        assert_eq!(
            "\u{1b}[31;40mXT\u{1b}[0m \u{1b}[34;42mT\u{1b}[39m\u{1b}[49m",
            colored_s.cut(2..6)
        );

        assert_eq!("TEXT TEXT", srip_ansi_sequences(&colored_s.cut(..)));
        assert_eq!("TEXT T", srip_ansi_sequences(&colored_s.cut(..6)));
        assert_eq!("XT T", srip_ansi_sequences(&colored_s.cut(2..6)));

        assert_eq!("\u{1b}[40m\u{1b}[49m", cut("\u{1b}[40m\u{1b}[49m", ..));
    }

    #[test]
    fn cut_keep_general_color_test() {
        assert_eq!(
            "\u{1b}[41m\u{1b}[30m\u{1b}[39m \u{1b}[34m12\u{1b}[39m\u{1b}[49m",
            "\u{1b}[41m\u{1b}[30msomething\u{1b}[39m \u{1b}[34m123123\u{1b}[39m\u{1b}[49m"
                .cut(9..12)
        );
    }

    #[test]
    fn cut_no_colored_str() {
        assert_eq!("something", cut("something", ..));
        assert_eq!("som", cut("something", ..3));
        assert_eq!("some", cut("something", ..=3));
        assert_eq!("et", cut("something", 3..5));
        assert_eq!("eth", cut("something", 3..=5));
        assert_eq!("ething", cut("something", 3..));
        assert_eq!("something", cut("something", ..));
        assert_eq!("", cut("", ..));
    }

    #[test]
    fn dont_panic_on_exceeding_upper_bound() {
        assert_eq!("TEXT", cut("TEXT", ..50));
        assert_eq!("EXT", cut("TEXT", 1..50));
        assert_eq!(
            "\u{1b}[31;40mTEXT\u{1b}[0m",
            cut("\u{1b}[31;40mTEXT\u{1b}[0m", ..50)
        );
        assert_eq!(
            "\u{1b}[31;40mEXT\u{1b}[0m",
            cut("\u{1b}[31;40mTEXT\u{1b}[0m", 1..50)
        );
    }

    #[test]
    fn dont_panic_on_exceeding_lower_bound() {
        assert_eq!("", cut("TEXT", 10..));
        assert_eq!("", cut("TEXT", 10..50));
    }

    #[test]
    #[should_panic = "One of indexes are not on a UTF-8 code point boundary"]
    fn cut_a_mid_of_emojie_2_test() {
        cut("😀", 1..2);
    }

    #[test]
    #[should_panic = "One of indexes are not on a UTF-8 code point boundary"]
    fn cut_a_mid_of_emojie_1_test() {
        cut("😀", 1..);
    }

    #[test]
    #[should_panic = "One of indexes are not on a UTF-8 code point boundary"]
    fn cut_a_mid_of_emojie_0_test() {
        cut("😀", ..1);
    }

    #[test]
    fn cut_emojies_test() {
        let emojes = "😀😃😄😁😆😅😂🤣🥲😊";
        assert_eq!(emojes, emojes.cut(..));
        assert_eq!("😀", emojes.cut(..4));
        assert_eq!("😃😄", emojes.cut(4..12));
        assert_eq!("🤣🥲😊", emojes.cut(emojes.find('🤣').unwrap()..));
    }

    #[test]
    // todo: We probably need to fix it.
    fn cut_colored_x_x_test() {
        assert_ne!("", cut("\u{1b}[31;40mTEXT\u{1b}[0m", 3..3));
        assert_ne!(
            "",
            cut(
                "\u{1b}[31;40mTEXT\u{1b}[0m \u{1b}[34;42mTEXT\u{1b}[0m",
                1..1
            )
        );
        assert_ne!("", cut("\u{1b}[31;40mTEXT\u{1b}[0m", ..0));
    }

    #[test]
    fn cut_partially_colored_str_test() {
        let s = "zxc_\u{1b}[31;40mTEXT\u{1b}[0m_qwe";
        assert_eq!("zxc", s.cut(..3));
        assert_eq!("zxc_\u{1b}[31;40mT\u{1b}[39m\u{1b}[49m", s.cut(..5));
        assert_eq!("\u{1b}[31;40mEXT\u{1b}[0m_q", s.cut(5..10));
        assert_eq!("\u{1b}[31;40m\u{1b}[0m", s.cut(12..));
    }

    #[test]
    fn chunks_not_colored_test() {
        assert_eq!(
            vec!["som".to_string(), "eth".to_string(), "ing".to_string()],
            chunks("something", 3)
        );
        assert_eq!(
            vec![
                "so".to_string(),
                "me".to_string(),
                "th".to_string(),
                "in".to_string(),
                "g".to_string()
            ],
            chunks("something", 2)
        );
        assert_eq!(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            chunks("abc", 1)
        );
        assert_eq!(vec!["something".to_string()], chunks("something", 99));
    }

    #[test]
    #[should_panic]
    fn chunks_panic_when_n_is_zero() {
        chunks("something", 0);
    }

    #[test]
    fn chunks_colored() {
        let text = "\u{1b}[31;40mTEXT\u{1b}[0m";
        assert_eq!(
            vec![
                "\u{1b}[31;40mT\u{1b}[39m\u{1b}[49m",
                "\u{1b}[31;40mE\u{1b}[39m\u{1b}[49m",
                "\u{1b}[31;40mX\u{1b}[39m\u{1b}[49m",
                "\u{1b}[31;40mT\u{1b}[0m"
            ],
            chunks(text, 1)
        );
        assert_eq!(
            vec![
                "\u{1b}[31;40mTE\u{1b}[39m\u{1b}[49m",
                "\u{1b}[31;40mXT\u{1b}[0m"
            ],
            chunks(text, 2)
        );
        assert_eq!(
            vec![
                "\u{1b}[31;40mTEX\u{1b}[39m\u{1b}[49m",
                "\u{1b}[31;40mT\u{1b}[0m"
            ],
            chunks(text, 3)
        );
    }

    #[test]
    fn chunk_emojies_test() {
        let emojes = "😀😃😄😁😆😅😂🤣🥲😊";
        assert_eq!(
            vec!["😀", "😃", "😄", "😁", "😆", "😅", "😂", "🤣", "🥲", "😊",],
            chunks(emojes, 1)
        );
        assert_eq!(
            vec!["😀😃", "😄😁", "😆😅", "😂🤣", "🥲😊",],
            chunks(emojes, 2)
        );
        assert_eq!(vec!["😀😃😄", "😁😆😅", "😂🤣🥲", "😊",], chunks(emojes, 3));
        assert_eq!(vec!["😀😃😄😁", "😆😅😂🤣", "🥲😊",], chunks(emojes, 4));
        assert_eq!(vec!["😀😃😄😁😆", "😅😂🤣🥲😊",], chunks(emojes, 5));
        assert_eq!(vec!["😀😃😄😁😆😅", "😂🤣🥲😊",], chunks(emojes, 6));
        assert_eq!(vec!["😀😃😄😁😆😅😂", "🤣🥲😊",], chunks(emojes, 7));
        assert_eq!(vec!["😀😃😄😁😆😅😂🤣", "🥲😊",], chunks(emojes, 8));
        assert_eq!(vec!["😀😃😄😁😆😅😂🤣🥲", "😊",], chunks(emojes, 9));
        assert_eq!(vec!["😀😃😄😁😆😅😂🤣🥲😊"], chunks(emojes, 10));
        assert_eq!(vec!["😀😃😄😁😆😅😂🤣🥲😊"], chunks(emojes, 11));
    }
}
