# Scroll Extents (DOM/GPUI Parity) — TODO Tracker

Status: Draft

Tracking format:

- ID: `SE-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## Evidence & Baseline

- [x] SE-010 Repro and evidence for “scroll probe stall” in UI Gallery.
  - Script: `tools/diag-scripts/ui-gallery-nav-card-click-latency.json`
  - Tracker: `docs/workstreams/ui-gallery-perf-scroll-measure.md`
- [ ] SE-020 Record a stable before/after table for dt_ms + layout_time_us for the repro script.
  - Goal: p95/max dt_ms < 33ms in dev builds for nav clicks on common pages.

## Design (Contract)

- [ ] SE-100 Define the scroll-extent contract: how extents are derived from layout geometry.
  - Include: padding/border, negative origins policy, rounding policy.
  - Include: interaction with overlays and anchoring.
- [ ] SE-110 Identify current “available size clamping” behaviors that prevent overflow.
  - List the minimum set of element types that must change to allow DOM-like overflow.

## Prototype (Behind a Gate)

- [ ] SE-200 Add an opt-in implementation (env flag or compile-time flag) for “post-layout scroll extents”.
  - Must not change default behavior yet.
- [ ] SE-210 Add focused unit tests around offset clamping + scrollbars + overlay reanchoring.

## Rollout

- [ ] SE-300 Turn the prototype into the default for vertical scrolling surfaces.
- [ ] SE-310 Keep an escape hatch for horizontal “true unbounded” cases if needed.

