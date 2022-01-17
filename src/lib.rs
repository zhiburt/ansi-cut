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
    /// 
    /// Range is defined in terms of `char`s of the string not containing ANSI
    /// control sequences.
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

pub fn chunks(s: &str, n: usize) -> Vec<String> {
    assert!(n > 0);

    let length = s.chars().count();
    let mut acc = Vec::new();
    let mut start_index = 0;
    while start_index < length {
        let part = s.cut(start_index..start_index + n);
        start_index += n;
        if str_len(&part) == 0 {
            continue;
        }

        acc.push(part);
    }

    acc
}

fn cut<S, R>(string: S, bounds: R) -> String
where
    S: AsRef<str>,
    R: RangeBounds<usize>,
{
    let string = string.as_ref();
    let string_width = str_len(string);
    let (start, end) = bounds_to_usize(bounds.start_bound(), bounds.end_bound(), string_width);

    assert!(start <= end, "Starting character index exceeds the last character index! Make sure to use character indices instead of byte indices!");
    // assert!(end <= string_width);

    cut_str(string, start, end)
}

fn cut_str(string: &str, start: usize, end: usize) -> String {
    let parsed = string.ansi_parse();

    let mut index = start;
    let mut need = end - start;
    let mut buffer = String::with_capacity(start + end);
    let mut escapes = Vec::new();
    for block in parsed.into_iter() {
        match block {
            Output::TextBlock(text) => {
                if need == 0 {
                    break;
                }

                let block_len = str_len(text);
                let is_nesessary_block = index < block_len;
                if is_nesessary_block {
                    escapes
                        .iter()
                        .for_each(|esc: &AnsiSequence| buffer.push_str(esc.to_string().as_str()));

                    let taken_chars = std::cmp::min(need, block_len - index);
                    let block = text.chars().skip(index).take(taken_chars);

                    buffer.extend(block);

                    need -= taken_chars;
                    index = 0;

                    if escapes.is_empty() && need == 0 {
                        break;
                    }
                } else {
                    index -= block_len;
                }

                escapes.clear();
            }
            Output::Escape(seq) => {
                if let AnsiSequence::SetGraphicsMode(_) = seq {
                    escapes.push(seq);
                }
            }
        }
    }

    escapes
        .iter()
        .for_each(|esc: &AnsiSequence| buffer.push_str(esc.to_string().as_str()));

    buffer
}

fn str_len(string: &str) -> usize {
    let bytes = strip_ansi_escapes::strip(string).expect("Wierd things happen");
    let string = String::from_utf8_lossy(&bytes);
    let len = string.chars().count();
    len
}

fn bounds_to_usize(
    left: Bound<&usize>,
    right: Bound<&usize>,
    count_elements: usize,
) -> (usize, usize) {
    match (left, right) {
        (Bound::Included(x), Bound::Included(y)) => (*x, y + 1),
        (Bound::Included(x), Bound::Excluded(y)) => (*x, *y),
        (Bound::Included(x), Bound::Unbounded) => (*x, count_elements),
        (Bound::Unbounded, Bound::Unbounded) => (0, count_elements),
        (Bound::Unbounded, Bound::Included(y)) => (0, y + 1),
        (Bound::Unbounded, Bound::Excluded(y)) => (0, *y),
        (Bound::Excluded(_), Bound::Unbounded)
        | (Bound::Excluded(_), Bound::Included(_))
        | (Bound::Excluded(_), Bound::Excluded(_)) => {
            unreachable!("A start bound can't be excluded")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use owo_colors::{colors::*, OwoColorize};

    #[test]
    fn cut_str() {
        assert_eq!("som", cut("something", ..3));
        assert_eq!("some", cut("something", ..=3));
        assert_eq!("et", cut("something", 3..5));
        assert_eq!("eth", cut("something", 3..=5));
        assert_eq!("ething", cut("something", 3..));
        assert_eq!("something", cut("something", ..));
        assert_eq!("", cut("", ..));
    }

    #[test]
    fn dont_panic_on_index_higher_then_length() {
        assert_eq!("som", cut("som", ..5));
    }

    #[test]
    fn cut_colored_str() {
        let s = "something".fg::<Black>().bg::<Blue>().to_string();
        assert_eq!("som".fg::<Black>().bg::<Blue>().to_string(), cut(&s, ..3));
        assert_eq!("some".fg::<Black>().bg::<Blue>().to_string(), cut(&s, ..=3));
        assert_eq!("et".fg::<Black>().bg::<Blue>().to_string(), cut(&s, 3..5));
        assert_eq!("eth".fg::<Black>().bg::<Blue>().to_string(), cut(&s, 3..=5));
        assert_eq!(
            "ething".fg::<Black>().bg::<Blue>().to_string(),
            cut(&s, 3..)
        );
    }

    #[test]
    fn cut_partially_colored_str() {
        let s = format!("zxc_{}_qwe", "something".fg::<Black>().bg::<Blue>());
        assert_eq!("zxc", cut(&s, ..3));
        assert_eq!(
            format!("zxc_{}", "s".fg::<Black>().bg::<Blue>()),
            cut(&s, ..5)
        );
        assert_eq!(
            "ometh".fg::<Black>().bg::<Blue>().to_string(),
            cut(&s, 5..10)
        );
        assert_eq!(
            format!("{}_qwe", "g".fg::<Black>().bg::<Blue>()),
            cut(&s, 12..)
        );
    }

    #[test]
    fn cut_emojies() {
        assert_eq!("😀😃😄", cut("😀😃😄😁😆😅😂🤣🥲😊", ..3));
        assert_eq!("😅😂", cut("😀😃😄😁😆😅😂🤣🥲😊", 5..7));
        assert_eq!("😊", cut("😀😃😄😁😆😅😂🤣🥲😊", 9..));
        assert_eq!("🧑‍🏭", cut("🧑‍🏭🧑‍🏭🧑‍🏭", ..3));
    }

    #[test]
    fn chunks_test() {
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
        let s = "something".fg::<Black>().bg::<Blue>().to_string();
        assert_eq!(
            vec![
                "som".fg::<Black>().bg::<Blue>().to_string(),
                "eth".fg::<Black>().bg::<Blue>().to_string(),
                "ing".fg::<Black>().bg::<Blue>().to_string()
            ],
            chunks(&s, 3)
        );
        assert_eq!(
            vec![
                "so".fg::<Black>().bg::<Blue>().to_string(),
                "me".fg::<Black>().bg::<Blue>().to_string(),
                "th".fg::<Black>().bg::<Blue>().to_string(),
                "in".fg::<Black>().bg::<Blue>().to_string(),
                "g".fg::<Black>().bg::<Blue>().to_string()
            ],
            chunks(&s, 2)
        );
        assert_eq!(
            vec![
                "s".fg::<Black>().bg::<Blue>().to_string(),
                "o".fg::<Black>().bg::<Blue>().to_string(),
                "m".fg::<Black>().bg::<Blue>().to_string(),
                "e".fg::<Black>().bg::<Blue>().to_string(),
                "t".fg::<Black>().bg::<Blue>().to_string(),
                "h".fg::<Black>().bg::<Blue>().to_string(),
                "i".fg::<Black>().bg::<Blue>().to_string(),
                "n".fg::<Black>().bg::<Blue>().to_string(),
                "g".fg::<Black>().bg::<Blue>().to_string(),
            ],
            chunks(&s, 1)
        );
        assert_eq!(vec![s.clone()], chunks(&s, 99));
    }
}
