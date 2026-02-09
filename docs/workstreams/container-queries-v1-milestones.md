---
title: Container Queries (v1) — Milestones
status: draft
date: 2026-02-09
scope: container queries contract + implementation + recipe migrations
---

# Container Queries (v1) — Milestones

This milestone plan is intentionally contract-adjacent: it defines "done" in terms of observable
outcomes and evidence anchors (tests / scripts), not internal implementation details.

ADR anchor:

- `docs/adr/1170-container-queries-and-frame-lagged-layout-queries-v1.md`

## Status (as of 2026-02-09)

- M0–M4 are implemented on `main` (mechanism + kit helpers + initial shadcn migrations).
- M5 is the next validation milestone (docking/panel reality check).

## M0 — Contract locked (mechanism + policy split)

Definition of done:

- ADR 1170 is `Accepted` and referenced from the relevant indices (`docs/adr/README.md`,
  `docs/todo-tracker.md`).
- Workstream docs exist (this file + TODO list).

Evidence:

- Links resolve and are discoverable from docs entrypoints.

## M1 — Runtime mechanism (frame-lagged layout queries)

Definition of done:

- A query-region mechanism exists in `crates/fret-ui`:
  - stable identity,
  - last-committed bounds query,
  - invalidation when the observed bounds change (epsilon + coalescing allowed).
- Diagnostics: query regions can be inspected (best-effort is fine in v1).

Evidence:

- Unit tests in `crates/fret-ui` cover:
  - frame-lagged behavior (no same-frame recursion),
  - invalidation on bounds change,
  - jitter threshold prevents rebuild storms.

## M2 — Policy helpers (kit surface)

Definition of done:

- `ecosystem/fret-ui-kit` exposes a small typed surface for container breakpoints:
  - width queries,
  - hysteresis helpers,
  - recommended breakpoint tokens (optional; viewport breakpoints may remain separate).

Evidence:

- Unit tests in `ecosystem/fret-ui-kit` cover:
  - hysteresis (up/down thresholds),
  - no oscillation under a shrinking/growing container loop.

## M3 — shadcn-facing integration scaffolding

Definition of done:

- `ecosystem/fret-ui-shadcn` has a single recommended way to express container-query-driven
  responsiveness (no ad-hoc `cx.bounds.size.width` thresholds scattered across recipes).
- Any remaining viewport-breakpoint approximations are explicitly labeled as temporary.

Evidence:

- A small helper surface exists in `ecosystem/fret-ui-shadcn` (or `fret-ui-kit`) used by migrated
  recipes.
- A docs note (audit or workstream entry) points to ADR 1170 as the contract anchor.

## M4 — First recipe migrations + regression gates

Definition of done:

- Multiple shadcn-aligned recipes stop using viewport-width approximation and use container queries
  instead (minimum: 2 recipes; prefer one "forms" recipe and one "navigation" recipe).
- Each migrated recipe is gated by an automated test (and optionally a web-vs-fret golden where
  relevant).

Recommended first targets:

- `Field(orientation="responsive")` (currently approximates `@md/field-group` via viewport width).
- `NavigationMenu` responsive behaviors that should track the menu's container width, not the window.

Evidence:

- `ecosystem/fret-ui-shadcn` tests updated/added for the migrated recipe.

## M5 — Docking/panel validation (editor reality check)

Definition of done:

- A docking demo/harness places migrated recipes into resizable panels and demonstrates correct
  adaptation as panels are resized, independent of the window size.
- Validation focuses on **panel resize without changing the window size**, to ensure we are not
  accidentally testing viewport breakpoints.

Evidence:

- A `fretboard diag` script (or a minimal harness test) that exercises panel resize and captures a
  stable outcome across DPIs.

## M6 — Ecosystem adoption sweep (incremental, not "big bang")

Definition of done:

- A tracked list of known viewport-width approximations exists, and we are steadily migrating them
  to container queries with regression gates.
- New responsive variants added to `fret-ui-shadcn` prefer container queries by default.

Evidence:

- The TODO list for this workstream contains an explicit "remaining approximations" section.
- PR review guidance (informal is fine) points contributors to the helper surface introduced in M3.
