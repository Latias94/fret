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
- Perf workflow skill (how to run/gate/log): `.agents/skills/fret-perf-workflow/SKILL.md`

---

## Workstream contract (how we measure “closing the gap”)

This workstream uses a strict “numbers or it didn’t happen” contract.

Protocol:

- Every gap item must map to at least one **probe** that can be run via `fretboard diag perf`.
- Every milestone must be guarded by a **gate** (baseline + thresholds) and recorded in the perf log with:
  - exact command line,
  - suite/baseline version,
  - artifacts directory,
  - worst bundles (for tail explanation),
  - commit hash (for rollback).

Recommended default gate set (global sanity):

- `ui-gallery-steady` (canonical baseline)
- `ui-resize-probes` (attempts=3)
- `ui-code-editor-resize-probes` (attempts=3)

Evidence template (copy/paste into the perf log):

```text
## YYYY-MM-DD HH:MM:SS (commit `<hash>`)

Change:
- <what changed>

Suite(s):
- <suite list>

Commands:
<exact commands>

Artifacts:
- <out dir paths>

Results:
- <PASS/FAIL + key deltas>

Worst bundles:
- <paths>
```

---

## 0) Current state (Fret)

Recent editor-class wins (evidence lives in the perf log):

- Code editor autoscroll torture: `p95 paint ~23ms → ~5ms` via `perf(fret-code-editor): cache syntax rich rows`
  (commit `81159325`).
- Post-merge editor regression fix: `paint_time_us p95 ~30ms → ~0.7ms` by eliminating allocator churn
  (per-row `Theme` clone) in syntax paint (commit `0d8ad27ac`).
- Code editor resize drag smoothness: `top_total_time_us ~42ms → ~16ms` by making `CodeEditorHandle::set_language(...)`
  idempotent (avoid per-frame syntax/rich cache resets), guided by in-bundle Canvas phase attribution (commits
  `f664ead2d`, `1778ba563`).

This removes an obvious “can’t ever feel like Zed” bottleneck, but it does **not** yet guarantee Tier B (120Hz)
budgets across editor-class pages. The remaining work is mainly about *systemic* caching + allocation strategy.

Recent “make GPU churn measurable” win (so we can explain tail hitches, not just average frame time):

- Diagnostics bundles now export best-effort renderer churn signals, and `fretboard` / `tools/perf/perf_log.py`
  can surface them in a commit-addressable way:
  - text atlas upload / eviction signals,
  - non-text upload bytes (SVG + image),
  - SVG raster cache occupancy + eviction signals,
  - intermediate pool lifecycle signals (budget / peak / in_use, allocations / reuses / releases / evictions, free bytes / textures).
  - Evidence: perf log entries for commits `3d1510a7` (SVG cache churn) and `52f555d5` (intermediate pool lifecycle).
- A deterministic blur/effects workload exists to make intermediate pool counters non-zero:
  - Harness: `FRET_UI_GALLERY_HARNESS_ONLY=effects_blur_torture`
  - Scripts: `tools/diag-scripts/ui-gallery-effects-blur-torture-steady.json`,
    `tools/diag-scripts/ui-gallery-effects-blur-thrash-steady.json`

---

## 0.1 Is GPUI the right north star?

GPUI is a strong reference for “Zed feel”, but it is not a universal renderer template.

What is most transferable for Fret:

- explicit frame-to-frame reuse contracts (cached views + `notify` semantics),
- side-effect free view updates (render-time setters must be idempotent; no per-frame cache resets),
- aggressive per-frame scratch / arena allocation discipline,
- a deliberate text layout cache model (double-buffered reuse, visible-window aware),
- scene replay primitives that make caching explicit and cheap.

What is less transferable 1:1:

- effect-heavy pipelines that require pooled intermediates (blur, clip masks, soft clipping),
- “multi-viewport editor chrome + docking” interaction arbitration (where policy and mechanism boundaries differ).

For effect/renderer architecture, it is often more productive to cross-check against render-graph style engines
(and existing large-scale UIs like Flutter/Skia) while using GPUI as the interaction + caching reference model.

## 0.2 Profiling playbook (bottom-up, editor-class)

To close the gap responsibly, treat perf as a contract and work from the “lowest primitives” upward:

0) **Eliminate per-frame side effects in declarative render loops** (the “idempotent setters” footgun class).
   - Heuristic: if `handle.set_*` is called from render, it must be a no-op for identical values.
   - Reference: `docs/workstreams/ui-perf-setter-idempotency-v1.md`.
