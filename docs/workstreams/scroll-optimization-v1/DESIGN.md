# Scroll Optimization Workstream (v1)

Date: 2026-03-03  
Status: Draft (planning + gates-first)

## Motivation

Scrolling is a core interaction that touches multiple mechanisms:

- layout barriers (virtualization, retained hosts, view-cache),
- scroll handle revision + binding invalidation,
- hit-testing / paint transforms,
- content extent measurement (probes vs observed overflow).

The goal of this workstream is to improve correctness, stability, and performance of scrolling
without violating Fret’s layering rules (mechanism in `fret-ui`, policy in `ecosystem/*`).

## Goals

- Correctness
  - No scroll-state drift across frames (especially under view-cache reuse).
  - No “pinned scroll range” when content grows near the scroll extent edge.
  - Nested scrollables behave predictably (deepest scrollable consumes wheel first).
- Performance
  - Reduce redundant invalidation/redraw work under high-frequency wheel/trackpad input.
  - Avoid expensive unbounded extent probes on steady-state frames.
  - Keep virtual list scrolling on HitTestOnly in common cases.
- Maintainability
  - Reduce foot-guns around barrier relayout + invalidation accounting.
  - Keep diagnostics evidence and regression gates small and repeatable.

## Non-goals

- Changing the public “policy” behavior (hover intent, default paddings, component sizing).
- Adding new scroll physics (inertia curves, overscroll behaviors) unless required by correctness.
- Replacing the virtual list implementation strategy.

## Current architecture (Fret)

Key mechanism pieces:

- Scroll handle and revision: `crates/fret-ui/src/scroll/mod.rs`
  - External setters bump `revision`; internal setters used during layout do not.
- Scroll layout + extent probing: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - Supports `probe_unbounded` plus the authoritative post-layout overflow observation path
    used to derive scroll extents without relying on legacy probe-first behavior.
- Scroll / vlist events: `crates/fret-ui/src/declarative/host_widget/event/scroll.rs`
  - Wheel is handled by the deepest scrollable first (capture returns early for Wheel).
  - Scroll offset changes are treated as `HitTestOnly` invalidations (fast path).
- Scroll-handle binding registry and change classification:
  - `crates/fret-ui/src/declarative/frame.rs`
  - Applied in `crates/fret-ui/src/tree/layout/state.rs` (including `windowed_paint` view-cache rules).

## Evidence (existing gates)

Diagnostics scripts:

- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-wheel-scroll.json`
  - Asserts semantics scroll max is finite and wheel moves offset for both axes.
  - Self-contained navigation via `ui-gallery-nav-search` -> `ui-gallery-nav-scroll-area`.
  - Suite: `tools/diag-scripts/suites/ui-gallery-scroll-area/` (run: `cargo run -p fretboard -- diag suite ui-gallery-scroll-area --launch -- cargo run -p fret-ui-gallery --release`).
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`
  - Starts a scrollbar thumb drag, triggers content growth mid-drag, and asserts scrollbar semantics `y` stays stable.
  - Harness diagnostics module (keep `test_id`s stable): `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/drag_baseline.rs`.
  - Suite: `tools/diag-scripts/suites/ui-gallery-scroll-area/`.
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-wheel-torture.json`
  - Repeated wheel input for perf/robustness evidence (captures a bundle; no perf threshold gate yet).
  - Suite: `tools/diag-scripts/suites/perf-ui-gallery-scroll-area/`.
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-wheel-burst-coalescing.json`
  - Runner-level wheel burst injection to exercise frame-boundary coalescing.
  - Gate: `check.wheel_events_max_per_frame.json` enforces `pointer.wheel` events-per-frame ≤ 1.
  - Suite: `tools/diag-scripts/suites/diag-hardening-smoke/`.
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-nested-scroll-routing.json`
  - Nested scroll routing: an inner horizontal scroll surface must not consume vertical wheel input.
  - Harness snippet (keep `test_id`s stable): `apps/fret-ui-gallery/src/ui/snippets/scroll_area/nested_scroll_routing.rs`.
  - Suite: `tools/diag-scripts/suites/ui-gallery-scroll-area/`.
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-toggle-code-tabs.json`
  - Toggles doc section `Preview`/`Code` tabs for the scroll-area page (smoke coverage for subtree bookkeeping).
  - Suite: `tools/diag-scripts/suites/ui-gallery-scroll-area/`.
- `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json`
  - Expands content while already at the scroll extent edge (pinned extents regression coverage).
  - Harness diagnostics module: `apps/fret-ui-gallery/src/ui/diagnostics/scroll_area/expand_at_bottom.rs`.
  - Suite: `tools/diag-scripts/suites/diag-hardening-smoke/`.
- `tools/diag-scripts/ui-gallery/virtual-list/ui-gallery-virtual-list-wheel-torture.json`
  - Repeated wheel input against the VirtualList torture harness (captures a bundle for perf attribution).
  - Suites:
    - `tools/diag-scripts/suites/ui-gallery-virtual-list/`
    - `tools/diag-scripts/suites/perf-ui-gallery-virtual-list/`

Unit / integration tests (non-exhaustive):

