# GPUI Parity Refactor (Experience + Performance) — Workstream Plan

Status: Draft (design document for alignment)

This document proposes a “fearless refactor” plan to close the **experience** (interaction feel, authoring ergonomics)
and **performance** (CPU frame time stability, cache effectiveness, predictable invalidation) gap between:

- **Fret runtime substrate**: `crates/fret-ui`, `crates/fret-app`, `crates/fret-runtime`, `crates/fret-render`
- **Zed/GPUI reference substrate** (non-normative): `repo-ref/zed/crates/gpui`
- **gpui-component policy/recipes** (non-normative): `repo-ref/gpui-component/crates/ui`

This is a workstream note (not an ADR). Any “hard-to-change” contract changes must be captured as ADRs.

Tracking:

- TODO tracker (keep updated during implementation): `docs/workstreams/gpui-parity-refactor-todo.md`
- Cache roots + cached subtree semantics (ViewCache v1): `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
- GPUI-aligned contract gates (must stay in sync with implementation):
  - Dirty views + notify: `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
  - Interactivity pseudoclasses + structural stability: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`
  - Prepaint + interaction stream range reuse: `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
  - Prepaint-windowed virtual surfaces: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md`

---

## 0. Executive Summary (What We Should Refactor, Not What We Should Add)

The gap you’re feeling is usually not “missing widgets”. It’s missing a **closed loop** between:

1) authoring model (how easy it is to express UI),
2) identity + state (what persists across frames),
3) observation + invalidation (what causes recomputation),
4) caching (what can be replayed safely),
5) introspection (how we debug correctness and performance).

Fret already has many of the building blocks:

- Declarative per-frame element tree + cross-frame element state (ADR 0028) implemented via `GlobalElementId -> NodeId` reuse
  - Fret anchors: `crates/fret-ui/src/declarative/mount.rs:113` (`render_root`), `crates/fret-ui/src/declarative/mount.rs:381` (`mount_element`)
- Element state store + GC lag frames
  - Fret anchors: `crates/fret-ui/src/elements/access.rs:13` (`with_element_state`), `crates/fret-ui/src/elements/runtime.rs:31` (`ElementRuntime`), `crates/fret-ui/src/elements/runtime.rs:48` (`set_gc_lag_frames`)
- Model observation + invalidation propagation (layout/paint record dependencies; model changes invalidate nodes)
  - Fret anchors: `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`), `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)
- Subtree replay caching for paint ops (ADR 0055) + per-window counters
  - Fret anchors: `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`), `crates/fret-ui/src/tree/mod.rs:589` (`set_paint_cache_policy`),
    `crates/fret-ui/src/tree/paint.rs:135` (replay), `crates/fret-ui/src/tree/mod.rs:110` (`UiDebugFrameStats`)

What GPUI adds (and what gives Zed its “feel”) is the *integration polish*:

- A first-class “view caching” authoring pattern (`AnyView::cached`) that reuses recorded prepaint/paint ranges when not dirty
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/view.rs:103` (`AnyView::cached`), `repo-ref/zed/crates/gpui/src/view.rs:216` (`reuse_prepaint`),
    `repo-ref/zed/crates/gpui/src/view.rs:280` (`reuse_paint`)
- A path-based `GlobalElementId` (debuggable identity) built via `Window::with_global_id`
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/window.rs:2049` (`with_global_id`), `repo-ref/zed/crates/gpui/src/window.rs:2871` (`with_element_state`)
- A single, consistent mental model for invalidation: “notify -> dirty views -> reuse ranges unless dirty/refreshing/inspecting”
  - GPUI anchors: `repo-ref/zed/crates/gpui/src/_ownership_and_data_flow.rs` (ownership + observe/notify narrative),
    `repo-ref/zed/crates/gpui/src/subscription.rs` (subscriber mechanics)

This plan focuses on refactoring Fret to gain:

- **View-level caching semantics** (GPUI-style) on top of the existing paint cache (ADR 0055), without bloating `fret-ui` into a kit.
- **Authoring density** improvements in ecosystem (`fret-ui-kit`/`fret-ui-shadcn`) so writing UI is “fluent” like gpui-component.
- **Debuggability** improvements (identity/inspector/perf HUD) so regressions are obvious.

---

## 1. Scope, Constraints, and Principles

### 1.1 Scope (in)

- Refactor experience/perf primitives in:
  - `crates/fret-ui` (runtime substrate, caching, routing, element bridge)
  - `crates/fret-app` / `crates/fret-runtime` (effects scheduling + model store integration)
  - `ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn` (authoring ergonomics + policy surfaces)
- Add targeted instrumentation and acceptance harnesses.

### 1.2 Out of scope (for this workstream)

- Replacing renderer architecture end-to-end (tracked elsewhere).
- New “big features” (complex widgets) unless required as perf/UX harness.
- API stabilization / public crate split changes unless they directly unblock parity.

### 1.3 Hard constraints (“fearless” does not mean “contractless”)

- Keep `crates/fret-ui` mechanism-only (ADR 0066): interaction policy stays in ecosystem.
- Preserve the “build every frame” contract for declarative roots (ADR 0028).
- Preserve ordering semantics (ADR 0002 / ADR 0009).
- Keep portability boundaries (`fret-runtime` effects, no `wgpu` types in UI runtime).

### 1.5 Rebuild vs Retain (GPUI-aligned mental model)

When we say “GPUI parity”, the goal is **not** “everything is rebuilt every frame” and also not “a traditional retained
widget tree”. It is a split:

- **Rebuilt each frame (per dirty view/cache root)**:
  - Declarative element tree (structure, props, style) is rebuilt on demand (dirty views), with explicit cross-frame
    state living outside the tree (`ElementRuntime`).
  - Hover/focus/pressed/capture-derived chrome should be *paint-only by default* (ADR 0181), i.e. it should not
    change the layout tree shape in steady state.
- **Ephemeral per frame (derived during prepaint, not a retained subtree)**:
  - Large “virtual surfaces” should derive their visible window during prepaint and emit per-frame items without
    forcing a rerender of the entire view-cache root for small scroll deltas.
  - Reference pattern: gpui-component’s `VirtualList` derives visible range + consumes `scroll_to_item` in `prepaint`
    and only lays out/prepaints the visible items (`repo-ref/gpui-component/crates/ui/src/virtual_list.rs`).
- **Retained across frames (keyed caches, not implicit state)**:
  - `ElementRuntime` state, view-cache recorded ranges, prepaint interaction caches, layout engine solves, text shaping
    caches, and GPU resources must remain retained and reused behind explicit “dirty + cache key” gates (ADR 0180/0182).

Candidate migrations (v1; performance-first):

- Virtual surfaces (prepaint-driven window + per-frame items):
  - `VirtualList` (`crates/fret-ui`, plus `ecosystem/fret-ui-kit` table/tree/list helpers).
  - Table/tree windows built on top of virtualization primitives (row/section windowing).
  - Text/code/markdown windows (e.g. visible line windows; long documents and code blocks).
  - Canvas/node graph culling windows (visible nodes/edges derived from viewport).
  - Large chart/plot surfaces (data sampling windows derived from viewport/time range).
