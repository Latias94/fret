# ImUi Models Text Picker Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane records the first small split of the growing `fret-imui` text-model test surface after the
completion/history picker policy slices closed.

## Assumptions

- Area: lane ownership
  - Assumption: picker test decomposition is a new narrow follow-on, not a reopening of the closed
    text-input policy or picker accessibility lanes.
  - Evidence: `docs/workstreams/imui-text-input-picker-a11y-v1/CLOSEOUT_AUDIT_2026-05-04.md`;
    `docs/audits/imui-imgui-gap-audit-2026-04-22.md`.
  - Confidence: Confident
  - Consequence if wrong: test-only cleanup would be recorded inside a behavior lane and blur the
    next maintainer's first-open state.
- Area: implementation shape
  - Assumption: these picker tests should remain Rust tests for now because they exercise multi-frame
    focus, text input, popup, keyboard, and semantics interactions.
  - Evidence: `ecosystem/fret-imui/src/tests/models_text_picker.rs`.
  - Confidence: Likely
  - Consequence if wrong: a later fixture harness may be justified, but it should start from a
    smaller repeated matrix with stable case IDs.
- Area: public surface
  - Assumption: this split must not change IMUI runtime, kit policy, or app-facing APIs.
  - Evidence: only `ecosystem/fret-imui/src/tests/mod.rs`,
    `ecosystem/fret-imui/src/tests/models_text.rs`, and
    `ecosystem/fret-imui/src/tests/models_text_picker.rs` are in the code scope.
  - Confidence: Confident
  - Consequence if wrong: behavior gates must catch any accidental policy drift.

## Ownership

- `fret-imui` owns the immediate-mode proof tests and module registration.
- `fret-ui-kit::imui` continues to own text-input picker policy.
- `crates/fret-ui` remains out of scope; no runtime contract changes are part of this lane.

## Must-Be-True Outcomes

- Completion/history picker tests are isolated in `models_text_picker.rs`.
- `models_text.rs` no longer owns picker-specific test bodies or picker-only imports.
- `tests/mod.rs` registers the new module.
- Focused picker tests and the broader `models_text` filter both pass.

## Fixture Decision

The current picker tests are procedural interaction proofs, not a pure repeated data matrix. Keep
them as Rust tests for now. Use fixture-driven extraction later only for case tables such as named
input filters, where the runner can stay thin and each failure can be case-ID addressable.

## Non-Goals

- No input-text behavior change.
- No picker semantics or keyboard policy change.
- No new JSON fixture schema in this slice.
- No broad cleanup of other large `fret-imui` test files.
