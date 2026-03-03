# Scroll Optimization Workstream (v1) — TODO

Date: 2026-03-03  
Status: Draft

## Gates-first checklist

- [x] Confirm baseline scripts pass:
  - [x] `ui-gallery-scroll-area-wheel-scroll` (bundle: `target/fret-diag/1772468071457-scroll-area-wheel-scroll`, 2026-03-02)
  - [x] `ui-gallery-scrollbar-drag-baseline-content-growth` (bundle: `target/fret-diag/1772498133742-scrollbar-drag-baseline-content-growth`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-wheel-torture` (bundle: `target/fret-diag/1772498149599-scroll-area-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-nested-scroll-routing` (bundle: `target/fret-diag-scroll-nested-debug6/sessions/1772508480737-75452/1772508483614-scroll-area-nested-scroll-routing`, 2026-03-03)
  - [x] `ui-gallery-virtual-list-wheel-torture` (bundle: `target/fret-diag-vlist-wheel/sessions/1772508526189-62940/1772508528623-virtual-list-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-toggle-code-tabs` (bundle: `target/fret-diag-underflow-check/sessions/1772500876247-61448/1772500879851-scroll-area-toggle-code-tabs`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-scroll-area` (bundle: `target/fret-perf-scroll-area/sessions/1772501734226-65632/1772501741770`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-virtual-list` (bundle: `target/fret-perf-vlist/1772508561962`, 2026-03-03)
- [x] Promote nested scroll routing into `diag-hardening-smoke`:
  - redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-scroll-area-nested-scroll-routing.json`

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
- [x] Collect repeatable perf evidence (repeat=11, warmup=10):
  - `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `30777/46060` / `29402/43910` / `3072/4510`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-off-r11/1772509265134/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `28134/29352` / `26956/28203` / `2859/3036`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-on-r11/1772509316761/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-on-r11.log`
  - `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `10910/11393` / `10180/10595` / `2996/3363`
      - worst bundle: `target/fret-perf-vlist-coalesce-off-r11/1772509365874/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `11870/18175` / `11185/17468` / `3437/5012`
      - worst bundle: `target/fret-perf-vlist-coalesce-on-r11/1772509420507/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-on-r11.log`
- [ ] Add diag evidence:
  - [x] stress wheel in a scroll area (`ui-gallery-scroll-area-wheel-torture`),
  - [x] stress wheel in a virtual list (`ui-gallery-virtual-list-wheel-torture`),
  - [x] nested scrollable case (inner X should not consume Y wheel: `ui-gallery-scroll-area-nested-scroll-routing`).

## Perf harness plumbing

- [x] Allow `fretboard diag perf perf-ui-gallery-scroll-area` to resolve via the promoted scripts registry:
  - `crates/fret-diag/src/perf_seed_policy.rs`

## Scrollbar drag stability

- [x] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [x] Update thumb math while dragging to use baseline.
- [x] Add diag script + semantics assertions (`ui-gallery-scrollbar-drag-baseline-content-growth`).

## Extents probing / observation

- [ ] Add diag script for “expand at bottom” (pinned extents regression).
- [ ] Validate post-layout observation budgets:
  - [ ] wrapper peel budget hit triggers a probe next frame,
  - [ ] deep scan budget hit triggers a probe next frame.
