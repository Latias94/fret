# GPUI Parity Refactor — TODO Tracker (Unafraid)

Status: Active (workstream tracker; keep updated during refactors)

This document tracks executable TODOs for the GPUI parity refactor workstream. It is intentionally “task-first”:

- The narrative plan lives in: `docs/workstreams/gpui-parity-refactor.md`
- “Hard-to-change” contracts live in ADRs (see Contract Gates below)

## Contract Gates (Must Drive Implementation)

- Dirty views + notify: `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
- Interactivity pseudoclasses + structural stability: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`
- Prepaint + interaction stream range reuse: `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
- Cache roots (ViewCache v1): `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
- Paint-stream replay caching baseline: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`

## Defaults (v1; performance-first)

These defaults are intentionally “cache-root-first” to maximize performance impact with minimal surface-area change:

- `ViewId` is defined at cache boundary granularity (a `ViewCache` root).
- `notify()` (no explicit target) marks the current/nearest cache root dirty; if no cache root is active, it falls back
  to the window root.
- Dirty cache roots propagate to ancestor cache roots (nested boundaries must not replay stale ranges).

## Tracking Format

Each TODO is labeled:

- ID: `GPUI-MVP{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Existing Building Blocks)

Keep this list short and evidence-backed:

- ViewCache (v1) mechanics and correctness scaffolding exist:
  - Evidence: `crates/fret-ui/src/tree/tests/view_cache.rs`, `crates/fret-ui/src/declarative/tests/view_cache.rs`,
    `crates/fret-ui/src/tree/paint.rs`, `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/elements/cx.rs`.
