# ImUi Edit Lifecycle Hardening v1 Milestones

Status: active
Last updated: 2026-04-24

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

## M2 - Hardening Slice

Exit criteria:

- Duplicated lifecycle state is removed or centralized where that makes the implementation more
  correct.
- Existing public runtime and authoring contracts remain stable unless explicitly justified.
- Focused unit tests cover the changed behavior.

## M3 - Diagnostics Proof

Exit criteria:

- Existing response-signals and editor-proof suites still pass.
- Drag-value and numeric-input lifecycle coverage is added or promoted if M1 finds a real gap.
- Selectors remain stable and demo-matched.

## M4 - Closeout

Exit criteria:

- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` reflect the shipped
  state.
- A closeout note records what remains out of scope.
- Residual gaps route to narrower follow-ons instead of reopening this folder indefinitely.