1) **Pick a single hot-path probe** (pointer move, wheel, resize, scroll) and gate it.
   - Pointer move gate: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
   - Gate flags: `fretboard diag perf --max-pointer-move-dispatch-us/--max-pointer-move-hit-test-us/--max-pointer-move-global-changes`
   - Extract derived pointer-move stats from a captured bundle via:
     `fretboard diag triage <bundle.json> --json` (`stats.pointer_move.*`)
2) **Explain tail latency** with bundles, not averages.
   - Use `fretboard diag stats <bundle.json> --sort time --top 30` to find the heaviest frame and why it was heavy.
3) **Separate CPU vs GPU** early.
   - Use `fretboard diag repro ... --with tracy` / `--with renderdoc` (best-effort) to confirm whether a hitch is CPU
     dispatch/layout/prepaint vs GPU encoding/upload/pipeline churn.
4) **Focus on allocation discipline** as a first-order knob.
   - If pointer move is not “paint-only”, treat it as a bug: eliminate model/global churn and per-event allocations
     before attempting deeper algorithmic changes.

This playbook is intentionally compatible with “fearless refactors”: each change should produce a measurable delta and
an entry in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (commit-addressable) so regressions are reversible.

## 0.2.1 Resize smoothness: gaps to investigate vs GPUI/Zed

Resize-drag is a “stress multiplier”: it invalidates layout constraints, which tends to trigger the widest set of
reflow + paint work. Zed feels smooth here primarily because it keeps the per-frame work bounded and predictable.

What we already do in Fret (evidence in the perf log):

- **Coalesce resizes to once per frame** at the runner boundary (apply pending size at `RedrawRequested`).
  - Code: `crates/fret-launch/src/runner/desktop/app_handler.rs` (`pending_surface_resize` is applied inside
    `WindowEvent::RedrawRequested`).
- **Defer known-expensive scroll measurement** while the viewport is actively resizing (unbounded probe deferral).
- **Make resize probes stable and reproducible** (so baselines measure “work per resize” rather than “scheduler timing”).

Open questions / likely gaps (need code-level confirmation against `repo-ref/zed/crates/gpui`):

Baseline fact (quick reference):
- On macOS, GPUI invokes a resize callback from the view `set_frame_size` path when the frame size actually changes
  (see `repo-ref/zed/crates/gpui/src/platform/mac/window.rs`).
  - It also sets the view’s layer redraw policy to redraw during live resize:
    `NSViewLayerContentsRedrawDuringViewResize` (same file).
- On Wayland, GPUI explicitly throttles interactive resizes to once per vblank (`configure.resizing` + `resize_throttle`)
  (see `repo-ref/zed/crates/gpui/src/platform/linux/wayland/window.rs`).

1) **Text layout cache model under width jitter**
   - Hypothesis: GPUI amortizes shaping/line-break work via a cache keyed by font+style+wrap width buckets or by a
     layout index (visible-window aware), so “resize drag” does not reshuffle all paragraphs every frame.
   - Fret TODO: make “width jitter” a first-class acceptance probe for editor surfaces (not just UI chrome).
     - Implemented probe: `ui-code-editor-resize-probes` (`tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`).
   - Interim win: for plain LTR paragraphs, use a “shape once → slice lines” wrap path to avoid per-line shaping on
     long text (commit `4f2009408`, default-on threshold in `10e7d97fc`).
   - Recent win: stabilize `TextService::measure` wrapped-text shaping reuse working-set to reduce rare
     `layout_engine_solve_time_us` tail spikes during interactive resize (commit `f2c08b806`).
     - Default: `FRET_TEXT_MEASURE_SHAPING_CACHE_ENTRIES=4096`
     - Short-label avoidance: `FRET_TEXT_MEASURE_SHAPING_CACHE_MIN_TEXT_LEN_BYTES=128`
     - This is still a FIFO, process-global cache; a more GPUI-like end state likely involves a length-bucketed
       or LRU policy and/or “visible window aware” caching so long-lived steady suites don't accumulate
       low-value entries.
   - Fret stopgap (default-on for jitter-class interactive resize):
     - `FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_BUCKET_PX` (default: `32`; set `0`/`1` to disable).
     - `FRET_UI_TEXT_WRAP_WIDTH_SMALL_STEP_MAX_DW_PX` (default: `64`; widens the “small-step” class so bucketing
       applies under common per-frame drag deltas; commit `53aa6534a`).
     - Applies only for small-step resizes (e.g. `drag-jitter`), and only while interactive resize is active.
     - Small-step detection is symmetric (back-and-forth drags keep the same policy/caches enabled).
       - Implementation: `perf(fret-ui): treat small-step resize symmetrically` (commit `0de40863f`).
       - Evidence: perf log entry `2026-02-09 16:37:00` (jitter probe p95 total improves by ~0.3ms).
   - Fret experiment knob (still default-off, broader scope): `FRET_UI_TEXT_WRAP_WIDTH_BUCKET_PX`.
   - Latest evidence: see the perf log entries dated `2026-02-08` for `ui-resize-probes` gate stability before/after
     the small-step default bucketing change.
   - Conclusion: quantization is a pragmatic “make live-resize bounded” lever, but it is not the end state; the
     longer-term direction remains improving wrapped-text reuse (separate shaping vs wrapping keys and reuse line
     layouts across frames), closer to GPUI’s amortization model.

