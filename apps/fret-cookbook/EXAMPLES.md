# Cookbook examples (Index)

This page is a **Bevy-style index** of all runnable cookbook examples under
[`apps/fret-cookbook/examples/`](./examples/).

Design goals:

- The cookbook can be large, but it should never feel chaotic.
- The default onboarding path stays short and stable (see [`apps/fret-cookbook/README.md`](./README.md)).
- Higher-ceiling topics are feature-gated to keep cold compile time down.

Taxonomy guide:

- **Default**: the short, boring ladder plus stable copy/paste-ready lessons
- **Comparison**: explicit side-by-side evidence surfaces such as `simple_todo_v2_target`
- **Advanced**: feature-gated routing/query/table/interop/renderer/editor-grade topics

If you are new to the repo, stay on the Default sections first and treat the Advanced sections
below as later reference material.

Repo-wide ladder reminder:

1. `hello` (template)
2. `simple-todo` (template)
3. `todo` (template / richer baseline)

This cookbook index starts after the first two rungs and is meant to provide focused follow-up
lessons, not replace the ladder itself.

How to run an example (recommended):

```bash
cargo run -p fretboard-dev -- dev native --example <name>
```

How to run an example (direct):

```bash
cargo run -p fret-cookbook --example <name>
```

Note: some higher-ceiling examples are feature-gated when running directly. If you run via
`fretboard-dev dev native --example <name>`, `fretboard-dev` will auto-enable required cookbook features
for known Lab examples and print what it enabled.

Diagnostics (optional, scripted smoke):

- If an example has a suite listed below, you can run it with:

```bash
FRET_DIAG=1 cargo run -p fretboard-dev -- diag suite <suite-name> --launch -- \
  cargo run -p fret-cookbook --features cookbook-diag --example <name>
```

Notes:

- Most suites are “smoke + screenshot + bundle”.
- Some examples require additional features beyond `cookbook-diag` (listed below).

## Default — the bare minimum

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`hello.rs`](./examples/hello.rs) | Official | `cargo run -p fretboard-dev -- dev native --example hello` | `cookbook-hello`
[`simple_todo.rs`](./examples/simple_todo.rs) | Official | `cargo run -p fretboard-dev -- dev native --example simple_todo` | `cookbook-simple-todo`

