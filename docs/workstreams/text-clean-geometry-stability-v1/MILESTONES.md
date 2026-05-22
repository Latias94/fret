# Text Clean Geometry Stability v1 Milestones

Date: 2026-05-21

## M0 - Boundary Split

Status: Complete.

The text stop condition from `scroll-optimization-v1` now has its own owner lane. Existing focused
tests prove the current boundary:

- wrapped auto-height text rejects as `text_reflow`,
- cached nowrap text with stable height can skip authoritative layout,
- nowrap text whose height changes rejects as `text_reflow`.

## M1 - Eligibility Matrix

Status: Complete.

Document the exact clean-geometry eligibility and rejection paths for `Text`, `StyledText`, and
`SelectableText` so future performance work can reason from a stable contract.

Evidence:

- `DESIGN.md` now records the width-delta eligibility matrix.
- The matrix keeps cached `TextWrap::None` text as the only accepted text fast path and names every
  current `text_reflow` rejection source.
- The matrix stays workstream-local until a future behavior change needs promotion to a runtime
  contract doc.

## M2 - Diagnostics Adequacy

Status: Complete.

Verify whether current rejection stats are enough to diagnose text clean-geometry failures. Add only
source-stable diagnostic fields if future proofs need them.

Evidence:

- `UiDebugCleanGeometrySolveSkipRejection` now carries optional `detail`.
- Diagnostics schema `UiCleanGeometrySolveSkipRejectionV1` serializes that optional `detail`
  additively.
- `clean_geometry_small_resize_rejects_auto_height_text_reflow` now proves wrapped text reports
  `reason=text_reflow` and `detail=text_wrap_not_none`.
- UI Gallery text-measure-overlay resize diagnostics now prove the field reaches a real shareable
  bundle at
  `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/bundle.schema2.json`,
  with `text_wrap_not_none` and `text_overflow_not_clip` clean-geometry rejection details.

## M3 - Wrapped Text Stability Proof

Status: Deferred follow-on.

No wrapped-text eligibility change ships from this lane. Consider wrapped text only after fresh
evidence shows the cost is material and a line-break/computed-box signature can prove unchanged
layout, measure, and paint outcomes.

## M4 - Closeout

Status: Complete.

This lane closes as a boundary and diagnostics record:

- `TextWrap::None` text with stable cached metrics remains the only accepted text fast path,
- wrapped, overflow-changing, alignment-changing, height-changing, missing-cache, stale-cache, and
  fingerprint-changing text stays on the authoritative path,
- diagnostics now expose additive text rejection `detail`,
- future wrapped-text clean geometry must start as a new behavior-changing proof lane.
