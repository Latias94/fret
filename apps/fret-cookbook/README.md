# fret-cookbook

Small, topic-focused runnable examples for learning Fret (cookbook-style).

This crate intentionally favors:

- tiny files (one concept per example),
- the ecosystem entry surface (`fret` + shadcn),
- stable `test_id` naming where interactive automation is expected.

## Recommended order (boring ladder)

Start with these before jumping into the UI Gallery:

```bash
cargo run -p fret-cookbook --example hello
cargo run -p fret-cookbook --example simple_todo
cargo run -p fret-cookbook --example overlay_basics
cargo run -p fret-cookbook --example text_input_basics
cargo run -p fret-cookbook --example commands_keymap_basics
```

Then pick a topic:

- State + derived state: `undo_basics`, `async_inbox_basics`, `virtual_list_basics`
- Theming + assets: `theme_switching_basics`, `icons_and_assets_basics`
- Rendering/effects: `effects_layer_basics`, `customv1_basics`
- Interop (higher ceiling): `embedded_viewport_basics`, `external_texture_import_basics`, `docking_basics`

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

## Optional feature-gated examples

Some examples are kept behind Cargo features to reduce cold compile time.

- Markdown + code view:
  - `cargo run -p fret-cookbook --features cookbook-markdown --example markdown_and_code_basics`
- Canvas:
  - `cargo run -p fret-cookbook --features cookbook-canvas --example canvas_pan_zoom_basics`
- Charts:
  - `cargo run -p fret-cookbook --features cookbook-chart --example chart_interactions_basics`
- Gizmo:
  - `cargo run -p fret-cookbook --features cookbook-gizmo --example gizmo_basics`

Notes:

- The recommended golden path examples use the view runtime + typed actions.
- Some advanced/legacy examples still use MVU for now; see
  `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`.

For faster iteration (skip linking):

```bash
cargo check -p fret-cookbook --example hello
```