- Chrome derived from interaction (paint-only; structural stability):
  - Scrollbar visibility/fade, hover toolbars, selection highlights, focus rings, cursor affordances.

The refactor strategy is to introduce a reusable “prepaint-driven ephemeral subtree/items” primitive, migrate
`VirtualList` first (largest multiplier), then apply the same primitive to tables/trees/code views/canvas surfaces.

Alignment checklist (v1; what must be derived per frame / per dirty view):

- View rebuild boundaries:
  - Dirty views/cache roots decide *whether we rebuild*; rebuilding should be the default authoring model, not a special-case.
- Interaction-derived visuals (should not force structural/layout churn):
  - Hover/pressed/focus/capture states (ADR 0181).
  - Scrollbar visibility, opacity fades, thumb geometry.
  - Cursor icon and caret/selection visuals.
- Virtualization windows (should not require rerender on small deltas):
  - Visible-range windows for lists/tables/trees.
  - Visible-line windows for text/code surfaces.
  - Viewport-culling windows for canvas/node graph surfaces.
  - Data-window/sampling windows for charts/plots (avoid rebuilding full series on pan/zoom).

Audit heuristic: if a component’s child set (or a large portion of its render work) depends on scroll offset / viewport
and the current implementation rebuilds that structure in the declarative render pass, it is a prime candidate for
prepaint-driven windows + per-frame ephemeral items.

### 1.5.1 Remaining gaps vs Zed/GPUI (what still blocks the “stable feel” loop)

Even with dirty views + cache roots in place, “stable feel + stable perf” requires closing a few practical gaps:

- **Windowed surfaces beyond VirtualList**: code/text/markdown surfaces and 2D canvas/node graphs must be able to update
  their visible window (scroll/camera) without forcing cache-root rerenders for small deltas (ADR 0190, MVP5).
- **Nested-cache composition**: our v1 cache-root model is subtree-replay-based, which tends to “bubble dirtiness” to
  ancestor cache roots. If we want GPUI-like “local dirtiness, stable outer shells”, we likely need a more composable
  recording boundary (e.g. prepaint/paint range reuse that can tolerate dirty descendants) or better cache-root
  placement guidance in ecosystem surfaces.
- **Cache key precision**: view-cache reuse must remain gated by explicit cache keys, but we should gradually refine the
  recommended key inputs toward GPUI’s `bounds/content_mask/text_style` model (ADR 1152 §7).
- **Explainability under reuse**: diagnostics bundles should make it obvious whether we missed reuse due to dirtiness vs
  key mismatch vs inspection mode, so “why didn’t this update?” questions have a single-run answer.
- **Notify hotspot regression gates**: input-driven `notify()` hotspots should be attributable by callsite and
  budgeted in scripted harnesses, so “this got slower” regressions are caught automatically.
  - Gate: `fretboard diag stats <bundle> --check-notify-hotspot-file-max <file> <max>` (writes `check.notify_hotspots.json`).
  - Default: `fretboard diag suite ui-gallery` applies a `pressable.rs=0` notify-hotspot budget for
    `tools/diag-scripts/ui-gallery-virtual-list-torture.json` unless overridden via CLI.
  - Workstream tracker: `docs/workstreams/gpui-parity-refactor-todo.md` (MVP5-perf-002).

### 1.6 Ecosystem adoption patterns (how we get ROI beyond a single widget)

To avoid a second “big rewrite” later, we should treat the v1 primitive as an ecosystem-facing building block, not as a
one-off VirtualList optimization.

Recommended patterns:

- **`fret-ui` provides the mechanism**: a small contract for prepaint-driven windowing and per-frame ephemeral items
  (e.g. “windowed surfaces”), plus diagnostics hooks to make it explainable.
- **`fret-ui-kit` provides policy + convenience**:
  - a `windowed_list` helper that can back lists, tables, and trees,
  - a `windowed_text_lines` helper for code/text views,
  - a `windowed_canvas_cull` helper for node graphs/canvas scenes.
- **`fret-ui-shadcn` and apps consume the helpers**: demos and policy-heavy components migrate first to validate
  real-world ergonomics and performance.

This keeps `crates/fret-ui` mechanism-only (ADR 0066) while enabling multiple ecosystem crates to benefit from the same
closed-loop caching and invalidation semantics.

Concrete ecosystem entry points (current state):

- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`: a Scroll + Canvas pattern that paints only the
  visible row window while keeping the element tree structurally stable (good for huge simple lists/inspectors).
  - UI Gallery harness: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_WINDOWED_ROWS_SURFACE_TORTURE`) and
    `apps/fret-ui-gallery/src/ui.rs` (`preview_windowed_rows_surface_torture`).
  - Scripted scroll capture: `tools/diag-scripts/ui-gallery-windowed-rows-surface-scroll-refresh.json` (run via
    `cargo run -p fretboard -- diag run ...`).

### 1.6.1 Retained vs. “Rebuilt Each Frame” (GPUI-style hybrid)

It is easy to talk about “retain vs rebuild” as a binary. GPUI (and our target) is a **hybrid**:

- **Retained** (cross-frame, must be reused):
  - stateful view/controller entities and models,
  - text buffers, syntax state, selection/cursor state,
  - layout caches (text shaping, line-breaking, row measurement),
  - GPU resources (atlases, pipelines, bind groups),
  - stable identity paths / element state (ADR 0028 / ADR 1151).
- **Per-frame rebuilt** (ephemeral, derived from retained state + viewport):
  - the declarative element tree for dirty views (ADR 0028),
  - “visible window” item sets for large virtual surfaces (ADR 0190),
  - interaction chrome that should normally be paint-only (ADR 0181): hover/pressed/focus rings, selection highlights,
    caret blink, drag previews, scrollbars.

The goal is not “rebuild everything always”. The goal is: **rebuild only when a view is dirty**, and for “windowed
surfaces”, let small scroll/camera deltas update via **prepaint-driven windows** instead of forcing cache-root rerender.

Current vs target (VirtualList example):

- **Current** (Fret v1): `VirtualList` computes `visible_items` during the declarative render pass
  (`crates/fret-ui/src/elements/cx.rs`), so when the visible window leaves the last rendered overscan window we must
  mark the nearest cache root dirty on the next tick to rebuild the item subtree
  (`crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`).
- **Target** (GPUI-aligned): derive the visible window during `prepaint` (ADR 0190) so that scroll-driven window
  changes can be applied as “ephemeral items” without requiring a full cache-root rerender/relayout for small deltas.

Concrete alignment targets (beyond VirtualList):

- **Windowed 1D surfaces (rows/lines)**:
  - code/text views (`ecosystem/fret-code-view`, editor surfaces): visible line windows,
  - long documents (`ecosystem/fret-markdown`, logs/traces): visible line/row windows,
  - tables/trees/lists (via `ecosystem/fret-ui-kit` helpers): row windows + stable item keys,
  - command/search surfaces (palette, search results, outline): list-of-rows windows under rapid updates.
