# Examples (Index)

This repo has multiple “example-shaped” entry points. This page is the shortest path to choosing
the right one.

## 0) Boring ladder (recommended)

These are intentionally stable and should be your default onboarding path:

1. `hello` (template) — smallest runnable UI surface.
   - Generate: `cargo run -p fretboard -- new hello --name hello-world`
2. `simple-todo` (template) — view runtime + typed actions + models + keyed lists (no selectors/queries).
   - Generate: `cargo run -p fretboard -- new simple-todo --name my-simple-todo`
3. `todo` (template) — “best practice baseline” (selectors + queries).
   - Generate: `cargo run -p fretboard -- new todo --name my-todo`
   - Read: `docs/examples/todo-app-golden-path.md`

## 1) In-tree Cookbook (small, focused lessons)

Cookbook examples live under `apps/fret-cookbook/examples/`. Each file is intended to be one
lesson. They are runnable and designed for copy/paste.

Run one directly:

```bash
cargo run -p fret-cookbook --example simple_todo
```

Or use the tooling runner (recommended on Windows):

```bash
cargo run -p fretboard -- dev native --example simple_todo
```

Recommended starting points (action-first + view runtime):

- `hello`, `simple_todo`, `hello_counter`
- `virtual_list_basics` (virtualization + keyed identity + reordering)
- `icons_and_assets_basics` (semantic icon ids + svg/image loading + reload epoch)
- `effects_layer_basics` (EffectLayer + EffectChain: pixelate/blur)
- `markdown_and_code_basics` (markdown preview + fenced code blocks)
- `canvas_pan_zoom_basics` (canvas pan/zoom wiring + pointer capture)
- `commands_keymap_basics`, `overlay_basics`, `text_input_basics`
- `imui_action_basics` (cross-frontend action dispatch)

Some cookbook examples still use legacy MVU for now. Track remaining in-tree usage here:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`

Diagnostics scripts for cookbook examples live under:

- `tools/diag-scripts/cookbook/*`
- `tools/diag-scripts/suites/cookbook-*` (redirect stubs)

Note: cookbook examples are separate binaries today, so scripts are per-example (not one “mega suite”).

## 2) UI Gallery (component catalog + conformance)

The UI gallery is a larger, multi-page app intended for component discovery and parity testing.

- Native: `cargo run -p fret-ui-gallery`
- Web: `cargo run -p fretboard -- dev web --demo ui_gallery`

## 3) Labs / maintainer harnesses

Some demos are intentionally “high ceiling” (docking arbitration, renderer effects, node graph
stress). They are useful for maintainers and advanced users but are not the onboarding path.

Start from the “Examples redesign” workstream for the intended product surface:

- `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
