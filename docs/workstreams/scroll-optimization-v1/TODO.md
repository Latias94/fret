# Scroll Optimization Workstream (v1) — TODO

Date: 2026-03-02  
Status: Draft

## Gates-first checklist

- [ ] Confirm baseline scripts pass:
  - [ ] `ui-gallery-scroll-area-wheel-scroll`
- [ ] Add a repro/gate for “thumb drag stability under content growth”.
- [ ] Add a perf-oriented wheel torture script (no layout solves, no jank threshold yet).

## Mechanism hardening

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
- [ ] Implement behind a runtime knob (opt-in) with a clear default.
- [ ] Add diag evidence:
  - [ ] stress wheel in a scroll area,
  - [ ] stress wheel in a virtual list,
  - [ ] nested scrollable case (vlist containing a horizontal scroll surface).

## Scrollbar drag stability

- [ ] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [ ] Update thumb math while dragging to use baseline.
- [ ] Add diag script + pixel or semantics assertions.

## Extents probing / observation

- [ ] Add diag script for “expand at bottom” (pinned extents regression).
- [ ] Validate post-layout observation budgets:
  - [ ] wrapper peel budget hit triggers a probe next frame,
  - [ ] deep scan budget hit triggers a probe next frame.

