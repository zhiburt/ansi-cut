use ansi_cut::AnsiCut;
use owo_colors::{colors::*, OwoColorize};

pub fn main() {
    let colored_text = format!(
        "{hello} {world}",
        hello = "Hello".fg::<Black>().bg::<White>(),
        world = "World".fg::<Magenta>().bg::<Green>(),
    );

    println!("{}", colored_text);

    let cutted_text = colored_text.cut(4..8);

    println!("{}", cutted_text);
}
