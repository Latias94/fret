# Fret Node Paint Root Cache Plan Adapter v1 - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This lane is a narrow follow-on from `fret-node-paint-prepaint-adapter-v1`. The parent lane proved
the prepaint cull-window adapter seam and rejected a broad paint-root adapter because paint root
contains multiple operation families.

The first implementation slice should target `paint_root/cache_plan.rs` only.

## Active Task

- Task ID: CPA-010
- Owner: unassigned
- Files: `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1`
- Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/WORKSTREAM.json`
- Status: NEEDS_CONTEXT
- Review: not started
- Evidence: `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/EVIDENCE_AND_GATES.md`

## Decisions Since Open

- Cache-plan preparation is the first paint-root adapter proof.
- Frame setup, static layer replay/store, cached/immediate passes, and tail cleanup are explicit
  non-goals for the first implementation slice.

## Blockers

- None known.

## Next Recommended Action

- Execute CPA-010 to freeze scope, then CPA-020 to introduce the cache-plan context seam.
