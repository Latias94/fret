---
title: UI Performance: Zed-level Smoothness v1 (TODO)
status: draft
date: 2026-02-02
scope: performance, profiling, data-structures, caching, input, layout, paint
---

# UI Performance: Zed-level Smoothness v1 (TODO)

This file tracks milestones and concrete tasks for:

- `docs/workstreams/ui-perf-zed-smoothness-v1.md`

Conventions:

- “Contract” items should land with an ADR (or an update to an existing ADR).
- “Perf gate” items should land with a runnable `fretboard diag perf` command and a baseline/threshold update.
- “Fearless refactor” items should include: (1) perf evidence, (2) correctness evidence, (3) rollback plan.

## Current priorities (updated 2026-02-07)

- [ ] **P0 Resize-drag smoothness**: reduce `layout/solve` costs and eliminate avoidable secondary probes under
  `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`.
  - Companion probe (width jitter / live-drag approximation):
    `tools/diag-scripts/ui-gallery-window-resize-drag-jitter-steady.json`.
- [ ] **P1 Text under width jitter**: stabilize wrapped-text cache keys (and consider bucketed widths during resize).
- [ ] **P2 GPU vs CPU attribution**: make “GPU stall vs CPU work” obvious from diag bundles / captures.

## Milestones

Execution plan:

- `docs/workstreams/ui-perf-zed-smoothness-v1-execution-plan.md`

### M0: Baseline + suite gates (make perf a contract)

- [ ] Decide Tier A / Tier B thresholds per script (initially “best-effort”, then tighten).
- [x] Decide what `--launch` represents (cold-start gate vs steady-state gate) and codify it.
  - `ui-gallery` + `--launch`: cold-start gate (mount + first interaction).
  - `ui-gallery-steady` + `--reuse-launch` + `--launch`: steady-state gate (post-mount interactions).
- [ ] Finalize the acceptance suite list (see `ui-perf-zed-smoothness-v1.md`) and keep it small.
  - Ensure it includes at least one editor-grade text surface (`ui-gallery-code-editor-torture-autoscroll-steady.json`).
