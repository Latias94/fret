# Cookbook examples (Index)

This page is a **Bevy-style index** of all runnable cookbook examples under
[`apps/fret-cookbook/examples/`](./examples/).

Design goals:

- The cookbook can be large, but it should never feel chaotic.
- The default onboarding path stays short and stable (see [`apps/fret-cookbook/README.md`](./README.md)).
- Higher-ceiling topics are feature-gated to keep cold compile time down.

How to run an example:

```bash
cargo run -p fret-cookbook --example <name>
```

For feature-gated examples:

```bash
cargo run -p fret-cookbook --features <feature> --example <name>
```

Diagnostics (optional, scripted smoke):

- If an example has a suite listed below, you can run it with:

```bash
FRET_DIAG=1 cargo run -p fretboard -- diag suite <suite-name> --launch -- \
  cargo run -p fret-cookbook --features cookbook-diag --example <name>
```

Notes:

- Most suites are “smoke + screenshot + bundle”.
- Some examples require additional features beyond `cookbook-diag` (listed below).

## The bare minimum

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`hello.rs`](./examples/hello.rs) | Official | `cargo run -p fret-cookbook --example hello` | `cookbook-hello`
[`simple_todo.rs`](./examples/simple_todo.rs) | Official | `cargo run -p fret-cookbook --example simple_todo` | `cookbook-simple-todo`

## Core UI + input + commands (onboarding-friendly)

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`overlay_basics.rs`](./examples/overlay_basics.rs) | Official | `cargo run -p fret-cookbook --example overlay_basics` | `cookbook-overlay-basics`
[`text_input_basics.rs`](./examples/text_input_basics.rs) | Official | `cargo run -p fret-cookbook --example text_input_basics` | `cookbook-text-input-basics`
[`commands_keymap_basics.rs`](./examples/commands_keymap_basics.rs) | Official | `cargo run -p fret-cookbook --example commands_keymap_basics` | `cookbook-commands-keymap-basics`

## App patterns (small, copy/paste-ready)

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`toast_basics.rs`](./examples/toast_basics.rs) | Official | `cargo run -p fret-cookbook --example toast_basics` | `cookbook-toast-basics`
[`date_picker_basics.rs`](./examples/date_picker_basics.rs) | Official | `cargo run -p fret-cookbook --example date_picker_basics` | `cookbook-date-picker-basics`
[`form_basics.rs`](./examples/form_basics.rs) | Official | `cargo run -p fret-cookbook --example form_basics` | `cookbook-form-basics`
[`drag_basics.rs`](./examples/drag_basics.rs) | Official | `cargo run -p fret-cookbook --example drag_basics` | `cookbook-drag-basics`
[`hello_counter.rs`](./examples/hello_counter.rs) | Official | `cargo run -p fret-cookbook --example hello_counter` | `cookbook-hello-counter`

## Routing (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`router_basics.rs`](./examples/router_basics.rs) | Lab | `cookbook-router` | `cargo run -p fret-cookbook --features cookbook-router --example router_basics` | `cookbook-router-basics`

## State + derived state

Example | Status | Run | Diag suite
--- | --- | --- | ---
[`virtual_list_basics.rs`](./examples/virtual_list_basics.rs) | Official | `cargo run -p fret-cookbook --example virtual_list_basics` | `cookbook-virtual-list-basics`

## Async state / queries (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`query_basics.rs`](./examples/query_basics.rs) | Lab | `cookbook-query` | `cargo run -p fret-cookbook --features cookbook-query --example query_basics` | `cookbook-query-basics`

## Data / tables (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`data_table_basics.rs`](./examples/data_table_basics.rs) | Lab | `cookbook-table` | `cargo run -p fret-cookbook --features cookbook-table --example data_table_basics` | `cookbook-data-table-basics`