- **Windowed 2D surfaces (viewport culling)**:
  - node graphs/canvas scenes (`ecosystem/fret-node`, `ecosystem/fret-canvas`): world-space culling windows,
  - gizmos/viewport overlays (`ecosystem/fret-gizmo`, `ecosystem/fret-viewport-tooling`): cull + paint-only chrome.
- **Sampling surfaces**:
  - plots/charts (`ecosystem/fret-chart`, `ecosystem/fret-plot`, `ecosystem/fret-plot3d`, `ecosystem/delinea`):
    pan/zoom adjusts a data window / sampling window without rebuilding full series.

- **Paint-only chrome surfaces** (should avoid rerender by default; ADR 0181):
  - hover/focus/pressed style refinements across common controls,
  - caret blink + selection highlights in text/code views,
  - scrollbars and drag/drop indicators.
  - Harness: UI Gallery `PAGE_CHROME_TORTURE` + `tools/diag-scripts/ui-gallery-chrome-torture.json`.
  - Pattern: prefer pointer-hook `invalidate(Paint)` and schedule frames via `request_animation_frame_paint_only()` when
    only visuals change; reserve `notify()` (or `request_animation_frame()`) for structural/state changes that must rerender.

What should be “per-frame rebuilt” (and where):

- **Declarative render (structural)**: should be as stable as possible; this is where we pay the “rerender/relayout” cost.
  - Allowed: changing subtree shape due to real state/data changes.
  - Avoid: hover/pressed toggles, caret blink, transient chrome.
- **Prepaint (ephemeral windows; ADR 0190)**: compute the visible window from viewport/scroll/camera and emit ephemeral items.
  - Examples: list row windows, code/text visible line windows, markdown long-doc windows, node graph viewport culling windows, plot sampling windows.
  - Goal: small scroll/pan deltas should not force a cache-root rerender when the view is otherwise clean.
- **Paint-only (chrome; ADR 0181)**: pointer-driven visuals that refine style without structural changes.
  - Examples: hover highlight, focus ring, pressed state, selection rectangles, scrollbar fade/hover, drag/drop indicators.
  - Goal: update via paint invalidation + redraw under view-cache reuse (no rerender unless the component explicitly opts in).

Fearless refactor tactic:

1) Keep the *retained* state stable (data revisions, selection, scroll/camera state).
2) Move “what’s visible” derivation into `prepaint` (ADR 0190), and make it explainable via diagnostics bundles.
3) Ensure out-of-band commands (e.g. `scroll_to_item`, `ensure_line_visible`, `zoom_to_fit`) deterministically schedule
   a redraw and invalidate the right cache boundary (ADR 0180 / ADR 0190).
4) Migrate surfaces one-by-one via ecosystem helpers (`fret-ui-kit`), so multiple crates get the same perf/correctness
   loop without duplicating invalidation logic.

Recommended migration order (maximize ROI, minimize churn):

1) **1D line windows** (code/text/markdown): the “editor core” payoff and easiest to validate with scroll harnesses.
2) **Row windows** (lists/inspectors/palettes): broad ecosystem reuse via `fret-ui-kit` helper patterns.
3) **2D culling windows** (node graph/canvas): unlock stable perf under pan/zoom with a single stress harness.
4) **Sampling windows** (charts/plots): separate “sampling math” from rendering and make it explainable in bundles.

### 1.4 Contract gates (ADRs)

This workstream is “fearless” in implementation scope, but not in contract hygiene. Any meaningful shifts to
invalidation/caching/interactivity semantics MUST be expressed as ADRs, and the refactor should be driven by those ADRs
as the source of truth:

- View model and identity:
  - Declarative element tree + externalized state: `docs/adr/0028-declarative-elements-and-element-state.md`
  - Element identity debug paths + frame-staged state: `docs/adr/1151-element-identity-debug-paths-and-frame-staged-element-state.md`
- Observation and invalidation:
  - Model observation + propagation (baseline): `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
  - Dirty views + notify (GPUI-aligned target): `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
- Caching:
  - Paint-stream replay caching: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
  - Cache roots (ViewCache v1): `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
  - View-cache subtree reuse + state retention: `docs/adr/1152-view-cache-subtree-reuse-and-state-retention.md`
  - Prepaint + interaction stream range reuse (GPUI-aligned target): `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
- Interactivity:
  - Pseudoclasses + structural stability (paint-only by default): `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`
- Tooling:
  - Diagnostics snapshot + scripted interaction tests: `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

---

## 2. Target Outcomes (What “Parity” Means)

### 2.1 Experience targets (user-visible)

- Overlays: tooltips/popovers/menus feel stable (no flicker), dismissal and focus restore are consistent.
- Text: basic single-line and multi-line editing behave predictably under IME and high DPI.
- Lists: virtualized lists scroll smoothly, selection/cursor follow user expectations, and hover/press state is stable.
- Docking: cross-window drag and panel keep-alive avoid one-frame “holes”.

### 2.2 Performance targets (developer-visible, measurable)

We should define explicit acceptance thresholds (initial proposal):

- Idle frame CPU cost approaches “near-zero” (no full traversal/paint in steady state unless animating).
- Cache effectiveness is visible and trustworthy:
  - `UiDebugFrameStats.paint_cache_hits/misses/replayed_ops` trend upward on stable scenes (`crates/fret-ui/src/tree/mod.rs:110`).
- Large UI surfaces remain stable:
  - 10k-row virtual list: scrolling does not trigger full relayout of unrelated subtree.
  - Virtualized surfaces do not require “input-driven notify” as a hidden dependency for correctness (e.g. `scroll_to_item`
    must schedule a redraw + invalidate the right cache root deterministically).

### 2.3 Definition of Done (workstream-level)

The refactor is “done enough” when these are true for the stress harnesses in `tools/diag-scripts/`:

- Hover/focus/pressed transitions do not trigger layout invalidation by default (0 layout invalidations attributed to
  pseudoclass edges in the overlay torture and virtual list torture scripts).
- View/cache effectiveness is visible:
  - Stable scenes show high cache root reuse (cache root hits dominate misses) and high paint replayed ops.
- Invalidation is explainable:
  - We can answer “why did this view rerender” and “why did this cache root miss” via diagnostics output.
- Scripted interactions are stable:
  - Running the same script produces the same high-level hit targets and focus path (within platform tolerance).

### 2.4 Metrics we track (minimum set)

The refactor should always move these metrics in the right direction, as reported by the gallery driver and diagnostics:

- Frame work: layout time, paint time, layout engine solves, cache-root relayout counts.
  - Evidence surface: `crates/fret-ui/src/tree/mod.rs`, `apps/fret-ui-gallery/src/driver.rs`
- Caching: paint cache hits/misses/replayed ops, cache root hits and per-root replayed ops.
  - Evidence surface: `crates/fret-ui/src/tree/mod.rs`, `apps/fret-ui-gallery/src/driver.rs`
- Invalidation: count of layout invalidations attributed to pseudoclass edges (MVP1), and dirty view reasons (MVP2).
  - Evidence surface: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`, `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`