- Wheel invalidation is HitTestOnly:
  - `crates/fret-ui/src/tree/tests/scroll_invalidation.rs`
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`
- Deferred scroll-to-item consumption:
  - `crates/fret-ui/src/declarative/tests/virtual_list/scroll_to_item.rs`
- Scroll-into-view correctness:
  - `crates/fret-ui/src/tree/tests/scroll_into_view.rs`
- Occlusion vs scroll forwarding:
  - `crates/fret-ui/src/tree/tests/pointer_occlusion.rs`
- Scrollbar thumb drag baseline stability (content growth mid-drag):
  - `crates/fret-ui/src/declarative/tests/layout/scroll.rs`

## Known risks / pain points

### 1) Barrier + invalidation accounting foot-guns

Barrier flows intentionally bypass normal “bubble invalidations to ancestors” rules. Any path that
mutates:

- `Node::children` for a barrier, or
- `Node::invalidation.layout` directly

must keep “subtree dirty aggregation” consistent, or debug underflow/drift can occur. This is
especially likely in scroll/virtualization surfaces.

### 2) High-frequency wheel input

Currently each wheel event can trigger:

- scroll handle mutation,
- binding invalidations,
- redraw request.

This is correct but can be expensive under trackpads (many events per frame).

Concrete GPUI anchor: `accumulated_scroll_delta = accumulated_scroll_delta.coalesce(event.delta)` in
`repo-ref/zed/crates/gpui/src/elements/list.rs`.

### 3) Scrollbar drag stability while content grows/measures

Zed/GPUI’s list state includes an explicit “scrollbar drag started/ended” mode that stabilizes the
height used for scrollbar math while dragging, preventing the thumb from “moving away” as measured
content changes.

Fret’s current scrollbar mechanism tracks `dragging_thumb` but does not explicitly lock a baseline
for content extent during drag.

Concrete GPUI anchor (for parity): `scrollbar_drag_start_height` + `scrollbar_drag_started/ended` +
`max_offset_for_scrollbar` in `repo-ref/zed/crates/gpui/src/elements/list.rs`.

## Proposed work items (mechanism-safe)

### A) Wheel/trackpad delta coalescing (gated)

Goal: reduce redundant invalidation under high-frequency input while preserving determinism.

Options:

1. Platform/runner coalescing (preferred):
   - Coalesce wheel deltas into one per frame/tick per target window.
2. UI-layer coalescing:
   - Maintain a per-window accumulator applied during `layout` or `prepaint`.

Current implementation (native, opt-in):

- Desktop runner buffers wheel deltas and delivers at most one `PointerEvent::Wheel` per frame when
  `FRET_WINIT_COALESCE_WHEEL=1`:
  - Buffering/flush: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - Per-window storage: `WindowRuntime.pending_wheel` in `crates/fret-launch/src/runner/desktop/runner/window.rs`
  - Mapping still uses winit semantics (`WindowEvent::MouseWheel` -> `PointerEvent::Wheel`) via
    `crates/fret-runner-winit/src/state/input/mod.rs`.
  - Guardrail: cap the absolute delta of a single delivered wheel event, carrying remainder over
    subsequent frames:
    - `FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` (default: `120`)

Design note (cap sensitivity):

- Under “adjacent event” coalescing (before frame-boundary buffering), repeat=11 perf evidence
  showed `FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` is workload-sensitive: a cap that is “safe” for
  `ScrollArea` could still cause spikes in `VirtualList`. After switching to frame-boundary buffering,
  `cap=120` is stable for both scripts in repeat=11 (see `docs/workstreams/scroll-optimization-v1/TODO.md`
  for the before/after bundles and logs).

Evidence gate:

- A perf-oriented diag script wheels repeatedly and captures a bundle for later perf regression gating:
  - `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-wheel-torture.json`
  - Suite: `tools/diag-scripts/suites/perf-ui-gallery-scroll-area/`

Perf entrypoint:

- `fretboard diag perf perf-ui-gallery-scroll-area ...` resolves via `crates/fret-diag/src/perf_seed_policy.rs`.

### B) Scrollbar drag baseline lock (correctness/UX)

Goal: keep thumb position stable during drag even if content extent changes due to measurement or
overflow observation growth.

Mechanism idea:

- Store a “drag baseline” in `ScrollbarState` (content size and/or max offset at drag start).
- Use the baseline for:
  - mapping pointer movement -> offset,
  - computing thumb rect while dragging.

Current implementation (mechanism-only):

- `ScrollbarState` stores baseline viewport/content while dragging, so thumb math does not drift if
  content extents change mid-drag:
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/declarative/host_widget/event/scrollbar.rs`

Evidence gate:

- A diag script that:
  - starts thumb drag,
  - triggers content growth (timer-driven for determinism),
  - asserts scrollbar semantics remain within an epsilon of the baseline max offset:
    - `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scrollbar-drag-baseline-content-growth.json`

### C) Consolidate barrier invalidation helpers (hardening)

Goal: reduce repeated “manual bookkeeping” code paths.

Approach:

- Prefer a single helper (or a small set) in `UiTree` that:
  - mutates barrier children,
  - schedules contained relayout,
  - updates subtree dirty aggregation,
  - records diagnostics detail.

### D) Extents growth at scroll edge (already partly addressed)

Keep improving the “avoid pinned extents” behavior:

- `pending_extent_probe` scheduling on clamp-at-edge is good.
- Post-layout observation (wrapper peel + deep scan budgets) is promising; expand gates to ensure:
  - budget-hit fallback probes happen,
  - extents grow when needed,
  - no infinite/oscillating probe loops.

## Reference: Zed/GPUI patterns

- Scroll handle (div): `repo-ref/zed/crates/gpui/src/elements/div.rs`
- List wheel delta coalescing + scrollbar drag stabilization:
  - `repo-ref/zed/crates/gpui/src/elements/list.rs`
- Scroll routing concept (“should handle scroll” separate from hover):
  - `repo-ref/zed/crates/gpui/src/window.rs`

## Open questions

- Where should wheel delta coalescing live for Fret (runner vs UI core)?
- For scrollbar drag baseline: what baseline is sufficient (content height only, or full size)?
- Should “restrict scroll axis” be a mechanism knob (like GPUI) or remain policy-level?
