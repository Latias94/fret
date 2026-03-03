# fret-cookbook

Small, topic-focused runnable examples for learning Fret (cookbook-style).

This crate intentionally favors:

- tiny files (one concept per example),
- the ecosystem entry surface (`fret` + shadcn),
- stable `test_id` naming where interactive automation is expected.

## Run (native)

```bash
cargo run -p fret-cookbook --example hello
cargo run -p fret-cookbook --example simple_todo
cargo run -p fret-cookbook --example hello_counter
cargo run -p fret-cookbook --example overlay_basics
cargo run -p fret-cookbook --example commands_keymap_basics
cargo run -p fret-cookbook --example text_input_basics
cargo run -p fret-cookbook --example imui_action_basics
cargo run -p fret-cookbook --example effects_layer_basics
cargo run -p fret-cookbook --example theme_switching_basics
cargo run -p fret-cookbook --example utility_window_materials_windows
```

Notes:

- The recommended golden path examples use the view runtime + typed actions.
- Some advanced/legacy examples still use MVU for now; see
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`.

For faster iteration (skip linking):

```bash
cargo check -p fret-cookbook --example hello
```