2) **Layout invalidation granularity**
   - Hypothesis: GPUI keeps invalidation scope tight (subtree diffs) and avoids re-walking “known static” chrome.
   - Fret TODO: tighten layout-root construction and subtree invalidation so a resize does not always imply
     “layout the whole tree” when only a small set of constraints changed.
   - Current Fret mechanism cost center:
     - The flow layout engine request/build phase currently walks (and “requests”) the mounted subtree each frame to
       keep stable identity (`TaffyLayoutEngine::seen` + stale-node GC at `end_frame`).
     - `TaffyLayoutEngine` still uses hashing-heavy per-frame tables (`HashMap`/`HashSet`) keyed by `NodeId`
       (a `slotmap` key), which is a strong candidate explanation for
       `layout_request_build_roots_time_us ~= 2–4ms` under resize drag-jitter.
     - Direction: M1 “hashing → dense tables” (e.g. `slotmap::SecondaryMap` + generation stamps) in the layout engine.

3) **Per-frame allocation discipline on hot resize frames**
   - GPUI likely relies heavily on per-frame scratch arenas and stable caches; sporadic allocations can manifest as
     rare tail hitches even when p90 looks fine.
   - Fret TODO: track allocation and cache miss reasons directly in resize bundles (already partially available via
     layout and view-cache counters) and close remaining blind spots.
   - Recent win: reduce avoidable allocations in the flow layout request/build phase (no more
     `UiTree::children(...).to_vec()` clones in flow build; avoid cloning the previous children vec in
     `TaffyLayoutEngine::set_children`).
     - Implementation: commit `10e30dac1`
     - Evidence: perf log entry `2026-02-09 09:10:11` (drag-jitter worst-case max total `27.5ms → 21.1ms`).
   - Negative result: a wrapper-chain memoization attempt using a per-build `HashMap` regressed
     `layout_request_build_roots_time_us` on drag-jitter (commit `96661c49c`).
     - Evidence: perf log entry `2026-02-09 15:28:00` in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.
     - Takeaway: prefer the M1 dense-table refactor over adding more per-frame hashed caches.

4) **GPU work scaling with surface area**
   - Even if CPU layout is stable, large resizes can spike GPU cost if we re-rasterize masks, upload atlases, or
     thrash intermediate textures.
   - Fret TODO: ensure resize probes include renderer churn counters in the log (text atlas, SVG cache, intermediate
     pool) and classify whether tail spikes are CPU-only or GPU-influenced.

## 0.3 Pointer-move hit-test status (current probe)

Current “Zed feel” probe:

- Script: `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-steady.json`
- Gate: `fretboard diag perf ... --max-pointer-move-dispatch-us/--max-pointer-move-hit-test-us`

Findings (macOS Apple M4; repeat=7):

- With bounds-tree enabled *and cached-path skipped*, pointer-move hit testing is effectively solved for this probe:
  - `hit_test_time_us`: p50 ~3us, p95 ~3us, max ~10us (across runs).
  - `dispatch_time_us`: **bimodal** due to timer dispatch:
    - Overall: p50 ~30us, p95 ~250us, max ~303us (across runs).
    - No-timer pointer-move frames: p50 ~16us, p95 ~25us, max ~38us.
    - Timer pointer-move frames: p50 ~241us, p95 ~254us, max ~303us.
- Micro timer breakdown shows why this mattered:
  - Before the skip, `try_hit_test_along_cached_path` dominated hit-test time due to conservative sibling scanning.
  - Bounds-tree query time was already single-digit microseconds.
- Dispatch micro timers are now exported to attribute the post-hit-test remainder:
  - `dispatch_widget_bubble_time_us`, `dispatch_input_context_time_us`, `dispatch_hover_update_time_us`, etc.
  - Follow-up finding (commit `5ab4ba71`): the pointer-move “dispatch tail” is dominated by **timer event dispatch**
    (`dispatch_timer_event_time_us`), not pointer routing.