### 2.5 Harness recipes (repeatable)

These commands exist to make A/B perf and correctness regressions easy to reproduce locally.

Prefer `diag run` for a fast pass/fail signal, and `diag perf` when you want the per-frame counters in the exported bundle.

When using `diag perf`, inspect these exported debug surfaces to decide which parts should become prepaint-windowed:

- `windows[].snapshots[].debug.dirty_views` (why a view/cache root was marked dirty, including `notify_call` callsites).
- `windows[].snapshots[].debug.cache_roots[].contained_relayout_time_us` (which cache roots dominate layout time).
- `windows[].snapshots[].debug.cache_roots[].reuse_reason` (whether we are missing reuse due to dirtiness vs cache-key gates).

Run a script without view cache (baseline):

```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --warmup-frames 5 --timeout-ms 300000 --poll-ms 200 --launch -- cargo run -p fret-ui-gallery --release
```

Run with view-cache enabled (no shell reuse):

```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --warmup-frames 5 --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --launch -- cargo run -p fret-ui-gallery --release
```

Run with view-cache enabled (shell reuse):

```sh
cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --warmup-frames 5 --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
```

Scroll regression harness (detect stale paint after scroll):

```sh
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json --dir target/fret-diag-sidebar-scroll --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
bundle_dir=$(cat target/fret-diag-sidebar-scroll/latest.txt)
cargo run -p fretboard -- diag stats target/fret-diag-sidebar-scroll/$bundle_dir/bundle.json --check-stale-paint ui-gallery-nav-intro
```

Notes:

- The UI Gallery supports `FRET_UI_GALLERY_VIEW_CACHE{,_SHELL,_INNER,_CONTINUOUS}` (bool parsing) to keep harnesses deterministic.
- When a script step cannot resolve its semantics selector, diagnostics fails fast with `reason=click_no_semantics_match`
  (or `no_semantics_snapshot`) and writes an auto-dumped `script-step-XXXX-...` bundle in `target/fret-diag/`.
- If overlay torture flakes under shell reuse, use `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json` as a stable A/B baseline while debugging.

---

### 2.3 Definition of Done (workstream-level)

The refactor is “done enough” when these are true for the stress harnesses in `tools/diag-scripts/`:

- Hover/focus/pressed transitions do not trigger layout invalidation by default (0 layout invalidations attributed to
  pseudoclass edges in the overlay torture and virtual list torture scripts).
- View/cache effectiveness is visible:
  - Stable scenes show high cache root reuse (cache root hits dominate misses) and high paint replayed ops.
- Invalidation is explainable:
  - We can answer “why did this view rerender” and “why did this cache root miss” via diagnostics output.
- Scripted interactions are stable:
  - Running the same script produces the same high-level hit targets and focus path (within platform tolerance).

### 2.4 Metrics we track (minimum set)

The refactor should always move these metrics in the right direction, as reported by the gallery driver and diagnostics:

- Frame work: layout time, paint time, layout engine solves, cache-root relayout counts.
  - Evidence surface: `crates/fret-ui/src/tree/mod.rs`, `apps/fret-ui-gallery/src/driver.rs`
- Caching: paint cache hits/misses/replayed ops, cache root hits and per-root replayed ops.
  - Evidence surface: `crates/fret-ui/src/tree/mod.rs`, `apps/fret-ui-gallery/src/driver.rs`
- Invalidation: count of layout invalidations attributed to pseudoclass edges (MVP1), and dirty view reasons (MVP2).
  - Evidence surface: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`, `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`

---

## 3. Current-State Snapshot (Fret vs GPUI)

### 3.1 Fret declarative bridge (already GPUI-aligned)

- Element tree is rebuilt each frame for each root:
  - `crates/fret-ui/src/declarative/mount.rs:113` (`render_root`)
- Cross-frame state lives in `ElementRuntime`:
  - `crates/fret-ui/src/elements/access.rs:13` (`with_element_state`)
- Identity mapping avoids a reconcile engine:
  - `crates/fret-ui/src/declarative/mount.rs:381` (`mount_element` reuses `GlobalElementId -> NodeId`)
- Continuous frames are requestAnimationFrame-driven:
  - `crates/fret-runtime/src/effect.rs:161` (`Effect::RequestAnimationFrame`)
  - `crates/fret-ui/src/declarative/mount.rs:229` (push RAF effect when `wants_continuous_frames`)

### 3.2 Fret invalidation + caching (already has the seed, but not author-facing)

- Model observation is recorded during layout/paint, then used to propagate invalidation:
  - `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`)
  - `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)
- Paint cache is range-based and replays ops with translation (ADR 0055):
  - `crates/fret-ui/src/tree/paint.rs:135` (`replay_ops_translated`)
  - `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`)

### 3.3 GPUI’s “missing glue” (what we should emulate)

- Identity is path-based and debuggable:
  - `repo-ref/zed/crates/gpui/src/window.rs:2049` (`with_global_id`)
- Element state is staged across frames and guarded against reentrancy:
  - `repo-ref/zed/crates/gpui/src/window.rs:2871` (`with_element_state`)
- View caching is an explicit authoring pattern (`AnyView::cached`) with robust correctness gates:
  - `repo-ref/zed/crates/gpui/src/view.rs:103` (`cached`)
  - `repo-ref/zed/crates/gpui/src/view.rs:216` (`reuse_prepaint`)
  - `repo-ref/zed/crates/gpui/src/view.rs:227` (`detect_accessed_entities`)
  - `repo-ref/zed/crates/gpui/src/view.rs:280` (`reuse_paint`)

### 3.4 Code map (Fret surfaces vs Zed/GPUI references)

This table is intentionally “relative path first”, so that refactors can be scoped and reviewed precisely.

| Concern | Fret (authoritative) | Zed/GPUI reference (non-normative) |
| --- | --- | --- |
| View/state authoring | `crates/fret-ui/src/element.rs` (`Render`, `RenderOnce`, `IntoElement`) | `repo-ref/zed/crates/gpui/src/element.rs`, `repo-ref/zed/crates/gpui/src/_ownership_and_data_flow.rs` |
| Element identity + state | `crates/fret-ui/src/elements/*` (state store, identity, runtime) | `repo-ref/zed/crates/gpui/src/window.rs` (`with_global_id`, `with_element_state`) |
| Root render + mount reuse | `crates/fret-ui/src/declarative/mount.rs` | `repo-ref/zed/crates/gpui/src/window.rs`, `repo-ref/zed/crates/gpui/src/view.rs` |
| Node tree (bounds/routing) | `crates/fret-ui/src/tree/{mod.rs,dispatch.rs,hit_test.rs}` | `repo-ref/zed/crates/gpui/src/window.rs` (dispatch + hitboxes), `repo-ref/zed/crates/gpui/src/elements/div.rs` (hitbox hover) |
| Layout + caches | `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/declarative/taffy_layout.rs` | `repo-ref/zed/crates/gpui/src/taffy.rs` |
| Paint + range replay | `crates/fret-ui/src/tree/paint.rs` | `repo-ref/zed/crates/gpui/src/view.rs` (`reuse_paint`) |
| ViewCache boundaries | `crates/fret-ui/src/element.rs` (`ViewCacheProps`), `crates/fret-ui/src/tree/mod.rs` | `repo-ref/zed/crates/gpui/src/view.rs` (`AnyView::cached`) |
| Interactivity pseudoclasses | `crates/fret-ui/src/element.rs` (`HoverRegion`, `Opacity`, `InteractivityGate`) | `repo-ref/zed/crates/gpui/src/elements/div.rs` (hover style refinements + notify) |
| Diagnostics + scripts | `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fretboard/src/diag.rs`, `tools/diag-scripts/*` | `repo-ref/zed/crates/gpui/src/inspector.rs` |

