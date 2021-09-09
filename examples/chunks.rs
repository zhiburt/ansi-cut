use owo_colors::{colors::*, OwoColorize};

pub fn main() {
    let colored_text = format!(
        "{hello} {world}",
        hello = "Hello".fg::<Black>().bg::<White>(),
        world = "World".fg::<Magenta>().bg::<Green>(),
    );

    println!("text={}", colored_text);

    println!("chunks");
    for chunk in ansi_cut::chunks(&colored_text, 4) {
        println!("{}", chunk);
    }
}
