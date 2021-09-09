# ansi-cut [![Build Status](https://github.com/zhiburt/ansi-cut/actions/workflows/ci.yml/badge.svg?style=for-the-badge)](https://github.com/zhiburt/ansi-cut/actions) [![codecov](https://codecov.io/gh/zhiburt/ansi-cut/branch/main/graph/badge.svg?token=ZUCG3Q9F1I)](https://codecov.io/gh/zhiburt/ansi-cut) [![Crate](https://img.shields.io/crates/v/ansi-cut)](https://crates.io/crates/ansi-cut) [![docs.rs](https://img.shields.io/docsrs/ansi_cut?color=blue)](https://docs.rs/ansi_cut/0.1.0/ansi_cut/) [![license](https://img.shields.io/crates/l/ansi-cut)](./LICENSE.txt)


A library for cutting a string while preserving colors.

## Cut

```rust
use ansi_cut::AnsiCut;
use owo_colors::{colors::*, OwoColorize};

pub fn main() {
    let colored_text = "When the night has come"
        .fg::<Black>()
        .bg::<White>()
        .to_string();
    let cutted_text = colored_text.cut(5..);

    println!("{}", cutted_text);
}
```

## Chunks

```rust
use owo_colors::{colors::*, OwoColorize};

pub fn main() {
    let colored_text = "When the night has come"
        .fg::<Black>()
        .bg::<White>()
        .to_string();

    let chunks = ansi_cut::chunks(colored_text, 5);
}
```

### Question

Are any other usefull ansi sequense that would be usefull to keep in mind?