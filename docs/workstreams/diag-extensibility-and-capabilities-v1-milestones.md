---
title: Diagnostics Extensibility + Capabilities v1 (Milestones)
status: draft
date: 2026-02-10
scope: diagnostics, automation, protocol, ecosystem, portability
---

# Diagnostics Extensibility + Capabilities v1 (Milestones)

This file tracks milestones for `docs/workstreams/diag-extensibility-and-capabilities-v1.md`.

Conventions:

- prefer “contract + evidence” over ad-hoc behavior;
- never require coordinates when a semantics selector can work;
- every milestone should end with a runnable demo (native and/or web).

## Milestones

### M0: Contract closure (docs-first)

- [x] Add an ADR for extensibility and capability negotiation:
  - [x] `docs/adr/0204-ui-diagnostics-extensibility-and-capabilities-v1.md`
- [x] Add this workstream doc + TODO + milestones.
- [ ] Decide the initial capability vocabulary to treat as stable.
- [ ] Decide the minimal script metadata shape (`meta`) and enforce “unknown fields ignored”.

### M1: Typed authoring ergonomics (keep JSON as the artifact)

- [x] Add a small, typed Rust script builder that produces Script v2.
- [x] Add a minimal script generator tool that emits JSON from Rust templates.
- [ ] Add a “normalize/pretty-print script JSON” tool (for stable diffs and reviews).
- [ ] Add a “schema validate” tool for scripts (CI-friendly).

### M2: Capability negotiation (fail fast, not by timeout)

- [ ] Define `required_capabilities` in script metadata.
- [ ] Implement capability discovery for:
  - [ ] filesystem-trigger transport,
  - [ ] devtools WS transport.
- [ ] Make `fretboard diag run/repro/suite` fail fast when required capabilities are missing.
- [ ] Emit a machine-readable evidence file (e.g. `check.capabilities.json`) when gating fails.

### M3: Ecosystem-friendly script discovery and suites

- [ ] Allow `fretboard diag suite` to accept:
  - [ ] `--script-dir <path>`,
  - [ ] `--glob <pattern>`,
  - [ ] multiple directories (workspace + `.fret/diag/scripts` + external).
- [ ] Add a recommended “smoke” suite definition that is stable across platforms.

### M4: Multi-window targeting (native first, portable shape)

- [ ] Define a window selector shape usable from scripts (avoid index-only selection).
- [ ] Add optional window scoping to actions and predicates.
- [ ] Provide at least one cross-window smoke script to validate the contract.

### M5: Canvas / embedded viewport automation (semantics projection first)

- [ ] Add guidelines for projecting canvas-interactive objects into semantics (`test_id`).
- [ ] Define an anchored coordinate injection fallback (element-local normalized coordinates).
- [ ] Gate coordinate injection behind a capability (avoid silent flake).

### M6: Touch/gesture (future mobile alignment)

- [ ] Define pointer-kind support and minimal gesture steps (tap/long-press/swipe/pinch).
- [ ] Ensure all gesture steps are optional and capability-gated.

### M7: CI guardrails for contract evolution

- [ ] Add CI checks for:
  - [ ] script schema validation,
  - [ ] script normalization (stable formatting),
  - [ ] capability metadata correctness for smoke suites.
- [ ] Add a minimal native smoke that can run reliably on CI runners with a window system.

