# ImUi Control Geometry Stability v1 - Evidence and Gates

Status: active execution lane
Last updated: 2026-04-28

## Smallest Repro

Manual/local:

- `cargo run -p fret-demo --bin imui_interaction_showcase_demo`
- `cargo run -p fret-demo --bin imui_response_signals_demo`

Inherited automated floor:

- `cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast`
- `cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui base_control_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui menu_and_tab_trigger_state_changes_keep_outer_bounds_stable --no-fail-fast`

Linux/Wayland real-host acceptance is not part of this lane. That proof remains owned by
`docs/workstreams/docking-multiwindow-imgui-parity/`.

## Current Evidence

- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-text-control-chrome-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-control-geometry-stability-v1/M1_BASE_CONTROL_GEOMETRY_GATE_2026-04-28.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gate Set

Initial lane gates:

- `cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast`
- `cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui base_control_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui menu_and_tab_trigger_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 2`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-control-geometry-stability-v1/WORKSTREAM.json`
- `git diff --check`

## Non-Gates

- No Linux compositor acceptance.
- No screenshot-only gate as the sole proof for geometry stability.
- No full workspace test requirement for the first refactor slice unless the changed surface grows.