- Cached-path reuse remains low on the stripes sweep workload (pointer crosses many regions):
  - With bounds-tree disabled (A/B), hit testing rises to ~2ms p50 and can spike to ~4ms, and cached-path hit rate is
    still ~2.1%.

Implication:

- For Tier B “Zed feel”, a spatial index (bounds-tree or equivalent) is mandatory, and “cached-path hit testing” should
  not be attempted when the index is enabled (it can be slower than the index on sibling-heavy trees).

Evidence:

- Perf log entries under:
  - `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (commits `763bf8e7`, `8bc15eda`, `7fa76fd5`, `5ab4ba71`)

---

## Milestones (v1)

These milestones are intentionally “mechanism-first” and map to measurable probes/gates.

### M0: Measurement discipline (keep experiments reversible)

Goal:

- Every hot-path change lands with a probe/gate and a perf log entry.

Acceptance:

- `ui-gallery-steady` (canonical baseline) stays green.
- `ui-resize-probes` and `ui-code-editor-resize-probes` are stable under attempts=3 on the primary dev machine(s).
  - Evidence: perf log entries `2026-02-09 13:31:35` (commit `1778ba563`) and `2026-02-09 13:46:46`
    (commit `007006b28`) in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`.

### M1: Resize-drag becomes predictable (bounded tail)

Goal:

- Keep resize-drag work bounded and predictable under both:
  - stress resize (`ui-gallery-window-resize-stress-steady`), and
  - width-jitter resize (`ui-gallery-window-resize-drag-jitter-steady`).

Acceptance:

- The `ui-resize-probes` gate is stable under attempts=3 (low tail flake) and worst bundles are explainable.

### M2: “Text under width jitter” closes the GPUI amortization gap

Goal:

- Move from “wrap-width quantization as a stopgap” toward GPUI-like frame-to-frame layout reuse.

Acceptance:

- `ui-code-editor-resize-probes` stays under budget with lower `Text::prepare` churn signals and fewer rare tail spikes.
  - Evidence: perf log entry `2026-02-09 13:46:46` (commit `007006b28`) shows worst frames `~15.1–15.8ms` vs `16308us` target.

### M3: Frame scratch / allocation discipline (arena-first)

Goal:

- Reduce allocator amplification on hot frames by moving scratch-heavy hot paths onto arenas/pools.

Acceptance:

- Key steady-state scripts show fewer tail outliers while maintaining correctness gates.

### M4: GPU churn is explainable and bounded

Goal:

- When we do hitch, bundles clearly show whether it is CPU or GPU (uploads/evictions/intermediate pool thrash).

Acceptance:

- Deep triage runs (`FRET_DIAG_RENDERER_PERF=1`) produce consistent churn tables for the worst bundle(s), and fixes can be gated indirectly (CPU) while explained by churn deltas.

### Recommended next steps (short-horizon)

1) Keep the gate set stable after large refactors.
   - Re-run: `ui-gallery-steady` + resize gates attempts=3.
   - If `ui-resize-probes` tail becomes flaky, cut a new baseline with `tools/perf/diag_perf_baseline_select.sh`.
2) Start “GPUI-like text reuse” as the next high-leverage gap closure.
   - Target: reduce wrapped-text churn under width jitter without relying primarily on wrap-width quantization.
   - Probe: `ui-code-editor-resize-probes` (plus the editor torture steady script for warm caches).
3) Push “paint-only” frames as a first-class goal.
   - Add/maintain an explicit gate check (where applicable) that asserts drag frames can be replayed from cache roots
     without triggering extra layout/paint work.

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

### Gap → probe / gate map (quick index)

| Gap | Primary probe(s) | Gate / baseline |
| --- | --- | --- |
| Frame arena / scratch discipline | `ui-gallery-steady` (pick worst steady script), `ui-gallery-chrome-torture-steady` | `ui-gallery-steady` (canonical baseline) |
| Timer work on interactive frames | `ui-gallery-hit-test-torture-stripes-move-sweep-steady` | pointer-move thresholds (`--max-pointer-move-*`) |
| Text under width jitter | `ui-code-editor-resize-probes`, `ui-resize-probes` | `ui-code-editor-resize-probes` baseline + attempts=3 |
| Resize tail predictability | `ui-resize-probes` | `ui-resize-probes` baseline + attempts=3 |
| “Paint-only” drag frames (cache replay) | `ui-resize-probes` (and future “paint-only replay” checks) | `ui-resize-probes` + `--check-drag-cache-root-paint-only <test_id>` (when available/maintained) |
| GPU churn / intermediate pool thrash | `ui-gallery-effects-blur-torture-steady`, `ui-gallery-effects-blur-thrash-steady` | deep triage only (`FRET_DIAG_RENDERER_PERF=1`); gate CPU separately |

