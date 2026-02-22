---
title: Diagnostics Fearless Refactor v1 (Milestones)
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1 (Milestones)

## Milestone 1: Mechanical modularization is landed

Exit criteria:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` no longer contains the full per-frame script driver implementation.
- Script engine code (driver + helpers) lives in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- Script runner state types live in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`.
- `cargo check -p fret-ui-gallery` is green.

## Milestone 2: Sidecar indexes reduce day-to-day pain

Exit criteria:

- Tools can locate relevant snapshots without opening full `bundle.json` in memory.
- Sidecars are documented and versioned.
- Missing sidecars degrade gracefully (no “hang until timeout”).

## Milestone 3: Agent-friendly triage loop

Exit criteria:

- A maintainer can run a scripted repro, collect evidence, and generate a small “triage bundle” quickly.
- Evidence anchors in docs stay in sync with the implementation.