## Default — Core UI + input + commands

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`overlay_basics.rs`](./examples/overlay_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example overlay_basics` | `cookbook-overlay-basics`
[`text_input_basics.rs`](./examples/text_input_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example text_input_basics` | `cookbook-text-input-basics`
[`commands_keymap_basics.rs`](./examples/commands_keymap_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example commands_keymap_basics` | `cookbook-commands-keymap-basics`

## Default — App patterns (small, copy/paste-ready)

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`toast_basics.rs`](./examples/toast_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example toast_basics` | `cookbook-toast-basics`
[`date_picker_basics.rs`](./examples/date_picker_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example date_picker_basics` | `cookbook-date-picker-basics`
[`form_basics.rs`](./examples/form_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example form_basics` | `cookbook-form-basics`
[`drag_basics.rs`](./examples/drag_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example drag_basics` | `cookbook-drag-basics`
[`hello_counter.rs`](./examples/hello_counter.rs) | Official | `cargo run -p fretboard-dev -- dev native --example hello_counter` | `cookbook-hello-counter`

## Advanced — Async submit / feedback (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`mutation_toast_feedback_basics.rs`](./examples/mutation_toast_feedback_basics.rs) | Lab | `cookbook-mutation` | `cargo run -p fretboard-dev -- dev native --example mutation_toast_feedback_basics` | `cookbook-mutation-toast-feedback-basics`

## Comparison targets (reference-only)

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`simple_todo_v2_target.rs`](./examples/simple_todo_v2_target.rs) | Comparison (payload-row / handler-placement) | `cargo run -p fretboard-dev -- dev native --example simple_todo_v2_target` | -

## Advanced — Routing (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`router_basics.rs`](./examples/router_basics.rs) | Lab | `cookbook-router` | `cargo run -p fretboard-dev -- dev native --example router_basics` | `cookbook-router-basics`

## Default follow-up — State + derived state

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`virtual_list_basics.rs`](./examples/virtual_list_basics.rs) | Official | `cargo run -p fretboard-dev -- dev native --example virtual_list_basics` | `cookbook-virtual-list-basics`

## Advanced — Async state / queries (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`query_basics.rs`](./examples/query_basics.rs) | Lab | `cookbook-query` | `cargo run -p fretboard-dev -- dev native --example query_basics` | `cookbook-query-basics`

## Advanced — Data / tables (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`data_table_basics.rs`](./examples/data_table_basics.rs) | Lab | `cookbook-table` | `cargo run -p fretboard-dev -- dev native --example data_table_basics` | `cookbook-data-table-basics`

## Advanced — Theme + assets (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`theme_switching_basics.rs`](./examples/theme_switching_basics.rs) | Official | (none) | `cargo run -p fretboard-dev -- dev native --example theme_switching_basics` | `cookbook-theme-switching-basics`
[`assets_reload_epoch_basics.rs`](./examples/assets_reload_epoch_basics.rs) | Lab | `cookbook-assets` | `cargo run -p fretboard-dev -- dev native --example assets_reload_epoch_basics` | `cookbook-assets-reload-epoch-basics`
[`icons_and_assets_basics.rs`](./examples/icons_and_assets_basics.rs) | Lab | `cookbook-assets` | `cargo run -p fretboard-dev -- dev native --example icons_and_assets_basics` | `cookbook-icons-and-assets-basics`
[`image_asset_cache_basics.rs`](./examples/image_asset_cache_basics.rs) | Lab | `cookbook-image-assets,cookbook-renderer` | `cargo run -p fretboard-dev -- dev native --example image_asset_cache_basics` | `cookbook-image-asset-cache-basics`

## Advanced — Rendering / effects (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`effects_layer_basics.rs`](./examples/effects_layer_basics.rs) | Official | (none) | `cargo run -p fretboard-dev -- dev native --example effects_layer_basics` | `cookbook-effects-layer-basics`
[`compositing_alpha_basics.rs`](./examples/compositing_alpha_basics.rs) | Lab | `cookbook-renderer` | `cargo run -p fretboard-dev -- dev native --example compositing_alpha_basics` | `cookbook-compositing-alpha-basics`
[`drop_shadow_basics.rs`](./examples/drop_shadow_basics.rs) | Lab | `cookbook-renderer` | `cargo run -p fretboard-dev -- dev native --example drop_shadow_basics` | `cookbook-drop-shadow-basics`
[`customv1_basics.rs`](./examples/customv1_basics.rs) | Lab | `cookbook-customv1` | `cargo run -p fretboard-dev -- dev native --example customv1_basics` | `cookbook-customv1-basics`

## Advanced — Interop + editor-grade (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`docking_basics.rs`](./examples/docking_basics.rs) | Lab | `cookbook-docking` | `cargo run -p fretboard-dev -- dev native --example docking_basics` | `cookbook-docking-basics`
[`embedded_viewport_basics.rs`](./examples/embedded_viewport_basics.rs) | Lab | `cookbook-interop` | `cargo run -p fretboard-dev -- dev native --example embedded_viewport_basics` | `cookbook-embedded-viewport-basics`
[`external_texture_import_basics.rs`](./examples/external_texture_import_basics.rs) | Lab | `cookbook-interop` | `cargo run -p fretboard-dev -- dev native --example external_texture_import_basics` | `cookbook-external-texture-import-basics`
[`imui_action_basics.rs`](./examples/imui_action_basics.rs) | Lab | `cookbook-imui` | `cargo run -p fretboard-dev -- dev native --example imui_action_basics` | `cookbook-imui-action-basics`

## Advanced — Content (feature-gated, still evolving)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`markdown_and_code_basics.rs`](./examples/markdown_and_code_basics.rs) | Preview | `cookbook-markdown` | `cargo run -p fretboard-dev -- dev native --example markdown_and_code_basics` | `cookbook-markdown-and-code-basics`

## Advanced — Charts / gizmo (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`chart_interactions_basics.rs`](./examples/chart_interactions_basics.rs) | Lab | `cookbook-chart` | `cargo run -p fretboard-dev -- dev native --example chart_interactions_basics` | `cookbook-chart-interactions-basics`
[`gizmo_basics.rs`](./examples/gizmo_basics.rs) | Lab | `cookbook-gizmo` | `cargo run -p fretboard-dev -- dev native --example gizmo_basics` | `cookbook-gizmo-basics`

## Advanced — Utility / platform notes (feature-gated)

Example | Status | Feature | Run
--- | --- | --- | ---
[`utility_window_materials_windows.rs`](./examples/utility_window_materials_windows.rs) | Lab | `cookbook-bootstrap` | `cargo run -p fretboard-dev -- dev native --example utility_window_materials_windows`
[`undo_basics.rs`](./examples/undo_basics.rs) | Lab | `cookbook-undo` | `cargo run -p fretboard-dev -- dev native --example undo_basics`
[`async_inbox_basics.rs`](./examples/async_inbox_basics.rs) | Lab | `cookbook-async` | `cargo run -p fretboard-dev -- dev native --example async_inbox_basics`
