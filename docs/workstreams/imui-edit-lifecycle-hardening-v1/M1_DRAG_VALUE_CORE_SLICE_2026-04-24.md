# M1 DragValueCore Lifecycle Slice

Status: landed slice
Date: 2026-04-24

## Summary

This slice audited the first value-edit lifecycle paths and landed the narrow fix where the code
had a real mismatch against the target invariant.

## Findings

- `fret-ui-kit::imui` slider lifecycle already uses an explicit private lifecycle session and the
  focused `fret-imui` slider lifecycle test remains green.
- `fret-ui-kit::imui` input text and textarea derive lifecycle from focus plus model diffs; this is
  still the correct owner model for text-entry `ResponseExt` because the underlying text elements
  own focus/writeback semantics.
- `fret-ui-editor::NumericInputOutcome::{Committed, Canceled}` is a session-close result, not an
  `edited` / `deactivated_after_edit` signal. It must keep firing on Enter/Escape so wrapper
  controls can exit typing mode, even when the committed value is unchanged.
- `DragValueCore` had the actual gap: it used `EditSession` but emitted commit callbacks whenever
  a drag session crossed the drag threshold, even if the live value never meaningfully changed.

## Landed Change

- `DragValueScalar` now requires `PartialEq`, matching the numeric scalar intent of the primitive.
- `DragState` now records `edited_during_session`.
- Live scrub updates are ignored when the constrained value equals the current value.
- Commit callbacks fire only after a drag session that had at least one meaningful value change.
- A session that changes away and returns to the pre-edit value still counts as edited, matching
  Dear ImGui's session-level "edited before deactivation" behavior.
- `DragValueCoreResponse::{committed, canceled}` was removed because those fields were never
  populated and therefore implied a false query surface.
- Pointer capture is now symmetrical for scrub sessions: pointer down captures and pointer up /
  cancel paths release.

## Evidence

- `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`

## Gates Run

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

## Follow-Up

The next behavior proof should target `DragValue` / `NumericInput` through a real rendered
editor-control harness or the existing `imui_editor_proof_demo` diagnostics suite. Do not widen
`ResponseExt` or runtime APIs just to expose editor session-close outcomes.
