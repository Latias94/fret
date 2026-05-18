# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - Milestones

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- Lane docs exist and name the frozen VCRJ-030 evidence.
- `Canvas` paint tail is the declared owner under investigation.
- `ViewCache`, `Scroll`, and RTX 4090 baselines are explicitly out of this slice.

Status: Complete.

## M1 - Source Attribution

Exit criteria:

- Code-editor canvas/root surface, windowed rows surface, row scene/cache paths, and diagnostics
  counters are mapped to source owners.
- The lane records why `code_editor.paint_perf` is zero while `Canvas` paint owns the tail.

Status: Complete.

Evidence:

- `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`

## M2 - Fresh Repro Or Instrumentation

Exit criteria:

- A fresh bundle confirms or rejects the VCRJ-030 `Canvas` paint signature.
- Any missing diagnostics needed for owner attribution are added or explicitly split.

Status: Pending.

## M3 - Proof Or Split

Exit criteria:

- A runtime optimization lands only after a focused owner proof.
- Otherwise, the lane records a no-change verdict or splits the actual owner.

Status: Pending.

## M4 - Closeout

Exit criteria:

- Workstream docs and gates reflect the final owner verdict.
- Remaining work is split by ownership boundary.

Status: Pending.
