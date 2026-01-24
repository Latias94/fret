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
- `request_animation_frame()` requested from within a view implies `notify()` for the nearest cache root on the next
  tick (GPUI-aligned), so view-cache reuse cannot replay stale output indefinitely during animations.
- Dirty cache roots propagate to ancestor cache roots (nested boundaries must not replay stale ranges).
- `request_animation_frame()` parity note: implemented as `request_animation_frame() -> (next tick) notify(nearest cache root)`
  (see `GPUI-MVP2-rt-003` evidence below).

## Tracking Format

Each TODO is labeled:

- ID: `GPUI-MVP{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Existing Building Blocks)

Keep this list short and evidence-backed:

- ViewCache (v1) mechanics and correctness scaffolding exist:
  - Evidence: `crates/fret-ui/src/tree/tests/view_cache.rs`, `crates/fret-ui/src/declarative/tests/view_cache.rs`,
    `crates/fret-ui/src/tree/paint.rs`, `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/elements/cx.rs`.
  - Notes: declarative conformance now also asserts cache-hit behavioral equivalence for painted scene ops and semantics/hit targets
    (see `crates/fret-ui/src/declarative/tests/view_cache.rs`).
  - Notes: conformance includes modal overlay barrier gating under view-cache reuse.
- Diagnostics + scripted interaction runner exists (foundation for regression harnesses):
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fretboard/src/diag.rs`, `tools/diag-scripts/*`.
  - Notes: `diag matrix ui-gallery` runs cached vs uncached variants per script and can gate on view-cache reuse and overlay cached-synthesis. Evidence: `tools/diag_matrix_ui_gallery.ps1`, `docs/ui-diagnostics-and-scripted-tests.md`.
- Cache-root and paint-cache counters are exposed in the UI gallery driver:
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (cache roots and paint cache stats).

## MVP0 — Instrumentation / Regression Harnesses

- [x] GPUI-MVP0-perf-001 Cache-root perf breakdown in HUD/log output.
  - Touches: `apps/fret-ui-gallery/src/driver.rs`, `crates/fret-ui/src/tree/mod.rs`
- [x] GPUI-MVP0-perf-002 Tracing spans for layout/paint per cache root.
  - Touches: `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/tree/paint.rs`
- [x] GPUI-MVP0-diag-003 Overlay torture scripted scenario exists.
  - Touches: `tools/diag-scripts/ui-gallery-overlay-torture.json`, `apps/fretboard/src/diag.rs`
- [x] GPUI-MVP0-diag-012 Modal barrier underlay-block scripted scenario exists (overlay regression scaffold).
  - Touches: `tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json`, `apps/fretboard/src/diag.rs`
- [x] GPUI-MVP0-diag-013 Add a bundle comparison command (cached vs uncached) to enforce “behavior preserving” view-cache.
  - Touches: `apps/fretboard/src/diag.rs`, `apps/fretboard/src/cli.rs`, `docs/ui-diagnostics-and-scripted-tests.md`
  - Notes: compares stable semantics anchors (`debug.semantics.nodes[].test_id`) and can include paint `scene_fingerprint`.
- [x] GPUI-MVP0-diag-014 Add post-run view-cache reuse gating for scripted regressions.
  - Touches: `apps/fretboard/src/diag.rs`, `apps/fretboard/src/cli.rs`, `docs/ui-diagnostics-and-scripted-tests.md`
  - Notes: `--check-view-cache-reuse-min N` counts `debug.cache_roots[].reused == true` events after warmup frames.
- [x] GPUI-MVP0-diag-004 Virtual list torture scripted scenario exists.
  - Touches: `tools/diag-scripts/ui-gallery-virtual-list-torture.json`, `apps/fretboard/src/diag.rs`