### Gap A: No per-frame arena for UI “element allocations / scratch”

GPUI:

- Uses `Arena` for element allocation during `Window::draw` (`repo-ref/zed/crates/gpui/src/window.rs`).

Fret:

- Partial progress: Fret now reuses a small set of retained scratch collections for hot per-frame traversals:
  - GC reachability scratch (`HashSet<NodeId>` + `Vec<NodeId>`) in declarative mount/GC

### Gap B: Timer scheduling contracts (avoid timer work on interactive frames)

GPUI:

- Has a tight “frame boundary” model (draw/build happens in a bounded window scope). The executor/event loop is
  designed around coalescing and doing minimal work per tick to maintain smoothness.

Fret:

- Pointer-move frames can include timer event dispatch, and the timer work can dominate per-frame dispatch time.
- Evidence (stripes pointer-move probe, commit `5ab4ba71`): timer pointer-move frames spend ~220–230us in
  `dispatch_timer_event_time_us` p50, while pointer routing itself is ~10–40us.
- Follow-up attribution (commit `98ca4fe3`) showed the slow timer frames were dominated by a single broadcast token
  (`TimerToken(1)`) rather than targeted timer routing.
- For scripted harness perf probes, suppressing ui-gallery’s dev-only config poller timer
  (`with_config_files_watcher(...)`) removed timer dispatch from pointer-move frames entirely and collapsed
  pointer-move dispatch p95 from ~247us → ~26us (commit `06feeb41`).
- Re-check on current `main`: even when force-enabling the watcher in harness mode
  (`FRET_UI_GALLERY_ENABLE_CONFIG_WATCHER=1`, added by `e978fe85`), pointer-move dispatch remains at the “noise floor”
  for this probe (p95 ~16us; see perf log entry 2026-02-05 15:59:00, commit `1293364f`).

Implication:

- To reach Zed feel, “timer work” must either be (a) coalesced into frames where it is expected, (b) made cheap enough
  to be effectively free, or (c) moved off the critical input path (defer/batch/background where possible).
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

### Gap B.2: Editor-class scene construction still happens in hot paint closures (Canvas-heavy)

GPUI’s “Zed feel” comes from a combination of:

- explicit cached-view reuse (`AnyView::cached` + `notify` discipline), and
- cheap scene replay primitives (`Scene::replay`) on top of that contract.

Fret has paint-cache replay mechanisms at the `UiTree` level, but editor-class pages can still spend multiple
milliseconds per frame in **CPU-side scene construction** inside `Canvas` paint closures.

Evidence (post-merge regression + fix; see perf log entry 2026-02-06, commits `72e6c32df` and `0d8ad27ac`):

- Script: `tools/diag-scripts/ui-gallery-code-editor-torture-autoscroll-steady.json`
- Regression symptom: `paint_time_us` p95 ~30ms, dominated by a single `ElementInstance::Canvas` host widget.
  - Renderer self-time was not the dominant slice in the worst bundle (`encode_scene_us~0.7ms`, `prepare_text_us~0.6ms`),
    which points away from “GPU encode is the bottleneck” and toward CPU-side work.
- Root cause: allocation churn (accidental per-row `Theme` clone during syntax span → rich text construction).
- Fix (commit `0d8ad27ac`): `paint_time_us` p95 collapses to sub-millisecond (~0.7ms) in the same probe.

Why this matters:

- It’s easy to misattribute “Canvas paint is slow” to renderer work or missing replay primitives. In practice, the most
  immediate “Zed feel” killers are often accidental allocations in tight loops (per-row/per-glyph/per-span).
- GPUI is a useful reference here not just for caching primitives, but for its strict draw-scope allocation discipline.

What to do about it (transferable lessons from GPUI, not literal copying):

1) Make **windowed content reuse** explicit for large scroll surfaces (code editor / code view):
   - reuse per-line/per-row prepared layout + paint ranges across frames,
   - when scrolling, prefer translating cached ranges instead of re-emitting all primitives.
2) Introduce a retained sub-surface contract for complex paint closures:
   - a `Canvas`-like API should be able to record a stable “display list” and replay it when inputs are unchanged,
   - tie invalidation to explicit keys (frame-local ids are poison for perf).
3) Treat scene ops as a budget:
   - track ops per frame and reduce them via batching/layering.

Fret reference points:

- `ElementInstance::Canvas` paint path: `crates/fret-ui/src/declarative/host_widget/paint.rs`
- Canvas resource caching (text/path/svg) exists but does not by itself eliminate per-frame scene construction:
  `crates/fret-ui/src/canvas.rs` (`CanvasCache`, `CanvasCachePolicy`)

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