---

## 4. Refactor Strategy Overview (MVPs, with Explicit Design Decisions)

### MVP0 — Instrumentation First (1–2 weeks)

Goal: make it impossible to regress “feel” silently.

- Add a “Perf HUD” and/or tracing spans for:
  - layout time, paint time, engine solves, paint cache hit/miss (already in `UiDebugFrameStats`: `crates/fret-ui/src/tree/mod.rs:110`)
  - element-state GC churn (`gc_lag_frames` effects; `crates/fret-ui/src/elements/runtime.rs:48`)
  - model invalidation fan-out (how many nodes invalidated per model change; `crates/fret-ui/src/tree/mod.rs:1269`)
- Add two harness demos:
  1) Overlay torture test (popover + menu + tooltip + focus trap + outside press)
  2) Virtual list torture test (10k+ rows, variable heights, selection + hover + inline text input)

Deliverable: a repeatable “before/after” baseline (numbers, not vibes).

### MVP1 — Pseudoclasses + Structural Stability (2–4 weeks)

Goal: make hover/focus/pressed “cheap by default” (ADR 0181) so stable scenes stay stable:

- Structural stability rule: pseudoclasses MUST NOT change subtree shape.
- Invalidation rule: pseudoclass transitions default to paint-only, but MUST still invalidate paint output when the
  visuals change (i.e. a cache root must not replay stale paint ranges and expect hover/focus rings to appear “for
  free”).
- Add debug-only attribution and enforcement so offenders are obvious (cache-friendly ecosystem hygiene).

### MVP2 — Dirty Views + View-Level Caching (runtime + ecosystem, 3–6 weeks)
Goal: converge on the GPUI mental model: `notify -> dirty views -> reuse ranges unless dirty/refreshing/inspecting`
(ADR 0180 + ADR 1152 + ADR 0055).

This is the highest leverage performance refactor, and it should be implemented before introducing a full prepaint
stream.

#### v1 view identity (recommended)

In v1, Fret defines “View” at the cache boundary granularity:

- `ViewId` == cache root identity (a `ViewCache` root), derived from `GlobalElementId` / `NodeId`.
- `notify()` (no explicit target) marks the **current/nearest cache root** dirty. If no cache root is active, it
  falls back to the window root (coarse, but correct).

Mapping vs Zed/GPUI (v1):

| Concept | Zed/GPUI | Fret (v1, recommended) | Notes |
| --- | --- | --- | --- |
| “View” unit | `EntityId` (view entity) | cache root (`ViewCache` root `GlobalElementId`/`NodeId`) | Cache-root-first to maximize perf impact with minimal surface changes. |
| `notify()` default target | current view | current/nearest cache root | Fallback: window root when no cache root is active. |
| Dirty propagation | mark ancestors dirty | mark ancestor cache roots dirty | Required for nested boundary correctness (no stale replay). |
| Cache reuse predicate | `!dirty && key matches && !refreshing && !inspecting` | same | Key includes size/scale/theme and any future fields (ADR 0055). |
| Inspector/picking | disables caching | disables caching | Tooling correctness > cache hits. |

This is intentionally “cache-root-first” to maximize performance impact with minimal surface area changes, while keeping
the door open for a future “entity-first” view model (GPUI-like) in a breaking-change window.

#### Dirty propagation rule (must match nested cache-root correctness)

Marking a view dirty MUST also mark ancestor cache roots dirty when nested boundaries exist, so an ancestor never reuses
a paint range that includes stale descendant output (this mirrors GPUI’s “mark ancestor views dirty” behavior).

We implement a “cached subtree” primitive that:

- is keyed by stable identity (`GlobalElementId` / `NodeId`),
- captures **cache keys** (bounds/scale/theme) plus **dependency keys** (observed models/globals),
- reuses recorded ranges (ADR 0055) and reuses observation sets on hit,
- is automatically disabled in inspection/picking modes.

Implementation note (current state):

- The declarative element GC now treats view-cache reuse as a performance optimization only: cache-hit frames MUST NOT
  change lifetime semantics (ADR 0191).
- The previous global stopgap (“skip sweep while any reuse roots exist”) has been removed as part of MVP2-cache-005 by
  making liveness explicit under reuse:
  - reachability roots include installed layer roots + view-cache reuse roots,
  - cache-hit frames refresh liveness via per-root subtree membership lists,
  - diagnostics attribute structural detaches (e.g. `set_children`) so “why was this swept?” is explainable from a
    single failing bundle.

Proposed ecosystem-facing API surface (runtime internal may differ):

- `cx.cached(cache_key, |cx| -> Vec<AnyElement>) -> Vec<AnyElement>`
  - where `cache_key` is an explicit, typed key for “layout-affecting inputs”.

### MVP3 — “Prepaint” + Multi-stream Frame Recording (optional but recommended, 4–8 weeks)

Goal: converge toward GPUI’s “request_layout / prepaint / paint” separation so that:

- interaction/semantics streams can be constructed deterministically,
- caching can reuse *more than paint ops*.

This aligns directly with ADR 0055’s “multi-stream” direction.

Incremental on-ramp (semantics-preserving, before full prepaint):

- Introduce cached hit-test path reuse for pointer-move routing, with conservative fallbacks to full hit-testing.
- This is a deliberately small “interaction range reuse” step that exercises the correctness constraints we will need
  once interaction output becomes replayable per cache root.
  - Evidence: `crates/fret-ui/src/tree/hit_test.rs`, `crates/fret-ui/src/tree/dispatch.rs`.
- Add a minimal prepaint pass that records an interaction stream and supports cache-root range reuse (v0).
  - Evidence: `crates/fret-ui/src/tree/prepaint.rs`, `crates/fret-ui/src/tree/layout.rs`.

### MVP4 — Authoring Density + Adoption (ecosystem, ongoing)

Goal: make the new contracts the “default obvious” way to build UI while keeping `crates/fret-ui` mechanism-only.

- Introduce/extend fluent authoring helpers in `ecosystem/fret-ui-kit` (typed “styled density”).
  - Reference: `repo-ref/gpui-component/crates/ui/src/styled.rs`
- Add recipe guidance for cache-root placement (panel granularity; avoid micro-boundaries).
- Add a thin ecosystem helper for cached subtrees so authors make cache-key inputs explicit without leaking policy.

