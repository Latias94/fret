# Editor Canvas Paint Replay Slice v1 Handoff

## Status

Closed. Created after the `20260523-r58` Windows RTX4090 editor-paint closeout selected
`owner=canvas-paint-replay`. The lane completed ECPR-030 and ECPR-040 and is now closed.

## Current State

- Parent lane: `docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json`
- Seed closeout:
  `target/fret-diag/editor-paint-contract-windows-handoff-20260523-r58/closeout/editor-paint-contract-closeout.summary.json`
- Implementation validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`
  `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`
- Existing closed historical lane:
  `docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/WORKSTREAM.json`

## Next Task

No next task in this lane. Use the parent workstream or open a fresh lane if a new owner proof appears.

## Assumptions

- Confident: the formal Windows closeout is complete and selected Canvas paint replay.
  - Evidence: closeout summary reports `ok=true`, `owner_decision.status=decided`, `owner=canvas-paint-replay`.
  - If wrong: rerun verifier/closeout before implementation.
- Confident: the closed historical Canvas attribution lane should not be reopened by default.
  - Evidence: its `WORKSTREAM.json` has `status=closed` and `default_action=closed`.
  - If wrong: reopen only with a fresh scoped evidence note.
- Likely: the first implementation owner is inside code-editor Canvas / row-surface paint callback or adjacent
  paint-cache bookkeeping, not clean-geometry.
  - Evidence: parent closeout reason names `paint.widget / Canvas`.
  - If wrong: ECPR-010 should reject the lane or redirect it before code changes.
- Confident: existing attribution fields are sufficient for the first implementation attempt.
  - Evidence: `ECPR_010_SOURCE_AUDIT_2026-05-23.md` rejects generic Canvas wrapper overhead, WindowedRowsSurface loop
    overhead, renderer payload, and missing row replay.
  - If wrong: reopen ECPR-020 and add only the missing summary split.
- Confident: the ECPR-030 implementation did not weaken row replay/cache or renderer payload guardrails.
  - Evidence: focused replay tests passed, the r59 attribution validation passed with `with_paint_perf=true`, and the
    closeout kept the baseline policy unchanged.
  - If wrong: reopen a fresh lane with a new bundle proof.

## Verification Before Handoff

Run:

```powershell
python -m json.tool docs/workstreams/editor-canvas-paint-replay-slice-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```
