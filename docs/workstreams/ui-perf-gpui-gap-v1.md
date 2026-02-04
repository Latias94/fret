# UI Performance: GPUI Gap Analysis (v1)

Status: Draft (workstream note; ADRs remain the source of truth)

This document captures a concrete, code-linked list of **performance gaps** between:

- **Fret** (this repo), and
- the **GPUI** substrate as used by Zed (reference: `repo-ref/zed/crates/gpui`).

The goal is not to copy GPUI, but to identify the *mechanisms* that matter for “Zed feel” and make them measurable
via `fretboard diag perf` scripts and perf logs.

Related:

- Zed smoothness workstream: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- TODO tracker: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Perf log: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`

---

## 0) Current state (Fret)

Recent editor-class win (evidence lives in the perf log):

- Code editor autoscroll torture: `p95 paint ~23ms → ~5ms` via `perf(fret-code-editor): cache syntax rich rows`
  (commit `81159325`).

This removes an obvious “can’t ever feel like Zed” bottleneck, but it does **not** yet guarantee Tier B (120Hz)
budgets across editor-class pages. The remaining work is mainly about *systemic* caching + allocation strategy.

Recent “make GPU churn measurable” win (so we can explain tail hitches, not just average frame time):

- Diagnostics bundles now export best-effort text atlas + intermediate pool churn signals, and `fretboard` can sort by them.
  - Evidence: commits `d10cac5a` + `c9a8b168` + perf log entries in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
- A deterministic blur/effects workload exists to make intermediate pool counters non-zero:
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Scripts: `tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json`,
    `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`

---

## 1) What GPUI does that matters for smoothness

The following are “load-bearing” for Zed feel. Each entry links to the GPUI reference location.

### 1.1 Per-frame element allocation arena (eliminate heap churn)

GPUI uses an **Arena** for element allocation during a window draw:

- Arena implementation: `repo-ref/zed/crates/gpui/src/arena.rs`
- Per-App arena + draw scope: `repo-ref/zed/crates/gpui/src/window.rs` (`ElementArenaScope`, `Window::draw`)

Key property:

- Elements can be allocated with near-zero per-element overhead and cleared in bulk after the frame.

### 1.2 Explicit “view caching” contract (recycle layout + paint)

GPUI provides an explicit “cached view” mechanism:

- `AnyView::cached`: `repo-ref/zed/crates/gpui/src/view.rs`

The contract is:

- If the view does not call `Context::notify` between frames, GPUI can recycle its previous layout + paint
  (except under explicit `Window::refresh`).

This is a correctness + performance *surface area* that’s easy to reason about.

### 1.3 Text system: frame-to-frame layout reuse (double-buffered cache)

GPUI’s text system uses a **current/previous frame cache** for line layouts:

- `LineLayoutCache`: `repo-ref/zed/crates/gpui/src/text_system/line_layout.rs`
  - `previous_frame: Mutex<FrameCache>`
  - `current_frame: RwLock<FrameCache>`
  - `finish_frame()` swaps and clears
  - `layout_index()` / `reuse_layouts(range)` / `truncate_layouts(index)` allow fine-grained reuse for partial updates

This is the kind of mechanism that keeps “scrolling a code editor” stable even while edits happen, because the
dominant cost (layout/shaping) is amortized and re-used across frames.

### 1.4 Scene replay as a first-class mechanism

GPUI’s `Scene` supports replaying a range of previous paint operations:

- `Scene::replay`: `repo-ref/zed/crates/gpui/src/scene.rs`

This is tightly coupled with view caching: cached views can effectively “replay” previously built primitives.

---

## 2) Where Fret is already aligned (or close)

These areas are already on the right track, but may still require tightening keys and defaults:

### 2.1 Paint-cache replay exists

Fret has an explicit paint-cache with replay:

- `crates/fret-ui/src/tree/paint_cache.rs`
- `crates/fret-ui/src/tree/paint.rs` (replay path)

This is analogous in spirit to `gpui::Scene::replay`, but GPUI appears to have a more explicit “cached view”
contract on top.

---

## 3) Primary performance gaps (GPUI vs Fret)

This section is the actionable “gap list”.

### Gap A: No per-frame arena for UI “element allocations / scratch”

GPUI:

- Uses `Arena` for element allocation during `Window::draw` (`repo-ref/zed/crates/gpui/src/window.rs`).

Fret:

- Partial progress: Fret now reuses a small set of retained scratch collections for hot per-frame traversals:
  - GC reachability scratch (`HashSet<NodeId>` + `Vec<NodeId>`) in declarative mount/GC
  - Semantics snapshot traversal scratch (`HashSet<NodeId>` + `Vec<(NodeId, Transform2D)>`)
  - Evidence: `perf(fret-ui): reuse GC/semantics scratch via frame arena` (commit `3d6e2431`).
- Partial progress: reduce per-frame heap churn during declarative element ID derivation by removing
  per-scope `HashMap` allocations for callsite counters.
  - Evidence: `perf(fret-ui): remove callsite counter HashMap churn` (commit `2dd36fde`).
- Diagnostics now export “frame arena scratch” proxies into perf bundles:
  - `top_frame_arena_capacity_estimate_bytes`
  - `top_frame_arena_grow_events`
  - Evidence: `feat(diag): export frame arena scratch stats` (commit `fe0ad7c3`) + perf log entry for `1b0364e9`.
- Still missing: a true “allocate elements in an arena, bulk-clear after draw” model akin to GPUI’s `Arena`.
  - Partial progress: pooled `Vec<AnyElement>` child buffers across frames (arena-adjacent; reduces allocation churn but is not a bump arena).
    - `perf(fret-ui): pool element children vectors` (commit `07a4c252`) + `perf(fret-ui): make element children vec pool LIFO` (commit `693a55b0`)
    - Diagnostics: `top_element_children_vec_pool_reuses` / `top_element_children_vec_pool_misses` (commit `cbcd81ed`)
    - Evidence: the pool reaches stable “0 misses” steady state on:
      - `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
      - `tools/diag-scripts/ui-gallery-chrome-torture-steady.json`
      (see perf log entries under `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`)