### Parallel track — Text System & Editor-Grade Inputs

Goal: close the biggest “editor feel” gap after perf substrate is stable (IME, caret geometry, font stacks).

- Implement font stack bootstrap + stable key propagation (ADR 0162).
- Make TextInput/TextArea integrate tightly with:
  - `TextFontStackKey` changes,
  - theme revisions,
  - cursor rect scheduling,
  - selection/caret stability under caching.

---

## 5. Module-by-Module Refactor Plan

### 5.1 Authoring & Composition (ecosystem)

#### Problem

Fret’s declarative authoring currently pushes developers toward “construct enums/props” directly.
This is correct but not *dense*. The resulting UI code is harder to scan and harder to keep consistent.

#### Reference

- gpui-component “styled density”: `repo-ref/gpui-component/crates/ui/src/styled.rs`

#### Proposal

In `ecosystem/fret-ui-kit`:

1) Add a fluent “style patch” surface:
   - Methods mirror Tailwind-ish semantics but remain typed.
   - Output is a `LayoutStyle` + “chrome style” patch.
2) Add “recipes” in `ecosystem/fret-ui-shadcn` that map shadcn taxonomy to these patches.

Key rule: `crates/fret-ui` remains mechanism-only.

#### Acceptance

- The demo UI code for a representative screen reduces ~30–50% line count.
- Styling is consistent (one source of truth for spacing/density).

#### Open questions

- Do we want proc-macros for derive/DSL, or keep everything as plain Rust methods?

---

### 5.2 Identity, Debuggability, and Inspector UX (runtime + ecosystem)

#### Problem

Fret’s `GlobalElementId(u64)` is great for performance and portability, but weak for:

- debugging (“what is this id?”),
- inspector “navigate-to-source”,
- parity with GPUI’s readable path ids.

#### Reference

- GPUI path ids: `repo-ref/zed/crates/gpui/src/window.rs:2049` (`with_global_id`)

#### Proposal (non-breaking)

1) Keep `GlobalElementId(u64)` as the stable runtime key.
2) Add an *optional* debug registry (feature-gated, e.g. `diagnostics`) that records:
   - callsite location (`Location::caller()`),
   - keyed hash inputs,
   - parent chain (a human-readable path for inspector only).
3) Extend `WindowElementDiagnosticsSnapshot` (already behind feature) to include:
   - focused/hovered elements (exists),
   - element id debug strings,
   - last bounds/visual bounds in a readable form.

#### Acceptance

- Given a hovered/focused element, the inspector can show a stable, human-readable id path.
- “unkeyed list reorder” warnings point to source callsite.

---

### 5.3 Observation + Invalidation: Make It “Closed Loop” (runtime)

#### Problem

Fret’s observation model is already robust, but it’s distributed across:

- element runtime’s “observed models/globals per root” (for declarative authoring),
- UiTree’s “observed_in_layout/paint” (for retained widget runtime).

This makes it harder to build “view caching” with a single story.

#### Anchors

- Element observation: `crates/fret-ui/src/elements/cx.rs:304` (`observe_model_id`)
- Invalidation propagation: `crates/fret-ui/src/tree/mod.rs:1269` (`propagate_model_changes`)

#### Proposal

Introduce a unified “dependency token” concept that both pipelines can speak:

- `DependencySet = { observed_models, observed_globals }`
- `DependencyRevision = hash(DependencySet + relevant key revisions)`

Then implement:

1) For retained widgets: dependency set is produced per node during layout/paint (already exists implicitly).
2) For declarative elements: dependency set is produced per root (already exists in `WindowElementState.observed_models`).
3) Standardize how caching consumes dependency sets:
   - Cache hit must retain the previous dependency set for the subtree, not drop it.

This mirrors GPUI’s “detect_accessed_entities” for cached views:

- `repo-ref/zed/crates/gpui/src/view.rs:227` (`detect_accessed_entities`)

#### Acceptance

- Cached subtrees continue to invalidate correctly on model changes (no “stale UI”).

---

### 5.4 View-Level Caching (runtime + ecosystem)

#### Problem

We have internal paint-cache (ADR 0055), but we lack an authoring-facing, composition-friendly caching boundary,
equivalent to GPUI’s `AnyView::cached` (`repo-ref/zed/crates/gpui/src/view.rs:103`).

Node-level caching is necessary but not sufficient for editor-grade UI:

- Large editor UIs benefit from caching *semantic subtrees* (view/panel level), not just widget nodes.
- The author needs a simple, intentional way to say “this subtree is expensive; cache it unless dependencies change”.

#### Current anchors

- Paint replay: `crates/fret-ui/src/tree/paint.rs:135`
- Cache key policy: `crates/fret-ui/src/tree/mod.rs:589` (`set_paint_cache_policy`)
- Cache ingestion: `crates/fret-ui/src/tree/mod.rs:632` (`ingest_paint_cache_source`)

#### Proposal A (recommended): `CachedSubtree` element (declarative)

Add a new declarative element kind, conceptually:

- `ElementKind::Cached(CachedProps { key: u64, policy: CachePolicy, ... })`

Behavior:

1) During render, the author wraps expensive content:
   - `cx.cached(key_inputs, |cx| children)`
2) The runtime creates/uses a dedicated `NodeId` boundary for the cached subtree root.
3) Cache key includes:
   - v1: `hash(theme_revision, scale_factor, cache_root_bounds.size, explicit_cache_key)` (implemented as `ViewCacheProps.cache_key`; currently width/height only)
   - v2+: extend toward GPUI’s `bounds/content_mask/text_style` key as those inputs become explicit at the cache boundary.
   - Helpers: `fret_ui::cache_key::{CacheKeyBuilder, text_style_key, rect_key, corners_key}` (ecosystem sugar: `CachedSubtreeProps::{cache_key_text_style, cache_key_clip_rect, cache_key_clip_rrect}`).
4) Dependency sets:
   - track observed models/globals for that subtree (unify with §5.3)
5) Inspection/picking disables caching:
   - consistent with `PaintCachePolicy::Auto` and GPUI inspector behavior.

Why declarative-first?

- Because it’s the closest analog to `AnyView::cached`, and it composes with your long-term authoring direction (ADR 0028).

#### Proposal B: Widget-only “cache boundary” (retained)

Alternative is to introduce a retained widget that acts as a cache boundary.
This can work, but it’s less aligned with the long-term declarative model and risks duplicating authoring patterns.

#### Acceptance

- In the “virtual list torture test”, non-visible panels remain cached while list scrolls.
- Hovering tooltips/menus does not blow away unrelated cached panels.

#### Open questions

- Should the cache boundary be opt-in only, or should we provide a default heuristic (like `PaintCachePolicy::Auto`)?

---

### 5.5 Event Dispatch, Default Prevention, and Action Availability (runtime)

#### Problem

Editor-grade “feel” depends on subtle consistency:

- capture vs bubble phase semantics,
- ability to prevent default focus changes on pointer down,
- action availability queries along the dispatch path (used by menus/palette/shortcuts).

