---
title: Container Queries (v1) — TODO
status: draft
date: 2026-02-09
---

# Container Queries (v1) — TODO

Workstream entry:

- `docs/workstreams/container-queries-v1.md`

ADR anchor:

- `docs/adr/1170-container-queries-and-frame-lagged-layout-queries-v1.md`

## Contract / docs

- [x] Land ADR 1170 (Accepted).
- [x] Add workstream docs + milestones + TODO tracker.
- [x] Add ADR 1170 to `docs/adr/README.md` task jump table under layout.
- [x] Add a `docs/known-issues.md` entry for "viewport breakpoint approximation" until recipes are migrated.

## Runtime mechanism (`crates/fret-ui`)

- [x] Define a query region wrapper element kind (pass-through layout; records committed bounds).
- [x] Expose a frame-lagged query API (read committed bounds by stable ID).
- [x] Dependency tracking: record which view roots observed which regions.
- [x] Invalidation: bounds change invalidates dependents (coalescing OK; avoid same-frame recursion).
- [ ] Add diagnostics hooks (inspector snapshot / debug logging) beyond best-effort naming.
- [ ] Add unit tests for:
  - [x] frame-lagged semantics (no same-frame recursion),
  - [x] invalidation on bounds change,
  - [ ] jitter threshold / epsilon handling.

## Policy helpers (`ecosystem/fret-ui-kit`)

- [x] Add container query helper surface:
  - [x] query region wrappers (mechanism-friendly; paint/input transparent),
  - [x] breakpoint selection with hysteresis,
  - [x] optional Tailwind-compatible breakpoint tokens.
- [x] Add unit tests for hysteresis / non-oscillation.

## Recipe migrations (`ecosystem/fret-ui-shadcn`)

- [x] Migrate `Field(orientation="responsive")` away from viewport-width breakpoint.
  - Evidence: `ecosystem/fret-ui-shadcn/src/field.rs` no longer hard-codes `>=768px` for the
    container-query approximation path.
- [x] Migrate a second shadcn recipe that currently keys off viewport width but should key off panel
  width (`NavigationMenu`), and gate it with an automated test.
- [ ] Add a regression gate for the migrated behavior in a resizable panel (docking harness or a
  focused unit test).

## Remaining approximations (audit list)

- [x] `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`: responsive footer layout now tracks the
  dialog's committed container width.
- [ ] `ecosystem/fret-ui-shadcn/src/calendar*.rs`: audit which responsive branches should follow
  container width (panels) vs viewport width (device-size).
- [ ] `ecosystem/fret-ui-shadcn/src/empty.rs`: padding/layout should follow container width, not
  viewport width.
