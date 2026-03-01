# fret-cookbook

Small, topic-focused runnable examples for learning Fret (cookbook-style).

This crate intentionally favors:

- tiny files (one concept per example),
- the ecosystem entry surface (`fret` + shadcn),
- stable `test_id` naming where interactive automation is expected.

## Run (native)

```bash
cargo run -p fret-cookbook --example hello
cargo run -p fret-cookbook --example hello_counter
cargo run -p fret-cookbook --example overlay_basics
cargo run -p fret-cookbook --example commands_keymap_basics
cargo run -p fret-cookbook --example text_input_basics
cargo run -p fret-cookbook --example effects_layer_basics
cargo run -p fret-cookbook --example theme_switching_basics
```
