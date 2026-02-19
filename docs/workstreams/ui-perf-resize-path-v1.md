---
title: UI Performance: Resize Path (Fret vs GPUI) v1
status: draft
date: 2026-02-09
scope: performance, resize, layout, gpui-gap
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# UI Performance: Resize Path (Fret vs GPUI) v1

This note exists to stop “resize feels janky” work from turning into untracked experiments.

Goals:

1. Explain what work Fret performs during interactive resize (what is paid per frame).
2. Cross-check the mechanism against GPUI/Zed (what is transferable vs not).
3. Turn the main cost centers into measurable, gated milestones.

Related:

- Workstream: `docs/workstreams/ui-perf-zed-smoothness-v1.md`
- TODO: `docs/workstreams/ui-perf-zed-smoothness-v1-todo.md`
- Perf log (commit-addressable evidence): `docs/workstreams/ui-perf-zed-smoothness-v1-log.md`
- GPUI gap map: `docs/workstreams/ui-perf-gpui-gap-v1.md`

## What happens on resize in Fret (today)

### 1) Runner coalescing (desktop)

Fret coalesces OS resize events and applies a pending surface resize at redraw time (once per frame).

This avoids “N resizes per frame”, but does not eliminate per-frame layout/paint work while the window size is
changing.

Reference:
- `crates/fret-launch/src/runner/desktop/app_handler.rs` (`pending_surface_resize` applied on `RedrawRequested`)

Note:
- Today we coalesce *surface reconfigure* and *bounds used for layout* to once-per-frame, but we still deliver a
  `WindowResized` event each time we apply the pending resize. Unlike GPUI’s `set_frame_size` guard (`old_size == new_size`),
  we do not currently keep an explicit “last delivered (quantized) logical size” to drop no-op resize deliveries.
  - This is likely a small win, but it also reduces “float noise” churn for higher-level code that reads window metrics.

### 2) Layout engine solves are multi-root (window root + viewport roots)

In steady resize probes we observe:

- `top_layout_engine_solves_max` is typically `~4` for the resize drag-jitter probe.

This is expected given Fret’s “explicit layout barrier” design:

- the **window root** is solved once, and
- additional **viewport roots** (scroll/virtualization/docking/contained cache roots) are solved separately with
  their own bounds.

This is not necessarily worse than GPUI; it is a mechanism trade-off. The key is to keep the total work bounded:

- reduce the number of viewport roots solved on a typical editor page, and/or
- make each viewport solve cheaper (especially text measure/shaping).

## Why resize can still hitch even with coalescing

Interactive resize is a *stress multiplier*:

- Bounds change forces layout constraint recomputation.
- Many widgets request viewport roots (scroll content, lists, docking viewports), which increases solve count.
- Text wrap widths churn under small-step jitter, which amplifies measure/shaping costs.

### Do we relayout during live drag?

Yes — and so does GPUI/Zed.

During a live resize drag, the window bounds are changing, so Fret will typically:

- rebuild the layout-engine request/build view of the tree,
- solve layout roots (window root + any active viewport roots),
- run paint (often dominated by text prep under width jitter).

The goal is not to “avoid all layout while resizing” (that usually implies visual lag), but to make live-resize
frames *predictable*:

- amortize text work under width jitter (wrap/shaping reuse),
- keep layout request/build overhead bounded (data structure + traversal discipline),
- keep viewport-root solves bounded (count + solve cost),
- ensure the “small-step” path remains stable when the user drags back-and-forth (avoid toggling policies).

Recent evidence:
- Commit `0de40863f` makes small-step interactive resize detection symmetric; this keeps the same bucketing/caching
  policy enabled on back-and-forth drags and improves the `ui-resize-probes` p95 total by ~0.3ms on the jitter probe.
  - Evidence: `docs/workstreams/ui-perf-zed-smoothness-v1-log.md` entry `2026-02-09 16:37:00`.
- Commit `d834481b3` drops no-op `WindowResized` deliveries (quantized logical size unchanged), mirroring GPUI’s
  `set_frame_size` early-return.
  - This is a “reduce churn/noise” change, not a primary budget win.

To feel “Zed smooth”, we need:

1) stable per-frame work (low tail variance), and
2) a strict budget for *layout request/build* + *solve* + *paint* under resize.

## Observed hotspots (from recent resize probe bundles)

From the resize probe runs recorded in the perf log (see the entries around `2026-02-08`):

- `top_layout_request_build_roots_time_us` can be `~2.4ms` in worst frames for the drag-jitter probe.
  - This is “flow subtree request/build” overhead (walking nodes, setting styles/children, stable identity).