#### Gap C.2: Window resize currently defeats view-cache reuse (hit-test coupling)

In the steady-state suite, `ui-gallery-window-resize-stress-steady` is still the worst overall script, and the
worst frame shows a characteristic signature:

- `layout_time_us` dominates (root/widget layout traversal),
- `paint_time_us` is then dominated by `paint_text_prepare_time_us` where the reason mask is entirely
  `width_changed` (word wrap reflow), and
- view-cache roots are present but do not reuse on these frames (reported as `not_marked_reuse_root`).

Evidence (macOS Apple M4; `ui-gallery-steady` baseline v11 era):

- Worst bundle: `target/fret-diag-codex-perf-v11/1770350673752-ui-gallery-window-resize-stress-steady/bundle.json`
- `fretboard diag stats --sort time --top 1` (selected lines from the worst frame):
  - `time.us(total/layout/prepaint/paint)=16136/11331/98/4707`
  - `paint_text_prepare.us(time/calls)=3656/18`
  - `paint_text_prepare.reasons(.../width/...)=.../18/...`
  - `top_cache_roots: ... reason=not_marked_reuse_root ...` (3 roots)

Interpretation:

- Fret’s current view-cache reuse gate is conservative and effectively ties “paint reuse” to “hit-test stability”.
  Window resizing forces hit-test invalidation in the chrome-heavy view-cache page, so cache roots cannot reuse,
  and the runtime ends up paying repeated layout + text prepare work across frames.

Proposal (fearless refactor friendly):

1) Decouple “paint reuse” from “hit-test invalidation” for view-cache roots:
   - allow reusing cached paint output while recomputing hit-test data (bounds-tree) when only hit-test is dirty.
2) Make width-change text reflow cheaper by introducing GPUI-like frame-to-frame line-layout reuse:
   - keep shaping results stable,
   - recompute line breaks incrementally for new widths,
   - gate reuse by explicit keys (text/style/font-stack/scale/wrap width).

Acceptance:

- The `ui-gallery-window-resize-stress-steady` row improves measurably:
  - reduce `max top_total_time_us` and `max paint_text_prepare_time_us` outliers,
  - keep correctness (no stale scene/hit-test failures on scripted resize probes).

Update:

- Desktop runner now coalesces resize application to once per frame (commit `beb2fa315`), closer to GPUI’s
  “resize marks dirty; draw happens at the frame boundary” model.
  - GPUI reference: `repo-ref/zed/crates/gpui/src/window.rs` wires `platform_window.on_resize(...)` to
    `Window::bounds_changed(cx)`, which updates `viewport_size` and calls `refresh()`; layout/paint happens later
    during `Window::draw` at the frame boundary.
  - Implication: during an interactive drag-resize, it is expected to “re-layout every frame” (because constraints
    are changing), but not expected to re-layout multiple times per frame due to resize spam.
- Evidence (macOS Apple M4):
  - Single-script probe worst `top_total_time_us`: `16935` (v12 baseline era) → `14219` (post-coalesce run)
  - Suite baseline worst `top_total_time_us`: `16935` (v12) → `15532` (v13)
  - Details and bundles are recorded in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (2026-02-06 13:20).
- Additional experiment (env-gated scroll optimization):
  - Enabling deferred unbounded scroll probes during resize (`FRET_UI_SCROLL_DEFER_UNBOUNDED_PROBE_ON_INVALIDATION=1`)
    improves the same single-script resize probe further to `top_total_time_us=11810` (repeat=7).
  - This suggests a non-trivial portion of the resize tail is `Scroll` “unbounded probe” measurement work.
  - Evidence and command are recorded in `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (2026-02-06 13:45).
- Correctness gates added to make the resize policy safe to iterate:
  - Scroll offset stability: `--check-scroll-offset-stable <test_id>` (commit `6c248d9e1`).
  - Scrollbar thumb geometry validity: `--check-scrollbar-thumb-valid all` (commit `e20637f92`).
- Latest spot check (commit `5208b6883`, repeat=7; reuse-launch):
  - `p95 time.us(total/layout/solve/prepaint/paint)=15204/11659/1799/101/3444`
  - Interpretation: resize remains layout-dominant; primary leverage is reducing layout plumbing overhead and
    width-jitter-induced text churn (not the solve itself).
  - Evidence: perf log entry `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (2026-02-07 08:45).

#### Gap C.1: Stable-frame paint overhead is still opaque (even with cache reuse)

Motivation:

