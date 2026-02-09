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

- [ ] Define a query region wrapper element kind (pass-through layout; records committed bounds).
- [ ] Expose a frame-lagged query API (read committed bounds by stable ID).
- [ ] Dependency tracking: record which view roots observed which regions.
- [ ] Invalidation: bounds change invalidates dependents (epsilon + coalescing).
- [ ] Add diagnostics hooks (inspector snapshot / debug logging).
- [ ] Add unit tests for:
  - [ ] frame-lagged semantics (no same-frame recursion),
  - [ ] invalidation on bounds change,
  - [ ] jitter threshold.

## Policy helpers (`ecosystem/fret-ui-kit`)

- [ ] Add `ContainerQuery` helper types:
  - [ ] `width_at_least(threshold, hysteresis_px)`,
  - [ ] `breakpoint_md_like()` (optional convenience).
- [ ] Add unit tests for hysteresis / non-oscillation.

## Recipe migrations (`ecosystem/fret-ui-shadcn`)

- [ ] Migrate `Field(orientation="responsive")` away from viewport-width breakpoint.
  - Evidence: `ecosystem/fret-ui-shadcn/src/field.rs` no longer hard-codes `>=768px` for the
    container-query approximation path.
- [ ] Pick a second shadcn recipe that currently keys off viewport width but should key off panel
  width (candidate: `NavigationMenu`), and migrate it with a regression gate.
- [ ] Add a regression gate for the migrated behavior in a resizable panel (docking harness or a
  focused unit test).