- `top_layout_engine_solve_time_us` can be `~2.2ms` (worst frames) for the drag-jitter probe.
  - This includes `TextService::measure` costs for wrapped text.
- View-cache reuse attribution matters: it is possible for `top_view_cache_roots_total > 0` while
  `top_view_cache_roots_reused == 0` because the observed roots were not marked as reuse roots
  (`top_view_cache_roots_not_marked_reuse_root`), even when there is no cache-key mismatch.
  - This is a key diagnostic for whether “multiple viewport solves” are expected or accidental.

The next step is to reduce the sum of:

- request/build overhead + solve overhead (especially tail outliers),
- without regressing `ui-gallery-steady`.

Local sample (for quick orientation; do not treat as canonical baseline evidence):
- Resize probes gate run (attempts=3): `target/perf-samples/ui-resize-probes.noopdrop.20260209-200004/summary.json`
  - `ui-gallery-window-resize-stress-steady.json` p95 total ≈ `15.7ms` (p95 layout ≈ `9.1ms`, p95 solve ≈ `2.2ms`, p95 paint ≈ `6.6ms`)
  - `ui-gallery-window-resize-drag-jitter-steady.json` p95 total ≈ `16.3ms` (p95 layout ≈ `9.1ms`, p95 solve ≈ `2.2ms`, p95 paint ≈ `7.3ms`)

Tail note:
- A representative `drag-jitter` outlier that breaks the baseline threshold tends to be “paint text prepare (width)”.
  - Example bundle: `target/perf-samples/ui-resize-probes.a86f390f8.20260209-1957/attempt-1/1770638303403-ui-gallery-window-resize-drag-jitter-steady/bundle.json`
  - `fretboard diag stats ... --sort time` shows `paint_text_prepare.reasons=width` dominating the worst frame.

### Finding (2026-02-17): viewport-size branching can defeat ViewCache reuse during resize

The resize probe scripts navigate to the UI-gallery “View Cache Torture” page with:

- `FRET_UI_GALLERY_VIEW_CACHE=1`
- `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`

This wraps the gallery shell (including the content panel) in a declarative `ViewCache` root.

We observed a perf regression where `ui-resize-probes` failed due to high `top_layout_engine_solve_time_us`.
Attribution showed that the content-panel `ViewCache` root had `cache_key_mismatch` on nearly every resize step
because it observed the `viewport_size` environment query with `Invalidation::Layout`.

Concrete repro (macOS M4):

- Gate command: `tools/perf/diag_resize_probes_gate.sh --attempts 1`
- Failing run: `target/fret-diag-resize-probes-gate-1771310400`
  - `top_layout_engine_solve_time_us` max:
    - resize-stress ≈ `5885us` (threshold `3060us`)
    - drag-jitter ≈ `3949us` (threshold `2816us`)

Root cause:

- `apps/fret-ui-gallery/src/ui/content.rs` computed a responsive header mode via:
  - `cx.environment_viewport_bounds(Invalidation::Layout)`
- Because the content view was inside a `ViewCache` root, this observation became part of the view-cache key and
  the key churned on each resize step (viewport size revision changes), preventing reuse.

Fix direction:

- Prefer **layout-driven adaptation** (flex wrap / intrinsic layout) over `viewport_size` branching inside cached
  subtrees when the goal is to keep cached shells reusable during interactive resize.

This specific case was fixed by making the UI-gallery header wrap-friendly and removing the viewport-size query:

- Passing run: `target/fret-diag-resize-probes-gate-1771312171`
  - `top_layout_engine_solve_time_us` max:
    - resize-stress ≈ `1155us`
    - drag-jitter ≈ `1117us`
- Revalidated after rebuilding `target/release/fret-ui-gallery`: `target/fret-diag-resize-probes-gate-1771315079`

Tooling / guardrails:

- Use `fretboard diag stats <bundle.json> --sort time --top 30` and inspect `top_cache_roots`:
  - `reuse_reason=cache_key_mismatch` during resize is often a sign that a cache boundary depends on rapidly-changing
    “external” deps (viewport environment, layout queries).
- `fretboard diag triage <bundle.json> --json` includes perf hints. A new hint code:
  - `view_cache.cache_key_mismatch` (warn): emitted when the worst frame contains view-cache roots that missed reuse
    due to cache key mismatches.
- Turn this into an enforceable suite contract by adding:
  - `--check-perf-hints --check-perf-hints-deny view_cache.cache_key_mismatch`

### Finding (2026-02-17): `ui-gallery-steady` overlay scripts still exceed `top_layout_engine_solve_time_us`

