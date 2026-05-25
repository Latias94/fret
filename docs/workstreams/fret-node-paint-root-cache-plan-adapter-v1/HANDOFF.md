# Fret Node Paint Root Cache Plan Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-prepaint-adapter-v1`. The parent lane proved
the prepaint cull-window adapter seam and rejected a broad paint-root adapter because paint root
contains multiple operation families.

The first implementation slice should target `paint_root/cache_plan.rs` only.

## Active Task

- Task ID: CPA-030
- Owner: planner
- Files: `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/EVIDENCE_AND_GATES.md`,
  `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/HANDOFF.md`,
  optional closeout audit
- Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Open

- Cache-plan preparation is the first paint-root adapter proof.
- Frame setup, static layer replay/store, cached/immediate passes, and tail cleanup are explicit
  non-goals for the first implementation slice.
- CPA-010 froze the cache-plan-only scope.
- CPA-020 introduced `PaintRootCachePlanCx` and isolated retained `PaintCx` binding in
  `cache_plan_retained_cx.rs`.

## Blockers

- None known.

## Next Recommended Action

- Execute CPA-030: close this lane or split the next paint family. Candidate follow-ons are frame
  setup, static layer replay/store, or scene pass emission.
