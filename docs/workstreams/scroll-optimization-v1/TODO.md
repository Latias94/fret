# Scroll Optimization Workstream (v1) — TODO

Date: 2026-03-03  
Status: Draft

## Gates-first checklist

- [x] Confirm baseline scripts pass:
  - [x] `ui-gallery-scroll-area-wheel-scroll` (bundle: `target/fret-diag/1772468071457-scroll-area-wheel-scroll`, 2026-03-02)
  - [x] `ui-gallery-scrollbar-drag-baseline-content-growth` (bundle: `target/fret-diag/1772498133742-scrollbar-drag-baseline-content-growth`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-wheel-torture` (bundle: `target/fret-diag/1772498149599-scroll-area-wheel-torture`, 2026-03-03)

## Mechanism hardening

- [x] Fix view-cache contained relayout bookkeeping (layout invalidation clears must keep subtree aggregation in sync):
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- [ ] Audit all barrier-related paths that can affect scroll surfaces:
  - [ ] child list mutation helpers,
  - [ ] contained relayout scheduling,
  - [ ] subtree dirty aggregation bookkeeping.
- [ ] Add/extend unit tests to cover:
  - [ ] barrier relayout sets `subtree_layout_dirty_count` consistently,
  - [ ] scroll handle revision-only bumps stay classified correctly.

## Wheel/trackpad delta coalescing

- [ ] Decide coalescing layer:
  - [ ] runner/platform (preferred),
  - [ ] UI core (fallback).
- [x] Implement behind a runtime knob (opt-in) with a clear default.
  - [x] Native (winit): `FRET_WINIT_COALESCE_WHEEL=1` (coalesce consecutive wheel events).
- [ ] Add diag evidence:
  - [x] stress wheel in a scroll area (`ui-gallery-scroll-area-wheel-torture`),
  - [ ] stress wheel in a virtual list,
  - [ ] nested scrollable case (vlist containing a horizontal scroll surface).

## Scrollbar drag stability

- [x] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [x] Update thumb math while dragging to use baseline.
- [x] Add diag script + semantics assertions (`ui-gallery-scrollbar-drag-baseline-content-growth`).

## Extents probing / observation

- [ ] Add diag script for “expand at bottom” (pinned extents regression).
- [ ] Validate post-layout observation budgets:
  - [ ] wrapper peel budget hit triggers a probe next frame,
  - [ ] deep scan budget hit triggers a probe next frame.
