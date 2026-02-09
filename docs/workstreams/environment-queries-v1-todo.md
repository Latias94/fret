---
title: Environment Queries (v1) — TODO
status: draft
date: 2026-02-09
---

# Environment Queries (v1) — TODO

Workstream entry:

- `docs/workstreams/environment-queries-v1.md`

ADR anchor:

- `docs/adr/1171-environment-queries-and-viewport-snapshots-v1.md`

## Contract / docs

- [ ] Land ADR 1171 (Accepted).
- [ ] Add ADR 1171 to `docs/adr/README.md` task jump table near ADR 1170.
- [ ] Add a `docs/todo-tracker.md` entry for environment queries (viewport/device).

## Runtime mechanism (`crates/fret-ui`)

- [ ] Define a per-window committed environment snapshot storage surface.
- [ ] Expose a typed query API that records dependencies during declarative rendering.
- [ ] Dependency tracking: record which view roots observed which environment keys.
- [ ] Invalidation: environment changes invalidate dependents (coalescing OK).
- [ ] Add diagnostics hooks (inspector snapshot / debug logging).
- [ ] Add unit tests for:
  - [ ] invalidation on viewport bounds change,
  - [ ] view-cache key participation via deps fingerprint,
  - [ ] stability under resize jitter (optional epsilon/hysteresis at policy layer).

## Policy helpers (`ecosystem/fret-ui-kit`)

- [ ] Add environment query helper surface:
  - [ ] viewport breakpoint tokens (Tailwind-aligned labels, optional),
  - [ ] pointer capability gates (hover vs touch-first),
  - [ ] reduced-motion preference helpers (if available).
- [ ] Add unit tests for hysteresis / non-oscillation where applicable.

## Ecosystem adoption (initial targets)

- [ ] Migrate `Combobox(responsive)` to use environment query helpers instead of `cx.bounds` magic
  numbers for the mobile shell decision (Drawer vs Popover).
- [ ] Add a regression gate (test or `fretboard diag` script) for the migration.

