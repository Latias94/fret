---
title: Environment Queries (v1) — Milestones
status: draft
date: 2026-02-09
scope: viewport/device capability queries contract + implementation + ecosystem adoption
---

# Environment Queries (v1) — Milestones

ADR anchor:

- `docs/adr/1171-environment-queries-and-viewport-snapshots-v1.md`

This milestone plan defines “done” in terms of observable outcomes (tests / diagnostics evidence),
not internal implementation details.

## M0 — Contract locked (mechanism vs policy)

Definition of done:

- ADR 1171 is `Accepted` and referenced from the relevant indices (`docs/adr/README.md`,
  `docs/todo-tracker.md`).
- Workstream docs exist (this file + TODO list).

## M1 — Runtime mechanism (committed per-window snapshot)

Definition of done:

- `crates/fret-ui` exposes a typed way to read a committed per-window environment snapshot (or
  fields from it) during declarative rendering.
- Observations participate in dependency tracking and invalidation.

Evidence:

- Unit tests in `crates/fret-ui` cover:
  - dependencies and invalidation when viewport bounds change,
  - view-cache key participation via an environment deps fingerprint.

## M2 — Diagnostics hooks

Definition of done:

- Diagnostics bundles capture:
  - the committed snapshot (or a summarized subset),
  - per-root observation lists and deps fingerprints (best-effort).

Evidence:

- A diagnostics bundle contains fields under a stable schema path (e.g. `debug.environment`), and
  tests (or a scripted gate) assert the fields exist.

## M3 — Policy helpers (kit surface)

Definition of done:

- `ecosystem/fret-ui-kit` exposes typed helpers for:
  - viewport/device breakpoints (tokens + optional hysteresis),
  - pointer capability gates (hover vs touch-first),
  - safe-area insets (future mobile),
  - reduced-motion defaults (if provided by the runner/app).

Evidence:

- Unit tests in `ecosystem/fret-ui-kit` cover hysteresis and non-oscillation behavior where
  applicable.

## M4 — Ecosystem adoption (shadcn + demos)

Definition of done:

- Known viewport-driven responsive recipes stop hard-coding `cx.bounds` thresholds and instead use
  the environment query helpers.
- Known hover-driven affordances are gated by pointer capability helpers (touch-first should not
  open hover-only UI).
- At least one migration is gated by an automated test or `fretboard diag` script.

Recommended first target:

- `Combobox(responsive)` Drawer-vs-Popover selection (device/viewport-driven shell behavior).
- `Tooltip` / `HoverCard` hover gating (pointer-capability-driven affordance).
