# Jellyflow Package Split v1 - Milestones

## M0 - Boundary Decision

- ADR 0331 records Jellyflow as the reusable engine brand and `fret-node` as the Fret adapter.
- Standalone repository extraction is deferred until in-workspace package seams stabilize.

## M1 - First Headless Crate

- `jellyflow-core` exists as a workspace crate.
- The moved modules compile outside `fret-node`.
- `fret-node` preserves old module paths with wrapper re-exports.
- A source-policy gate prevents UI/render/platform dependencies in `jellyflow-core`.

Status: complete for JF-001. Fresh evidence is recorded in `EVIDENCE_AND_GATES.md`.

## M2 - Ops Boundary

- `ops`, `GraphHistory`, and fragment/diff/normalize helpers live in `jellyflow-core`.
- `fret-node` keeps the XyFlow-style change projection and runtime glue.

Status: complete for JF-010. Fresh evidence is recorded in `EVIDENCE_AND_GATES.md`.

## M3 - Runtime Follow-Up

- `jellyflow-runtime` exists as a workspace crate.
- `io`, `profile`, `rules`, `schema`, and `runtime` moved out of `fret-node`.
- `fret-node` preserves old module paths with wrapper re-exports.
- `DataflowProfile` remains kit-owned in `fret-node`, not in the runtime crate.

Status: complete for JF-020. Fresh evidence is recorded in `EVIDENCE_AND_GATES.md`.

## M4 - Geometry Follow-Up

- Decide whether canvas-space geometry, route math, spatial indexes, and hit-test helpers belong in
  `jellyflow-geometry`.
- Move geometry only after the runtime package gates stay green.
