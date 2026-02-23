# A11y semantics closure (v1) — Milestones

Last updated: 2026-02-23

## Current progress (2026-02-23)

- M1: Complete (pressed semantics: contract + AccessKit mapping + shadcn gates + ADR 0290).
- M2: Complete (required/invalid semantics: contract + AccessKit mapping + shadcn gates + ADR 0291).

## M0 — Inventory and priority agreement

Exit criteria:

- The closure checklist is agreed (contract → adapter → production → diagnostics → gates).
- P0 surfaces are selected (pressed, required/invalid, busy).
- Each surface has a chosen smallest adoption target (shadcn component) and a gate type (snapshot vs diag script).

## M1 — Pressed semantics closed

Exit criteria:

- Portable contract exists in `crates/fret-core` and is documented.
- AccessKit adapter maps it (or documents non-support) with unit tests.
- shadcn publishes pressed semantics for at least one toggle control.
- Snapshot gate(s) assert pressed semantics deterministically.

## M2 — Required/invalid semantics closed

Exit criteria:

- Portable contract exists and is documented.
- AccessKit mapping exists with unit tests (or explicit fallback behavior).
- shadcn input-like control publishes required/invalid state.
- Gate exists (snapshot and/or diag script).

## M3 — Busy semantics closed

Exit criteria:

- Portable contract exists and is documented.
- AccessKit mapping exists (or documented fallback).
- shadcn adoption exists for at least one real loading surface.
- Gate exists (snapshot and/or diag script).
