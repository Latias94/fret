# Diagnostics Architecture (Fearless Refactor v1) — Artifact Invariants

Status: Draft (workstream contract note)

This file defines the “hard to change” expectations for what diagnostics tooling produces. These
invariants are intended to remain stable across refactors and across FS vs WS transport modes.

If a refactor breaks one of these invariants, it should be treated as a behavior change and gated
explicitly (tests + scripted repro evidence).

## Invariant 1: Every run writes a stable script result

Regardless of success/failure/timeouts/tooling failures, a run must produce a structured script
result artifact (typically `script.result.json`) that includes:

- a clear `stage`/outcome,
- a stable `reason_code` when failing,
- bounded structured evidence (paths, counts, caps; no unbounded dumps).

## Invariant 2: Bundles are portable directories with a primary schema

Every bundle dump yields a local shareable directory containing a primary artifact:

- prefer `bundle.schema2.json` when present,
- treat legacy `bundle.json` as optional compatibility.

“Portable” means: a bundle directory can be zipped/shared and re-opened by tooling without access
to the original runtime process.

## Invariant 3: Failures have stable, actionable reason codes

Tooling must avoid “opaque failures”. When something fails, the output must be attributable to a
small number of stable reason codes (plus bounded evidence) so CI and humans can route issues.

## Invariant 4: “Latest bundle” resolution is deterministic and session-aware

When an out dir contains per-session subdirectories, tooling must resolve “latest bundle” without
races (avoid global `latest.txt` ambiguity when possible).

This invariant is critical for:

- `diag suite` (post-run checks and pack),
- `diag perf` (repeat comparisons),
- DevTools workflows that stream bundles to disk.

## Invariant 5: Evidence clipping is explicit

Any clipping/dropping of evidence due to caps must be visible in the artifact:

- counts of rows/frames dropped,
- byte caps applied,
- and which payloads were omitted.

Silent truncation is not allowed.

## References

- Contract ADR: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- Workflow guide: `docs/ui-diagnostics-and-scripted-tests.md`
- Workstream gates: `docs/workstreams/diag-architecture-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

