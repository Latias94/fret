# ImUi Edit Lifecycle Hardening v1 Milestones

Status: closed
Last updated: 2026-04-25

## M0 - Baseline And Lane Start

Exit criteria:

- The closed diagnostics follow-on remains closed.
- This lane names the Dear ImGui lifecycle target and the Fret owner split.
- First repro, gate, and evidence anchors are recorded.

Status: started by `M0_BASELINE_AUDIT_2026-04-24.md`.

## M1 - Value-Edit Audit

Exit criteria:

- Slider pointer and keyboard edit semantics are compared against the target invariant.
- Text input and textarea focus/edit/blur semantics are compared against same-frame writeback
  requirements.
- `fret-ui-editor` drag-value and numeric-input outcomes are classified by owner layer.
- Any needed refactor is described as a private-kernel cleanup or a proof-surface repair before
  code changes land.

Status: completed for the first hardening slice by
`M1_DRAG_VALUE_CORE_SLICE_2026-04-24.md`.

## M2 - Hardening Slice

Exit criteria:

- Duplicated lifecycle state is removed or centralized where that makes the implementation more
  correct.
- Existing public runtime and authoring contracts remain stable unless explicitly justified.
- Focused unit tests cover the changed behavior.

Status: first private `DragValueCore` slice landed. Retained node portal text/number editors now
also share fixed input sizing and fixed control line-box policy through
`M2_PORTAL_INPUT_STABILITY_SLICE_2026-04-25.md`. Public IMUI single-line inputs now share the same
fixed field-height/control-line-box direction through
`M2_IMUI_INPUT_STABILITY_SLICE_2026-04-25.md`. Remaining M2 scope should stay open only for
additional concrete mismatches found by diagnostics or editor-control harnesses.

## M3 - Diagnostics Proof

Exit criteria:

- Existing response-signals and editor-proof suites still pass.
- Drag-value and numeric-input lifecycle coverage is added or promoted if M1 finds a real gap.
- Selectors remain stable and demo-matched.

Status: completed for the current hardening scope. Focused `DragState` unit coverage landed and the
existing response-signals plus editor-proof diagnostics suites pass. Public IMUI single-line input
bounds stability now has a rendered diagnostics gate through
`M3_IMUI_INPUT_BOUNDS_DIAG_GATE_2026-04-25.md`. Numeric-input validation, reset, and Escape-cancel
proof are promoted into the editor-proof suite through
`M3_NUMERIC_INPUT_RENDERED_PROOF_2026-04-25.md`.

## M4 - Closeout

Exit criteria:

- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` reflect the shipped
  state.
- A closeout note records what remains out of scope.
- Residual gaps route to narrower follow-ons instead of reopening this folder indefinitely.

Status: completed by `CLOSEOUT_AUDIT_2026-04-25.md`.