- [x] GPUI-MVP0-diag-005 Export prepaint timing + add warmup filtering for perf runs.
  - Touches: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fretboard/src/diag.rs`, `apps/fretboard/src/cli.rs`
  - Notes: `--warmup-frames <n>` skips early `frame_id` snapshots when ranking; short scripts auto-fallback to unfiltered stats when warmup would skip everything.
- [x] GPUI-MVP0-diag-008 Allow env injection for launched diag targets.
  - Touches: `apps/fretboard/src/diag.rs`, `apps/fretboard/src/cli.rs`
  - Notes: supports repeating `--env KEY=VALUE` and passes them only to the launched target process (not the diag runner).
  - Notes: reserved variables are protected (e.g. `FRET_DIAG`, `FRET_DIAG_DIR`, `FRET_DIAG_READY_PATH`).
- [x] GPUI-MVP0-diag-009 Add env-driven UI Gallery toggles for perf harnesses.
  - Touches: `apps/fret-ui-gallery/src/driver.rs`
  - Notes: `FRET_UI_GALLERY_VIEW_CACHE{,_SHELL,_INNER,_CONTINUOUS}` are parsed as bools (with sensible defaults) to keep scripts deterministic.
- [x] GPUI-MVP0-diag-010 Add scroll + stale-paint regression hooks.
  - Touches: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fretboard/src/diag.rs`, `apps/fret-ui-gallery/src/ui.rs`, `tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json`
  - Notes: scripts support `wheel` steps; bundles export `scene_fingerprint`; `fretboard diag stats --check-stale-paint <test_id>` flags “bounds moved but scene fingerprint did not change”.
