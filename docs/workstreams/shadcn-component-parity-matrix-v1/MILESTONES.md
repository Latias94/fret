---
title: Shadcn Component Parity Matrix v1 Milestones
status: active
date: 2026-05-25
---

# Milestones

## M0 - Lane Opened

Status: complete on 2026-05-25.

The lane owns the component-level harness matrix. It does not reopen
`component-parity-fact-harness-v1`; that lane remains the packet-shape foundation.

## M1 - Initial Matrix Generated

Status: complete on 2026-05-25.

Completed criteria:

- The generator produces a 59-component matrix from repo-local evidence.
- The matrix distinguishes `regression_locked`, `harness_hardening`, `coverage_targeted`,
  `inventory_only`, and `not_in_harness`.
- The first summary shows the current automation ceiling:
  - 18 components have source refs,
  - 14 have upstream DOM/CSS snapshots,
  - 18 have Fret layout evidence,
  - 10 have Fret bundle semantics evidence,
  - 1 has Fret text/paint evidence,
  - 15 have behavior scripts,
  - 5 have responsive/non-desktop coverage.

## M2 - First Matrix-Driven Repair Seed

Status: pending.

Pick one high-risk `inventory_only` or `coverage_targeted` row and promote it through the full
harness path before broadening the matrix model again.
