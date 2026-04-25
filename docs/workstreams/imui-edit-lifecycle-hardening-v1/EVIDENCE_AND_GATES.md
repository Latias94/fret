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
cargo nextest run -p fret-ui deferred_dirty_sync_does_not_consume_model_revision --jobs 2
cargo nextest run -p fret-ui forced_sync_applies_model_revision_even_when_dirty --jobs 2
cargo check -p fret-ui -p fret-ui-editor -p fret-examples --jobs 2
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
```

The first three commands are the focused lifecycle floor. The `fret-node` commands cover retained
portal editor input sizing policy. The IMUI input bounds test covers the public single-line helper.
The input-bounds diag script renders the click/focus/type path and captures layout sidecars. The
fret-ui bound text tests cover the controlled-buffer revision rule used by rendered numeric-input
proof. The numeric-input diagnostics scripts render validation, reset, and Escape-cancel outcomes.
The registry check keeps named suite membership in sync with promoted scripts. The two diag suites
keep the proof demos from drifting while this lane hardens value-edit behavior.

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
cargo nextest run -p fret-ui deferred_dirty_sync_does_not_consume_model_revision --jobs 2
cargo nextest run -p fret-ui forced_sync_applies_model_revision_even_when_dirty --jobs 2
cargo fmt --package fret-ui --package fret-ui-editor --check
cargo check -p fret-ui -p fret-ui-editor -p fret-examples --jobs 2
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
python tools/check_workstream_catalog.py
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json
python -m json.tool tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json
python -m json.tool tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json
python -m json.tool tools/diag-scripts/index.json
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
- `ecosystem/fret-ui-editor/src/primitives/numeric_text_entry.rs`
- `ecosystem/fret-ui-editor/src/composites/property_row.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `crates/fret-ui/src/text/input/bound.rs`
- `crates/fret-ui/src/text/area/bound.rs`
- `ecosystem/fret-node/src/ui/editors/chrome.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `ecosystem/fret-imui/src/tests/models_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json`
- `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json`
- `tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json`
- `tools/diag-scripts/index.json`
- `tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/M3_NUMERIC_INPUT_RENDERED_PROOF_2026-04-25.md`
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

Passed on 2026-04-25 for the rendered IMUI input bounds diagnostics gate:

```bash
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-response-signals-edit-lifecycle --launch -- cargo run -p fret-demo --bin imui_response_signals_demo
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-response-signals-input-bounds-stability.json
python -m json.tool tools/diag-scripts/suites/imui-response-signals-edit-lifecycle/suite.json
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```

Passed on 2026-04-25 for the numeric-input rendered proof promotion:

```bash
cargo nextest run -p fret-ui deferred_dirty_sync_does_not_consume_model_revision --jobs 2
cargo nextest run -p fret-ui forced_sync_applies_model_revision_even_when_dirty --jobs 2
cargo fmt --package fret-ui --package fret-ui-editor --check
cargo check -p fret-ui -p fret-ui-editor -p fret-examples --jobs 2
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fretboard-dev -- diag registry check
cargo run -p fretboard-dev -- diag suite imui-editor-proof-edit-outcomes --launch -- cargo run -p fret-demo --bin imui_editor_proof_demo
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json
python -m json.tool tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json
python -m json.tool tools/diag-scripts/suites/imui-editor-proof-edit-outcomes/suite.json
python -m json.tool tools/diag-scripts/index.json
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Bundle evidence:

- `target/fret-diag/1777113547959-imui-editor-proof-numeric-input-validation`
- `target/fret-diag/1777114129957-imui-editor-proof-numeric-input-escape-cancel`
- `target/fret-diag/1777114748599-imui-editor-proof-drag-value-outcomes`
- `target/fret-diag/1777114771734-imui-editor-proof-numeric-input-escape-cancel`
- `target/fret-diag/1777114794864-imui-editor-proof-numeric-input-validation`
- `target/fret-diag/1777114836767-imui-editor-proof-text-numeric-baseline-policy`
