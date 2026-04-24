# M0 Baseline Audit - ImUi Edit Lifecycle Hardening

Status: active baseline
Date: 2026-04-24

## Dear ImGui Reference Shape

The local `repo-ref/imgui` snapshot shows the value-edit lifecycle model this lane should match at
the outcome level:

- `repo-ref/imgui/imgui.h` exposes `IsItemDeactivatedAfterEdit()` as the public query for "last
  item ended an active edit session after changing value".
- `repo-ref/imgui/imgui.cpp` keeps global active-item state such as `ActiveId`,
  `ActiveIdHasBeenEditedBefore`, `ActiveIdPreviousFrame`, and deactivation snapshots.
- `repo-ref/imgui/imgui_widgets.cpp` routes sliders, drags, temp scalar input, and text input
  through `MarkItemEdited(...)` plus active/deactivated handoff logic.

Fret should not copy those APIs directly. The useful invariant is the behavior:

- one active owner per edit session,
- one per-item query result after a helper returns,
- an edited flag only when the model meaningfully changes,
- and a deactivated-after-edit edge when the session ends after an edit.

## Current Fret State

- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` already exposes facade-local
  `ResponseExt::{activated, deactivated, edited, deactivated_after_edit}`.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs` provides a private lifecycle
  session model plus transient events for explicit pointer sessions and instant edits.
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs` uses that private lifecycle session for
  pointer drag edits and synthesizes lifecycle events for keyboard edits.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs` derives text lifecycle from focus state and
  model diffs rather than the explicit private lifecycle session.
- `ecosystem/fret-ui-editor/src/controls/drag_value.rs` and
  `ecosystem/fret-ui-editor/src/controls/numeric_input.rs` own editor-grade value controls whose
  proof currently flows through `apps/fret-examples/src/imui_editor_proof_demo.rs`.

## Initial Gap Hypothesis

The likely gap is not the public response vocabulary. It is the consistency of private lifecycle
ownership across value-edit families:

- slider has an explicit session path,
- text has a focus/model-diff path,
- drag-value and numeric input live in the editor layer and need outcome proof before any shared
  helper grows,
- and diagnostics currently prove outcomes but not yet every edit-session edge those controls need
  for undo/redo-class workflows.

## First Slice Recommendation

Start M1 with a code audit before editing behavior:

1. Trace `ResponseExt` population for slider pointer, slider keyboard, input text, and textarea.
2. Trace `fret-ui-editor` drag-value and numeric-input commit/cancel outcomes.
3. Decide whether a shared private "value edit session" kernel removes duplication or whether the
   existing split is correct with stronger tests.
4. Add proof only for a concrete mismatch; do not widen runtime or authoring APIs speculatively.

## First Gate Set

```bash
cargo check -p fret-diag
cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit
cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges
cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges
cargo check -p fret-examples
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json
git diff --check
```
