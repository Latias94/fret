# Generated Icon Presentation Defaults v1 — TODO

Status: Closed
Last updated: 2026-04-09

## Lane opening

- [x] GIP-001 Open this as a dedicated follow-on instead of widening the closed acquisition lane.
- [x] GIP-002 Record the shipped runtime/generator boundary and the assumptions for this narrower
  presentation-defaults lane.

## M0 — Scope and evidence freeze

- [x] GIP-010 Audit the current presentation baseline:
  - runtime icon contract,
  - generator registration path,
  - generator intermediate data model,
  - and acquisition metadata already available.
- [x] GIP-011 Freeze the non-goals so this lane does not turn into another runtime or acquisition
  redesign.

## M1 — Presentation policy contract freeze

- [x] GIP-020 Decide the source of truth for default presentation:
  - explicit import config,
  - SVG analysis,
  - collection-level hints,
  - or a layered policy.
- [x] GIP-021 Decide where that policy lives:
  - generator library contract,
  - `fretboard` CLI surface,
  - generated crate metadata,
  - or a split across those layers.
- [x] GIP-022 Decide the first proof target:
  - Iconify multicolor subset import,
  - local SVG directory with authored colors,
  - or both.

## M2 — Proof surface

- [x] GIP-030 Land one smallest proof that generated packs can register imported icons with the
  correct default presentation.
- [x] GIP-031 Prove monochrome imports still keep the existing mask-mode posture.
- [x] GIP-032 Prove authored-color imports reach the existing `OriginalColors` runtime path without
  manual pack edits.

## M3 — Gates and docs

- [x] GIP-040 Leave deterministic generator/runtime gates for the shipped policy.
- [x] GIP-041 Teach the presentation-defaults contract only after the policy is real.
- [x] GIP-042 Close this lane or split another narrower follow-on instead of widening scope by
  default.

## Boundaries to protect

- Do not reopen `iconify-acquisition-prestep-v1`.
- Do not hide presentation inference inside runtime rendering code.
- Do not silently change existing first-party pack policy without explicit proof.
- Do not mix semantic alias policy into this lane.

Completed M0 evidence:

- `docs/workstreams/generated-icon-presentation-defaults-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/generated-icon-presentation-defaults-v1/M1_CONTRACT_FREEZE_2026-04-09.md`

Completed M2 proof:

- `docs/workstreams/generated-icon-presentation-defaults-v1/M2_PROOF_SURFACE_2026-04-09.md`

Closeout:

- `docs/workstreams/generated-icon-presentation-defaults-v1/CLOSEOUT_AUDIT_2026-04-09.md`