- [x] Record initial baselines (one per machine profile) using `fretboard diag perf --perf-baseline-out`.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v1.json` (commit `50bfcc54`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v2.json` (see perf log entry).
    - v1 was slightly flaky on `ui-gallery-window-resize-stress-steady` `max_top_solve_us` when checked with repeat=3.
      v2 bumps headroom to 30% to reduce false positives.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v4.json` (see perf log entry).
    - Includes the new `ui-gallery-hover-layout-torture-steady.json` script in the `ui-gallery-steady` suite.
    - v3 exists but is superseded by v4 (hover script cleanup to reduce cross-script state contamination).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v5.json` (see perf log entry).
    - Switches perf protocol to `FRET_DIAG_SCRIPT_AUTO_DUMP=0` to avoid per-step bundle dumps dominating I/O.
    - Supersedes v4 for perf gating; keep v4 only if you explicitly want “auto dump on” behavior for debugging.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v6.json` (see perf log entry).
    - Includes pointer-move maxima in the baseline rows (newer perf protocol) and reflects the current steady-state
      costs of the menubar script after recent diagnostics/runtime changes.
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v7.json` (see perf log entry).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v8.json` (post-merge snapshot;
    evidence + drift notes in the perf log entry for commit `72e6c32df`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v9.json` (refresh after the
    post-merge editor regression fix; evidence + drift notes in the perf log entry for commit `0d8ad27ac`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v10.json` (refresh under the
    steady-state protocol: `--reuse-launch` + diagnostics envs pinned; evidence + drift notes in the perf log entry
    for commit `09ecac494`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v11.json` (adds the editor-grade
    autoscroll probe to the suite; evidence + drift notes in the perf log entry for commit `f21a0aa82`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json` (pointer-move threshold slack/quantum stabilization; see perf log entry around 2026-02-06 12:36).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v13.json` (refresh after resize-event coalescing work; see perf log entry for commit `beb2fa315`).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json` (schema refresh including run-max hit-test-replay gate fields; evidence + drift notes in perf log entry 2026-02-06 20:12).
  - macOS (Apple M4): `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json` (adds anti-noise threshold seeding metadata, policy header, and resize-script p95 seeding with interpolated percentile; evidence + drift notes in perf log entries 2026-02-06 21:05 and 2026-02-06 21:35).
- [x] Add a “how to run locally” snippet to the workstream doc (keep it copy/paste friendly).
- [ ] Create a “known-noise sources” section (thermal, background apps, debug vs release, shader compile).
- [x] Pick one canonical view-cache setting for the suite and enforce it via `--env` in scripts.
  - Candidate: `FRET_UI_GALLERY_VIEW_CACHE=1` + `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
- [x] Create a commit-addressable perf log:
  - `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- [x] Add a helper to append suite results to the log:
  - `tools/perf/perf_log.py`
- [x] Extend `tools/perf/perf_log.py` to include churn signals (top frame p95/max) alongside CPU breakdown.
  - Signals: text atlas uploads/evictions, intermediate pool peak bytes, intermediate pool evictions.
  - Implemented by `feat(perf): include churn signals in perf_log` (commit `76d2dfd6`).
- [x] Record an initial suite run in the log (repeat=7).
- [x] Add a steady-state suite and reuse-launched-process support:
  - `fretboard diag perf ui-gallery-steady --reuse-launch --launch -- cargo run -p fret-ui-gallery --release`
- [x] Record a `ui-gallery-steady` baseline in the perf log (repeat=7, `--reuse-launch`).
  - See `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `686bebe1`.
- [x] Stabilize view-cache key to avoid resize-driven `cache_key_mismatch`.
  - Implemented by `perf(fret-ui): stabilize view-cache key` (commit `b6f1b580`).
- [x] Add a resize-smoothness knob for scroll extents: defer unbounded probes while the viewport is resizing.
  - Implemented by `perf(fret-ui): defer unbounded scroll probe on resize` (commit `05d2d56c`).
  - Env: `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`
  - Debounce: `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES` (default: 2)
- [x] Add correctness gates for the resize + scroll probe policy:
  - Scroll offset stability gate: `--check-scroll-offset-stable <test_id>` (commit `6c248d9e1`).
  - Scrollbar thumb geometry validity gate: `--check-scrollbar-thumb-valid all` (commit `e20637f92`).
- [ ] Decide whether scroll unbounded-probe deferral should become the default (remove env gating) and
  update the canonical perf suite env set accordingly.
- [x] Export view-cache reuse “miss reasons” as perf-visible counters (so regressions are explainable).
  - Implemented by `feat(diag): export view-cache reuse miss counters` (commit `43f9c73e`).
- [x] Export a coarse layout-phase breakdown (so `layout_time_us` is explainable in bundles and stable-frame fast paths).
  - Add: `layout_collect_roots_time_us`, `layout_invalidate_scroll_handle_bindings_time_us`,
    `layout_expand_view_cache_invalidations_time_us`, `layout_request_build_roots_time_us`,
    `layout_pending_barrier_relayouts_time_us`, `layout_repair_view_cache_bounds_time_us`,
    `layout_contained_view_cache_roots_time_us`, `layout_collapse_layout_observations_time_us`,
    `layout_prepaint_after_layout_time_us`, `layout_skipped_engine_frame`.
  - Wire into: `fretboard diag stats --json` so a worst bundle can be inspected without manual JSON digging.
  - Implemented by `feat(diag): export layout phase breakdown` (commit `b02744a8`).
- [x] Export initial paint-pass breakdown metrics (to disprove/confirm “paint-cache replay is the hotspot”).
  - Adds: `paint_cache_replay_time_us`, `paint_cache_bounds_translate_time_us`,
    `paint_cache_bounds_translated_nodes`, `paint_record_visual_bounds_time_us`,
    `paint_record_visual_bounds_calls`.
  - Implemented by `feat(diag): add paint pass breakdown metrics` (commit `f2bee87a`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`
- [x] Export initial paint micro-breakdown timers (paint-all plumbing).
  - Adds: `paint_input_context_time_us`, `paint_scroll_handle_invalidation_time_us`,
    `paint_collect_roots_time_us`, `paint_publish_text_input_snapshot_time_us`,
    `paint_collapse_observations_time_us`.
  - Implemented by `feat(diag): add paint micro-breakdown timers` (commit `b20a1280`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`
- [x] Export paint node breakdown timers (paint-cache key/hit checks, widget paint, observation recording).
  - Adds: `paint_cache_key_time_us`, `paint_cache_hit_check_time_us`, `paint_widget_time_us`,
    `paint_observation_record_time_us`.
  - Implemented by `feat(diag): add paint node breakdown timers` (commit `c512be81`).
  - Tracking: `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`
- [ ] Keep `diag perf` runs comparable by splitting “gate checks” vs “deep profiling”:
  - Gate check (CPU regressions): keep `FRET_DIAG_RENDERER_PERF` off (avoid instrumentation overhead).
  - Deep profiling (churn / GPU triage): turn `FRET_DIAG_RENDERER_PERF=1` on and record churn tables in the log.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries on 2026-02-04 show the delta.

### M1: Frame data structures (hashing → dense)

Primary targets (highest leverage):

- [x] Refactor `WindowFrame` stores to avoid per-frame `HashMap` churn:
  - `crates/fret-ui/src/declarative/frame.rs` (`WindowFrame.instances`, `WindowFrame.children`)
  - Landed as `slotmap::SecondaryMap<NodeId, ...>` (commit `448c34ad`).
- [x] Avoid rewriting `WindowFrame.children` when the child list is unchanged (reduce per-frame `Arc<[NodeId]>` allocations).
  - Implemented by `perf(fret-ui): skip unchanged window frame children` (commit `cce827ad`).
- [x] Avoid cloning child lists when calling `UiTree::set_children*` from declarative mount (reduce per-frame heap churn).
  - Implemented by `perf(fret-ui): avoid cloning child lists in mount` (commit `089bac9b`).
- [ ] Replace `Arc<[NodeId]>` for `WindowFrame.children` with a reuse-friendly representation.
  - Candidate: store `Vec<NodeId>` in a slab/arena and reference by index + generation.
- [x] Replace invalidation “visited”/scratch `HashMap<NodeId, u8>` with generation-stamped tables:
  - `crates/fret-ui/src/tree/mod.rs` invalidation propagation caches.
  - Implemented by `perf(fret-ui): generation-stamp invalidation propagation` (commit `a540829e`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries for commit `a540829e`.
- [x] Avoid per-dispatch `HashMap<NodeId, u8>` churn when deduplicating invalidations during input dispatch.
  - Use the existing generation-stamped `InvalidationDedupTable` for dispatch-time invalidation dedup.
  - Implemented by `perf(fret-ui): reuse invalidation dedup in dispatch` (commit `bcb329e6`).
- [ ] Ensure deterministic ordering is preserved where diagnostics rely on it (bundle stability).

Perf acceptance:

- [ ] `ui-gallery-overlay-torture.json`: p95 total improves; invalidation nodes/calls do not regress.
- [ ] `ui-gallery-virtual-list-torture.json`: tail latency improves or stays flat.
- [x] Investigate post-`a540829e` suite deltas (noise vs real regression) and decide next step:
  - If real: profile invalidation propagation micro-costs and consider alternative dense map strategy (or env gating).
  - If noise: standardize suite runs on explicit `--dir` and pin a baseline via `--perf-baseline-out`.
  - Result: A/B rerun at `448c34ad` is within noise vs the current baseline (see perf log).

### M2: Allocation model (per-frame scratch arena)

- [ ] Introduce a `FrameArena` (or equivalent) for UI runtime scratch allocations.
  - Reference: `repo-ref/zed/crates/gpui/src/arena.rs`.
- [x] Reuse a small set of per-frame scratch buffers to reduce allocator churn.
  - `perf(fret-ui): reuse frame scratch buffers` (commit `a39e79c4`).
- [x] Reuse view-cache GC “keep-alive” scratch collections (HashSet/Vec) to reduce per-frame allocations.
  - `perf(fret-ui): reuse view-cache keepalive scratch` (commit `cb3ff2d9`).
  - A/B gate: `perf(fret-ui): gate view-cache keepalive scratch` (commit `968305b9`)
    - `FRET_UI_VIEW_CACHE_KEEPALIVE_SCRATCH_DISABLE=1` disables scratch reuse.
  - Status: A/B is within noise on:
    - code editor autoscroll (`tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`)
    - view-cache toggle perf steady (`tools/diag-scripts/ui-gallery-view-cache-toggle-perf-steady.json`)
    - overlay torture steady (`tools/diag-scripts/ui-gallery-overlay-torture-steady.json`)
    (see perf log entries for `968305b9`).
- [x] Convert at least 2 hot scratch paths to arena-backed allocation (scratch reuse, v0):
  - Semantics snapshot traversal scratch (stack + visited).
  - GC reachability scratch sets / traversal stack in mount/GC.
  - Implemented by `perf(fret-ui): reuse GC/semantics scratch via frame arena` (commit `3d6e2431`).
  - Evidence: perf log entry for `1b0364e9` (exports `top_frame_arena_*` counters).
- [x] Export “frame arena scratch” counters into perf-visible diagnostics:
  - Implemented by `feat(diag): export frame arena scratch stats` (commit `fe0ad7c3`).
  - Fix: `fix(fret-ui): restore keepalive scratch after diagnostics` (commit `1b0364e9`).
- [x] Remove per-scope `HashMap` churn during element ID derivation (callsite counters).
  - Implemented by `perf(fret-ui): remove callsite counter HashMap churn` (commit `2dd36fde`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for `2dd36fde`.
- [x] Pool declarative element child buffers (`Vec<AnyElement>`) across frames (arena-adjacent, v0).
  - Implemented by `perf(fret-ui): pool element children vectors` (commit `07a4c252`).
  - Perf-visible counters exported by `feat(diag): export element build pool counters` (commit `cbcd81ed`).
  - Follow-up: `perf(fret-ui): make element children vec pool LIFO` (commit `693a55b0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for `693a55b0`.
- [x] Validate element children vec pool steady-state behavior on editor-class pages.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries for:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` (0 misses; paint-dominant).
    - `tools/diag-scripts/ui-gallery-chrome-torture-steady.json` (0 misses; very light total).
- [ ] Add an optional allocation counter hook for local profiling (feature-gated).
  - Keep it portable; do not require a global allocator swap for normal builds.

Correctness acceptance:

- [x] Existing `cargo nextest run -p fret-ui` remains green.
  - Evidence: passed locally after `perf(fret-ui): skip layout-engine rebuild on stable frames` (commit `1905de1e`).
- [ ] `fretboard diag repro ui-gallery` smoke suite passes.

### M3: Hit testing (bounds tree / spatial index)

- [x] Implement a bounds tree built during prepaint per hit-testable layer root.
  - Implemented by `perf(fret-ui): add bounds tree hit-test index` (commit `75a9fde3`).
  - Note: current implementation supports axis-aligned transforms only (no rotation/shear).
- [x] Route pointer move/down hit-testing through the bounds tree for large trees.
  - Implemented by `75a9fde3` (hooked via `UiTree::hit_test_layers_cached`).
- [x] Define “fallback” conditions clearly (transforms, clips, non-axis-aligned bounds).
  - Supports `clips_hit_test=false` (overflow-visible hit testing) by propagating the ancestor clip (instead of
    disabling the index for the entire layer).
  - Disabled for a layer if any transform is non-axis-aligned (`b!=0` or `c!=0`).
  - Env toggles:
    - `FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1` disables the index.
    - `FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS` (default: 256) gates building for small trees.
- [x] Add a pointer-move stress gate that fails on dispatch/hit-test regressions.
  - Use:
    - `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
    - `--max-pointer-move-dispatch-us`, `--max-pointer-move-hit-test-us`,
      `--max-pointer-move-global-changes` (fretboard `diag perf`)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `6da92d3d`.
  - TODO: Investigate occasional flakiness when running this gate with `--reuse-launch --repeat 7`
    (observed: a run gets stuck early in the script, e.g. `set_window_inner_size`).
    Short-term workaround: use `--repeat 3` for local iteration and keep a stable Tier B gate at repeat=7 once the
    harness is robust.
    - Evidence: a repeat=7 run completed when launching a prebuilt binary
      (`--launch -- target/release/fret-ui-gallery`); see the perf log entry for commit `b83ae7a5`.
- [x] Make pointer-move gate outliers explainable (include snapshot id for pointer-move maxima).
  - Implemented by `feat(diag): include pointer-move max frame ids in triage` (commit `c2ea017b`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `c2ea017b`.
- [x] Eliminate changed-but-unobserved model churn on pointer-move frames.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `dd1a22e8` shows pointer-move
    frames with `changed_models=2` and `propagated_model_change_unobserved_models=2` while remaining paint-only.
  - Fixed in `perf(ui-gallery): avoid per-frame undo/redo model churn` (commit `eb6c6b2e`).
  - Goal: pointer-move frames should have `changed_models=0` unless the interaction explicitly updates observed state.
  - Candidate fix: move per-frame pointer-move bookkeeping out of `Model` updates into a window-scoped scratch store
    (or a “set-if-changed” model update discipline similar to the global churn fix).
- [x] Add a dispatch/hit-test time metric to diagnostics so we can gate pointer-move cost explicitly.
  - Implemented by `perf(diag): expose dispatch and hit-test timing` (commit `4b0be50e`).
  - Adds new `fretboard diag perf --sort dispatch|hit_test` modes and exports:
    - `top_dispatch_time_us`, `top_hit_test_time_us`
    - `top_dispatch_events`, `top_hit_test_queries`
- [x] Add a dedicated hit-test drag stress script (high pointer event density).
  - Script: `tools/diag-scripts/ui-gallery-hit-test-drag-sweep-steady.json`
  - Use with: `fretboard diag perf ... --sort hit_test`
- [x] Add a multi-frame pointer-move sweep step for realistic hover/hit-test measurements.
  - Implemented by `perf(diag): add move_pointer_sweep script step` (commit `4941baa1`).
  - Scripts:
    - `tools/diag-scripts/ui-gallery-hit-test-move-sweep-steady.json`
    - `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`
- [x] Find (or construct) a workload where `top_hit_test_time_us` is a meaningful slice of the frame budget.
  - Page: `apps/fret-ui-gallery/src/ui.rs` (`hit_test_torture`)
  - Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
  - Harness-only mode (to remove gallery chrome noise): `FRET_UI_GALLERY_HARNESS_ONLY=hit_test_torture`
  - Evidence + metrics: see `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries after commit `811101c3`.
- [x] Record baseline numbers for the two “realistic move sweep” probes:
  - Data table sweep: `tools/diag-scripts/ui-gallery-hit-test-data-table-move-sweep-steady.json`
  - Stripes torture (via nav): `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries on 2026-02-04 (commit `9b2f9fc9`).
- [x] Add a smaller torture script variant to make scaling runs practical (avoid 10GB+ bundles).
  - Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-mini.json`
  - Implemented by `feat(diag-scripts): add mini hit-test torture sweep` (commit `1b3d2db3`).
  - Use: `FRET_DIAG_SCRIPT_AUTO_DUMP=0` + `FRET_DIAG_SEMANTICS=0` + `FRET_DIAG_MAX_SNAPSHOTS=120`.
- [x] Export cached-path hit-test reuse counters (to measure whether the fast path helps).
  - Counters:
    - `debug.stats.hit_test_path_cache_hits`
    - `debug.stats.hit_test_path_cache_misses`
  - Implemented by `feat(diag): track hit-test path-cache reuse` (commit `55dd923d`).
- [ ] Investigate why the torture workload is still layout/prepaint-dominant on the sampled frames.
  - Goal: create (or tune) a variant where pointer moves are paint-only and hit-test cost is isolated.
  - Hypotheses:
  - TODO: Use the new bounds-tree “work” counters to determine whether `hit_test_time_us` tails are algorithmic or
    wall-time noise:
    - `debug.stats.hit_test_bounds_tree_nodes_visited`
    - `debug.stats.hit_test_bounds_tree_nodes_pushed`
    - Implemented by `feat(fret-ui): track bounds-tree query work in debug stats` (commit `913ee260`).
    - hover policy triggers layout
    - retained tree has a per-frame relayout
    - noise elements invalidate layout
    - diagnostics/script harness accidentally forces expensive work every frame (e.g. semantics refresh)
  - Progress:
    - `1905de1e` reduces this probe's `layout_time_us` max from ~74ms → ~31ms by skipping layout-engine rebuild on stable frames.
    - `prepaint_time_us` remains ~9–10ms and `hit_test_time_us` stays measurable; next isolate remaining ~20ms inside `layout_all_with_pass_kind`.
    - `470708b2` reduces the same probe's top frame max total from ~56ms → ~39ms by gating semantics snapshot refresh
      to only the frames that actually need selector resolution (3/201 frames in the inspected bundle).
    - `ba3fd15d` fixes a diagnostics accounting bug (layout time no longer double-counts prepaint).
    - `6cca2cf1` removes prepaint rebuild work on layout-stable frames by reusing hit-test bounds trees:
      - `top_prepaint_time_us` drops to ~0 for the probe's worst frames.
      - Pointer-move frames become paint-only with `layout_time_us ~ 0` and `prepaint_time_us ~ 0` (see perf log entry).
  - Deliverable: a new/updated script + a log entry demonstrating low `layout_time_us` while `hit_test_time_us` remains measurable.
  - [x] Add hit-test micro timers so tail latency is attributable to concrete work.
    - Exports (per-frame, accumulated across hit-test queries):
      - `hit_test_cached_path_time_us`
      - `hit_test_bounds_tree_query_time_us`
      - `hit_test_candidate_self_only_time_us`
      - `hit_test_fallback_traversal_time_us`
    - Implemented by `feat(diag): break down hit-test timing` (commit `763bf8e7`).
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries for commits `763bf8e7` and `8bc15eda`.
  - [x] Remove cached-path overhead when bounds-tree is enabled.
    - Implemented by `perf(fret-ui): skip cached-path hit-test under bounds-tree` (commit `8bc15eda`).
    - Result: pointer-move `hit_test_time_us` p50 ~575us → ~3us on the stripes torture probe.
  - [x] Export a coarse dispatch sub-step timing breakdown for pointer-move triage.
    - Exports (per-frame, accumulated across the frame’s dispatch work):
      - `dispatch_hover_update_time_us`
      - `dispatch_scroll_handle_invalidation_time_us`
      - `dispatch_active_layers_time_us`
      - `dispatch_input_context_time_us`
      - `dispatch_event_chain_build_time_us`
      - `dispatch_widget_capture_time_us`
      - `dispatch_widget_bubble_time_us`
      - `dispatch_cursor_query_time_us`
      - `dispatch_pointer_move_layer_observers_time_us`
    - Wired into: `fretboard diag stats --json` (so a worst bundle can be inspected without manual JSON digging).
    - Implemented by `feat(diag): break down dispatch timing` (commit `7fa76fd5`).
    - Evidence: perf log entry for commit `7fa76fd5`.
  - [x] Attribute dispatch time by dispatched event class (pointer vs timer vs other).
    - Exports (per-frame, accumulated across the frame’s dispatch work):
      - `dispatch_pointer_events`, `dispatch_pointer_event_time_us`
      - `dispatch_timer_events`, `dispatch_timer_event_time_us`
      - `dispatch_other_events`, `dispatch_other_event_time_us`
    - Wired into: `fretboard diag stats --json` (bundle triage without manual JSON digging).
    - Implemented by `feat(diag): attribute dispatch time by event class` (commit `5ab4ba71`).
    - Evidence: perf log entry for commit `5ab4ba71`.
  - [x] Reduce timer-driven dispatch work during pointer-move workloads.
    - Why: In the stripes pointer-move probe, the “dispatch gap” was primarily **timer event dispatch** (not pointer
      routing). On the worst pointer-move frame, `dispatch_timer_event_time_us` accounted for ~95%+ of `dispatch_time_us`.
    - Root cause: ui-gallery’s dev-only config polling (`with_config_files_watcher(...)`) installs a repeating global
      timer, and the timer could co-occur with scripted pointer-move frames.
    - Deliverable:
      - Timer routing attribution exported (commit `98ca4fe3`).
      - Harness runs avoid config watcher timer traffic (commit `06feeb41`).
      - Evidence: perf log entries for commits `98ca4fe3` and `06feeb41` (p95 dispatch drops to ~tens of microseconds).
    - Remaining follow-ups (generalizing beyond the ui-gallery harness):
      - [ ] Make “background timers” avoid the UI dispatch hot path by default (or run them out-of-band).
      - [ ] Add a configurable “timer budget / priority” contract so non-UX-critical timers cannot steal time from
        interactive input frames.
  - A/B experiments:
    - [x] Run the pointer-move gate with `FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE=1` and record:
      - `hit_test_time_us` distribution, and
      - `hit_test_path_cache_hits/misses` hit rate.
      - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `8bc15eda` (gate fails expectedly).
    - [ ] Sweep `FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS` to find the break-even point (small trees vs index build).

Perf acceptance:

- [ ] Pointer-move heavy cases should stay paint-only (no layout) unless explicitly required.
- [ ] Hit-test CPU time should be bounded as node count scales.
- [x] Ensure the perf log captures pointer-move dispatch/hit-test costs (not just “top frame” totals).
  - Today, `perf_log.py` reports “top frame” metrics for each run, which can show `dispatch=0` for probes
    where the worst total frame is a non-dispatch settle/selector frame.
  - `tools/perf/perf_log.py` now emits a derived “Pointer-move frames” section by scanning the run bundles and
    summarizing per-run maxima over frames where `dispatch_events > 0`.
- [x] Eliminate changed-but-unobserved global churn in hover-only pointer-move probes.
  - Current hotspots reported by `fretboard diag stats`: `WindowInputContextService`,
    `WindowCommandActionAvailabilityService` (often changed but unobserved).
  - Goal: reduce pointer-move dispatch tails by making these globals “notify only on actual value change”
    (or avoid publishing them every frame unless explicitly needed).
  - Implemented by `perf(fret-ui): avoid global churn on hover moves` (commit `d4adf37f`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for `d4adf37f`
    (`dispatch_time_us` run-max p95 drops from ~4.1ms → ~1.2ms; `snapshots_with_global_changes` becomes 0).

### M7: Renderer primitive profiling (bottom-up)

- [x] Add renderer perf logging to UI gallery (primitive-level signals).
  - Enable: `FRET_UI_GALLERY_RENDERER_PERF=1`
  - Optional pipeline breakdown: `FRET_RENDERER_PERF_PIPELINES=1`
  - Goal: provide low-level “are we draw-call/pipeline-switch bound?” signals before deeper refactors.
- [x] Add a short “profiling playbook” that links `diag perf` → renderer perf → Tracy → RenderDoc.
  - `docs/workstreams/ui-perf-renderer-profiling-v1.md` (commit `22671e06`)
- [x] Export renderer perf snapshots into diagnostics bundles for perf log correlation.
  - Data lands in `bundle.json` under `.windows[].snapshots[].debug.stats.renderer_*` (commit `0e4928fe`).
  - `fretboard diag stats/perf` supports sorting by renderer metrics (commit `cf8975ca`).
- [x] Export renderer churn metrics (text atlas + intermediate pool) into bundles and wire them into `fretboard`.
  - Commits: `feat(render): add text atlas + intermediate churn perf stats` (`d10cac5a`) +
    `feat(fretboard): add renderer churn sort modes` (`c9a8b168`).
  - Text atlas (per-frame signals): `renderer_text_atlas_revision`, `renderer_text_atlas_upload_bytes`,
    `renderer_text_atlas_evicted_pages`, `renderer_text_atlas_resets` (and related counters).
  - Intermediate pool (per-frame signals): `renderer_intermediate_peak_in_use_bytes`,
    `renderer_intermediate_pool_evictions` (and related counters).
  - New sort modes:
    - `atlas_upload_bytes`, `atlas_evicted_pages`, `intermediate_peak_bytes`, `pool_evictions`
- [ ] Add a GPU-time signal (where supported) to separate “CPU is fine” vs “GPU stalls”.
  - Candidate: timestamp queries in the renderer + export `gpu_render_us` (best-effort).
  - If unsupported on a backend, export `None` and keep the field stable in the bundle schema.
- [ ] Establish per-script renderer complexity budgets (to prevent silent GPU regressions).
  - Track at minimum: `renderer_draw_calls`, `renderer_pipeline_switches`, `renderer_bind_group_switches`,
    `renderer_scissor_sets`, and `renderer_text_atlas_upload_bytes`.
  - Add at least one acceptance script that is renderer-heavy (effects/blur, large text surface, SVG churn).
- [ ] Make RenderDoc captures repeatable for the acceptance scripts.
  - Pin marker names and a canonical `--renderdoc-after-frames` per script so “capture the hitch” is low-friction.

### M7.1: Renderer churn correlation (tail latency)

Goal:
- Turn “jank” into a correlation between **slow frames** and a **churn signature** (GPU-side or resource-side),
  and then close that churn.

TODO:

- [x] Add a deterministic workload/script that actually exercises blur/effects so intermediate pool counters become non-zero.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (entry for `effects_blur_torture`).
- [x] Add an eviction stress variant to force intermediate pool churn for correlation work.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Budget override: `FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520` (20MB)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (pool evictions > 0).
- [x] Add additional churn accounting beyond text atlas (non-text uploads):
  - Bundles now export (best-effort) non-text texture upload counters:
    `renderer_svg_upload_bytes`, `renderer_svg_uploads`,
    `renderer_image_upload_bytes`, `renderer_image_uploads`.
  - Commits: `d01d3190` + `4bade395` + `dfbc02d3` (workload). Evidence:
    `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `dfbc02d3`.
  - Harness/script:
    - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=svg_upload_torture`
    - Script: `tools/diag-scripts/ui-gallery-svg-upload-thrash-steady.json`
    - Budget override: `FRET_UI_GALLERY_SVG_RASTER_BUDGET_BYTES=262144` (256KB)
- [x] Add an eviction stress protocol for intermediate pool churn correlation.
  - Env: `FRET_UI_GALLERY_RENDERER_INTERMEDIATE_BUDGET_BYTES=20971520` (20MB) to force pool evictions.
  - Script: `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (entry for `effects_blur_thrash`).
- [ ] Extend churn accounting beyond uploads:
  - [x] SVG raster cache occupancy + eviction counts (to distinguish warmup vs thrash).
    - Commits: `6bd82329` + `5f7e4fd0` + `3d1510a7`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `3d1510a7`
      (see `svg_cache_misses` / `svg_evictions` columns).
  - [x] Intermediate pool lifecycle churn signals (alloc/reuse/release/free bytes/texture counts + budget/in_use/peak).
    - Commit: `52f555d5`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `52f555d5`.
  - [ ] Path/MSAA per-pass churn (uploads/resolves/temporary targets) beyond the pooled intermediate counters.
  - [x] Reduce intermediate pool housekeeping overhead by enforcing budget once per frame (instead of per release).
    - Commit: `3b792646`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `3b792646`.
- [x] Replace keyed repaint forcing with a representative invalidation-driven workload.
  - The legacy `svg_upload_torture` harness keys the Canvas subtree by frame to bypass paint-cache replay.
  - Added an invalidation-driven scroll workload that uses wheel input to shift the VirtualList window:
    - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=svg_scroll_torture` (commit `dd8bc0f8`)
    - Script: `tools/diag-scripts/ui-gallery-svg-scroll-thrash-steady.json`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `dd8bc0f8`.
- [x] Standardize “churn triage checklist” in the perf log template:
  - `tools/perf/perf_log.py` now emits churn + intermediate pool lifecycle tables and includes captured stdout paths.
  - Commit: `2c40a3fb`
- [x] Keep ADRs and audits in sync with the diagnostics bundle schema.
  - Update ADR 0174 bundle/export notes when schema changes (renderer counters, script steps, screenshot wiring).
  - Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` evidence and gaps when tooling contracts change.

### M4: Windowed surfaces (prepaint-driven visible windows)

- [x] Pick the first “editor-class” migration target: **Option A (VirtualList)**.
  - Rationale: fastest path to validate retained prepaint-window behavior and rerender suppression under wheel traffic.
  - Evidence: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json`,
    `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries 2026-02-07 00:46 and 2026-02-07 00:56.
- [ ] Reduce editor-class per-frame scene construction when scrolling/animating.
  - Baseline hotspot: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` can be dominated by
    `paint_widget_hotspots kind=Canvas` (see perf log entry 2026-02-05 15:43:55).
  - Goal: translate/replay cached ranges where possible instead of re-emitting large display lists each frame.
- [ ] Ensure cache-root reuse remains stable under steady scroll/pan.
- [x] Suppress avoidable non-retained prefetch rerenders on steady wheel crossing.
  - Change: `crates/fret-ui/src/tree/prepaint.rs` now disables preemptive/forced prefetch shifts for
    non-retained + view-cache path while visible range remains covered by the rendered overscan envelope.
  - Non-retained sample (`FRET_UI_GALLERY_VLIST_RETAINED=0`, 3 runs):
    - before: `prefetch=1`, `non_retained=1` per run
    - after: `prefetch=0`, `non_retained=0` per run
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 01:04.
- [x] Add a “window boundary crossing” probe script for retained VirtualList scrolling.
  - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json`
  - Sampling status: with `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`,
    `FRET_UI_GALLERY_VLIST_MINIMAL=1`, runs `r3..r6` show `total_shifts=1`, `prefetch=1`, `escape=0`, `non_retained=0`.
- [x] Promote the boundary-crossing probe into a stable acceptance gate recipe (repeat runs + threshold rationale).
  - Gate runner: `tools/perf/diag_vlist_boundary_gate.sh`
  - Validation summary: `target/fret-diag-codex-vlist-boundary-gate-r1/summary.json` (`runs=3`, `run_failures=0`, `pass=true`).

Perf acceptance:

- [ ] `ui-gallery-virtual-list-torture.json`: steady scroll should avoid cache-root rerender in most frames.
- [x] `ui-gallery-virtual-list-window-boundary-crossing-steady.json`:
  - Retained gate target: `prefetch<=3`, `escape<=0`, `non_retained<=0`
  - Command profile: enable view-cache env (`FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`) and run `tools/perf/diag_vlist_boundary_gate.sh --runs 3`.
- [x] `ui-gallery-virtual-list-window-boundary-crossing-steady.json` (non-retained fallback profile):
  - Run profile: add `FRET_UI_GALLERY_VLIST_RETAINED=0`
  - Current sampled expectation (3 runs): `prefetch=0`, `escape=0`, `non_retained=0`
- [x] Add strict non-retained fallback gate and cache-key budgets.
  - Gate runner: `tools/perf/diag_vlist_boundary_gate.sh` now supports
    `--retained`, `--max-cache-key-mismatch`, `--max-needs-rerender`.
  - Validation summary: `target/fret-diag-codex-vlist-boundary-nonretained-gate-r1/summary.json`
    (`runs=3`, `pass=true`, `prefetch=0`, `escape=0`, `non_retained=0`,
    `cache_key_mismatch_max=0`, `needs_rerender_max=0`).
- [x] Add non-retained boundary stress probe and strict gate recipe.
  - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json`
  - Gate command:
    `tools/perf/diag_vlist_boundary_gate.sh --runs 3 --script tools/diag-scripts/ui-gallery-virtual-list-window-boundary-nonretained-stress-steady.json --retained 0 --prefetch-max 0 --escape-max 0 --non-retained-max 0 --max-cache-key-mismatch 0 --max-needs-rerender 0`
  - Validation summary: `target/fret-diag-codex-vlist-boundary-nonretained-stress-gate-r1/summary.json` (`pass=true`, `run_failures=0`).
- [ ] `ui-gallery-code-view-scroll-refresh-baseline.json`: no hitch spikes after warmup.
- [x] `ui-gallery-code-editor-torture-autoscroll-steady.json`: eliminate the post-merge Canvas paint hotspot.
  - Root cause: accidental per-row `Theme` clone in syntax paint (allocator churn).
  - Fix: `perf(code-editor): avoid per-row Theme clone in syntax paint` (commit `0d8ad27ac`).
  - Evidence + numbers: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for 2026-02-06 (commit `0d8ad27ac`).
  - Follow-up: still track tail outliers (max spikes) and ensure the probe stays within Tier B on high-end HW.

### M5: Text pipeline stabilization (editor-ready)

- [ ] Document stable cache keys for measure/shaping (wrap width, font stack, style).
- [ ] Reduce redundant text measurements under intrinsic probes (layout engine + `TextWrap::None` paths).
- [x] Add a fast path for “min-content probes” (e.g. `wrap=Word` + `max_width=0`) to avoid O(n²) text wrapping.
  - Implemented by `perf(fret-render): fast-path wrapped text measure` (see perf log entry for commit `9440648a`).
- [x] Reduce repeated shaping work when taffy calls `measure()` under multiple intrinsic modes (min/max/definite).
  - Implemented by caching single-line shaping + cluster-based wrap stats (see `ui-perf-zed-smoothness-v1-log.md`).
- [x] Cut code editor syntax paint cost in the “autoscroll torture” probe (p95 paint drops from ~23ms → ~5ms).
  - Implemented by `perf(fret-code-editor): cache syntax rich rows` (commit `81159325`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries for commit `bd709f88` (baseline) and `81159325`.
- [x] Eliminate allocation churn in editor syntax paint by avoiding per-row `Theme` clones.
  - Implemented by `perf(code-editor): avoid per-row Theme clone in syntax paint` (commit `0d8ad27ac`).
  - Evidence + numbers: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for 2026-02-06 (commit `0d8ad27ac`).
- [x] Add diagnostics hooks to identify text cache misses that correlate with perf hitches.
  - `paint_widget_hotspots` now include `ElementInstance` kind attribution (commit `c80525b9`).
  - Paint-phase text prepare counters + reason counts:
    - `paint_text_prepare_time_us`, `paint_text_prepare_calls` (commit `07d2ccf2`)
    - `paint_text_prepare_reason_*` (commit `80a46d49`)
  - Per-frame top-N text prepare hotspots with node/element ids + constraints + reason mask:
    - `paint_text_prepare_hotspots` (commit `77979100`)
- [x] Add a steady-state menubar hover probe to confirm “text prepares happen only on first appearance”.
  - Script: `tools/diag-scripts/ui-gallery-menubar-open-hover-sweep-steady.json` (commit `0a8191eb`)
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for `ui-gallery-menubar-open-hover-sweep-steady`.
- [ ] Ensure atlas eviction and re-upload events are observable in perf snapshots.

Perf acceptance:

- [ ] Editor-class pages remain within Tier A budgets; Tier B progress is tracked.

### M6: Perf gates in CI (optional, but recommended)

- [ ] Define a reduced suite for CI (fast, stable, platform-agnostic as much as possible).
- [ ] Decide baseline storage approach (per platform, per hardware class).
- [ ] Add a “perf regression triage” template: which bundle artifacts to attach, how to compare.

## Cross-cutting hygiene

- [ ] When a refactor changes a hard-to-change behavior, capture it as an ADR and update
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md` if relevant.
- [ ] Prefer tooling-driven evidence: `bundle.json`, `check.*.json`, and reproducible scripts.
- [ ] Keep `fret-ui` policy-light (mechanisms only; policy stays in ecosystem; see ADR 0066).
- [ ] Track GPUI performance gaps explicitly and close them with measurable gates:
  - `docs/workstreams/ui-perf-gpui-gap-v1.md`
- [x] Stabilize `ui-gallery-steady` perf baseline gates against microsecond jitter.
  - Adjustment: add slack + quantum rounding for pointer-move thresholds in perf baseline generation.
  - Refresh baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v12.json`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 12:36.
- [x] Refresh steady baseline after perf-threshold schema update (run-max hit-test replay metrics).
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v14.json`
  - Validation: `target/fret-diag-codex-perf-v14-validate2/check.perf_thresholds.json` (failures=0).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 20:12.
- [x] Add anti-noise threshold seeding metadata for steady baselines.
  - Baseline row now records `measured_p90`, `measured_p95`, `threshold_seed`, `threshold_seed_source`.
  - Baseline header records `threshold_seed_policy` (default seed + per-script/metric rules).
  - Script-specific policy: resize steady uses p95 seed for `top_total/layout/solve`; other metrics stay max-seeded.
  - Percentile seeds use linear interpolation so repeat=7 no longer degenerates to max-only seeding.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v15.json`
  - Validation: `target/fret-diag-codex-perf-v15-validate-seed/check.perf_thresholds.json` (failures=0).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entries 2026-02-06 21:05 and 2026-02-06 21:35.
- [x] Make baseline seed policy configurable from CLI.
  - New flag: `--perf-baseline-seed <scope@metric=max|p90|p95>` (repeatable; scope supports suite names and `this-suite`).
  - Example: `--perf-baseline-seed ui-gallery-steady@top_total_time_us=p90`.
  - Template doc: `docs/workstreams/perf-baselines/seed-policy-template.md`.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 21:35.
- [x] Add versioned JSON presets for baseline seed policies.
  - New flag: `--perf-baseline-seed-preset <path>` (repeatable; merge order follows CLI argument order).
  - Merge precedence: built-in defaults -> preset rules -> explicit `--perf-baseline-seed` overrides.
  - Added preset example: `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json`.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 22:50.
- [x] Run v16 preset trial and evaluate gate stability.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v16.json`.
  - Validation sample: `target/fret-diag-codex-perf-v16-validate{,2,3}/check.perf_thresholds.json` (all `failures=1`).
  - Control: `target/fret-diag-codex-perf-v15-validate-recheck/check.perf_thresholds.json` (`failures=0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 23:20.
- [x] Publish `ui-gallery-steady.v2` preset to remove known false-fail hotspot.
  - Updated: `docs/workstreams/perf-baselines/policies/ui-gallery-steady.v2.json`.
  - Change: `tools/diag-scripts/ui-gallery-overlay-torture-steady.json` now uses `p95` override.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v17.json`.
  - Validation sample: `target/fret-diag-codex-perf-v17-validate{1,2,3}/check.perf_thresholds.json` (all `failures=0`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 23:55.
- [x] Harden baseline generation workflow against resize outliers (tooling).
  - Added: `tools/perf/diag_perf_baseline_select.sh` (candidate selection + validation sampling + summary JSON).
  - Rule: choose candidate by failures -> resize p90 -> threshold-sum.
  - Template doc updated: `docs/workstreams/perf-baselines/seed-policy-template.md` (`Candidate selection workflow`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 00:35.
- [x] Promote selected v18 baseline as canonical after candidate-selection run.
  - Baseline: `docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v18.json`.
  - Selection summary: `target/fret-diag-codex-perf-v18-select2/selection-summary.json`.
  - Stability: both candidates validated `3/3` with `failures=0`; winner copied to v18 baseline.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-07 00:35.


- [x] Coalesce window resizes to once per frame in the desktop runner.
  - Change: apply `WindowEvent::SurfaceResized` at `RedrawRequested` (keep latest pending size).
  - Commit: `beb2fa315`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 13:20.
- [ ] Decide whether “deferred unbounded scroll probes on resize” should become default behavior.
  - Current mechanism (env-gated):
    - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`
    - `FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_STABLE_FRAMES=2`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 13:45.
  - TODO:
    - [x] Add a correctness probe to ensure resize stress does not clamp scroll offsets incorrectly.
      - Script: `tools/diag-scripts/ui-gallery-window-resize-scroll-offset-stable.json`
      - Gate: `--check-scroll-offset-stable ui-gallery-content-viewport`
      - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 14:26.
    - If acceptable, flip the default for resize-only (keep invalidation deferral opt-in).
- [x] Add an experiment gate for paint-cache replay under `HitTestOnly` invalidation.
  - Env: `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1`
  - Commit: `e50173f13`
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 16:12.
- [x] Add diagnostics counters for the new gate path before deciding default behavior.
  - Export at least: “paint replay allowed by hit-test-only gate” and “hit-test-only replay attempts rejected by key mismatch”.
  - Implemented by `feat(diag): export hit-test-only paint-cache replay counters` (commit `f38f8c1d5`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 17:32.
  - [x] Add a focused script where `HitTestOnly` dominates and layout stays stable.
    - Added probe page + script: `hit_test_only_paint_cache_probe` + `tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json`.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 18:30.
  - [x] Export per-run counter maxima in `diag perf --json` for gate-path counters.
    - Implemented by `feat(diag): export per-run hit-test-only replay maxima in perf json` (commit `4c88f6696`); new fields `run_paint_cache_hit_test_only_replay_allowed_max` and `run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max`.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 19:28.
    - `top_*` rows can stay `0` even when bundle-level max counters are non-zero.
  - [x] Wire run-max counters into perf baseline + threshold gates.
    - Implemented by `feat(diag): gate hit-test replay run-max in perf baseline` (commit `f4a6f422b`).
    - Adds CLI thresholds: `--min-run-paint-cache-hit-test-only-replay-allowed-max`, `--max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max`.
    - Baseline export now includes both `measured_max` and `thresholds` for these counters.
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-06 19:56.
- [ ] Decide whether `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` should ever become default.
  - Current status: keep opt-in only; A/B evidence is mixed across repeated resize probes.
- [ ] Consider gating pointer-move thresholds only when pointer-move frames are present for the script.
- [ ] Keep diagnostics artifacts bounded (especially `target/fret-diag*` and `target/fret-diag-perf`).
  - Default script auto-dump can generate hundreds of GB if left on across long perf sessions.
  - Prefer `FRET_DIAG_SCRIPT_AUTO_DUMP=0` for perf probes and clean old run directories periodically.
