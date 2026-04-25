# ImUi Edit Lifecycle Hardening v1 Evidence And Gates

Status: active gate list
Last updated: 2026-04-25

## Smallest Repros

```bash
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-node --features compat-retained-canvas portal_text_input_ --jobs 2
cargo nextest run -p fret-node --features compat-retained-canvas portal_button_stack_height --jobs 2
cargo check -p fret-node --features compat-retained-canvas --jobs 2
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --jobs 2
cargo check -p fret-ui-kit --features imui --jobs 2
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
```

The first three commands are the focused lifecycle floor. The `fret-node` commands cover retained
portal editor input sizing policy. The IMUI input bounds test covers the public single-line helper.
The two diag suites keep the proof demos from drifting while this lane hardens value-edit behavior.

## Required Gates

```bash
cargo check -p fret-diag
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo check -p fret-examples
cargo nextest run -p fret-node --features compat-retained-canvas portal_text_input_ --jobs 2
cargo nextest run -p fret-node --features compat-retained-canvas portal_button_stack_height --jobs 2
cargo check -p fret-node --features compat-retained-canvas --jobs 2
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --jobs 2
cargo check -p fret-ui-kit --features imui --jobs 2
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
- `ecosystem/fret-ui-editor/src/controls/numeric_input.rs`
- `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-node/src/ui/editors/chrome.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `ecosystem/fret-imui/src/tests/models_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json`
- `tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json`

## Verified Gates

Passed on 2026-04-24 for the lane-start and diagnostics warning slice:

```bash
cargo check -p fret-diag
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```

The IMUI nextest and diag suite gates are the first required M1/M2 behavior gates; they were not
rerun by the lane-start documentation slice.

Passed on 2026-04-24 for the first `DragValueCore` hardening slice:

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor drag_state_
cargo check -p fret-ui-editor --features imui
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo check -p fret-examples
cargo nextest run -p fret-ui-editor --features imui editor_imui_adapter_option_defaults_compile
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```

Passed on 2026-04-25 for the retained node portal input sizing slice:

```bash
cargo fmt --package fret-node --check
cargo nextest run -p fret-node --features compat-retained-canvas portal_text_input_ --jobs 2
cargo nextest run -p fret-node --features compat-retained-canvas portal_button_stack_height --jobs 2
cargo check -p fret-node --features compat-retained-canvas --jobs 2
```

Passed on 2026-04-25 for the public IMUI single-line input sizing slice:

```bash
cargo fmt --package fret-ui-kit --package fret-imui --check
cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --jobs 2
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges --jobs 2
cargo check -p fret-ui-kit --features imui --jobs 2
```