- [x] GPUI-MVP0-diag-011 Gracefully stop launched diag targets.
  - Touches: `apps/fretboard/src/diag.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
  - Notes: `fretboard diag run/suite/perf --launch` sets `FRET_DIAG_EXIT_PATH` and touches it on completion; the target polls it and requests `Effect::QuitApp`.
- [x] GPUI-MVP0-perf-006 Avoid false global-change churn from stable “service globals”.
  - Touches: `ecosystem/fret-ui-kit/src/dnd/service.rs`
  - Notes: use `with_global_mut_untracked` for lazy init + stable read paths (prevents global-change tracking from firing on every frame).
- [x] GPUI-MVP0-perf-007 Avoid false global-change churn from frame-local overlay registries.
  - Touches: `ecosystem/fret-ui-shadcn/src/a11y_modal.rs`
  - Notes: `ModalA11yRegistry` is a frame-local registry used during modal overlay construction; it should not participate in host global-change tracking.

## MVP1 — Pseudoclasses + Structural Stability (Paint-only by Default)

Goal: make hover/focus/pressed “cheap by default” and stop subtree shape thrash (ADR 0181).

- [~] GPUI-MVP1-ui-001 Add debug attribution for “hover caused layout invalidation”.
  - Touches: `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/tree/mod.rs`, diagnostics export in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, CLI surfacing in `apps/fretboard/src/diag.rs`.
  - Progress: `bundle.json` exports hover-attributed declarative invalidation counters + top hotspots (`debug.hover_declarative_invalidation_hotspots`); `fretboard diag stats` can gate via `--check-hover-layout[(-max N)]`.
  - Progress: `fretboard diag run` / `fretboard diag suite` can enforce the same gate post-run via `--check-hover-layout-max 0`.
  - Done when: overlay torture + virtual list torture run with 0 hover-attributed layout invalidations (except explicitly whitelisted components).
- [x] GPUI-MVP1-eco-002 Refactor top hover offenders to be structurally stable.
  - Start with: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`, `ecosystem/fret-ui-shadcn/src/*scroll*`
  - Done when: no hover-driven `set_children` churn in these components (verified via diagnostics + manual UX sanity).
  - Evidence: `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
- [x] GPUI-MVP1-eco-003 Write “pseudoclass rules of thumb” for component authors.
  - Evidence: `docs/component-author-guide.md` (Interactivity pseudoclasses section)

## MVP2 — Dirty Views + `notify` (GPUI-Aligned Invalidation)

Goal: converge on `notify -> dirty views -> cached reuse` as the primary mental model (ADR 0180).

- [x] GPUI-MVP2-rt-001 Define `ViewId` and `notify` API shape at the `fret-ui` / `fret-app` boundary.
  - Touches: `crates/fret-core/src/ids.rs`, `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/tree/*`
  - Reference: `repo-ref/zed/crates/gpui/src/window.rs` (`WindowInvalidator`, `dirty_views`)
  - Evidence: `crates/fret-core/src/ids.rs` (`ViewId`), `crates/fret-ui/src/widget.rs` (`EventCx::notify`), `crates/fret-ui/src/tree/dispatch.rs` (notify targets the current view),
    `crates/fret-ui/src/tree/mod.rs` (`UiDebugInvalidationSource::Notify`, `debug_dirty_views`), `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_notify_marks_cache_root_needs_rerender`).
- [x] GPUI-MVP2-rt-002 Track per-window dirty view set and coalesce redraw scheduling.
  - Touches: `crates/fret-ui/src/tree/mod.rs`, runner glue in `crates/fret-launch/` if needed
  - Done when: repeated `notify` calls are coalesced; diagnostics can list dirty views (debug-only).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`dirty_cache_roots`, `request_redraw_coalesced`, `debug_dirty_views`),
    `crates/fret-ui/src/tree/dispatch.rs` (notify-driven redraw scheduling), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
    (`UiTreeDebugSnapshotV1.dirty_views`), `crates/fret-ui/src/tree/tests/view_cache.rs`
    (`view_cache_notify_propagates_to_ancestor_cache_roots`).
- [x] GPUI-MVP2-rt-003 Make `request_animation_frame()` dirty the current view (GPUI-aligned).
  - Touches: `crates/fret-ui/src/elements/cx.rs` (`ElementContext::request_animation_frame`), `crates/fret-ui/src/widget.rs` (`*Cx::request_animation_frame`).
  - Goal: if a subtree relies on frame-driven updates (animations), `request_animation_frame()` must not allow a cache-hit frame to replay stale output indefinitely.
  - Reference: `repo-ref/zed/crates/gpui/src/window.rs` (`request_animation_frame` -> notify view / dirty views).
  - Notes: v1 implements this as `request_animation_frame()` implying `notify()` on the nearest cache root.
  - Evidence:
    - `crates/fret-ui/src/elements/cx.rs` (`ElementContext::request_animation_frame`)
    - `crates/fret-ui/src/widget.rs` (`LayoutCx::request_animation_frame`, `MeasureCx::request_animation_frame`, `PaintCx::request_animation_frame`)
    - `crates/fret-ui/src/declarative/mount.rs` (drains animation-frame notify requests and invalidates with `UiDebugInvalidationDetail::AnimationFrameRequest`)
    - `crates/fret-ui/src/declarative/tests/view_cache.rs` (`request_animation_frame_marks_view_cache_root_dirty`)
    - `crates/fret-ui/src/tree/tests/view_cache.rs` (`widget_request_animation_frame_marks_nearest_view_cache_root_dirty`)
- [x] GPUI-MVP2-cache-003 Gate view-cache reuse on dirty views.
  - Touches: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/declarative/mount.rs`, `crates/fret-ui/src/elements/runtime.rs`
  - Done when: a notified view never reuses cached ranges; a clean view reliably reuses them.
  - Progress: `notify` marks the nearest cache root as `view_cache_needs_rerender`, which disables view-cache reuse for that root.
  - Progress: model/global observation invalidation also marks cache roots dirty (`view_cache_needs_rerender`) so reuse is disabled on data changes.
  - Progress: cache-hit frames still uplift element-recorded observations to cache roots (prevents stale cache-hit when an input event changes model state but the subtree is reused).
  - Evidence: `crates/fret-core/src/ids.rs` (`ViewId`), `crates/fret-ui/src/tree/dispatch.rs` (`notify_target_for_node`), `crates/fret-ui/src/tree/mod.rs` (`should_reuse_view_cache_node`, `invalidation_source_marks_view_dirty`), `crates/fret-ui/src/widget.rs` (`EventCx::notify`), `crates/fret-ui/src/elements/runtime.rs`,
    `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots`).

- [x] GPUI-MVP2-cache-004 Stabilize overlay interactions under `ViewCache` shell reuse.
  - Touches: `crates/fret-ui/src/declarative/mount.rs`, `crates/fret-ui/src/elements/runtime.rs`
  - Goal: `tools/diag-scripts/ui-gallery-overlay-torture.json` completes with `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
  - Root cause: the declarative element GC ("stale nodes after gc lag frames") is keyed off `last_seen_frame`, but view-cache reuse intentionally skips re-mounting cached subtrees.
    On the first cache-hit frame, stale-but-live overlay nodes (e.g. `ui-gallery-dialog-trigger`) could be swept, which then removed overlay semantics roots and broke scripted clicks.
  - Fix (v1):
    - When a cache root transitions into reuse, touch the existing retained subtree (`last_seen_frame`) and (re-)record subtree elements so cache-hit frames keep liveness/identity consistent.
    - Note: early iterations used a global "skip sweep while reuse exists" safety gate; MVP2-cache-005 replaces this with re-enabled sweeping under reuse.
  - Diagnostics: export `removed_subtrees` records in bundles to make sweeping behavior explainable from a single run.
  - Evidence (pass):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-torture.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
  - Follow-up: reintroduce declarative GC under cache-root reuse with explicit, GPUI-aligned liveness (dirty views + notify + cache key gates) rather than the global "skip sweep when reuse exists" stopgap.

- [x] GPUI-MVP2-cache-005 Reintroduce declarative node GC with explicit cache-root liveness.
  - Touches: `crates/fret-ui/src/declarative/mount.rs` (GC), `crates/fret-ui/src/tree/mod.rs` (parent pointer repair), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export).
  - Goal: collect truly-detached nodes without deleting live cached subtrees (keep `ui-gallery-overlay-torture.json` green under shell reuse).
  - Root cause: `node_layer` (and cache-root discovery) relies on parent pointers; a reachable subtree can retain correct child edges but have a broken parent chain,
    causing GC to mis-classify it as detached and sweep it on a cache-hit frame.
  - Fix (v1):
    - Repair parent pointers for nodes reachable from layer roots before running the declarative GC sweep (`UiTree::repair_parent_pointers_from_layer_roots`).
    - Keep reachability-based sweeping (layer roots + explicit view-cache subtree liveness), rather than a global "skip sweep when reuse exists" gate.
  - Diagnostics: `removed_subtrees` now include `root_parent_element`, a `root_path` sample, and `root_parent_children_last_set_location` (when available), plus `root_element_path` when resolvable.
  - Evidence (pass under reuse + shell):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-torture.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
  - Evidence (unit): `crates/fret-ui/src/declarative/tests/core.rs` (`stale_nodes_are_swept_after_gc_lag_under_view_cache_reuse`)

- [x] GPUI-MVP2-cache-008 Repair cache-root bounds when the runtime skips placement (view-cache + shell).
  - Touches: `crates/fret-ui/src/tree/layout.rs` (`repair_view_cache_root_bounds_from_engine_if_needed`)
  - Goal: cache-root semantics bounds remain in screen space so scripted clicks hit real widgets.
  - Root cause: some cache roots could end up with `Rect::default()` bounds even though the layout engine has a solved rect for them, causing the entire subtree's semantics bounds to be relative (0-based) and diagnostics clicks to miss the intended controls.
  - Fix (v1): after the main layout pass, if a view-cache root has default bounds but its parent has a solved engine child rect, synthesize the root bounds and translate the retained subtree by the implied delta.
  - Evidence (pass under reuse + shell):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-virtual-list-torture.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`

- [x] GPUI-MVP2-cache-006 Add an explicit cache key gate for view-cache reuse (GPUI-aligned).
  - Touches: `crates/fret-ui/src/element.rs` (`ViewCacheProps.cache_key`), `crates/fret-ui/src/elements/cx.rs` (reuse gating),
    `crates/fret-ui/src/elements/runtime.rs` (per-root key storage), `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`.
  - Goal: prevent reusing a view-cache root when key inputs like theme/bounds/text style/content mask changed.
  - v1 key: `hash(theme_revision, scale_factor, window_bounds, ViewCacheProps.cache_key)`.
    - Notes: this is a coarse proxy for GPUI's `bounds/content_mask/text_style` key and will be refined as more inputs become explicit.
    - Helpers: `fret_ui::cache_key::{text_style_key, rect_key, corners_key}`; `fret_ui_kit::declarative::CachedSubtreeProps::{cache_key_text_style, cache_key_clip_rect, cache_key_clip_rrect}`.
  - Reference: `repo-ref/zed/crates/gpui/src/view.rs` (`ViewCacheKey`: bounds/content_mask/text_style).
  - Evidence:
    - `crates/fret-ui/src/declarative/tests/view_cache.rs` (`view_cache_gates_reuse_on_explicit_cache_key`)
    - `cargo nextest run -p fret-ui`
  - Diagnostics: cache-root stats now distinguish key misses via `reuse_reason=cache_key_mismatch`.

- [x] GPUI-MVP2-diag-007 Keep debug identity stable on cache-hit frames (`--features diagnostics`).
  - Goal: inspector / debug paths remain resolvable even when a view-cache root is a cache hit and its render closure
    is skipped.
  - Touches: `crates/fret-ui/src/elements/runtime.rs` (`touch_view_cache_subtree_elements_if_recorded`).
  - Evidence: `crates/fret-ui/src/declarative/tests/view_cache.rs`
    (`view_cache_skips_child_render_when_clean_and_preserves_element_state`).

## MVP3 — Prepaint + Interaction Stream Range Reuse

Goal: make caching a closed loop across paint + interaction (+ semantics later), not “paint-only” (ADR 0182).

- [ ] GPUI-MVP3-virt-002 VirtualList: reduce rerender cost during scroll via incremental range reuse (GPUI-component parity).
  - Motivation: `ui-gallery-virtual-list-torture.json` remains layout-dominated even with view-cache + shell reuse.
  - Perf snapshot (release, `--warmup-frames 5`, `--sort time`):
    - Baseline: `sum.total_time_us=225911` / 10 frames; `max.total_time_us=30585` (layout `29252`).
    - ViewCache+Shell: `sum.total_time_us=214260` / 10 frames; `max.total_time_us=28999` (layout `27545`).
    - Smooth wheel (new harness: `tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json`):
      - Baseline: `max.total_time_us=27219` (layout `26948`, prepaint `23`, paint `248`).
      - ViewCache+Shell: `max.total_time_us=26890` (layout `26205`, prepaint `19`, paint `666`).
  - Commands:
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-baseline --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 10 --json perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-cache-shell --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 10 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json --warmup-frames 5 --sort time --top 15 --dir target/fret-diag-perf-vlist-smooth-scroll-baseline --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json --warmup-frames 5 --sort time --top 15 --dir target/fret-diag-perf-vlist-smooth-scroll-cache-shell --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`
  - Sketch:
    - Move “visible range / scroll_to_item” work toward `prepaint` (GPUI-style), so steady-state scroll can reuse paint + interaction ranges without rebuilding large declarative subtrees.
    - Keep per-item identity stable (do not recycle cells) while making the “range delta” path cheap.
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
- [x] GPUI-MVP3-test-003 Add correctness tests: cached subtree keeps correct hit-test / outside-press behavior.
  - Touches: `crates/fret-ui/src/tree/tests/*`, `crates/fret-ui/src/declarative/tests/*`
  - Progress: outside-press routing remains correct when the overlay root is a view-cache root and prepaint interaction ranges are reused.
  - Progress: hit-testing remains correct under render transforms when the overlay root is a view-cache root and prepaint interaction ranges are reused.
  - Evidence: `crates/fret-ui/src/tree/tests/outside_press.rs` (`outside_press_observer_works_with_view_cache_root_and_prepaint_reuse`),
    `crates/fret-ui/src/tree/tests/hit_test.rs` (`hit_test_works_with_view_cache_root_and_prepaint_reuse_under_render_transform`).
- [x] GPUI-MVP3-rt-004 Use prepainted hit-test caches for routing hot paths.
  - Touches: `crates/fret-ui/src/tree/prepaint.rs`, `crates/fret-ui/src/tree/hit_test.rs`, `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/tree/mod.rs`
  - Progress: prepaint caches and replays inverse transforms + clip metadata per node; hit-testing and event coordinate mapping reuse it when nodes are clean and inspection is inactive.
  - Evidence: `crates/fret-ui/src/tree/prepaint.rs` (`apply_interaction_record`), `crates/fret-ui/src/tree/hit_test.rs` (`prepaint_hit_test` fast path), `crates/fret-ui/src/tree/dispatch.rs` (`build_mapped_event_chain`).

## MVP4 — Migration + Adoption (Ecosystem + Demos)

Goal: make the new contracts “default obvious” by migrating a small set of representative components and demos.

- [x] GPUI-MVP4-eco-001 Add an ecosystem-facing “cached subtree” helper API (policy-free).
  - Touches: `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`
  - Evidence: `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs` (`CachedSubtreeExt`, `CachedSubtreeProps`)
- [ ] GPUI-MVP4-demo-002 Migrate `fret-ui-gallery` hotspots to the new patterns (hover chrome, scrollbars, code views).
  - Touches: `apps/fret-ui-gallery/src/*`, selected `ecosystem/*` components

## Open Questions (Keep Short)

- Which interaction stream comes first for maximum “feel” payoff: hit regions vs cursor styles vs outside-press observers?
- Do we want to keep cache roots strictly opt-in only, or also provide guided defaults in demos (never in core)?
