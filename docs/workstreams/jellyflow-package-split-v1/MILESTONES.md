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

- Decide whether runtime store/callback helpers and any remaining geometry seams belong in
  `jellyflow-runtime`.
- Extract runtime only after the previous slice's gates stay green.