GPUI has explicit dispatch phases:

- `DispatchPhase::Capture/Bubble` (`repo-ref/zed/crates/gpui/src/window.rs`).

Fret has capture and bubbling, but lacks a unified “default prevention” + action-availability contract.

#### Proposal

1) Formalize dispatch phases in `fret-ui`:
   - capture pass for “state cleanup” and outside-press observers,
   - bubble pass for normal handlers.
2) Add `prevent_default()` semantics for pointer down:
   - specifically to stop implicit focus shifts or parent activation.
3) Introduce “action availability” queries:
   - integrate with the command system in `fret-app`,
   - mirror gpui’s “is action available along dispatch path” mental model.

#### Acceptance

- Overlays no longer cause accidental focus steals.
- Keyboard shortcuts respect focus scopes consistently across windows and overlays.

---

### 5.6 Overlays, Dismissal, and Focus Restore (runtime substrate; policy in ecosystem)

#### Problem

Fret’s layering model is strong (multi-root overlays), but experience gaps appear when:

- focus needs to be restored predictably after dismissal,
- outside press dismissal interacts with docking drags and viewport capture,
- “initial focus before layout” is needed.

#### Proposal

1) Make overlay lifecycle hooks first-class:
   - `on_open`: capture focus snapshot + install focus trap policy (ecosystem)
   - `on_close`: restore focus or redirect focus deterministically
2) Ensure overlay anchor geometry uses *visual bounds*:
   - use `visual_bounds_for_element` for render-transform aware anchoring.
3) Provide a “policy harness” suite in ecosystem:
   - Radix-aligned dismissal/focus outcomes regression tests.

#### Acceptance

- Popover/menu focus behavior matches the reference stack (`docs/reference-stack-ui-behavior.md`).

---

### 5.7 Text System (runtime + renderer + platform)

#### Problem

Text is the most visible editor-grade subsystem. Missing font bootstrap + incomplete IME pipeline will dominate perceived gap.

#### References

- GPUI text system: `repo-ref/zed/crates/gpui/src/text_system.rs`
- Fret scheduling and cursor area effects:
  - `crates/fret-runtime/src/effect.rs:15` (`Effect`)

#### Proposal

Split into two refactor tracks:

1) **Font stack bootstrap**
   - Implement ADR 0162 (stable font stack key propagation).
   - Guarantee that changing font stack triggers relayout via `TextFontStackKey` dependency.
2) **IME + caret geometry correctness**
   - Ensure cursor rect effect is updated precisely when caret moves (including during preedit).
   - Add acceptance tests that replay IME sequences (even if partially mocked).

#### Acceptance

- IME acceptance checklist passes for representative cases.
- No “cursor jumping” under caching + high DPI.

---

### 5.8 Virtualization & Large Collections (runtime + ecosystem)

#### Problem

Virtualization must compose with:

- selection models,
- keyboard navigation,
- active-descendant semantics (cmdk-like),
- accessibility collection semantics (future).

gpui-component offers a composable range-driven API:

- `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

Fret already has `virtualizer`-backed metrics:

- `crates/fret-ui/src/virtual_list.rs`

#### Proposal

1) Standardize a “virtual list row recipe” in ecosystem:
   - keyed rows by stable item key,
   - per-row hover/press/focus behaviors implemented in policy layer.
2) Ensure virtualization integrates with caching boundaries:
   - the list itself is “hot”; surrounding panels should remain cached.
3) Add a “table/tree” scaffolding doc and demo:
   - make the “rich row” pattern canonical.

#### Acceptance

- 10k-row list scroll stays smooth and does not repaint unrelated panels.

---

### 5.9 Renderer/Scene: Ordering, Batching, and Recording Fingerprints (renderer + runtime)

#### Problem

Even with UI caching, we can lose perf if we still:

- re-encode identical scenes,
- re-upload unchanged resources,
- break batching due to overly fine-grained ops.

ADR 0055 already mentions renderer-side encoding reuse by scene fingerprint.

#### Proposal

1) Make `SceneRecording::fingerprint` a first-class debug metric (HUD/tracing).
2) Provide per-pass stats:
   - ops count by kind, clip/push/pop counts, text blobs, images.
3) Tie cache boundaries to renderer reuse:
   - if UI paint cache hits, renderer fingerprint should remain stable (when no external surfaces change).

#### Acceptance

- Stable UI produces stable fingerprints across frames.

---

## 6. Key Design Decisions to Confirm (Choose Defaults)

### D1 — Where does “View caching” live?

Options:

1) Runtime primitive in `crates/fret-ui` (recommended): because caching must be deterministic and contract-level.
2) Ecosystem-only wrapper: easier initially, but risks “caching that breaks invalidation” due to lack of tight integration.

Recommendation: (1), but with a small surface area and policy-free semantics.

### D2 — Do we introduce an explicit `prepaint` phase?

Options:

1) Keep current (layout + paint) and extend paint cache only.
2) Add `prepaint` to build future interaction/semantics streams and enable broader reuse (GPUI-like).

Recommendation: start with (1) for MVP2, but design MVP2 APIs so MVP3 can add prepaint without breaking.

### D3 — Identity: keep `GlobalElementId(u64)` or move to path ids?

Recommendation: keep `u64` for runtime, add debug registry for readability (feature gated).

### D4 — View identity granularity (v1)

Options:

1) Cache-root-first (recommended for MVP2): define `ViewId` at cache boundary granularity (a `ViewCache` root), and
   make `notify()` default to "mark nearest cache root dirty".
2) Entity-first (GPUI-like): introduce explicit long-lived view entities as the primary `ViewId`, and treat cache
   roots as an optimization detail that a view may opt into.

Recommendation: start with (1) to maximize performance impact with minimal surface-area change, and keep (2) as a
breaking-change corridor once the substrate and acceptance harnesses are stable.

ADR impact:

- Dirty views + notify: `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
- Cache roots + nested invalidation correctness: `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`

### D5 — Redraw scheduling: immediate vs coalesced per tick

Options:

1) Request redraw on every `notify`/invalidation call (simple, but noisy and hard to attribute).
2) Coalesce redraw per window per tick (recommended): schedule a single redraw at the driver boundary and aggregate
   reasons (notify/model/layout/inspection).

Recommendation: (2), because it is required for predictable "near-zero idle" behavior and for trustworthy
diagnostics ("why did we redraw?").

### Suggested defaults (based on current repo maturity)

These defaults are optimized for “ship demos fast while raising the performance/feel ceiling”:

1) **Primary feel target**: pick **E (overall perf/jank)** as the mainline, and use **A (overlays)** as the first acceptance harness.
   - Rationale: Zed-like “smoothness” is mostly “default idle is cheap” + “cache is trustworthy”.
   - Exception: if the current north star is “code editor-grade authoring & IME”, then bring **B (text/IME)** ahead of A, but still keep E as the substrate work.