After stabilizing the perf measurement surface (suite prewarm + per-script prelude, plus ensuring scripts do not
leave state behind), the remaining `ui-gallery-steady` baseline failures on macOS M4 are concentrated in
`top_layout_engine_solve_time_us` for a small set of scripts:

- `tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json`
- `tools/diag-scripts/ui-gallery-dialog-escape-focus-restore-steady.json`
- `tools/diag-scripts/ui-gallery-virtual-list-torture-steady.json`

Local repro command (release):

- Update (2026-02-19): the macOS M4 baseline was refreshed to `ui-gallery-steady.macos-m4.v26.json` and the
  steady-suite command now includes the view-cache flags so the measurement surface matches the baseline.
- `target/release/fretboard diag perf ui-gallery-steady --repeat 7 --warmup-frames 5 --reuse-launch --suite-prewarm tools/diag-scripts/tooling-suite-prewarm-fonts.json --suite-prelude tools/diag-scripts/tooling-suite-prelude-reset-diagnostics.json --perf-baseline docs/workstreams/perf-baselines/ui-gallery-steady.macos-m4.v26.json --dir target/fret-diag-perf-local/20260219-ui-gallery-steady --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_RENDERER_PERF=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery`

### Finding (2026-02-18): macOS M4 resize-stress worst frames are paint-dominated

On macOS (Apple M4 / Metal), `ui-gallery-window-resize-stress-steady` does not reproduce the “layout solve explodes”
profile observed on Windows. The worst frames in the script are dominated by paint (and in particular the per-node
element-bounds walk), with `layout.engine_solve` staying around ~1.1ms.

Evidence (single-script perf run, repeat=3):

- Bundle: `target/fret-diag/1771410780171-ui-gallery-window-resize-stress-steady/bundle.json`
- `fretboard diag stats ... --sort time --top 1` (same bundle) reports:
  - worst frame total ≈ `8.9ms`
  - `layout.solve_us ≈ 1159us`
  - `paint.elem_bounds_calls ≈ 2172` (dominant share of paint time)

This suggests the “primary” macOS optimization target for resize-stress is paint-side traversal/caching rather than
layout solving.

### Note (2026-02-18): VirtualList offset/viewport state should reflect the Final pass only

### Finding (2026-02-18): `ScrollDeferredProbe` follow-ups should be barrier-contained (avoid ancestor relayout)

When a `Scroll` element defers its unbounded probe (during resize or while descendants are transiently
layout-invalidated), it schedules a follow-up via `ScrollDeferredProbe`. The previous behavior used a full
`Invalidation::Layout` on the scroll node, which bubbles a relayout to ancestors.

On macOS M4 this showed up as tail spikes in `ui-gallery-steady` overlay scripts (dropdown/dialog), with
`scroll_deferred_probe` appearing in `top_invalidation_walks` but without enough attribution to answer:
“which scroll root is asking for follow-up frames?”

Diagnostics improvement:

- `UiInvalidationWalkV1` now includes `root_element_path` so `fretboard diag stats` can map invalidation walks
  back to stable debug paths (element identity chain).

Concrete attribution (dropdown open/select steady):

- Root: `ecosystem/fret-ui-shadcn/src/scroll_area.rs:306` / `:322` (gallery content scroll area)
- Root: `ecosystem/fret-ui-kit/src/primitives/menu/content_panel.rs:53` (popover/menu panel scroll subtree)

Mechanism fix direction:

- Treat `ScrollDeferredProbe` as **barrier-internal** work: schedule a contained relayout for the scroll barrier
  without forcing ancestor relayout.
- Implemented by adding `UiTree::schedule_barrier_relayout_with_source_and_detail(...)` and using it from the
  scroll layout code path instead of bubbling `Invalidation::Layout`.

Result (macOS M4, release, repeat=3):

- Before: `tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json` could hit
  `top_layout_engine_solve_time_us ≈ 721us` (large outlier solve rooted in the main content stack).
- After: the outlier is removed; remaining max is ~`164us` and is dominated by the popover `RovingFlex` solve.

Remaining gap (still failing `ui-gallery-steady` baseline v25):

- The remaining dropdown failure is not driven by `ScrollDeferredProbe` bubbling anymore.
- Next step is to reduce `RovingFlex` solve cost (or prevent unnecessary **layout** invalidations for menu
  state changes that should be paint-only).

For VirtualList, the element-local `VirtualListState.offset_*` / `viewport_*` values are used as the “last committed”
anchor for render-time range computation and other cross-frame heuristics.

