# fret-cookbook

Small, topic-focused runnable examples for learning Fret (cookbook-style).

This crate intentionally favors:

- tiny files (one concept per example),
- the ecosystem entry surface (`fret` + shadcn),
- stable `test_id` naming where interactive automation is expected.

## Labels

- **Official**: boring, stable, onboarding-friendly; should compile fast and avoid optional subsystems.
- **Lab**: higher ceiling; feature-gated to keep cold compile time down.

Repo-wide taxonomy mapping:

- **Default**: `Official` examples that belong on the onboarding ladder
- **Comparison**: explicit side-by-side evidence surfaces such as `simple_todo_v2_target`
- **Advanced**: `Lab` examples plus any interop/renderer/docking-oriented `Official` references that
  are useful after the boring ladder, not before it

Full index (Bevy-style table of contents): [apps/fret-cookbook/EXAMPLES.md](./EXAMPLES.md)

## Recommended order (boring ladder)

Repo-wide default ladder:

1. `hello` (template)
2. `simple-todo` (template)
3. `todo` (template / richer baseline)

Cookbook fits immediately after the first two rungs: use it for focused, boring lessons before
jumping into the gallery/reference surfaces.

Start with these cookbook lessons before jumping into the UI Gallery:

```bash
cargo run -p fretboard -- dev native --example hello
cargo run -p fretboard -- dev native --example simple_todo
cargo run -p fretboard -- dev native --example overlay_basics
cargo run -p fretboard -- dev native --example text_input_basics
cargo run -p fretboard -- dev native --example commands_keymap_basics
```

If you need the richer third rung after that ladder, read:

- `docs/examples/todo-app-golden-path.md`

Then pick a topic:

Comparison target (reference-only; not part of the boring ladder):

- `cargo run -p fretboard -- dev native --example simple_todo_v2_target`
  - keeps a tiny keyed todo list on the same `LocalState<Vec<_>>` + payload-action path now used by `simple_todo`, `todo_demo`, and the simple-todo scaffold, so its role is to stay as a denser comparison surface for row-handler placement rather than a missing-default-path preview

- Default follow-up topics: `hello_counter`, `form_basics`, `virtual_list_basics`
- State + derived state: `undo_basics` (feature-gated), `async_inbox_basics` (feature-gated)
- Queries (feature-gated): `query_basics`
- Routing (feature-gated): `router_basics`
- Theming + assets: `theme_switching_basics`, `icons_and_assets_basics` (feature-gated)
- App patterns: `toast_basics`, `form_basics`, `date_picker_basics`, `drag_basics`
- Rendering/effects: `effects_layer_basics`, `customv1_basics` (feature-gated)
- Interop (higher ceiling): `embedded_viewport_basics` (feature-gated), `external_texture_import_basics` (feature-gated), `docking_basics` (feature-gated)

## Run (native)

```bash
cargo run -p fretboard -- dev native --example hello
cargo run -p fretboard -- dev native --example simple_todo
cargo run -p fretboard -- dev native --example hello_counter
cargo run -p fretboard -- dev native --example overlay_basics
cargo run -p fretboard -- dev native --example commands_keymap_basics
cargo run -p fretboard -- dev native --example text_input_basics
cargo run -p fretboard -- dev native --example effects_layer_basics
cargo run -p fretboard -- dev native --example theme_switching_basics
```

## Optional feature-gated examples

Some examples are kept behind Cargo features to reduce cold compile time.
Tip: if you run examples via `fretboard dev native --example <name>`, `fretboard` will auto-enable
required cookbook features for known Lab examples and print what it enabled.

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
  - Query basics:
    - `cargo run -p fret-cookbook --features cookbook-query --example query_basics`
  - Router basics:
    - `cargo run -p fret-cookbook --features cookbook-router --example router_basics`
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
- If you are choosing where to start, prefer the boring ladder here first and leave `Lab` /
  comparison surfaces for after `hello` + `simple_todo`.
- Cookbook examples intentionally avoid teaching legacy MVU. The legacy MVU inventory applies to
  maintainer demos (non-onboarding):
  [docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md](../../docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md).

## Diagnostics (optional)

Fret includes an optional diagnostics + scripted UI automation toolchain (`fretboard diag`).
If you are new to it, start with the `hello` example because it already exposes stable `test_id`s.

Run a small script that clicks the button and asserts the count label updates:

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json \
  --launch -- cargo run -p fret-cookbook --features cookbook-diag --example hello
```

Note: `--features cookbook-diag` enables the optional `fret/diagnostics` integration for the example binary.

How it works (mental model):

- Your UI attaches stable `test_id`s (e.g. `cookbook.hello.button`).
- The script targets those IDs (and/or role/name selectors) and drives input.
- The runner records a diagnostics bundle you can inspect/share.

Next step (still boring): run the `simple_todo` smoke script (includes a screenshot + bundle):

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag run tools/diag-scripts/cookbook/simple-todo/cookbook-simple-todo-smoke.json \
  --launch -- cargo run -p fret-cookbook --features cookbook-diag --example simple_todo
```

For faster iteration (skip linking):

```bash
cargo check -p fret-cookbook --example hello
```
