use ansi_parser::AnsiSequence;
use ansi_parser::{AnsiParser, Output};
use std::ops::{Bound, RangeBounds};

pub trait AnsiCut {
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

fn cut<S, R>(string: S, bounds: R) -> String
where
    S: AsRef<str>,
    R: RangeBounds<usize>,
{
    let string = string.as_ref();
    let string_width = str_len(string);
    let (start, end) = bounds_to_usize(bounds.start_bound(), bounds.end_bound(), string_width);

    assert!(start <= end);
    assert!(end <= string_width);

    cut_str(string, start, end)
}

fn cut_str(string: &str, start: usize, end: usize) -> String {
    let parsed: Vec<Output> = string.ansi_parse().collect();

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
    #[should_panic]
    fn panic_on_index_higher_then_length() {
        cut("qwe", ..5);
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
}