Writing these fields during probe layout passes can:

- hide large-jump detection (probe sees the new offset first),
- make render-time “handle leads state” logic harder to reason about,
- and increase the risk of cross-pass oscillation under intrinsic measurement.

Fix direction: update committed offset/viewport only during `LayoutPassKind::Final`, while still allowing probe passes
to use the latest scroll handle offset for correctness within the pass.

Observed failures (suite12):

- dropdown: `top_layout_engine_solve_time_us` max `199us` (threshold `116us`)
- dialog: `top_layout_engine_solve_time_us` max `180us` (threshold `104us`)
- vlist: `top_layout_engine_solve_time_us` max `1242us` (threshold `988us`)

Evidence bundles:

- dropdown: `target/fret-diag-perf-local/20260217-suite12/1771342690685-ui-gallery-dropdown-apple-steady/bundle.json`
- dialog: `target/fret-diag-perf-local/20260217-suite12/1771342696495-ui-gallery-dialog-escape-steady/bundle.json`
- vlist: `target/fret-diag-perf-local/20260217-suite12/1771342703470-ui-gallery-virtual-list-bottom-steady/bundle.json`

Attribution notes:

- `fretboard diag stats <bundle.json> --sort time --top 60` shows `top_layout_engine_solves` dominated by overlay
  roots (e.g. `DismissibleLayer` / popover content). This is consistent with the “multi-root solves” model, but the
  *sum* of solves for a single frame still exceeds the baseline threshold.
- `top_walks` frequently includes `detail=scroll_deferred_probe` in overlay-heavy scripts, indicating the scroll
  unbounded-probe deferral path is active during steady interaction scripts (not just resize probes).

Next step (directional, not yet implemented):

- Reduce the number of overlay roots that need to be solved in a single frame for “simple overlay interactions”
  (dropdown open/select, dialog escape/restore), and/or make each overlay solve cheaper.
- Use `FRET_LAYOUT_PROFILE=1` (and optionally `FRET_LAYOUT_NODE_PROFILE=1`, `FRET_MEASURE_NODE_PROFILE=1`) on a single
  script repro to capture measure hotspots for the worst frame before attempting mechanism changes.

Node-level attribution snapshot (2026-02-18, macOS M4):

- Repro (single script; note this is for attribution, not baseline numbers):
  - `target/release/fretboard diag perf tools/diag-scripts/ui-gallery-dropdown-open-select-steady.json --repeat 1 --warmup-frames 5 --reuse-launch --env FRET_LAYOUT_NODE_PROFILE=1 --env FRET_LAYOUT_NODE_PROFILE_TOP=20 --env FRET_LAYOUT_NODE_PROFILE_MIN_US=200 --env FRET_MEASURE_NODE_PROFILE=1 --env FRET_MEASURE_NODE_PROFILE_TOP=20 --env FRET_MEASURE_NODE_PROFILE_MIN_US=200 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --launch -- target/release/fret-ui-gallery`
- In the worst frame for the script (frame id `124` in this run), `layout_node profile` points at the popover menu
  scroll subtree (dropdown menu content) as the top self-time layout node, consistent with the “overlay roots dominate
  solve time” model. Evidence: `target/fret-diag-layout-node-profile2-1771399625/fretboard.stdout.json`.

### Finding (2026-02-18): pre-solve barrier relayout roots (avoid widget-local fallback solves)

Some relayout work is intentionally **barrier-contained** and runs as independent solves:

- pending barrier relayout roots (scheduled by scroll/virtualization/etc.), and
- contained `ViewCache` relayout roots (post main viewport roots).

If these roots are solved “late” without a corresponding layout-engine pre-solve, the widget-local flow fallback path
can trigger (log: `layout engine child rects missing; falling back to widget-local solve`). This is bad for tail
latency because it adds additional out-of-band layout-engine passes within the same frame.

Fix direction:

- Pre-solve these roots via the layout engine (`UiTree::solve_barrier_flow_roots_if_needed(...)`) before entering the
  per-root layout pass, so child rects exist and widget-local fallback solves stay rare.
- When reusing cached flow subtrees (translation-only / cache-hit paths), mark “seen” via the **UiTree children**
  (not the layout-engine cached children list) to avoid relying on stale engine-side topology.
- Improve traceability when fallback does happen by logging the layout-engine stamp/seen state for the missing child.

Evidence (macOS M4, release, gate PASS; repeat=7, attempts=3):

- `ui-resize-probes` gate: `target/fret-diag-resize-probes-gate-1771435266/summary.json`
  - worst overall: `target/fret-diag-resize-probes-gate-1771435266/attempt-1/1771435304603-ui-gallery-window-resize-stress-steady/bundle.json`
  - `top_total_time_us` worst overall: `8635us`