Impact:

- Higher tail latency under editor-class workloads due to allocator + cache-miss amplification.

Proposal:

- Implement `FrameArena` (or equivalent) and migrate at least two hot paths first:
  1) declarative mount scratch allocations,
  2) prepaint/paint traversal scratch (stacks/vectors).

Acceptance:

- `ui-gallery-steady` suite p95 stable or improves; `max` outliers reduce.
- Add a micro-benchmark counter (feature gated) to show allocation drop (counts / bytes).

### Gap B: Code editor / code view text pipeline lacks GPUI-like frame reuse contract

GPUI:

- Double-buffered text layout cache with explicit reuse ranges (`LineLayoutCache`).

Fret:

- Has text shaping optimizations and now caches “syntax rich rows” in the code editor, but we do not yet have a
  general “frame-to-frame text layout reuse” API that:
  - supports partial reuse across frames for the visible line window,
  - is tightly integrated with editor scrolling + selection/cursor overlays.

Impact:

- Code editor “steady-state” can be good after warmup, but tends to be fragile when something triggers rebuilds
  (theme toggles, language changes, transient overlays, IME composition).

Proposal:

- Introduce a “visible window layout cache” for editor-like surfaces:
  - a per-editor `layout_index` concept (bookends) similar to GPUI’s `LineLayoutIndex`,
  - reuse previous frame layouts for unchanged rows,
  - make cache invalidation explicit via revision keys (buffer/theme/language/wrap width/scale factor).

Acceptance:

- `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json` stays within Tier B on high-end HW
  (p95 total <= 4ms, max <= 8ms), or at least closes the remaining gap.

### Gap C: “Cached view” ergonomics and defaults

GPUI:

- Explicit `AnyView::cached` contract, tied to `notify` semantics.

Fret:

- View-cache exists, but:
  - it’s easy to accidentally invalidate cache roots,
  - it’s non-obvious which “views” should opt into caching by default in an editor-class app.

