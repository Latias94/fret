# ImUi Edit Lifecycle Hardening v1 Evidence And Gates

Status: active gate list
Last updated: 2026-04-24

## Smallest Repros

```bash
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
```

The first three commands are the focused lifecycle floor. The two diag suites keep the proof demos
from drifting while this lane hardens value-edit behavior.

## Required Gates

```bash
cargo check -p fret-diag
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo check -p fret-examples
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
- `ecosystem/fret-ui-editor/src/imui.rs`
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
