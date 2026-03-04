# fret-cookbook

Small, topic-focused runnable examples for learning Fret (cookbook-style).

This crate intentionally favors:

- tiny files (one concept per example),
- the ecosystem entry surface (`fret` + shadcn),
- stable `test_id` naming where interactive automation is expected.

## Labels

- **Official**: boring, stable, onboarding-friendly; should compile fast and avoid optional subsystems.
- **Lab**: higher ceiling; feature-gated to keep cold compile time down.

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

- State + derived state: `virtual_list_basics`, `undo_basics` (feature-gated), `async_inbox_basics` (feature-gated)
- Theming + assets: `theme_switching_basics`, `icons_and_assets_basics` (feature-gated)
- Rendering/effects: `effects_layer_basics`, `customv1_basics` (feature-gated)
- Interop (higher ceiling): `embedded_viewport_basics` (feature-gated), `external_texture_import_basics` (feature-gated), `docking_basics` (feature-gated)

## Run (native)

```bash
cargo run -p fret-cookbook --example hello
cargo run -p fret-cookbook --example simple_todo
cargo run -p fret-cookbook --example hello_counter
cargo run -p fret-cookbook --example overlay_basics
cargo run -p fret-cookbook --example commands_keymap_basics
cargo run -p fret-cookbook --example text_input_basics
cargo run -p fret-cookbook --example effects_layer_basics
cargo run -p fret-cookbook --example theme_switching_basics
```

## Optional feature-gated examples

Some examples are kept behind Cargo features to reduce cold compile time.

- Official (keep it boring):
  - `cargo run -p fret-cookbook --example hello`
  - `cargo run -p fret-cookbook --example simple_todo`
  - `cargo run -p fret-cookbook --example overlay_basics`
  - `cargo run -p fret-cookbook --example text_input_basics`
  - `cargo run -p fret-cookbook --example commands_keymap_basics`
  - `cargo run -p fret-cookbook --example theme_switching_basics`
  - `cargo run -p fret-cookbook --example virtual_list_basics`
  - `cargo run -p fret-cookbook --example effects_layer_basics`

- Lab (feature-gated):
  - Icons + assets:
    - `cargo run -p fret-cookbook --features cookbook-assets --example icons_and_assets_basics`
  - Undo history:
    - `cargo run -p fret-cookbook --features cookbook-undo --example undo_basics`
  - Async inbox / dispatcher:
    - `cargo run -p fret-cookbook --features cookbook-async --example async_inbox_basics`
  - IMUI + GenUI interop:
    - `cargo run -p fret-cookbook --features cookbook-imui --example imui_action_basics`
  - Docking:
    - `cargo run -p fret-cookbook --features cookbook-docking --example docking_basics`
  - Embedded viewport / external textures:
    - `cargo run -p fret-cookbook --features cookbook-interop --example embedded_viewport_basics`
    - `cargo run -p fret-cookbook --features cookbook-interop --example external_texture_import_basics`
  - Custom renderer probes:
    - `cargo run -p fret-cookbook --features cookbook-customv1 --example customv1_basics`
  - Window materials diagnostics:
    - `cargo run -p fret-cookbook --features cookbook-bootstrap --example utility_window_materials_windows`
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