- Several steady-state scripts still show a meaningful `paint_time_us` slice even when view-cache roots are reused
  and paint-cache hits replay prior ops.

Evidence:

- Added paint-phase attribution counters in `feat(diag): add paint pass breakdown metrics` (commit `f2bee87a`):
  - `paint_cache_replay_time_us`
  - `paint_cache_bounds_translate_time_us` / `paint_cache_bounds_translated_nodes`
  - `paint_record_visual_bounds_time_us` / `paint_record_visual_bounds_calls`
- Added paint-node micro timers in `feat(diag): add paint node breakdown timers` (commit `c512be81`):
  - `paint_cache_key_time_us`
  - `paint_cache_hit_check_time_us`
  - `paint_widget_time_us` (exclusive; pauses while painting children)
  - `paint_observation_record_time_us`
- Added top-N widget paint hotspots in `feat(diag): export paint widget hotspots` (commit `e1132c95`):
  - `debug.paint_widget_hotspots[]` (exclusive time + widget type + scene ops delta)
- On `tools/diag-scripts/ui-gallery-menubar-keyboard-nav-steady.json` (repeat=7), the worst frame shows:
  - `paint_time_us ~2.6ms`,
  - while `paint_cache_replay_time_us` is single-digit microseconds and bounds translation is ~0us.
- With widget hotspots enabled, the same probe shows `paint_widget_time_us` is almost entirely concentrated in a few
  `fret_ui::declarative::host_widget::ElementHostWidget` nodes (top-3 sum ~98% of the widget paint slice), and those
  nodes have `ops(excl/incl)=1/1` (suggesting CPU bookkeeping dominates, not scene encoding).
  - Evidence run: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry 2026-02-05 20:03 (commit `e1132c95`).
- Follow-up A/B attempts to remove/avoid observation vector clones did not materially reduce the hotspots on this
  probe (commits `424ca9fc`, `df5df0b7`).

Implication:

- Stable-frame paint cost is now attributable:
  - the remaining slice is dominated by a small number of host-widget paint calls, not “many tiny widgets”.
- This points at a GPUI gap: stable frames should avoid **per-frame allocations** and should reuse observation edges
  (or at least access them without cloning).
- Next likely win: remove per-frame `Vec` clones in element-runtime observation accessors and re-run the probe.

Tracking:

- Workstream doc: `docs/workstreams/ui-perf-paint-pass-breakdown-v1.md`
- TODO tracker: `docs/workstreams/ui-perf-paint-pass-breakdown-v1-todo.md`

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
  - Non-text upload churn counters are now exported (best-effort):
    `renderer_svg_upload_bytes`, `renderer_svg_uploads`,
    `renderer_image_upload_bytes`, `renderer_image_uploads`.
    - Commits: `d01d3190` + `4bade395` (export) + `dfbc02d3` (deterministic workload + script).
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `dfbc02d3`.
  - SVG raster cache occupancy + churn counters are now exported (best-effort):
    - occupancy: `renderer_svg_raster_budget_bytes`, `renderer_svg_rasters_live`,
      `renderer_svg_mask_atlas_pages_live`, `renderer_svg_mask_atlas_bytes_live`,
      `renderer_svg_mask_atlas_used_px`, `renderer_svg_mask_atlas_capacity_px`
    - churn: `renderer_svg_raster_cache_hits`, `renderer_svg_raster_cache_misses`,
      `renderer_svg_raster_budget_evictions`, `renderer_svg_mask_atlas_page_evictions`,
      `renderer_svg_mask_atlas_entries_evicted`
    - Commits: `6bd82329` + `5f7e4fd0` + `3d1510a7`
    - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry for commit `3d1510a7`.

Next:

- Extend the exported telemetry with additional “GPU churn” and “occupancy” signals:
  - glyph atlas occupancy / live page count (to distinguish “one-time warmup” vs “thrash”),
  - image/texture lifecycle signals beyond upload-bytes (live bytes, evictions, cache hit/miss) for non-text assets
    other than SVG,
  - (optional) GPU timestamp queries for render passes + present/submit time when supported.
- Promote churn into a first-class perf log surface:
  - require `diag perf --json` output to include churn vectors for the top frames,
  - record churn p95/max alongside the CPU breakdown for each perf run (so regressions are explainable).

Acceptance:

- Correlate tail hitches with a specific churn signature (CPU or GPU); fix by stabilizing caches or batching.

### Gap E: Invalidation granularity (hover/layout/paint separation)

GPUI:

- “Cached view” + `notify` semantics provide a clear contract for when layout/paint work can be reused.

Fret:

