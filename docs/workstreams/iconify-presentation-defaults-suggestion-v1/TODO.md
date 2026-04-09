# Iconify Presentation Defaults Suggestion v1 — TODO

Status: Closed
Last updated: 2026-04-09

## Lane opening

- [x] IPS-001 Open a narrow follow-on instead of reopening the closed generated-defaults lane.
- [x] IPS-002 Record assumptions and boundary ownership for provenance-driven suggestion only.

## M0 — Scope and evidence freeze

- [x] IPS-010 Audit the current generator/import contract, acquisition provenance contract, and
  the missing helper gap.
- [x] IPS-011 Freeze the non-goals so this lane does not turn into another hidden defaulting rule.

## M1 — Suggestion contract freeze

- [x] IPS-020 Decide where the helper lives.
- [x] IPS-021 Decide the v1 input/output contract.
- [x] IPS-022 Decide how missing provenance evidence behaves.

## M2 — Proof surface

- [x] IPS-030 Land a thin `icons suggest presentation-defaults` CLI surface.
- [x] IPS-031 Prove the helper emits the existing versioned config shape.
- [x] IPS-032 Prove the emitted config flows into the existing import path.

## M3 — Docs and closeout

- [x] IPS-040 Leave deterministic gates for the helper.
- [x] IPS-041 Teach the helper as advisory, not normative import policy.
- [x] IPS-042 Close this lane or split another narrower follow-on.

## Boundaries to protect

- Do not turn `palette` into the hidden default for `icons import`.
- Do not move this helper into runtime crates.
- Do not silently add SVG heuristics or mixed-pack inference here.
- Do not reopen the closed acquisition or generated-defaults lanes.

Completed M0 evidence:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Completed M2 proof:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/M2_PROOF_SURFACE_2026-04-09.md`

Closeout:

- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/CLOSEOUT_AUDIT_2026-04-09.md`