2) **API break budget**: do **ecosystem-first** authoring refactors; allow only **additive runtime** changes (no breaking public surface) until parity baselines are stable.
3) **Caching semantics**: implement view-level caching as **explicit opt-in** (`cx.cached(...)`) first; keep runtime node-level caching as-is (`PaintCachePolicy::Auto`).
4) **Inspector**: start with **HUD + debug picking + tracing/logging**, and postpone a full inspector UI until the caching/dispatch semantics are stable.

---

## 7. Migration Plan (Concrete, “No Big Bang”)

### Step A — Baseline harness

- Add two demos (overlay + virtual list) and record baseline stats.

### Step B — Ecosystem authoring

- Land fluent styling helpers in ecosystem and migrate the demos.

### Step C — Cached subtree primitive

- Implement `CachedSubtree` element kind and wire it to existing paint cache and dependency sets.
- Add regression tests:
  - cached subtree invalidates when a dependent model changes,
  - cached subtree does not repaint when unrelated models change,
  - caching disabled under inspection/picking.

### Step D — Prepaint/multi-stream (optional)

- Add prepaint pass behind a feature flag.

### Step E — Text bootstrap

- Implement ADR 0162 and re-run text/IME acceptance checks.

---

## 8. Questions for You (to lock the direction before coding)

1) **Primary “feel” target**: which one hurts most today?
   - A) overlays (dismiss/focus/portal), B) text/IME, C) lists/tables, D) docking/multi-window drag, E) overall perf/jank
2) **API break budget**: are we allowed to introduce a new authoring surface in ecosystem and migrate demos first, before touching runtime APIs?
3) **Caching semantics**: do you prefer explicit author opt-in (`cx.cached(...)`) only, or also an auto policy (like `PaintCachePolicy::Auto`)?
4) **Inspector story**: do you want a built-in inspector UI (like GPUI), or is “log + HUD + debug picking” enough for now?

---

## 9. Future “Big Break” Refactor Corridors (when breaking changes are allowed)

### Direction decision (no interfaces locked)

When a breaking-change window is available, the repository’s **v2 refactor north star** is:

- **Corridor A**: GPUI-aligned runtime pipeline — **view-level caching + explicit three-phase pipeline**
  (`request_layout` / `prepaint` / `paint`) + ADR 0055-style **multi-stream frame recording**.

This is a direction only. It intentionally does **not** lock concrete APIs or data structures.

This section exists specifically to avoid being “ADR-locked” into an implementation shape that later blocks
a necessary performance/experience redesign.

The idea is to keep ADRs locking **invariants/outcomes**, while reserving explicit “corridors” for large, planned
replacement refactors (v2/v3) without rewriting the whole ecosystem.

### 9.1 Triggers (when we should consider a big break)

We should treat these as objective signals that incremental refactors are no longer cost-effective:

- Sustained perf ceiling issues (e.g., editor-scale UIs cannot remain smooth even with caching boundaries and tuned invalidation).
- Architectural impedance mismatch (e.g., retained `UiTree` constraints block a GPUI-style `prepaint/paint` multi-stream model).
- Text pipeline constraints (font stack, shaping, IME) require deep changes across renderer/runtime/platform boundaries.
- Debuggability debt becomes a velocity killer (can’t attribute jank or correctness regressions reliably).

### 9.2 ADR evolution policy (how not to get locked)

We should treat ADRs as “contracts with evolution lanes”, not as “forever implementation”.

Recommended policy:

1) **Lock outcomes, not data structures**
   - Example: lock “ordering semantics” (ADR 0002/0009), not “how ops are stored”.
2) **Version hard-to-change contracts**
   - Prefer “v1/v2” explicitly in contract names and types (e.g., `*V1` structs already exist in the repo for keymap).
   - Allow ADRs to be superseded with a new ADR that states migration rules.
3) **Reserve escape hatches**
   - Keep IDs opaque (`GlobalElementId`, `TextBlobId`, etc.) so representations can change.
   - Keep effect boundaries data-driven (`Effect`) so scheduler/work loop can evolve.
4) **Ship experimental lanes behind feature flags**
   - Prototype the “next pipeline” in parallel, then migrate demos, then flip defaults.

This keeps the repo “decision-driven” without making early ADRs a permanent ceiling.

### 9.3 Corridor A — “UI Runtime v2”: unify around frame recording + view caching

This corridor keeps the current crate layering, but changes the runtime pipeline shape when breaks are allowed:

1) Introduce an explicit **three-phase pipeline**:
   - `request_layout` → `prepaint` → `paint`
   - Aligns with GPUI’s mental model and makes multi-stream recording natural
     - Reference: `repo-ref/zed/crates/gpui/src/element.rs` (request_layout/prepaint/paint pattern)
2) Promote ADR 0055’s “multi-stream frame recording” from conceptual to real:
   - Paint stream (already): `SceneOp`
   - Add interaction stream (hit regions/cursors/tab stops), and eventually semantics stream
3) Make **view-level caching** a first-class substrate mechanism (not just a node optimization):
   - Align with GPUI’s `AnyView::cached` semantics
     - Reference anchors: `repo-ref/zed/crates/gpui/src/view.rs:103` (cached), `repo-ref/zed/crates/gpui/src/view.rs:216` (reuse_prepaint), `repo-ref/zed/crates/gpui/src/view.rs:280` (reuse_paint)
4) Unify dependency tracking into a single “closed loop”:
   - “what was accessed” → “what is dirty” → “what ranges are replayable”

Compatibility strategy:

- Keep `crates/fret-ui` mechanism-only.
- Keep ecosystem policy/recipes stable by providing adapters/shims for a transition period.

### 9.4 Corridor B — “Authoring model v2”: first-class View/Entity composition (GPUI-like)

This corridor is about authoring ergonomics and caching boundaries becoming a primary unit of composition.

1) Introduce an explicit “view entity” authoring layer (ecosystem or new crate), providing:
   - view identity, caching, and dependency observation as a cohesive unit
2) Provide a migration path from today’s `ElementContext` composition:
   - Existing `AnyElement` trees remain valid; views become a wrapper that can host element trees.

Risk/benefit:

- Higher short-term migration cost, but the strongest parity with Zed’s “write UI like Rust” + “cache like views”.

### 9.5 Corridor C — “Identity v2”: debuggable path ids without paying the runtime cost

If/when we decide that `GlobalElementId(u64)` blocks tooling, we can move to a GPUI-like path id model while keeping
runtime performance by splitting identity into:

- a stable opaque key for runtime fast paths, plus
- a debug path representation only when diagnostics are enabled.

Reference:

- GPUI `with_global_id`: `repo-ref/zed/crates/gpui/src/window.rs:2049`

### 9.6 Practical rule: don’t block v2 with v1 decisions

When implementing MVP0–MVP2 work, enforce the following:

- Any new caching/dispatch APIs must be designed so MVP3 (prepaint/multi-stream) can be added without breaking semantics.
- Any new “helper” in ecosystem should be a thin layer over stable primitives, not a pile of bespoke runtime hooks.
- Any new “locked contract” should include a short “future v2 note”: what is invariant vs what may change.
