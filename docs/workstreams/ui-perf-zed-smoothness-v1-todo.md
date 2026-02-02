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

## Milestones

### M0: Baseline + suite gates (make perf a contract)

- [ ] Decide Tier A / Tier B thresholds per script (initially “best-effort”, then tighten).
- [x] Decide what `--launch` represents (cold-start gate vs steady-state gate) and codify it.
  - `ui-gallery` + `--launch`: cold-start gate (mount + first interaction).
  - `ui-gallery-steady` + `--reuse-launch` + `--launch`: steady-state gate (post-mount interactions).
- [ ] Finalize the acceptance suite list (see `ui-perf-zed-smoothness-v1.md`) and keep it small.
- [ ] Record initial baselines (one per machine profile) using `fretboard diag perf --perf-baseline-out`.
- [x] Add a “how to run locally” snippet to the workstream doc (keep it copy/paste friendly).
- [ ] Create a “known-noise sources” section (thermal, background apps, debug vs release, shader compile).
- [x] Pick one canonical view-cache setting for the suite and enforce it via `--env` in scripts.
  - Candidate: `FRET_UI_GALLERY_VIEW_CACHE=1` + `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
- [x] Create a commit-addressable perf log:
  - `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- [x] Add a helper to append suite results to the log:
  - `tools/perf/perf_log.py`
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
- [ ] Decide whether scroll unbounded-probe deferral should become the default (remove env gating) and
  update the canonical perf suite env set accordingly.

### M1: Frame data structures (hashing → dense)

Primary targets (highest leverage):

- [ ] Refactor `WindowFrame` stores to avoid per-frame `HashMap` churn:
  - `crates/fret-ui/src/declarative/frame.rs` (`WindowFrame.instances`, `WindowFrame.children`)
  - Candidate: `slotmap::SecondaryMap<NodeId, ...>` or a dense `Vec` store keyed by `NodeId`.
- [ ] Replace `Arc<[NodeId]>` for `WindowFrame.children` with a reuse-friendly representation.
  - Candidate: store `Vec<NodeId>` in a slab/arena and reference by index + generation.
- [ ] Replace invalidation “visited”/scratch `HashMap<NodeId, u8>` with generation-stamped tables:
  - `crates/fret-ui/src/tree/mod.rs` invalidation propagation caches.
- [ ] Ensure deterministic ordering is preserved where diagnostics rely on it (bundle stability).

Perf acceptance:

- [ ] `ui-gallery-overlay-torture.json`: p95 total improves; invalidation nodes/calls do not regress.
- [ ] `ui-gallery-virtual-list-torture.json`: tail latency improves or stays flat.

### M2: Allocation model (per-frame scratch arena)

- [ ] Introduce a `FrameArena` (or equivalent) for UI runtime scratch allocations.
  - Reference: `repo-ref/zed/crates/gpui/src/arena.rs`.
- [ ] Convert at least 2 hot scratch paths to arena-backed allocation:
  - Candidate A: semantics snapshot traversal scratch (stack + visited).
  - Candidate B: GC reachability scratch sets / temporary vectors in mount/GC.
- [ ] Add an optional allocation counter hook for local profiling (feature-gated).
  - Keep it portable; do not require a global allocator swap for normal builds.

Correctness acceptance:

- [ ] Existing `cargo nextest run -p fret-ui` remains green.
- [ ] `fretboard diag repro ui-gallery` smoke suite passes.

### M3: Hit testing (bounds tree / spatial index)

- [ ] Implement a bounds tree (R-tree variant) built during prepaint per layer root.
  - Reference: `repo-ref/zed/crates/gpui/src/bounds_tree.rs`.
- [ ] Route pointer move/down hit-testing through the bounds tree for large trees.
- [ ] Define “fallback” conditions clearly (transforms, clips, non-axis-aligned bounds).
- [ ] Add a stress script gate that correlates pointer-move frequency with stable frame time.
  - Candidate: extend `tools/diag-scripts/ui-gallery-hover-layout-torture.json`.

Perf acceptance:

- [ ] Pointer-move heavy cases should stay paint-only (no layout) unless explicitly required.
- [ ] Hit-test CPU time should be bounded as node count scales.

### M4: Windowed surfaces (prepaint-driven visible windows)

- [ ] Pick the first “editor-class” migration target:
  - Option A: VirtualList visible window derivation in prepaint (ADR 0190 alignment).
  - Option B: Code view visible-line window derivation in prepaint (code editor feel).
- [ ] Ensure cache-root reuse remains stable under steady scroll/pan.
- [ ] Add a “window boundary crossing” script that fails if it triggers full rerender too often.

Perf acceptance:

- [ ] `ui-gallery-virtual-list-torture.json`: steady scroll should avoid cache-root rerender in most frames.
- [ ] `ui-gallery-code-view-scroll-refresh-baseline.json`: no hitch spikes after warmup.

### M5: Text pipeline stabilization (editor-ready)

- [ ] Document stable cache keys for measure/shaping (wrap width, font stack, style).
- [ ] Reduce redundant text measurements under intrinsic probes (layout engine + `TextWrap::None` paths).
- [x] Add a fast path for “min-content probes” (e.g. `wrap=Word` + `max_width=0`) to avoid O(n²) text wrapping.
  - Implemented by `perf(fret-render): fast-path wrapped text measure` (see perf log entry for commit `9440648a`).
- [x] Reduce repeated shaping work when taffy calls `measure()` under multiple intrinsic modes (min/max/definite).
  - Implemented by caching single-line shaping + cluster-based wrap stats (see `ui-perf-zed-smoothness-v1-log.md`).
- [ ] Add diagnostics hooks to identify text cache misses that correlate with perf hitches.
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
