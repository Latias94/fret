# Iconify Acquisition Pre-step v1 — TODO

Status: Active
Last updated: 2026-04-09

## Lane opening

- [x] IAP-001 Open this as a dedicated follow-on instead of reopening
  `iconify-import-pack-generator-v1`.
- [x] IAP-002 Record the shipped generator-lane boundary and the assumptions for this narrower
  acquisition lane.

## M0 — Scope and evidence freeze

- [x] IAP-010 Audit the current acquisition reference surface:
  - closed generator lane docs,
  - current generator input expectations,
  - optional local `repo-ref/dioxus-iconify` fetch workflow reference.
- [x] IAP-011 Freeze the non-goals so this lane does not turn into another generator or runtime
  redesign.

## M1 — Acquisition contract freeze

- [x] IAP-020 Decide what artifact shape the acquisition pre-step emits:
  - full Iconify collection snapshot,
  - subset collection snapshot,
  - or snapshot + explicit provenance sidecar.
- [x] IAP-021 Decide where the public acquisition surface lives:
  - `fretboard` command only,
  - library + thin CLI split,
  - or another narrower ownership line if proof demands it.
- [x] IAP-022 Decide what pinning/provenance facts must be recorded:
  - collection prefix,
  - requested icon set or subset list,
  - source URL/template,
  - and whether acquisition-time metadata belongs in a sidecar file.
- [x] IAP-023 Decide whether the first proof should target:
  - one full collection snapshot,
  - or one explicit subset snapshot that still matches the generator's accepted input shape.

## M2 — Proof surface

- [ ] IAP-030 Land one smallest acquisition proof that produces a pinned local artifact from remote
  Iconify state without hiding network inside `icons import`.
- [ ] IAP-031 Prove the acquired artifact can flow into the existing generator/import path without
  manual cleanup.
- [ ] IAP-032 Keep the workflow explicitly two-step:
  - acquisition,
  - then pack generation.

## M3 — Gates and docs

- [ ] IAP-040 Leave one deterministic gate for the acquisition proof.
- [ ] IAP-041 Teach the acquisition pre-step in user-facing docs only after the contract is real.
- [ ] IAP-042 Close this lane or split another narrower follow-on instead of widening scope by
  default.

## Boundaries to protect

- Do not add live fetch to `fretboard icons import ...`.
- Do not reopen `iconify-import-pack-generator-v1`.
- Do not add a runtime Iconify client to framework crates.
- Do not guess semantic aliases from remote vendor data.

Completed M0 evidence:

- `docs/workstreams/iconify-acquisition-prestep-v1/BASELINE_AUDIT_2026-04-09.md`

Completed M1 decision:

- `docs/workstreams/iconify-acquisition-prestep-v1/M1_CONTRACT_FREEZE_2026-04-09.md`