- `ui-code-editor-resize-probes` gate: `target/fret-diag-resize-probes-gate-1771435434/summary.json`
  - worst overall: `target/fret-diag-resize-probes-gate-1771435434/attempt-1/1771435447121-ui-gallery-code-editor-window-resize-drag-jitter-steady/bundle.json`
  - `top_total_time_us` worst overall: `6700us`

## GPUI/Zed resize notes (transferable vs not)

GPUI is a strong reference for “Zed feel”, but it is not a complete template for Fret:

Transferable:

- strict “once per frame” work shaping + aggressive reuse contracts,
- allocation discipline (per-frame scratch arenas, bounded caches),
- text layout caching that amortizes width jitter (visible window aware).

Less transferable 1:1:

- Fret’s heavier mechanism set (docking + view cache + multiple viewports) means we may need more explicit barrier
  policies than GPUI.
- Renderer/effects architecture should also be cross-checked against engines like Flutter/Skia for pooled
  intermediates and GPU upload churn.

Local GPUI references:

- `repo-ref/zed/crates/gpui/src/platform/mac/window.rs`
- `repo-ref/zed/crates/gpui/src/platform/linux/wayland/window.rs` (interactive resize throttling)

### What GPUI actually does during interactive resize (high signal)

On macOS (Cocoa / layer-backed view):

- GPUI sets the view’s layer redraw policy to redraw during live resize:
  - `setLayerContentsRedrawPolicy: NSViewLayerContentsRedrawDuringViewResize`
- Size changes flow through `set_frame_size`:
  - updates the renderer drawable size, then calls the registered `resize_callback`.
  - importantly: it early-returns when the new size matches the old size.
- Actual redraw work is still driven by the frame pump:
  - `display_layer` calls `request_frame_callback` (and GPUI also has a display link `step` path).

Implication:
- Zed/GPUI is not “free” during drag-resize — it redraws continuously; it just keeps per-frame work bounded and
  stable via reuse + allocation discipline.

On Wayland (xdg configure floods are real):

- GPUI explicitly throttles interactive resizes to at most once per vblank.
  - When `configure.resizing` is true, it drops additional configure events while `resize_throttle` is set.
  - The throttle is cleared on `wl_surface.frame` (`frame()`), i.e. after the compositor has presented a frame.

This is similar in spirit to Fret’s “coalesce to once per redraw” behavior, but the key insight is:
- **interactive resize is shaped by the frame boundary**, not by “number of OS events”.

## Milestone candidates (make the path measurable)

These are deliberately phrased as “mechanism-first” items that can be gated.

### M4.4.a Resize layout request/build budget

Goal:
- Reduce `top_layout_request_build_roots_time_us` tails during resize probes.

Probe:
- `ui-resize-probes` (drag-jitter + stress)

Likely levers:
- reduce per-frame tree-walk overhead (dense data structures, fewer HashMap lookups),
- avoid work that is invariant under resize (only constraints changed, not styles/structure),
- tighten viewport root batching and avoid redundant work in `flush_viewport_roots_after_root`.

### M4.4.b Viewport roots: reduce solve count or solve cost

Goal:
- Reduce typical `top_layout_engine_solves` and/or reduce `top_layout_engine_solve_time_us` for each solve.

Probe:
- `ui-resize-probes` + editor resize jitter suite.

Likely levers:
- reduce the number of simultaneously active viewport roots on common pages,
- ensure translation-only and cache-hit paths are common under jitter-class resizes,
- continue improving text measure/shaping reuse under width churn.

### M4.4.c Interactive resize policy ADR

Goal:
- Document what is allowed to be bucketed/deferred during live resize (and what must remain exact).

Deliverable:
- ADR update/new ADR + alignment entry.

This prevents “fearless refactors” from silently breaking UX expectations.

### M4.4.d GPU resize budget and churn gates

Goal:
- Ensure “CPU good” isn’t hiding “GPU bad” under resize (120Hz requires both).

Probe:
- Reuse `ui-resize-probes`, but log renderer churn signals alongside CPU totals:
  - `top_renderer_encode_scene_us`, draw-call counts, bind-group switches,
  - text atlas upload/eviction signals (if present in bundles),
  - intermediate pool lifecycle signals (alloc/reuse/evict).

Notes:
- This is an observability milestone first: we should be able to explain a GPU hitch by pointing to a bundle that
  shows upload/eviction churn, not just “time got worse”.