## Theme + assets (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`theme_switching_basics.rs`](./examples/theme_switching_basics.rs) | Official | (none) | `cargo run -p fret-cookbook --example theme_switching_basics` | `cookbook-theme-switching-basics`
[`assets_reload_epoch_basics.rs`](./examples/assets_reload_epoch_basics.rs) | Lab | `cookbook-assets` | `cargo run -p fret-cookbook --features cookbook-assets --example assets_reload_epoch_basics` | `cookbook-assets-reload-epoch-basics`
[`icons_and_assets_basics.rs`](./examples/icons_and_assets_basics.rs) | Lab | `cookbook-assets` | `cargo run -p fret-cookbook --features cookbook-assets --example icons_and_assets_basics` | `cookbook-icons-and-assets-basics`
[`image_asset_cache_basics.rs`](./examples/image_asset_cache_basics.rs) | Lab | `cookbook-image-assets,cookbook-renderer` | `cargo run -p fret-cookbook --features cookbook-image-assets,cookbook-renderer --example image_asset_cache_basics` | `cookbook-image-asset-cache-basics`

## Rendering / effects (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`effects_layer_basics.rs`](./examples/effects_layer_basics.rs) | Official | (none) | `cargo run -p fret-cookbook --example effects_layer_basics` | `cookbook-effects-layer-basics`
[`compositing_alpha_basics.rs`](./examples/compositing_alpha_basics.rs) | Lab | `cookbook-renderer` | `cargo run -p fret-cookbook --features cookbook-renderer --example compositing_alpha_basics` | `cookbook-compositing-alpha-basics`
[`drop_shadow_basics.rs`](./examples/drop_shadow_basics.rs) | Lab | `cookbook-renderer` | `cargo run -p fret-cookbook --features cookbook-renderer --example drop_shadow_basics` | `cookbook-drop-shadow-basics`
[`customv1_basics.rs`](./examples/customv1_basics.rs) | Lab | `cookbook-customv1` | `cargo run -p fret-cookbook --features cookbook-customv1 --example customv1_basics` | `cookbook-customv1-basics`

## Interop + advanced (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`docking_basics.rs`](./examples/docking_basics.rs) | Lab | `cookbook-docking` | `cargo run -p fret-cookbook --features cookbook-docking --example docking_basics` | `cookbook-docking-basics`
[`embedded_viewport_basics.rs`](./examples/embedded_viewport_basics.rs) | Lab | `cookbook-interop` | `cargo run -p fret-cookbook --features cookbook-interop --example embedded_viewport_basics` | `cookbook-embedded-viewport-basics`
[`external_texture_import_basics.rs`](./examples/external_texture_import_basics.rs) | Lab | `cookbook-interop` | `cargo run -p fret-cookbook --features cookbook-interop --example external_texture_import_basics` | `cookbook-external-texture-import-basics`
[`imui_action_basics.rs`](./examples/imui_action_basics.rs) | Lab | `cookbook-imui` | `cargo run -p fret-cookbook --features cookbook-imui --example imui_action_basics` | `cookbook-imui-action-basics`

## Content (feature-gated, still evolving)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`markdown_and_code_basics.rs`](./examples/markdown_and_code_basics.rs) | Preview | `cookbook-markdown` | `cargo run -p fret-cookbook --features cookbook-markdown --example markdown_and_code_basics` | `cookbook-markdown-and-code-basics`

## Charts / gizmo (feature-gated)

Example | Status | Feature | Run | Diag suite
--- | --- | --- | --- | ---
[`chart_interactions_basics.rs`](./examples/chart_interactions_basics.rs) | Lab | `cookbook-chart` | `cargo run -p fret-cookbook --features cookbook-chart --example chart_interactions_basics` | `cookbook-chart-interactions-basics`
[`gizmo_basics.rs`](./examples/gizmo_basics.rs) | Lab | `cookbook-gizmo` | `cargo run -p fret-cookbook --features cookbook-gizmo --example gizmo_basics` | `cookbook-gizmo-basics`

## Utility / platform notes (feature-gated)

Example | Status | Feature | Run
--- | --- | --- | ---
[`utility_window_materials_windows.rs`](./examples/utility_window_materials_windows.rs) | Lab | `cookbook-bootstrap` | `cargo run -p fret-cookbook --features cookbook-bootstrap --example utility_window_materials_windows`
[`undo_basics.rs`](./examples/undo_basics.rs) | Lab | `cookbook-undo` | `cargo run -p fret-cookbook --features cookbook-undo --example undo_basics`
[`async_inbox_basics.rs`](./examples/async_inbox_basics.rs) | Lab | `cookbook-async` | `cargo run -p fret-cookbook --features cookbook-async --example async_inbox_basics`