- We can now *measure* dispatch and hit-test cost (`top_dispatch_time_us`, `top_hit_test_time_us`), but the more
  actionable gap is often **over-invalidation**: pointer-move / hover policies can accidentally trigger layout work
  (or prepaint work) even when the visual change is paint-only.
- Evidence pattern (see perf log on 2026-02-04):
  - `tools/diag-scripts/ui-gallery-hit-test-torture-stripes-move-sweep-via-nav-steady.json` shows p95
    `top_hit_test_time_us` in single-digit microseconds, while `layout_time_us` dominates the frame.

Impact:

- “Hit-testing is fast” but the UI still feels janky because hover invalidations pull in layout and/or prepaint work.

Proposal:

- Make invalidation intent explicit and enforceable:
  - introduce a paint-only invalidation path (or harden the existing one) so hover state flips are paint-only unless
    a layout-affecting style actually changed.
  - classify style changes into **layout-affecting** vs **paint-only** (padding/size/line-height vs color/opacity).
- Add a perf gate targeting “hover should not relayout”:
  - use `tools/diag-scripts/ui-gallery-hover-layout-torture-steady.json`,
  - add a variant where hover is guaranteed paint-only (no size/spacing changes) to isolate dispatch/hit-test cost.

Acceptance:

- Hover/pointer-move torture probes show `p95 layout_time_us ~ 0` (or near-zero) for paint-only hover changes.

### Gap F: Diagnostics harness semantics coupling (measurement distortion)

GPUI:

- Zed/GPUI inspection and testing flows do not require rebuilding a full accessibility semantics snapshot every frame
  to drive pointer-move stress probes.

Fret:

- The UI gallery driver requested a semantics snapshot every frame, even when diagnostics were configured to not
  capture semantics (`FRET_DIAG_SEMANTICS=0`).
- This made “hit-test torture” style scripts unrepresentative: pointer sweeps were dominated by per-frame semantics
  refresh cost, even though the scripted step had already cached its target geometry.

Impact:

- Perf probes intended to isolate hit-test/dispatch cost can end up measuring a heavy unrelated subsystem.

Proposal:

- Gate semantics snapshot refresh to the frames that actually require selector resolution.
  - Implemented by `perf(diag): gate semantics snapshot requests` (commit `470708b2`).
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` (entry for commit `470708b2`).

---

### Gap G: Changed-but-unobserved global churn (dispatch tail)

GPUI:

- “Do nothing if you didn’t `notify`” is a core performance contract: if an interaction does not update relevant state,
  layout/paint reuse stays valid and the runtime does minimal bookkeeping.

Fret:

- In pointer-move heavy probes, diagnostics frequently report the same globals as “changed” on most snapshots while also
  reporting them as **unobserved** (`unobs.globals`), e.g.:
  - `fret_runtime::window_input_context::WindowInputContextService`
  - `fret_runtime::window_command_action_availability::WindowCommandActionAvailabilityService`
- This suggests a “we publish every frame” pattern that adds dispatch bookkeeping and tail latency even when the UI does
  not actually observe the values for the current interaction.
- Evidence:
  - `fretboard diag stats <bundle> --sort dispatch --json` for the hit-test torture sweep bundles (see perf log entry
    for commit `1a9c1238`).

Impact:

- Dispatch tails become harder to bound at 120Hz, even when the frame’s layout/paint work is effectively zero.

Proposal:

- Make these window-scoped services publish changes only on actual value changes:
  - adopt “set-if-changed” semantics (`Eq` / structural equality), or store transient values in a per-frame scratch
    store rather than a globally observed service.
- Add (or tighten) an explicit perf gate for pointer-move dispatch:
  - for the hit-test torture sweep, record per-run max `dispatch_time_us` for frames where `dispatch_events > 0`
    and treat outliers as regressions.

Progress:

- Implemented a first cut that avoids publishing these globals on hover-only pointer moves (commit `d4adf37f`),
  collapsing `snapshots_with_global_changes` to 0 and reducing pointer-move dispatch tails in the stripes sweep
  (see the perf log entry for `d4adf37f`).
- Implemented a dedicated pointer-move gate in `fretboard diag perf`:
  - `--max-pointer-move-dispatch-us`, `--max-pointer-move-hit-test-us`, `--max-pointer-move-global-changes`
  - plus derived pointer-move stats in `tools/perf/perf_log.py`
  - Evidence: perf log entry for commit `6da92d3d`.

Acceptance:

- In hit-test torture sweep probes, `snapshots_with_global_changes` drops materially (or `unobs.globals` becomes rare),
  and per-run max `dispatch_time_us` stabilizes (no outlier spikes) while preserving correctness of cursor/hover/focus.

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
