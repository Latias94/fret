---
title: Diagnostics Extensibility + Capabilities v1 (TODO)
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, ecosystem, portability
---

# Diagnostics Extensibility + Capabilities v1 (TODO)

This file tracks tasks for `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Guiding idea: keep `diag` useful for day-to-day debugging *and* safe to depend on as an ecosystem contract surface.

## Contract tasks

- [x] Write an extensibility-focused ADR:
  - [x] `docs/adr/0204-ui-diagnostics-extensibility-and-capabilities-v1.md`
- [ ] Define the initial capability vocabulary (names, semantics, and stability guarantees).
- [ ] Define and document the script metadata surface (`meta`):
  - [ ] `name`, `tags`, `required_capabilities`, `target_hints`,
  - [ ] rule: tooling ignores unknown `meta` keys.
- [ ] Document window targeting shape (future multi-window):
  - [ ] window selection rules (prefer stable ids, avoid index-only),
  - [ ] scoping rules for selector resolution.
- [ ] Document coordinate spaces for anchored coordinate injection (canvas fallback):
  - [ ] `window_local`,
  - [ ] `element_local_normalized` (preferred),
  - [ ] forbid raw screen coordinates in scripts.

## Tooling tasks (authoring ergonomics)

- [x] Add typed Rust helpers for building Script v2:
  - [x] `crates/fret-diag-protocol/src/builder.rs`
- [x] Add an internal script generator tool that emits JSON from typed templates:
  - [x] `apps/fret-diag-scriptgen`
- [ ] Add a script normalization command:
  - [ ] parse JSON (v1/v2), re-emit `to_string_pretty`, stable ordering where relevant.
- [ ] Add a script schema validation command:
  - [ ] fail fast with clear errors (missing fields, invalid enum values, unknown schema_version).
- [ ] Add “generate + check” workflows suitable for CI:
  - [ ] ensure generated scripts match checked-in scripts (when applicable),
  - [ ] avoid editing `tools/diag-scripts` by default (prefer `.fret/diag/scripts`).

## Runner + transport tasks (capabilities)

- [ ] Implement capability discovery:
  - [ ] devtools WS transport: handshake ack exposes supported capabilities.
  - [ ] filesystem-trigger transport: write a deterministic `capabilities.json` under `FRET_DIAG_DIR`.
- [ ] Implement capability gating in tooling:
  - [ ] `fretboard diag run` fails fast if required capabilities are missing,
  - [ ] `diag repro` includes the gating failure in `repro.summary.json`,
  - [ ] emit evidence file `check.capabilities.json` (machine-readable).

## Scenario coverage tasks (future-proofing)

- [ ] Multi-window:
  - [ ] one deterministic “open window B, assert focus/barrier invariants” smoke.
- [ ] Embedded viewport/canvas:
  - [ ] one demo that projects objects into semantics (`test_id`) for stable automation.
  - [ ] one fallback demo using anchored normalized coordinates (capability-gated).
- [ ] Mobile alignment (future):
  - [ ] define touch pointer kind surface and basic gestures in protocol (capability-gated).

## CI tasks (guardrails)

- [ ] Add a small smoke suite for CI that:
  - [ ] avoids pixel assertions by default,
  - [ ] uses only `test_id`/role selectors,
  - [ ] runs with predictable timeouts.
- [ ] Add checks that protect contract evolution:
  - [ ] script schema validation for the smoke suite,
  - [ ] normalization check (avoid “random diff churn”).

