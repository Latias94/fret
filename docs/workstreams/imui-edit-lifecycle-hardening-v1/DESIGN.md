# ImUi Edit Lifecycle Hardening v1

Status: active follow-on
Last updated: 2026-04-24

## Scope

This lane hardens IMUI value-edit lifecycle semantics after the closed
`imui-edit-lifecycle-diag-gate-v1` diagnostics follow-on.

The previous lane proved the existing response lifecycle counters and repaired editor-proof script
drift. This lane is the next implementation surface: make slider, drag-value, numeric input, and
text-entry edit sessions behave like Dear ImGui-class controls under repeatable unit and diag
coverage.

## Assumptions

- Area: lane status
  - Assumption: `imui-edit-lifecycle-diag-gate-v1` is closed and should not be reopened for broader
    lifecycle work.
  - Evidence: `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json` and
    `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/TODO.md`.
  - Confidence: Confident.
  - Consequence if wrong: this follow-on would split scope that belongs in the old closeout folder.
- Area: upstream target
  - Assumption: Dear ImGui lifecycle parity here means `ActiveId` session ownership,
    `LastItemData`-style per-item query result, and `IsItemDeactivatedAfterEdit()`-style commit
    edge behavior, not a one-for-one C++ API clone.
  - Evidence: `repo-ref/imgui/imgui.h`, `repo-ref/imgui/imgui.cpp`, and
    `repo-ref/imgui/imgui_widgets.cpp`.
  - Confidence: Confident.
  - Consequence if wrong: the lane could overfit API names instead of preserving Fret's response
    return model.
- Area: current Fret surface
  - Assumption: `fret-ui-kit::imui::ResponseExt` is the right facade layer for lifecycle query
    vocabulary; `fret-authoring::Response` and `crates/fret-ui` should stay stable unless proof
    forces a contract change.
  - Evidence: `docs/adr/0066-fret-ui-runtime-contract-surface.md`,
    `ecosystem/fret-ui-kit/src/imui/response/hover.rs`, and
    `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`.
  - Confidence: Confident.
  - Consequence if wrong: implementation could leak policy-heavy IMUI semantics into the runtime.
- Area: likely gap
  - Assumption: the next useful slice is not another generic helper; it is a value-edit audit over
    slider, drag-value, numeric input, and text-entry handoff/writeback edges.
  - Evidence: `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`,
    `ecosystem/fret-ui-kit/src/imui/text_controls.rs`,
    `ecosystem/fret-ui-editor/src/controls/drag_value.rs`, and
    `ecosystem/fret-ui-editor/src/controls/numeric_input.rs`.
  - Confidence: Likely.
  - Consequence if wrong: we may add proof around an area that is already correct while missing a
    higher-impact IMUI owner problem.

## Ownership

- `fret-ui-kit::imui` owns generic immediate response lifecycle vocabulary and private interaction
  runtime state.
- `fret-ui-editor` owns editor-grade value-edit controls such as drag-value and numeric input.
- `apps/fret-examples` owns proof surfaces and selectors.
- `fretboard diag` owns repeatable interaction evidence.

## Target Invariant

- A value-edit helper reports `edited` only when the underlying model meaningfully changes.
- A continuous edit session reports `deactivated_after_edit` exactly when the active session ends
  after at least one edit.
- Instant edit helpers may synthesize activation/deactivation only when there is no active session.
- Text and numeric entry preserve same-frame writeback/deactivation semantics instead of losing a
  commit edge on blur, Enter, Escape, or focus handoff.
- The immediate facade returns per-item query results through `ResponseExt`; it does not require a
  global public `LastItemData` API.

## Out Of Scope

- Widening `crates/fret-ui`, `fret-authoring::Response`, or public `fret-imui` contracts by
  default.
- Key-owner / shortcut ownership work.
- Docking, multi-window, or runner/backend hand-feel work.
- Broad editor workbench product scope.
- Compatibility fallbacks for obsolete proof-demo behavior.
