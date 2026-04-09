# Icon Install Health Hardening v1 — TODO

Status: Closed
Last updated: 2026-04-09

## Lane opening

- [x] IIH-001 Open this as a narrow follow-on instead of widening the closed icon contract lane.
- [x] IIH-002 Record the assumptions and boundaries for explicit install failure semantics vs
  runtime helper fallback.

## M0 — Baseline and problem freeze

- [x] IIH-010 Audit the current install/freeze behavior across:
  - registry freeze helpers,
  - bootstrap registration,
  - first-party pack install seams,
  - generated pack install seams,
  - and installed-pack metadata recording.
- [x] IIH-011 Freeze the non-goals so this lane does not turn into a broad fallible bootstrap
  redesign.

## M1 — Contract freeze

- [x] IIH-020 Decide which surfaces are fail-fast:
  - explicit app install seams,
  - bootstrap contract registration,
  - and raw registry registration escape hatches.
- [x] IIH-021 Decide which surfaces remain best-effort:
  - runtime helper fallback,
  - preload helpers,
  - and other non-fallible convenience paths.
- [x] IIH-022 Freeze the metadata-conflict rule for `InstalledIconPacks`.

## M2 — Proof surface

- [x] IIH-030 Prove explicit install seams now fail fast on registry-freeze failure.
- [x] IIH-031 Prove explicit install seams now fail fast on metadata conflict.
- [x] IIH-032 Prove best-effort helper/preload paths preserve valid icons when unrelated entries
  are broken.
- [x] IIH-033 Prove generated pack output teaches the same strict install semantics.

## M3 — Gates and closeout

- [x] IIH-040 Run the cross-crate gate set for this narrow follow-on.
- [x] IIH-041 Refresh roadmap/index state and lane status after proof is real.
- [x] IIH-042 Close this lane explicitly instead of leaving install hardening as implicit drift.

## Boundaries to protect

- Do not widen this lane into a `Result`-based bootstrap redesign.
- Do not let helper convenience semantics leak into explicit install surfaces.
- Do not let one invalid entry collapse an otherwise usable frozen registry.
- Do not treat pack metadata conflicts as debug-only invariants now that provenance is explicit.

Completed M0 evidence:

- `docs/workstreams/icon-install-health-hardening-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/icon-install-health-hardening-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Completed M2 proof:

- `docs/workstreams/icon-install-health-hardening-v1/M2_PROOF_SURFACE_2026-04-09.md`

Closeout:

- `docs/workstreams/icon-install-health-hardening-v1/CLOSEOUT_AUDIT_2026-04-09.md`
