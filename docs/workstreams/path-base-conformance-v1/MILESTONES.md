# Path Base Conformance v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Base Path Gates

Exit criteria:

- GPU readback conformance distinguishes non-zero and even-odd intersecting same-winding fills.
- GPU readback conformance proves `SceneOp::Path` honors transform and clip composition together.
- Path module tests prove `PathMetrics.bounds` contain tessellated vertices for representative
  fill, v1 stroke, and v2 miter stroke styles.

Status: Met on 2026-05-18.

## M1 — Contract Evidence

Exit criteria:

- ADR 0080 remaining work no longer lists completed base conformance gaps.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` points to the new gates.
- Workstream evidence includes exact commands and pass/fail status.

Status: Met on 2026-05-18.

## M2 — Close Or Split

Exit criteria:

- Close the lane once M0/M1 are verified.
- If a separate issue appears, split it into a narrower follow-on with its own repro and gate rather
  than keeping this base conformance lane open.

Status: Met on 2026-05-18. No separate behavior issue was exposed.