- Diagnostics + scripted interaction runner exists (foundation for regression harnesses):
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fretboard/src/diag.rs`, `tools/diag-scripts/*`.
- Cache-root and paint-cache counters are exposed in the UI gallery driver:
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (cache roots and paint cache stats).

## MVP0 — Instrumentation / Regression Harnesses

- [x] GPUI-MVP0-perf-001 Cache-root perf breakdown in HUD/log output.
  - Touches: `apps/fret-ui-gallery/src/driver.rs`, `crates/fret-ui/src/tree/mod.rs`
- [x] GPUI-MVP0-perf-002 Tracing spans for layout/paint per cache root.
  - Touches: `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/tree/paint.rs`
- [x] GPUI-MVP0-diag-003 Overlay torture scripted scenario exists.
  - Touches: `tools/diag-scripts/ui-gallery-overlay-torture.json`, `apps/fretboard/src/diag.rs`
- [x] GPUI-MVP0-diag-004 Virtual list torture scripted scenario exists.
  - Touches: `tools/diag-scripts/ui-gallery-virtual-list-torture.json`, `apps/fretboard/src/diag.rs`

## MVP1 — Pseudoclasses + Structural Stability (Paint-only by Default)

Goal: make hover/focus/pressed “cheap by default” and stop subtree shape thrash (ADR 0181).

- [ ] GPUI-MVP1-ui-001 Add debug attribution for “hover caused layout invalidation”.
  - Touches: `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/tree/mod.rs`
  - Done when: overlay torture + virtual list torture run with 0 hover-attributed layout invalidations (except explicitly whitelisted components).
- [x] GPUI-MVP1-eco-002 Refactor top hover offenders to be structurally stable.
  - Start with: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`, `ecosystem/fret-ui-shadcn/src/*scroll*`
  - Done when: no hover-driven `set_children` churn in these components (verified via diagnostics + manual UX sanity).
  - Evidence: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
- [ ] GPUI-MVP1-eco-003 Write “pseudoclass rules of thumb” for component authors.
  - Touches: `docs/component-author-guide.md` or a dedicated addendum under `docs/workstreams/`
  - Done when: the guidance includes examples for scrollbar fade, hover toolbars, and focus rings without layout shifts.

## MVP2 — Dirty Views + `notify` (GPUI-Aligned Invalidation)

Goal: converge on `notify -> dirty views -> cached reuse` as the primary mental model (ADR 0180).

- [~] GPUI-MVP2-rt-001 Define `ViewId` and `notify` API shape at the `fret-ui` / `fret-app` boundary.
  - Touches: `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/elements/*`, `crates/fret-app/src/app.rs`
  - Reference: `repo-ref/zed/crates/gpui/src/window.rs` (`WindowInvalidator`, `dirty_views`)
  - Progress: `EventCx::notify()` exists and marks the nearest cache root as `view_cache_needs_rerender` via a dedicated invalidation source.
  - Evidence: `crates/fret-ui/src/widget.rs` (`EventCx::notify`), `crates/fret-ui/src/tree/mod.rs` (`UiDebugInvalidationSource::Notify`),
    `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_notify_marks_cache_root_needs_rerender`).
- [x] GPUI-MVP2-rt-002 Track per-window dirty view set and coalesce redraw scheduling.
  - Touches: `crates/fret-ui/src/tree/mod.rs`, runner glue in `crates/fret-launch/` if needed
  - Done when: repeated `notify` calls are coalesced; diagnostics can list dirty views (debug-only).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`dirty_cache_roots`, `request_redraw_coalesced`, `debug_dirty_views`),
    `crates/fret-ui/src/tree/dispatch.rs` (notify-driven redraw scheduling), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
    (`UiTreeDebugSnapshotV1.dirty_views`), `crates/fret-ui/src/tree/tests/view_cache.rs`
    (`view_cache_notify_propagates_to_ancestor_cache_roots`).
- [~] GPUI-MVP2-cache-003 Gate view-cache reuse on dirty views.
  - Touches: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/declarative/mount.rs`
  - Done when: a notified view never reuses cached ranges; a clean view reliably reuses them.
  - Progress: `notify` marks the nearest cache root as `view_cache_needs_rerender`, which disables view-cache reuse for that root.
  - Progress: model/global observation invalidation also marks cache roots dirty (`view_cache_needs_rerender`) so reuse is disabled on data changes.
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`should_reuse_view_cache_node`, `invalidation_source_marks_view_dirty`), `crates/fret-ui/src/widget.rs` (`EventCx::notify`),
    `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots`).

## MVP3 — Prepaint + Interaction Stream Range Reuse

Goal: make caching a closed loop across paint + interaction (+ semantics later), not “paint-only” (ADR 0182).

- [~] GPUI-MVP3-rec-001 Define the minimal interaction stream vocabulary for replay.
  - Candidates: hit regions, cursor requests, outside-press observers, focus traversal roots.
  - Touches: `crates/fret-ui/src/tree/*`, `crates/fret-core/src/*` (data-only shapes as needed)
  - Progress: add hit-test path reuse (cached “interaction range”) as an incremental, semantics-preserving step toward replayable interaction output.
  - Notes: reuse is currently enabled only for pointer-move dispatch; other pointer events rebuild the cache from a full hit-test pass.
  - Notes: reuse falls back to full hit-testing if the cached leaf can hit-test children (avoids stale routing when the pointer moves between descendants).
  - Evidence: `crates/fret-ui/src/tree/hit_test.rs` (`hit_test_layers_cached`, `try_hit_test_along_cached_path`),
    `crates/fret-ui/src/tree/dispatch.rs` (pointer-move-only reuse policy),
    `crates/fret-ui/src/tree/tests/hit_test.rs` (`hit_test_layers_cached_reuses_path_and_respects_layer_order`).
- [x] GPUI-MVP3-rec-002 Add a prepaint phase that records interaction ranges (per cache root) in a replayable way.
  - Touches: `crates/fret-ui/src/tree/*`
  - Reference: `repo-ref/zed/crates/gpui/src/element.rs` (prepaint), `repo-ref/zed/crates/gpui/src/view.rs` (`reuse_prepaint`)
  - Progress: `layout_all` triggers a prepaint pass that records an interaction stream and clears `InvalidationFlags.hit_test`.
  - Progress: cache roots (ViewCache v1) record and replay interaction ranges across frames via generation/key checks.
  - Evidence: `crates/fret-ui/src/tree/prepaint.rs` (`UiTree::prepaint_after_layout`),
    `crates/fret-ui/src/tree/layout.rs` (prepaint call site),
    `crates/fret-ui/src/tree/tests/prepaint.rs` (`prepaint_interaction_cache_replays_for_clean_view_cache_root`).
- [~] GPUI-MVP3-test-003 Add correctness tests: cached subtree keeps correct hit-test / outside-press behavior.
  - Touches: `crates/fret-ui/src/tree/tests/*`, `crates/fret-ui/src/declarative/tests/*`
  - Progress: outside-press routing remains correct when the overlay root is a view-cache root and prepaint interaction ranges are reused.
  - Evidence: `crates/fret-ui/src/tree/tests/outside_press.rs` (`outside_press_observer_works_with_view_cache_root_and_prepaint_reuse`).

## MVP4 — Migration + Adoption (Ecosystem + Demos)

Goal: make the new contracts “default obvious” by migrating a small set of representative components and demos.

- [ ] GPUI-MVP4-eco-001 Add an ecosystem-facing “cached subtree” helper API (policy-free).
  - Touches: `ecosystem/fret-ui-kit/src/*`
- [ ] GPUI-MVP4-demo-002 Migrate `fret-ui-gallery` hotspots to the new patterns (hover chrome, scrollbars, code views).
  - Touches: `apps/fret-ui-gallery/src/*`, selected `ecosystem/*` components

## Open Questions (Keep Short)

- Which interaction stream comes first for maximum “feel” payoff: hit regions vs cursor styles vs outside-press observers?
- Do we want to keep cache roots strictly opt-in only, or also provide guided defaults in demos (never in core)?
