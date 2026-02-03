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

- Still uses regular heap allocations for many per-frame intermediates (mount scratch, per-frame vectors, etc.).

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

- We have good CPU breakdown (`layout/prepaint/paint`), but we still lack a tight “GPU-side budget” feedback loop:
  - glyph atlas eviction/reupload,
  - texture uploads,
  - draw-call/batch counts,
  - GPU timing / present time.

Impact:

- CPU looks “fast enough” but end-to-end still jitters due to GPU / resource churn.

Proposal:

- Add optional renderer telemetry into perf bundles:
  - per-frame batch count, text glyph uploads, atlas evictions,
  - (if available) GPU timestamp queries for scene encoding / render passes.

Acceptance:

- Correlate tail hitches with a specific GPU churn signature; fix by stabilizing caches or batching.

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
