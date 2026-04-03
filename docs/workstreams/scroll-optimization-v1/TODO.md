# Scroll Optimization Workstream (v1) — TODO

Date: 2026-03-03  
Status: Active

## Current slice — Deferred probe seed vs authoritative extent

- [x] Make deferred probe policy read retained seed state before deciding to skip a deep probe.
- [x] Allow deferred invalidation frames to consume `intrinsic_measure_cache` as the seed extent
  when retained child measured sizes are absent.
- [x] Centralize authoritative extent commits so pending probe clearing only happens on explicit
  probe / authoritative observation paths.
- [x] Ensure unchanged authoritative post-layout observation still clears deferred invalidation
  pending state instead of forcing an extra at-edge probe on the next frame.
- [x] Record the dedicated verification results for the seed/authority regression gates in
  `EVIDENCE_AND_GATES.md`.

## Gates-first checklist

- [x] Confirm baseline scripts pass:
  - [x] `ui-gallery-scroll-area-wheel-scroll` (bundle: `target/fret-diag/1772468071457-scroll-area-wheel-scroll`, 2026-03-02)
  - [x] `ui-gallery-scrollbar-drag-baseline-content-growth` (bundle: `target/fret-diag/1772498133742-scrollbar-drag-baseline-content-growth`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-wheel-torture` (bundle: `target/fret-diag/1772498149599-scroll-area-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-nested-scroll-routing` (bundle: `target/fret-diag-scroll-nested-debug6/sessions/1772508480737-75452/1772508483614-scroll-area-nested-scroll-routing`, 2026-03-03)
  - [x] `ui-gallery-wheel-burst-coalescing` (new gate: wheel events per frame ≤ 1; suite: `diag-hardening-smoke`)
    - `diag run` evidence: `target/fret-diag-runs/1772530803405-wheel-burst/check.wheel_events_max_per_frame.json` (2026-03-03)
  - [x] `ui-gallery-virtual-list-wheel-torture` (bundle: `target/fret-diag-vlist-wheel/sessions/1772508526189-62940/1772508528623-virtual-list-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-toggle-code-tabs` (bundle: `target/fret-diag-underflow-check/sessions/1772500876247-61448/1772500879851-scroll-area-toggle-code-tabs`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-expand-at-bottom` (bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-scroll-area` (bundle: `target/fret-perf-scroll-area/sessions/1772501734226-65632/1772501741770`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-virtual-list` (bundle: `target/fret-perf-vlist/1772508561962`, 2026-03-03)
- [x] Promote nested scroll routing into `diag-hardening-smoke`:
  - suite manifest: `tools/diag-scripts/suites/diag-hardening-smoke/suite.json`
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-nested-scroll-routing.json`

## Mechanism hardening

- [x] Fix view-cache contained relayout bookkeeping (layout invalidation clears must keep subtree aggregation in sync):
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- [x] Keep layout-only contained relayout from forcing next-frame cache-root rerender / dirty-view carry-over:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- [x] Prune detached roots from layout follow-up scheduling before final layout:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- [ ] Audit all barrier-related paths that can affect scroll surfaces:
  - [ ] child list mutation helpers,
  - [ ] contained relayout scheduling,
  - [x] subtree dirty aggregation bookkeeping.
- [ ] Add/extend unit tests to cover:
  - [x] barrier relayout sets `subtree_layout_dirty_count` consistently,
  - [ ] scroll handle revision-only bumps stay classified correctly.

## Wheel/trackpad delta coalescing

- [ ] Decide coalescing layer:
  - [ ] runner/platform (preferred),
  - [ ] UI core (fallback).
- [x] Implement behind a runtime knob (opt-in) with a clear default.
  - [x] Native (winit): `FRET_WINIT_COALESCE_WHEEL=1` (coalesce consecutive wheel events).
- [x] Add a max-abs delta guardrail for a single coalesced wheel event (still needs perf validation on VirtualList):
  - `FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` (default: `120`)
- [x] Implement frame-boundary buffering in the desktop runner (deliver ≤ 1 wheel per frame when enabled):
  - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
- [x] Add a runner-level “same-frame wheel burst” regression gate:
  - Script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-wheel-burst-coalescing.json`
  - Suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-wheel-burst-coalescing.json`
  - Post-run check: `check.wheel_events_max_per_frame.json`
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
- [x] Re-run repeat=11 perf after adding the max-abs cap (2026-03-03):
  - Goal: keep `perf-ui-gallery-scroll-area` improved, remove `perf-ui-gallery-virtual-list` p95 regression.
  - Result (pre frame-boundary buffering): `cap=120` still shows high variance for VirtualList; see the “Full rerun (pre frame-boundary buffering)” section below.
  - Result (after frame-boundary buffering): `cap=120` is stable for both scripts; see the “Frame-boundary buffering rerun” section below.

### Rerun (2026-03-03) — max-abs cap default (`120`)

Short rerun (repeat=5, warmup=10) to sanity-check the new default cap behavior:

- `perf-ui-gallery-virtual-list`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `10963/11109`
    - worst bundle: `target/fret-diag/1772514040891/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `10424/11102`
    - worst bundle: `target/fret-diag/1772513954382/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r5.log`

- `perf-ui-gallery-scroll-area`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `30567/49376`
    - worst bundle: `target/fret-diag/1772513830516/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `28054/29181`
    - worst bundle: `target/fret-diag/1772513987367/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r5.log`
- [ ] Add diag evidence:
  - [x] stress wheel in a scroll area (`ui-gallery-scroll-area-wheel-torture`),
  - [x] stress wheel in a virtual list (`ui-gallery-virtual-list-wheel-torture`),
  - [x] nested scrollable case (inner X should not consume Y wheel: `ui-gallery-scroll-area-nested-scroll-routing`).

Full rerun (repeat=11, warmup=10) with explicit env overrides (2026-03-03, pre frame-boundary buffering):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10619/12213` / `9945/11573` / `2987/3319`
    - worst bundle: `target/fret-diag/1772517019308/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `11611/24223` / `10978/22757` / `3258/5983`
    - worst bundle: `target/fret-diag/1772517054654/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `10872/12343` / `10231/11625` / `3042/3281`
    - worst bundle: `target/fret-diag/1772517987201/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap60-r11-20260303c.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `27674/28643` / `26613/27521` / `2844/3019`
    - worst bundle: `target/fret-diag/1772517184852/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `27873/28904` / `26766/27801` / `2859/3254`
    - worst bundle: `target/fret-diag/1772517215826/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `29862/32033` / `28487/30812` / `2965/3549`
    - worst bundle: `target/fret-diag/1772518038237/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap60-r11-20260303c.log`

Notes:

- Current evidence suggests the cap is workload-sensitive:
  - `cap=120` is acceptable for `scroll-area` but shows high variance/regression in `virtual-list`.
  - `cap=60` removes the `virtual-list` spikes but regresses `scroll-area` in this torture script.
- Follow-up (partially resolved): frame-boundary buffering makes `cap=120` stable in repeat=11 for both scripts; next step is deciding if/when this becomes default-on across platforms.

Frame-boundary buffering rerun (repeat=11, warmup=10) (2026-03-03):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10927/12140` / `10263/11451` / `2986/3307`
    - worst bundle: `target/fret-diag/1772519046872/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `10729/11614` / `10099/10922` / `2985/3187`
    - worst bundle: `target/fret-diag/1772519094741/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-frame-cap120-r11-20260303.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `28544/52680` / `27404/50496` / `2855/5707`
    - worst bundle: `target/fret-diag/1772519164488/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `29282/31195` / `28203/30055` / `2957/3434`
    - worst bundle: `target/fret-diag/1772519183814/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-frame-cap120-r11-20260303.log`

## Perf harness plumbing

- [x] Allow `fretboard diag perf perf-ui-gallery-scroll-area` to resolve via the promoted scripts registry:
  - `crates/fret-diag/src/perf_seed_policy.rs`

## Scrollbar drag stability

- [x] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [x] Update thumb math while dragging to use baseline.
- [x] Add diag script + semantics assertions (`ui-gallery-scrollbar-drag-baseline-content-growth`).

## Extents probing / observation

- [x] Add diag script for “expand at bottom” (pinned extents regression):
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json`
  - suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-scroll-area-expand-at-bottom.json`
  - bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297` (2026-03-03)
- [x] Validate post-layout observation budgets:
  - [x] wrapper peel budget hit triggers a probe next frame,
  - [x] deep scan budget hit triggers a probe next frame.
  - test: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (`scroll_post_layout_observation_budget_hit_schedules_probe_next_frame`)