Impact:

- Developers overpay for redraws because the “happy path” is not naturally cached.

Proposal:

- Define a clearer caching surface (documentation + debug counters):
  - which widgets/views are cache roots by default,
  - which invalidations punch through (layout vs paint vs hit-test),
  - expose “cache-root misses by reason” in diagnostics.

Acceptance:

- Add a perf gate that fails if cache-root reuse rate drops for key scripts (scroll, hover, editor autoscroll).

### Gap D: Renderer-side batching and “glyph prep” observability

GPUI:

- Has a cohesive scene model and platform renderer pipelines around that `Scene`.

Fret:

- We have good CPU breakdown (`layout/prepaint/paint`), but historically lacked a tight “GPU-side budget” feedback loop:
  - glyph atlas eviction/reupload,
  - texture uploads,
  - draw-call/batch counts,
  - GPU timing / present time.

Impact:

- CPU looks “fast enough” but end-to-end still jitters due to GPU / resource churn.

Proposal:

- Status: **Partially implemented**.
  - Basic renderer telemetry is now exported into diagnostics bundles (best-effort) under
    `.windows[].snapshots[].debug.stats.renderer_*`:
    `encode_scene_us`, `prepare_text_us`, `prepare_svg_us`, `draw_calls`,
    `pipeline_switches`, `bind_group_switches`, `scissor_sets`,
    uniform/instance/vertex byte counts, and scene encoding cache hit/miss counts.
  - `fretboard diag stats` and `fretboard diag perf --json` can now sort/report by these renderer metrics.
  - Commits: `0e4928fe` + `cf8975ca`. Evidence runs are logged in
    `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
  - Renderer churn signals are now exported (best-effort) for tail-hitch correlation:
    - Text atlas: `renderer_text_atlas_revision`, `renderer_text_atlas_upload_bytes`,
      `renderer_text_atlas_evicted_pages`, `renderer_text_atlas_resets` (and related counters).
    - Intermediate pool: `renderer_intermediate_peak_in_use_bytes`,
      `renderer_intermediate_pool_evictions` (and related counters).
    - Commits: `d10cac5a` + `c9a8b168`.

Next:

- Extend the exported telemetry with additional “GPU churn” and “occupancy” signals:
  - glyph atlas occupancy / live page count (to distinguish “one-time warmup” vs “thrash”),
  - texture upload bytes for non-text assets (images, SVG masks, path intermediates),
  - (optional) GPU timestamp queries for render passes + present/submit time when supported.
- Promote churn into a first-class perf log surface:
  - require `diag perf --json` output to include churn vectors for the top frames,
  - record churn p95/max alongside the CPU breakdown for each perf run (so regressions are explainable).

Acceptance:

- Correlate tail hitches with a specific churn signature (CPU or GPU); fix by stabilizing caches or batching.

---

## 4) Proposed milestone mapping (additive to the Zed smoothness workstream)

These are intended as “fearless refactor” milestones, each with a measurable acceptance gate.

### G0: Instrumentation parity (CPU + GPU)

- Add renderer churn metrics to bundles (feature-gated).
- Add “cache-root miss reasons” counters (already partially tracked; make them actionable).

### G1: FrameArena (allocation model)

- Implement and migrate 2 hot scratch paths.
- Progress:
  - `perf(fret-ui): reuse frame scratch buffers` (commit `a39e79c4`) reuses a small set of per-frame scratch buffers
    without changing contracts (mount pending invalidations, paint-cache replay traversal stack, interaction-cache
    replay scratch).

### G2: Text frame cache reuse API (GPUI-like)

- Add “layout index” bookends + reuse range plumbing for editor surfaces.

### G3: Editor steady-state to Tier B

- Make `code-editor autoscroll` a first-class acceptance script for Tier B.

---

## 5) How to keep this document current

Whenever we close a gap:

- Add the commit hash + the exact `diag perf` command + p50/p95/max to
  `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
- Update the corresponding TODO entry in `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`.
