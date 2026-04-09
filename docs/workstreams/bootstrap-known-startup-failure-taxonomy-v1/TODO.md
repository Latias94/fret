# Bootstrap Known Startup Failure Taxonomy v1 — TODO

Status: Closed
Last updated: 2026-04-09

## Lane opening

- [x] BKSFT-001 Open this as a narrow follow-on instead of reopening the closed icon-reporting
  lane.
- [x] BKSFT-002 Record the assumptions and boundaries for a wider bootstrap taxonomy.

## M0 — Baseline and scope freeze

- [x] BKSFT-010 Audit the current returned startup error surfaces across:
  - `BootstrapError`,
  - `fret::Error`,
  - asset startup/manifest failures,
  - and diagnostics panic-hook logging.
- [x] BKSFT-011 Freeze the non-goals so this lane does not turn into a broad lifecycle redesign.

## M1 — Contract freeze

- [x] BKSFT-020 Decide where the shared bootstrap taxonomy lives.
- [x] BKSFT-021 Decide how returned errors and panic-only icon install failures converge on one
  report shape.
- [x] BKSFT-022 Decide how the `fret` facade exposes the taxonomy without widening the root
  direct re-export budget.

## M2 — Proof surface

- [x] BKSFT-030 Land the bootstrap known failure stage/kind/report primitives.
- [x] BKSFT-031 Map returned bootstrap/app asset failures into the shared taxonomy.
- [x] BKSFT-032 Map panic-only explicit icon install failures into the same taxonomy and log
  unified diagnostics fields.
- [x] BKSFT-033 Leave tests for bootstrap and `fret` bridge coverage.

## M3 — Docs and closeout

- [x] BKSFT-040 Record the gate set for the closed lane.
- [x] BKSFT-041 Update roadmap/workstream indexes for the new follow-on.
- [x] BKSFT-042 Close this lane explicitly instead of leaving bootstrap startup failures as
  scattered ad-hoc error handling.

## Boundaries to protect

- Do not widen this lane into startup recovery UI or persistent diagnostics bundle design.
- Do not redesign `.setup(...)` / `init_app(...)` around `Result`.
- Do not add a bootstrap dependency edge into pack crates.
- Do not widen `fret` root direct re-exports beyond the existing curated budget.

Completed M0 evidence:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Completed M2 proof:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/M2_PROOF_SURFACE_2026-04-09.md`

Closeout:

- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/CLOSEOUT_AUDIT_2026-04-09.md`
