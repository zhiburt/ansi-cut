# ansi-cut

A library for cutting a string while preserving colors.

## Example

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

### Question

Are any other usefull ansi sequense that would be usefull to keep in mind?