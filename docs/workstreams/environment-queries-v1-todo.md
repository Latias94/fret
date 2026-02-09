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

- [x] Land ADR 1171 (Accepted).
- [x] Add ADR 1171 to `docs/adr/README.md` task jump table near ADR 1170.
- [x] Add a `docs/todo-tracker.md` entry for environment queries (viewport/device).

## Runtime mechanism (`crates/fret-ui`)

- [x] Define a per-window committed environment snapshot storage surface.
- [x] Expose a typed query API that records dependencies during declarative rendering.
- [x] Dependency tracking: record which view roots observed which environment keys.
- [x] Invalidation: environment changes invalidate dependents (coalescing OK).
- [x] Add diagnostics hooks (inspector snapshot / debug logging).
- [ ] Add unit tests for:
  - [x] invalidation on viewport bounds change,
  - [x] view-cache key participation via deps fingerprint,
  - [ ] stability under resize jitter (optional epsilon/hysteresis at policy layer).

## Policy helpers (`ecosystem/fret-ui-kit`)

- [x] Add environment query helper surface:
  - [x] viewport breakpoint tokens (Tailwind-aligned labels, optional),
  - [ ] pointer capability gates (hover vs touch-first),
  - [ ] reduced-motion preference helpers (if available).
- [x] Add unit tests for hysteresis / non-oscillation where applicable.

## Ecosystem adoption (initial targets)

- [x] Migrate `Combobox(responsive)` to use environment query helpers instead of `cx.bounds` magic
  numbers for the mobile shell decision (Drawer vs Popover).
- [x] Add a regression gate (test or `fretboard diag` script) for the migration.
  - Evidence: `fret-ui-shadcn::web_vs_fret_overlay_placement::fret_combobox_responsive_drawer_blocks_underlay_scroll_on_mobile`
