---
title: UI Performance: Resize Path (Fret vs GPUI) v1
status: draft
date: 2026-02-09
scope: performance, resize, layout, gpui-gap
---

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
