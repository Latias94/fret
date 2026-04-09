# Icon Install Error Reporting v1 — TODO

Status: Closed
Last updated: 2026-04-09

## Lane opening

- [x] IER-001 Open this as a narrow follow-on instead of widening the closed install-health lane.
- [x] IER-002 Record the assumptions and boundaries for icon install error reporting.

## M0 — Baseline and scope freeze

- [x] IER-010 Audit how explicit install failures are reported today across:
  - first-party pack app installs,
  - generated pack app installs,
  - bootstrap contract registration,
  - and diagnostics panic-hook logging.
- [x] IER-011 Freeze the non-goals so this lane does not turn into a broad lifecycle redesign.

## M1 — Reporting contract freeze

- [x] IER-020 Decide where the shared report type lives.
- [x] IER-021 Decide how panic text remains human-readable while diagnostics gain structure.
- [x] IER-022 Decide how long the structured report is visible during panic handling.

## M2 — Proof surface

- [x] IER-030 Land a shared icon-install failure report and panic helpers.
- [x] IER-031 Route first-party and generated install seams through the shared helpers.
- [x] IER-032 Prove bootstrap diagnostics can compile against the structured reporting path.
- [x] IER-033 Leave tests/source-policy coverage for the shared helper usage.

## M3 — Docs and closeout

- [x] IER-040 Update ADR/alignment wording for the reporting contract.
- [x] IER-041 Record the final gate set for the reporting lane.
- [x] IER-042 Close this lane explicitly instead of leaving panic reporting as ad-hoc drift.

## Boundaries to protect

- Do not widen this lane into `Result`-based bootstrap redesign.
- Do not introduce a bootstrap dependency edge into pack crates.
- Do not regress ordinary panic output to opaque non-string payloads.
- Do not turn a panic-time report into stale cross-panic global state.

Completed M0 evidence:

- `docs/workstreams/icon-install-error-reporting-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/icon-install-error-reporting-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Completed M2 proof:

- `docs/workstreams/icon-install-error-reporting-v1/M2_PROOF_SURFACE_2026-04-09.md`

Closeout:

- `docs/workstreams/icon-install-error-reporting-v1/CLOSEOUT_AUDIT_2026-04-09.md`
