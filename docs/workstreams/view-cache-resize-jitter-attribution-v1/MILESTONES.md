# ViewCache Resize-Jitter Attribution v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist with an explicit `ViewCache` owner boundary.
- Starting evidence points to the fresh `Pressable` closeout bundle and layout summary.
- `ViewCache` is documented as a clean-geometry side-effect boundary.
- Catalog and JSON validation pass.

Status: Complete.

## M1 - Source Audit

Exit criteria:

- `ViewCacheProps`, runtime boundary store, view-boundary metadata, layout entrypoints, and
  clean-geometry classification are mapped to concrete responsibilities.
- The audit separates cache-root contained relayout, root-bound repair, invalidation collapse, and
  scroll follow-up scheduling.
- The lane records whether a runtime optimization is plausible before any code edit.

Status: Complete.

## M2 - Fresh Attribution Bundle

Exit criteria:

- A current UI Gallery resize-jitter bundle is captured or the latest valid bundle is explicitly
  reused.
- `diag stats` and `layout.perf.summary.v1.json` identify the current top layout owners.
- The evidence states whether `ViewCache` remains the top owner.

Status: Complete.

## M3 - Proof Or No-Change Verdict

Exit criteria:

- If a narrow runtime change is justified, a focused RED/GREEN proof lands before the change.
- If no runtime change is justified, the no-change verdict names the owning source phase and why it
  should not be optimized in this lane.
- Cache-root liveness, state retention, boundary tracing, and scroll extent repair remain protected.

Status: Complete with a no-runtime-change verdict. Fresh evidence does not justify a `ViewCache`
clean-geometry or contained-relayout runtime change.

## M4 - Closeout

Exit criteria:

- Final gates pass or any blocked gate is recorded with the exact blocker.
- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` reflect the shipped
  verdict.
- Remaining work is split only when it has a distinct owner and evidence.

Status: Complete. Closeout split the current worst owner into
`ui-gallery-code-editor-canvas-paint-tail-attribution-v1`.
