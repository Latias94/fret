# GPUI Parity Refactor — TODO Tracker (Unafraid)

Status: Active (workstream tracker; keep updated during refactors)

This document tracks executable TODOs for the GPUI parity refactor workstream. It is intentionally “task-first”:

- The narrative plan lives in: `docs/workstreams/gpui-parity-refactor.md`
- “Hard-to-change” contracts live in ADRs (see Contract Gates below)

## Contract Gates (Must Drive Implementation)

- Dirty views + notify: `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
- Interactivity pseudoclasses + structural stability: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`
- Prepaint + interaction stream range reuse: `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
- Prepaint-windowed virtual surfaces: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md`
- Cache roots (ViewCache v1): `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
- Paint-stream replay caching baseline: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`

## Defaults (v1; performance-first)

These defaults are intentionally “cache-root-first” to maximize performance impact with minimal surface-area change:

- `ViewId` is defined at cache boundary granularity (a `ViewCache` root).
- `notify()` (no explicit target) marks the current/nearest cache root dirty; if no cache root is active, it falls back
  to the window root.
- `request_animation_frame()` requested from within a view implies `notify()` for the nearest cache root on the next
  tick (GPUI-aligned), so view-cache reuse cannot replay stale output indefinitely during animations.
- A paint-only variant (e.g. `request_animation_frame_paint_only()`) MAY be used for chrome-only loops under view-cache
  reuse (hover/focus/selection/caret/drag indicators): it schedules a frame without marking the view dirty and should
  be paired with paint invalidation.
- Dirty cache roots propagate to ancestor cache roots (nested boundaries must not replay stale ranges).
- `request_animation_frame()` parity note: implemented as `request_animation_frame() -> (next tick) notify(nearest cache root)`
  (see `GPUI-MVP2-rt-003` evidence below).

## Tracking Format

Each TODO is labeled:

- ID: `GPUI-MVP{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Near-term Focus (keep tight)

- **MVP2-cache-005**: done (keep overlay/scroll-refresh harnesses green while refactoring other areas).
- **MVP5-virt-001**: move VirtualList window derivation toward prepaint so window shifts do not necessarily imply cache-root rerender.
- **MVP5-perf-002**: turn “notify hotspots no longer dominated by Pressable” into a repeatable perf gate (top bundles + callsites).

## Near-term Plan (2026-02; make progress measurable)

- **P1 — MVP5-virt-001 contract + explainability hardening**
  - Deliverable: tighten ADR 0190/0193 wording for non-retained v2 (“what can change in prepaint” vs “must rerender”), and keep the `ui-gallery-vlist-window-boundary` suite green with `--check-vlist-window-shifts-explainable`.
- **P2 — MVP5-perf-002 notify hotspots**
  - Deliverable: pick one stable harness (VirtualList boundary or overlay torture), record `notify_call` hotspots with *callsite attribution*, and add one budget gate that fails fast with evidence JSON.
  - Checklist:
    - Add `#[track_caller]` capture for `EventCx::notify()` / `ObserverCx::notify()` and propagate a single callsite per dispatch.
    - Export bounded per-frame `debug.notify_requests` (file/line/col + caller node + target dirty view/cache root).
    - Add a `fretboard diag stats` gate that can fail on a specific hotspot file (e.g. `pressable.rs`) and writes `check.notify_hotspots.json`.
    - Wire the gate into at least one perf suite (warmup-ranked) so it is exercised in CI-style runs.
- **P3 — MVP5-eco-003 code/text window pilot**
  - Deliverable: migrate one concrete surface (code-view lines or markdown) onto the prepaint-windowed pattern and add a diag script + stale-paint gate.

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

- [x] GPUI-MVP1-ui-001 Add debug attribution for “hover caused layout invalidation”.
  - Touches: `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/tree/mod.rs`, diagnostics export in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, CLI surfacing in `apps/fretboard/src/diag.rs`.
  - Progress: `bundle.json` exports hover-attributed declarative invalidation counters + top hotspots (`debug.hover_declarative_invalidation_hotspots`); `fretboard diag stats` can gate via `--check-hover-layout[(-max N)]`.
  - Progress: `fretboard diag run` / `fretboard diag suite` can enforce the same gate post-run via `--check-hover-layout-max 0`.
  - Done when: overlay torture + virtual list torture run with 0 hover-attributed layout invalidations (except explicitly whitelisted components).
  - Evidence: both scripts pass `--check-hover-layout-max 0` (warmup 5): `target/fret-diag-hover-check-overlay/` + `target/fret-diag-hover-check-vlist/`.
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
    - Note: early iterations used a global "skip sweep while reuse exists" safety gate; MVP2-cache-005 aims to remove this by making liveness explicit under reuse.
  - Diagnostics: export `removed_subtrees` records in bundles to make sweeping behavior explainable from a single run.
  - Evidence (pass):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-torture.json --timeout-ms 240000 --poll-ms 200 --check-gc-sweep-liveness --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
  - Follow-up: remove the global "skip sweep when reuse exists" stopgap by relying on explicit liveness under cache-root reuse (dirty views + notify + cache key gates).

- [x] GPUI-MVP2-cache-005 Reintroduce declarative node GC with explicit cache-root liveness.
  - Touches: `crates/fret-ui/src/declarative/mount.rs` (GC + cache-root subtree recording), `crates/fret-ui/src/tree/mod.rs` (liveness reachability + attachment/ownership bookkeeping repair), `crates/fret-ui/src/elements/runtime.rs` (per-root subtree lists), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export).
  - Goal: collect truly-detached nodes without deleting live cached subtrees (keep `ui-gallery-overlay-torture.json` green under shell reuse).
  - Contract: `docs/adr/0191-declarative-liveness-roots-and-gc-under-view-cache-reuse.md` (Accepted).
  - Fix (v1):
    - Removed the global “skip sweep while reuse exists” GC stopgap.
    - Made liveness explicit under reuse via layer roots + view-cache reuse roots + per-root subtree membership lists (ADR 0191).
    - Stabilized unkeyed element identity generation by using per-callsite counters (reduces accidental subtree swaps under conditional structure).
  - Progress (diagnostics): bundles now include retained keep-alive root samples (`debug.element_runtime.retained_keep_alive_roots_*`), so the full liveness root set can be audited from a single run.
    - Anchors: `crates/fret-ui/src/elements/runtime.rs` (`WindowElementDiagnosticsSnapshot::retained_keep_alive_roots_*`),
      `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`ElementDiagnosticsSnapshotV1.retained_keep_alive_roots_*`).
  - Root-cause framing (keep honest):
    - When a live interactive subtree is swept while view-cache reuse exists, the GC is usually behaving correctly on the graph it sees.
      The bug is typically that ownership/liveness bookkeeping allowed the subtree to become an *island root* (unreachable from both
      layer roots and view-cache reuse roots), most often due to incomplete/misaligned subtree membership lists under reuse (ADR 0191).
  - Evidence (cache+shell, sweep enabled):
    - `target/fret-diag-cache005-stopgap-removed-overlay-1769334929510/1769335008125-ui-gallery-overlay-torture/bundle.json`
    - `target/fret-diag-cache005-stopgap-removed-sidebar-1769335037056/1769335040362-ui-gallery-sidebar-scroll-refresh/bundle.json`
  - Evidence (stopgap disabled, before removal):
    - `target/fret-diag-cache005-stopgap-disabled-overlay-1769334562794/1769334640506-ui-gallery-overlay-torture/bundle.json`
    - `target/fret-diag-cache005-stopgap-disabled-sidebar-1769334672956/1769334676508-ui-gallery-sidebar-scroll-refresh/bundle.json`
  - Progress (v1):
    - Keep reachability-based sweeping (layer roots + explicit view-cache subtree liveness) as the foundation for removing the global stopgap gate.
    - GC reachability classification unions `UiTree` + `WindowFrame` retained child edges (unit test: `gc_reachability_unions_ui_and_window_frame_children` in `crates/fret-ui/src/declarative/mount.rs`).
    - Record ViewCache root subtree element lists on cache-miss frames and touch them on cache-hit frames so liveness does not depend on "visited this frame".
    - Diagnostics: `removed_subtrees[*]` exports `root_layer_visible`, `unreachable_from_liveness_roots`, and root-set counts (`liveness_layer_roots_len`, `view_cache_reuse_roots_len`, `view_cache_reuse_root_nodes_len`) so “island vs. broken parent chain” is distinguishable from one bundle. Gate: `fretboard diag stats <bundle> --check-gc-sweep-liveness`.
    - Regression test: cache-hit liveness remains correct even if child-edge reachability drifts for a reused cache root (membership list still prevents premature sweep).
      - Anchor: `crates/fret-ui/src/declarative/tests/view_cache.rs` (`view_cache_subtree_membership_keeps_detached_children_alive_under_cache_hit`).
  - Historical notes (pre-fix):
    - NOTE: this section is historical; the stopgap guard has been removed and the harnesses are green under cache+shell.
      - Removed guard: `crates/fret-ui/src/declarative/mount.rs` (previous `view_cache_has_reuse_roots` gate).
    - Explain why a subtree that should still be live becomes unreachable from *both* liveness sources on the failing frame:
      - identify the `root_root_parent_sever_parent` node (map to element id + debug path + whether it is a cache root / reuse root),
      - verify whether the `set_children` call at `mount.rs:1162` represents an intentional cache-root child swap (and identify old/new child element identities),
      - verify the expected cache root is present in `window_state.view_cache_reuse_roots()` and has a *complete* recorded subtree-element list (and whether it is being invalidated/cleared prematurely).
    - Ensure cache-root subtree element recording is complete even on cache-hit frames:
      - `collect_declarative_elements_for_existing_subtree(..)` MUST be able to recover element ids without relying solely on `WindowFrame.instances` (cache hits may skip instances).
      - Prefer the element runtime's node entries as an authoritative fallback (e.g. `WindowElementState::element_for_node(node)`), then re-run the stopgap-disabled harnesses.
    - If overlay torture still fails under stopgap disabled, prioritize proving whether the missing semantics targets live under a nested `CachedSubtree` (inner ViewCache root) that is not being kept alive when an outer cache root hits:
      - confirm whether the inner cache root appears in the outer root's recorded subtree-element list (and is therefore part of the keep-alive recursion),
      - and whether the subtree becomes an island due to missing ownership/attachment bookkeeping rather than pure parent-pointer drift.
    - Diagnostics follow-up: `--check-gc-sweep-liveness` writes `check.gc_sweep_liveness.json` next to `bundle.json` (even on failure), with bounded offender samples and the key root-set counters.
      Use this file as the first stop when classifying “island vs. broken parent chain”, without re-running the harness.
    - Diagnostics follow-up: some failing bundles still cannot resolve `root_element_path` / `root_parent_element_path` (paths are `null`).
      Prefer capturing debug paths at removal time (or extending the debug-path retention window) so cache-005 regressions stay explainable from a single bundle.
    - Verify we are not accidentally overwriting element-root ownership during “touch existing subtree” paths:
      - add debug-only diagnostics when updating `NodeEntry.root` for an element that already has a different `root`,
      - decide whether to preserve the original root (avoid cross-root pollution) or split bookkeeping per-root if overwrites are expected/legitimate.
  - Historical investigation plan (pre-fix):
    - Export/confirm the liveness roots on the failing frame: which layer roots are active (note: `UiTree::all_layer_roots` includes invisible layers; visibility is not a lifetime signal), plus the current `view_cache_reuse_roots` list, plus retained keep-alive roots (ADR 0192).
    - Export the sever-parent mapping (parent node -> element id/path + cache-root flags) so the detach callsite can be tied back to the authoring UI structure.
    - Add debug-only diagnostics for `NodeEntry.root` overwrites (element + old_root + new_root + debug paths) to validate or falsify the “cross-root ownership overwrite” hypothesis.
    - Re-run the overlay torture with the stopgap disabled and use the new fields to decide whether the fix is:
      - Note: in stopgap-disabled failing bundles we expect to see whether a swept subtree is a true island (both `reachable_from_layer_roots=false` and `reachable_from_view_cache_roots=false`), which points to liveness-root selection or attach/ownership bookkeeping drift (not just a “parent pointer broke” story).
      - missing liveness roots (root selection / visibility semantics),
      - root ownership / attachment bookkeeping drift (the subtree becomes an island even though the app still expects it to be interactive), or
      - a true structural detach (authoring/runtime edge drop) that must be attributed to a callsite.
  - Done when:
    - The `view_cache_has_reuse_roots` stopgap is removed and both overlay regression harnesses remain green under cache+shell reuse.
    - `fretboard diag suite ui-gallery-cache005` defaults `--check-gc-sweep-liveness` for the cache-005 harness scripts (overlay torture + sidebar scroll refresh), so island regressions are caught without manual inspection (without bloating the main `ui-gallery` suite).
  - Diagnostics:
    - `removed_subtrees` include `root_element_path` (when the element debug identity is still retained within the diagnostics lag window).
    - `removed_subtrees` include `root_parent_children_last_set_location` (when the parent has a recorded `set_children(..)` write in this run).
    - `removed_subtrees` include `root_path_edge_ui_contains_child` / `root_path_edge_frame_contains_child` to pinpoint whether the parent chain is internally consistent
      (and whether the authoritative `WindowFrame.children` agrees with `UiTree` edges).
    - `removed_subtrees.reachable_from_layer_roots` is computed using the same conservative reachability used by GC (liveness roots + union of `UiTree` and `WindowFrame` edges),
      so cache-hit frames with temporarily-incomplete `UiTree.children` do not misreport “islands”.
    - `removed_subtrees` include `reachable_from_view_cache_roots` to classify whether a swept subtree was still reachable from any view-cache reuse root node (child-edge reachability),
      vs. becoming a fully-detached island.
    - `removed_subtrees` include `trigger_element` / `trigger_element_root` and `trigger_element_root_path` to identify which element-runtime root produced the sweep.
    - `removed_subtrees` include `trigger_element_in_view_cache_keep_alive` / `trigger_element_listed_under_reuse_root` to explain whether view-cache subtree membership contributed to liveness decisions.
    - `removed_subtrees` include `root_root_parent_sever_*` to attribute detached-island roots to the structural operation that severed them.
    - `removed_subtrees` include `root_root_parent_sever_parent_element` / `root_root_parent_sever_parent_path` to map the sever parent node back to the authoring UI structure.
    - `debug.all_layer_roots` (derived from `layers_in_paint_order`) makes the GC liveness roots explicit per snapshot.
    - `element_runtime.view_cache_reuse_root_element_samples` includes a per-reuse-root `(root_element -> node)` mapping plus a bounded head/tail sample of the recorded subtree element list.
    - `element_runtime.node_entry_root_overwrites` records `NodeEntry.root` ownership overwrites (element + old/new root + debug paths + callsite).
    - If these fields are missing in a failing bundle, it usually means: the debug identity entry was pruned (not touched for `gc_lag_frames`), or the parent never issued a `set_children(..)` write in the current capture.
  - Evidence (pass under reuse + shell):
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-torture.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
    - Re-verified (2026-01-25): PASS with sweep enabled under cache+shell (stopgap removed):
      - `target/fret-diag-cache005-stopgap-removed-overlay-1769334929510/1769335008125-ui-gallery-overlay-torture/bundle.json`
      - `target/fret-diag-cache005-stopgap-removed-sidebar-1769335037056/1769335040362-ui-gallery-sidebar-scroll-refresh/bundle.json`
    - Re-verified (2026-01-31): PASS with `--check-gc-sweep-liveness` and stale-paint under cache+shell:
      - `target/fret-diag/1769824052379-ui-gallery-overlay-torture/bundle.json`
    - Re-verified (2026-02-02): PASS via `fretboard diag suite ui-gallery-cache005` (warmup=5, cache+shell, `--check-gc-sweep-liveness` defaulted for cache-005 scripts):
      - `target/fret-diag-suite-cache005-local1/1770011515732-ui-gallery-overlay-torture/bundle.json`
      - `target/fret-diag-suite-cache005-local1/1770011541379-ui-gallery-sidebar-scroll-refresh/bundle.json`
    - Re-verified (2026-02-02): PASS (cache+shell) with taxonomy evidence exported by the gate:
      - `target/fret-diag-suite-ui-gallery-cache005-cache-shell/1770035888620-script-step-0018-press_key/bundle.json`
      - `target/fret-diag-suite-ui-gallery-cache005-cache-shell/1770035916581-script-step-0007-wheel/bundle.json`
      - `target/fret-diag-suite-ui-gallery-cache005-cache-shell/1770035888620-script-step-0018-press_key/check.gc_sweep_liveness.json`
      - `target/fret-diag-suite-ui-gallery-cache005-cache-shell/1770035916581-script-step-0007-wheel/check.gc_sweep_liveness.json`
    - Re-verified (2026-01-24): PASS on this branch with the stopgap still enabled:
      - `target/fret-diag-overlay-torture-cache005-stopgap/1769240992350-ui-gallery-overlay-torture/bundle.json`
      - `target/fret-diag-sidebar-scroll-refresh-cache005-stopgap/1769241046228-ui-gallery-sidebar-scroll-refresh/bundle.json`
    - Re-verified (2026-01-24): PASS with `debug.overlay_policy_decisions` exported:
      - `target/fret-diag-overlay-torture-cache005-overlay-policy/1769266152963-ui-gallery-overlay-torture/bundle.json`
  - Historical failing evidence (pre-fix):
    - Removing the `view_cache_has_reuse_roots` GC guard regresses `ui-gallery-overlay-torture.json` at step 10 (`click_no_semantics_match`):
      - `target/fret-diag-overlay-torture-cache005/1769240888633-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag-overlay-torture-cache005-newdiags/1769260341209-script-step-0010-click-no-semantics-match/bundle.json`
    - Historical repro note: early iterations used `FRET_UI_DISABLE_VIEW_CACHE_GC_STOPGAP=1` to emulate stopgap removal; the env var has since been removed along with the stopgap.
      - `target/fret-diag-overlay-torture-cache005-stopgap-disabled/1769266528812-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag-overlay-torture-cache005-stopgap-disabled-path-edges/1769304660925-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag-overlay-torture-cache005-stopgap-disabled-vc-reachability/1769307484613-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag-overlay-torture-cache005-stopgap-disabled-trigger-element-path/1769308808141-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag-overlay-torture-cache005-stopgap-disabled-vc-elements/1769315204903-script-step-0010-click-no-semantics-match/bundle.json` (includes `element_runtime.view_cache_reuse_root_element_samples`)
  - Next:
    - In the failing bundle above, `ui-gallery-dialog-trigger` exists up to `frame_id=33`, then disappears on `frame_id=34` when `debug.removed_subtrees.len()` spikes (31).
      Use `frame_id=34` `debug.removed_subtrees[*].root/root_element/root_path` as the entry point for root-cause analysis.
    - Observation (2026-01-25): in the `stopgap-disabled-path-edges` bundle, the subtree containing `ui-gallery-dialog-trigger` is removed as part of the max-`removed_nodes` record:
      - `frame_id=34`: `removed_subtrees[*].root=4294967755`, `root_root=4294967733`, `root_layer=None`, `reachable_from_layer_roots=false`.
      - `root_path_edge_ui_contains_child` and `root_path_edge_frame_contains_child` are all `1` for the full `root_path` hop chain, which suggests the subtree is internally consistent,
        but the top `root_root` is an "island root" (not reachable from current layer roots), so it is treated as detached and swept.
    - Next unblock target: explain why `root_root` becomes an island root on the failing frame (lost attachment to layer roots / window root), then make that attachment or liveness explicit
      so we can remove the global `view_cache_has_reuse_roots` stopgap.
    - Observation (2026-01-25): in the `stopgap-disabled-vc-reachability` bundle, the max-`removed_nodes` record reports:
      - `reachable_from_layer_roots=false` and `reachable_from_view_cache_roots=false`, which indicates the swept subtree is *not* reachable from any current layer root or any view-cache reuse root node.
      - This points to a **true island** per ADR 0191’s taxonomy: liveness-root selection, ownership drift, or subtree membership bookkeeping under reuse (not just “a parent pointer broke”).
    - Observation (2026-01-25): in the `stopgap-disabled-trigger-element-path` bundle, the sweep is attributed to:
      - `trigger_element_root_path = root[fret-ui-gallery]`, i.e. the window root pass, not an overlay-only root.
    - Observation (2026-01-24): in the failing bundles above, `ui-gallery-dialog-trigger` is last present at `frame_id=33` and is removed as part of the max-`removed_nodes` subtree at `frame_id=34`
      (the node id `4294967671` appears in `debug.removed_subtrees[*].removed_head`).
      The record reports `root_layer=None` and `reachable_from_layer_roots=false`, so the subtree is treated as detached at sweep time (not just missing a `last_seen_frame` touch).
    - Note (2026-01-24): `debug.layer_visible_writes` attributes the `visible=false` toggle for the `4294967718` layer root to `ecosystem/fret-ui-kit/src/window_overlays/state.rs:159`.
    - Note (2026-01-24): `debug.overlay_policy_decisions` attributes overlay-manager policy (kind/present/interactive/reason) with an explicit callsite.
      - Regression triage checklist (only relevant if a new failing bundle appears):
        - Confirm whether the removed subtree was still reachable from any liveness root:
          - `debug.removed_subtrees[*].reachable_from_layer_roots` / `reachable_from_view_cache_roots`.
        - If the subtree is still reachable from a layer root, treat it as a reachability/classification bug:
          - Audit `debug.layers_in_paint_order[*].visible`, `debug.layer_visible_writes[*]`, and `debug.overlay_policy_decisions[*]`.
          - Audit parent edge consistency via `root_parent_children_contains_root` / `root_parent_frame_children_contains_root`.
        - If the subtree is an island (both reachability flags are `false`), treat it as bookkeeping drift:
          - Audit `element_runtime.view_cache_reuse_root_element_samples` and `element_runtime.node_entry_root_overwrites`.
          - If present, use `root_root_parent_sever_*` / `root_parent_children_last_set_*` attribution fields to find the first structural detach that created the island.
          - Audit cache-root `set_children` attribution via `debug.cache_roots[*].children_last_set_*`.
        - If the subtree is a true structural detach, attribute it to a callsite:
          - Use `root_root_parent_sever_*` and parent element debug-path fields.
      - If `root_element_path` stays `None`, extend the diagnostics lag window or capture debug paths at removal time so we can map swept subtrees back to authoring callsites.
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
  - v1 key: `hash(theme_revision, scale_factor, cache_root_bounds.size, ViewCacheProps.cache_key)` (currently width/height only).
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

- [~] GPUI-MVP3-virt-002 VirtualList: reduce rerender cost during scroll via incremental range reuse (GPUI-component parity).
  - Motivation: `ui-gallery-virtual-list-torture.json` remains layout-dominated even with view-cache + shell reuse.
  - Progress: measured-mode virtual lists skip redundant per-frame `measure_in` passes for already-measured, clean visible rows.
  - Progress: re-measure is forced when the cross-axis viewport extent changes or when a row is layout-invalidated.
  - Progress: apply scroll offsets via a children-only render transform (content-space child bounds), matching `Scroll` and avoiding translation-only layout rect churn.
  - Progress: wheel scroll invalidation is `HitTestOnly` (scroll updates coordinate mapping + paint without forcing a layout pass).
  - Progress: view-cache roots no longer rerender on scroll-handle `HitTestOnly` invalidations (only `Layout` invalidations with `detail=ScrollHandle` force rerender).
  - Progress: VirtualList wheel handling detects when the mounted visible range no longer covers the desired range and notifies the nearest cache root for a one-shot rerender (avoids per-frame rerenders but keeps virtualization correct under `HitTestOnly` scroll invalidation).
  - Progress: scroll-handle-driven `HitTestOnly` invalidations (e.g. scrollbar wheel/drag) also run the same range-escape check during event dispatch, so cache-hit frames can schedule a one-shot rerender even when the VirtualList widget does not handle the wheel event directly.
  - Progress: treat VirtualList visible-range structural churn as a layout barrier (avoid forcing ancestor relayout on child list changes; schedule a contained relayout for the list itself).
  - Progress: export which view-cache roots were re-laid out via the contained-relayout post-pass to diagnostics bundles (`debug.cache_roots[].contained_relayout_in_frame`) and surface it in `fretboard diag stats/perf` (`cache.contained_relayout_roots`, `top_contained_relayout_cache_roots`) so slow scroll frames are explainable and actionable.
  - Progress: avoid scroll-handle revision churn from runtime layout bookkeeping (viewport/content size writes do not bump revisions), and classify offset-driven scroll changes as `HitTestOnly` even when other scroll-handle fields update in the same frame (reduces spurious layout-driven rerender waves during steady scroll under view-cache reuse).
  - Progress: keep scroll-handle value baselines (offset/viewport/content) in sync even when the revision does not change, so a later revision-only bump (e.g. deferred `scroll_to_item`) is classified as `Layout` and still runs the layout consumption path.
  - Progress: fix focus/click scroll-into-view to map the viewport into the same (unscrolled) content coordinate space as `descendant_bounds` (prevents scroll jumps under render transforms).
  - Progress: wrap heavyweight rows in per-row view-cache roots (keyed by item key) so steady-state scroll can rerender the list shell without rebuilding row subtrees.
  - Evidence: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (measurement gate, content-space bounds + `scroll_child_transform`), `crates/fret-ui/src/virtual_list.rs` (cross-extent measurement reset), `crates/fret-ui/src/declarative/host_widget/paint.rs` (paint applies `children_render_transform`), `crates/fret-ui/src/declarative/host_widget.rs` (scroll-into-view viewport mapping), `crates/fret-ui/src/declarative/host_widget/event/scroll.rs` (wheel `HitTestOnly`), `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`virtual_list_skips_redundant_measures_for_clean_measured_rows`, `virtual_list_scroll_offsets_apply_in_semantics_snapshot`, `virtual_list_click_focus_does_not_trigger_scroll_jump_under_children_transform`, `virtual_list_scroll_transform_does_not_double_transform_per_row_clip_rects`).
  - Evidence: `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`virtual_list_row_view_cache_reuses_rows_across_small_scroll_deltas`), `apps/fret-ui-gallery/src/ui.rs` (virtual list torture rows wrapped in `cached_subtree`).
  - Evidence: `crates/fret-ui/src/declarative/frame.rs` (`scroll_handle_revision_only_bumps_after_internal_offset_updates_classify_as_layout`), `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`virtual_list_can_scroll_to_deep_index_then_to_end`).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`set_children_barrier`, `take_pending_barrier_relayouts`, `invalidation_marks_view_dirty`), `crates/fret-ui/src/tree/layout.rs` (`layout_pending_barrier_relayouts_if_needed`, fixed-mode `scroll_to_item` early consumption), `crates/fret-ui/src/tree/dispatch.rs` (dispatch-time range escape notify for `HitTestOnly` invalidations), `crates/fret-ui/src/declarative/mount.rs` (`VirtualList` uses barrier set-children when axis size is layout-definite), `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_scroll_handle_hit_test_only_does_not_mark_root_dirty`, `view_cache_scroll_handle_layout_invalidation_marks_root_dirty`), `crates/fret-ui/src/declarative/host_widget/event/scroll.rs` (`VirtualList` wheel range escape notify), `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached`, `virtual_list_triggers_visible_range_rerender_on_scrollbar_wheel_when_cached`, `virtual_list_fixed_scroll_to_item_does_not_force_layout_invalidation`).
  - Perf snapshot (release, `--warmup-frames 5`, `--sort time`, `--repeat 7`; nearest-rank p50/p95; updated after per-row view-cache roots):
    - Torture (`tools/diag-scripts/ui-gallery-virtual-list-torture.json`):
      - Baseline: `p50.total_time_us=30326` `p95.total_time_us=32747` (p50 layout `28856`, prepaint `36`, paint `1391`) (run dir: `target/fret-diag-perf-vlist-torture-baseline-r7-0125-1542`).
      - ViewCache+Shell: `p50.total_time_us=29842` `p95.total_time_us=31073` (p50 layout `28267`, prepaint `24`, paint `1496`) (run dir: `target/fret-diag-perf-vlist-torture-cache-shell-r7-0125-1549`).
      - Note: wall-clock timings are noisy; use repeat percentiles for comparisons, and expect occasional spikes.
    - Smooth wheel (`tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json`):
      - Baseline: `p50.total_time_us=23589` `p95.total_time_us=24397` (p50 layout `23282`, prepaint `29`, paint `277`) (run dir: `target/fret-diag-perf-vlist-smooth-baseline-r7-0125-1854`).
      - ViewCache+Shell: `p50.total_time_us=26188` `p95.total_time_us=27472` (p50 layout `25309`, prepaint `26`, paint `861`) (run dir: `target/fret-diag-perf-vlist-smooth-cache-shell-r7-0125-1857`).
      - Note: the smooth-wheel script primes scroll first, then uses `reset_diagnostics` before the measured segment so the bundle captures steady-state scroll behavior (not the initial range-refresh / mount churn).
  - Commands:
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-torture-baseline-r7 --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 12 --repeat 7 --json perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-torture-cache-shell-r7 --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 12 --repeat 7 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-smooth-baseline-r7 --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 15 --repeat 7 --json perf tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json --launch -- cargo run -p fret-ui-gallery --release`
    - `cargo run -p fretboard -- diag --dir target/fret-diag-perf-vlist-smooth-cache-shell-r7 --timeout-ms 300000 --poll-ms 200 --warmup-frames 5 --sort time --top 15 --repeat 7 --json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 perf tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json --launch -- cargo run -p fret-ui-gallery --release`
  - Notes:
    - The current per-row cache roots help reduce subtree rebuild cost, but the measured scroll paths are still layout-dominated.
    - The smooth wheel scenario improved after stopping view-cache rerenders on scroll-handle `HitTestOnly` invalidations, but long-term we still want the range bookkeeping to move earlier (prepaint) so the “range delta” path stays cheap.
    - Correctness: `fretboard diag compare target/fret-diag-vlist-torture-uncached-r1 target/fret-diag-vlist-torture-cache-shell-r1 --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint` reports `ok=true`.
  - Checklist:
    - [x] Avoid redundant per-row `measure_in` for clean measured rows (measured-mode gate).
    - [x] Apply scroll offsets via a children-only render transform (content-space bounds).
    - [x] Treat visible-range set-children churn as a layout barrier.
    - [x] Wrap heavyweight rows in per-row view-cache roots (stable per-item identity).
    - [x] Export VirtualList scroll/range diagnostics counters (range recompute, children churn, barrier relayouts) to bundles to make regressions explainable.
      - Fields: `set_children_barrier_writes`, `barrier_relayouts_scheduled`, `barrier_relayouts_performed`, `virtual_list_visible_range_checks`, `virtual_list_visible_range_refreshes`.
      - Tooling: `fretboard diag perf --json` includes these counters in both per-run rows and summarized percentiles (so perf diffs don't require manually inspecting bundle dumps).
      - Evidence: `crates/fret-ui/src/tree/mod.rs` (`UiDebugFrameStats`, `debug_record_virtual_list_visible_range_check`),
        `crates/fret-ui/src/tree/layout.rs` (`barrier_relayouts_performed`),
        `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1` export),
        `crates/fret-ui/src/declarative/host_widget/paint.rs` (range checks).
    - [x] Export contained-relayout cache-root hotspots to bundles and tooling (debug visibility for scroll perf regressions).
      - Fields: `debug.cache_roots[].contained_relayout_in_frame`, `fretboard diag stats/perf: cache.contained_relayout_roots`, `top_contained_relayout_cache_roots`.
      - Evidence: `crates/fret-ui/src/tree/layout.rs` (`layout_contained_view_cache_roots_if_needed` recording), `crates/fret-ui/src/tree/mod.rs` (`debug_view_cache_contained_relayout_roots`),
        `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export), `apps/fretboard/src/diag.rs` (stats/perf surfaces).
    - [x] Avoid scroll-handle revision churn from runtime layout bookkeeping (reduce rerender churn under view-cache reuse).
      - Touches: `crates/fret-ui/src/scroll.rs` (internal setters), `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (use internal setters), `crates/fret-ui/src/declarative/frame.rs` (change classification), `crates/fret-ui/src/tree/mod.rs` (`propagate_auto_sized_view_cache_root_invalidations` gating).
      - Evidence: `crates/fret-ui/src/scroll.rs` (`scroll_handle_internal_setters_do_not_bump_revision`), `crates/fret-ui/src/declarative/frame.rs` (`scroll_handle_changes_classify_value_changes_as_hit_test_only`), `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_auto_sized_repair_does_not_promote_hit_test_when_bounds_are_known`).
    - [x] Avoid view-cache rerenders on scroll-handle `HitTestOnly` invalidations (rerender only on `Layout` invalidations with `detail=ScrollHandle`).
    - [x] Trigger a one-shot rerender when the desired visible range escapes the mounted range (avoid per-frame rerenders but keep virtualization correct under wheel scroll).
    - [x] Move visible-range escape detection toward the runtime dispatch path (GPUI-style "range delta" gate), so scroll-handle changes can schedule a one-shot rerender even when the VirtualList widget does not handle the event directly.
    - [x] Use contained-relayout cache-root hotspot diagnostics to reduce post-pass contained relayouts during steady scroll (target: `cache.contained_relayout_roots` stays near 0 for smooth-wheel frames under view-cache + shell).
      - Evidence: in `tools/diag-scripts/ui-gallery-virtual-list-smooth-scroll.json` with `FRET_UI_GALLERY_VIEW_CACHE=1` + `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`, contained relayout is only observed on the two wheel frames where the mounted range escapes the overscan window (typical run: 2/18 snapshots; max 2 roots), e.g. `target/fret-diag-perf-vlist-smooth-cache-shell-r3/*-script-step-0023-wheel/bundle.json`.
    - [~] Move `scroll_to_item` consumption earlier than layout where possible (fixed-mode early consumption; measured-mode still consumed during final layout).
    - [x] Repeat the perf runs (baseline vs cache+shell) and update the p50/p95 snapshots after each structural change (see run dirs above).
  - Sketch (target shape):
    - Keep per-item identity stable (do not recycle cells) while making the “range delta” path cheap.
    - Make “range delta” a prepaint-plan update whenever possible (offset/visible range), and keep layout limited to true geometry changes.
- [~] GPUI-MVP3-rec-001 Define the minimal interaction stream vocabulary for replay.
  - Candidates: hit regions, cursor requests, outside-press observers, focus traversal roots.
  - Touches: `crates/fret-ui/src/tree/*`, `crates/fret-core/src/*` (data-only shapes as needed)
  - Progress: add hit-test path reuse (cached “interaction range”) as an incremental, semantics-preserving step toward replayable interaction output.
  - Progress: add a pure cursor request hook (`Widget::cursor_icon_at`) and route pointer-move cursor updates through it when present (cursor requests are now representable without relying on pointer-move side effects).
  - Progress: cache focus traversal gates in prepaint (focusable + traversal + scroll-ancestor) so command availability queries do not re-enter widget hit-test hooks for clean nodes.
  - Progress: export outside-press observer layer metadata (consume flag + branch list) so pointer-down-outside arbitration is explainable from bundles.
  - Notes: reuse is currently enabled only for pointer-move dispatch; other pointer events rebuild the cache from a full hit-test pass.
  - Notes: reuse falls back to full hit-testing if the cached leaf can hit-test children (avoids stale routing when the pointer moves between descendants).
  - Evidence: `crates/fret-ui/src/tree/hit_test.rs` (`hit_test_layers_cached`, `try_hit_test_along_cached_path`),
    `crates/fret-ui/src/tree/dispatch.rs` (pointer-move-only reuse policy),
    `crates/fret-ui/src/tree/tests/hit_test.rs` (`hit_test_layers_cached_reuses_path_and_respects_layer_order`),
    `crates/fret-ui/src/widget.rs` (`Widget::cursor_icon_at`),
    `crates/fret-ui/src/tree/tests/cursor_icon_query.rs`,
    `crates/fret-ui/src/tree/prepaint.rs` (`InteractionRecord` focus flags),
    `crates/fret-ui/src/tree/tests/focus_traversal_prepaint_cache.rs`,
    `crates/fret-ui/src/tree/mod.rs` (`UiDebugLayerInfo` outside-press fields),
    `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiLayerInfoV1` export).
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
- [~] GPUI-MVP4-demo-002 Migrate `fret-ui-gallery` hotspots to the new patterns (hover chrome, scrollbars, code views).
  - Touches: `apps/fret-ui-gallery/src/*`, selected `ecosystem/*` components
- [~] GPUI-MVP4-demo-003 VirtualList: move “visible range” derivation from declarative render into prepaint-driven state.
  - Motivation: reduce cache-root rerenders and layout invalidations during scroll/hover by keeping the element tree more structurally stable.
  - Reference: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs` (prepaint-driven range + reuse)
  - Touches: `ecosystem/fret-ui-kit/src/*`, `apps/fret-ui-gallery/src/*` (migration site), `crates/fret-ui/src/tree/*` (if new hooks needed)
  - Plan (v1, Fret-compatible):
    - Ensure out-of-band `ScrollHandle` revision changes (e.g. `scroll_to_item`) are detected during event dispatch and schedule a redraw even when no
      `notify()` occurred, so view-cache roots cannot replay stale virtual-list output indefinitely.
    - Keep steady wheel scroll “transform-only” within overscan, but upgrade to a single view-cache rerender only when the range window actually changes.
    - Remove the dependency on `Pressable`'s implicit post-activation `notify()` for scroll-to-item correctness (the scroll handle becomes the driver).
  - Done when (v1):
    - `tools/diag-scripts/ui-gallery-virtual-list-torture.json` passes with view-cache + shell enabled, and out-of-band scroll-handle updates (e.g.
      `scroll_to_item`) cannot get “stuck” behind cache-hit frames.
  - Done when (v2):
    - VirtualList no longer relies on post-activation `Pressable` `notify()` for scroll-to-item correctness (the scroll handle becomes the driver).
    - Worst-tick bundles no longer attribute the dominant `notify_call` hotspot to `pressable.rs:*` for the VirtualList torture scenario.
  - Progress (v1):
    - Wheel scrolling is now **transform-only** (hit-test-only invalidation) while the visible window remains stable; layout invalidation is used only
      when the visible window actually changes.
    - VirtualList children are laid out in “content space” (not offset-adjusted), and the scroll offset is applied via `ScrollChildTransform` so paint
      and hit-testing can track offset changes without a relayout.
    - Declarative render now prefers a layout-derived `VirtualListState.window_range` and records `render_window_range` so layout can detect window
      mismatches and only force a view-cache rerender when needed.
    - `scroll_descendant_into_view` maps the VirtualList viewport into content space before computing the scroll delta (prevents runaway scroll offsets
      and “invisible but interactable” rows during focus traversal).
  - Evidence:
      - `crates/fret-ui/src/declarative/host_widget/event/scroll.rs` (wheel invalidation gate)
      - `crates/fret-ui/src/tree/layout.rs` (`invalidate_scroll_handle_bindings_for_changed_handles` upgrades VirtualList scroll to `Layout` only when the viewport leaves the last rendered overscan window)
      - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (content-space layout + window_range)
      - `crates/fret-ui/src/declarative/host_widget/paint.rs` (row paint under scroll transform + per-row clips)
      - `crates/fret-ui/src/declarative/mount.rs` (pre-render scroll-handle invalidation gate for view-cache reuse)
      - `crates/fret-ui/src/declarative/frame.rs` + `crates/fret-ui/src/tree/layout.rs` (scroll-handle change classification)
      - `crates/fret-ui/src/tree/mod.rs` (scroll-handle invalidation detail gates view-cache dirtiness)
      - `crates/fret-ui/src/elements/cx.rs` + `crates/fret-ui/src/element.rs` (window_range + render_window_range state)
      - Tests: `crates/fret-ui/src/tree/tests/scroll_invalidation.rs` (`scroll_wheel_invalidation_is_hit_test_only`, `virtual_list_wheel_scroll_is_hit_test_only_within_overscan_window`, `virtual_list_out_of_band_scroll_upgrades_to_layout_after_overscan_window`), `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`virtual_list_paint_clips_each_visible_row`), `crates/fret-ui/src/declarative/tests/view_cache.rs` (`view_cache_rerenders_on_virtual_list_scroll_to_item`), `crates/fret-ui/src/tree/tests/scroll_into_view.rs` (`focus_traversal_does_not_scroll_visible_virtual_list_descendant_into_view`)
      - Diagnostics: in an exported `ui-gallery-virtual-list-edit-9000` bundle, find a snapshot where
        `debug.virtual_list_windows[*].source=prepaint`, `deferred_scroll_consumed=true` and `window_mismatch=true`; the next snapshot should include a
        `debug.dirty_views` entry with `detail=scroll_handle_layout`, and `render_window_range` should match `window_range`.
      - Perf capture: `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --top 10 --sort time --warmup-frames 5 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery`
        produced worst bundle `target/fret-diag/1769096169296-script-step-0011-click/bundle.json` (top.us(total/layout/prepaint/paint)=503161/476991/241/25929).
      - Bundle note (v1): `debug.dirty_views` reports the cache roots that were already marked dirty at the **start** of that frame
        (pre-mount). If a frame shows `window_mismatch=false` but still lists `scroll_handle_layout` dirtiness, inspect the *previous*
        snapshot: the typical pattern is `tick=N` has `window_mismatch=true` (VirtualList consumed a deferred scroll and changed its window),
        then `tick=N+1` starts dirty to force a rerender and rebuild the visible rows.
        Track ongoing explainability work under `GPUI-MVP5-perf-003`.

## MVP5 — Prepaint-driven Ephemeral Windows (Beyond VirtualList)

Goal: converge on GPUI’s “stable feel + stable perf” loop by moving large virtual surfaces to **prepaint-driven visible
windows** + per-frame ephemeral items, while keeping caching gated by dirty views and explicit cache keys (ADR 0180/0182).

### Candidate Map (What Should Become “Windowed”)

This list is intentionally biased toward editor-scale performance. The rule of thumb is:

- if the surface’s visible content is primarily a function of **viewport + scroll/camera** (plus overscan),
- and we currently “solve correctness” by forcing cache-root rerenders/relayout on scroll,
- then it likely belongs to the **prepaint-windowed** bucket (ADR 0190).

Initial candidates (to be evidence-backed via `diag perf` bundles):

- **1D scroll window (rows/lines)**
  - Code/text: `ecosystem/fret-code-view` (code blocks), `ecosystem/fret-markdown` (code blocks + long documents)
  - Tables/trees: `ecosystem/fret-ui-kit` (table/tree virtualization)
  - Workspace/inspectors: large “property inspectors” and pane sidebars in `apps/fret-editor` and `ecosystem/fret-workspace`
  - Diagnostics: list-like surfaces in `apps/fretboard` + `ecosystem/fret-bootstrap` diagnostics panes
  - Search/commands: command palette, search results, outline panels (typically “list-of-rows” UIs with large datasets)
  - Undo/history: `ecosystem/fret-undo` (history lists with frequent selection changes)
- **2D viewport culling (nodes/sprites)**
  - Node graph / canvas: `ecosystem/fret-node`, `ecosystem/fret-canvas`
  - Gizmos/overlays: `ecosystem/fret-gizmo`, `ecosystem/fret-viewport-tooling`
- **Sampling windows (data reduction)**
  - Charts/plots: `ecosystem/fret-chart`, `ecosystem/fret-plot`, `ecosystem/fret-plot3d`
  - AI/chat transcripts: `ecosystem/fret-ui-ai` (large scrolling transcripts, incremental append, selection/search)

Non-candidates (usually): small forms/menus/popovers where the “ephemeral window” complexity would outweigh the wins.

- **Dirty-view rebuild (ADR 0180; structural)**
  - Overlay open/close, portal attach/detach, and overlay stack reordering.
  - Docking graph mutations (split/merge, tab reorder, tear-out/reattach).
  - Text/content reflow that changes layout structure (wrap width, font/style changes, inline reflow).
  - Large data set updates that change the shape/order of rows/nodes (search results, outlines, histories).

- **Paint-only chrome (should not rerender by default; ADR 0181)**
  - Caret/selection blink and selection geometry updates (text/code views).
  - Hover/pressed/focus ring decoration layers (shadcn-style interaction chrome).
  - Drag previews / drop indicators (docking, lists/trees).
  - Scrollbars and subtle scroll affordances (thumb hover/fade; scroll shadow/edge fades).
  - Overlay arrows/anchors (position updates that should not imply rerender when content is stable).

### Ecosystem Alignment Checklist (First Pass)

This checklist is a “directional classification” to keep refactors incremental and avoid future rewrites. It is not
meant to be perfect on day one; it is meant to be explicit.

- **Prepaint-windowed (ADR 0190)**: viewport/scroll/camera primarily determines “what is visible”.
  - `VirtualList` (v2): visible window + ephemeral items move toward `prepaint`; scroll should stay transform-only while the view is clean.
  - Table/tree virtualization in `ecosystem/fret-ui-kit`: adopt the same window model to avoid rerendering huge row subtrees on scroll.
  - Code/text lines: `ecosystem/fret-code-view`, `ecosystem/fret-markdown`, `ecosystem/fret-syntax` (retained caches) with a windowed “line/run” surface.
  - 2D culling: `ecosystem/fret-node`, `ecosystem/fret-canvas`, `ecosystem/fret-viewport-tooling` (visible node/edge/handle windows).
  - Sampling/data windows: `ecosystem/fret-chart`, `ecosystem/fret-plot`, `ecosystem/fret-plot3d`, `ecosystem/delinea` (pan/zoom updates a sampling window).
  - Large transcripts/logs: `ecosystem/fret-ui-ai`, diagnostics panes in `ecosystem/fret-bootstrap`.

- **Paint-only by default (ADR 0181)**: visual refinement derived from pointer/focus/selection state.
  - Shadcn interaction chrome: `ecosystem/fret-ui-shadcn` (hover/pressed/focus rings, subtle transitions).
  - Docking chrome: `ecosystem/fret-docking` (drop indicators, drag previews, tab hover).
  - Text/code chrome: caret blink, selection highlight, IME underline/caret geometry (avoid `notify()` loops for blink/hover).
  - Scrollbars and edge fades: hover/fade should be paint-only; structural changes stay in layout.

- **Rerender on dirty view (ADR 0180)**: structural or semantic changes that must rebuild the declarative subtree.
  - Data/model changes that alter the element tree shape (insert/remove/reorder).
  - Overlay open/close (but keep overlay positioning/arrow placement paint-only when possible).
  - Docking layout graph changes (but keep drag previews paint-only).

- **Retained caches (cross-frame, must persist)**: moving these to “rebuild each frame” is usually a perf regression.
  - Text shaping / line breaking caches, syntax parsing state (`ecosystem/fret-syntax`, renderer text systems).
  - GPU caches/atlases (`crates/fret-render`, `ecosystem/fret-ui-assets`).
  - Stable identity + element state stores (ADR 0028 / ADR 1151 / ADR 1152).

### ADR Follow-ups (Create Only When We Cross a “Hard-to-change” Boundary)

We should add a new ADR only when we are about to lock in behavior that would be expensive to change later. Likely ADR
topics (if/when we implement them):

- The exact contract and liveness rules for “ephemeral prepaint items” (debuggability, replay caching, inspector behavior).
- Cache key gates for reuse beyond `layout` invalidation (bounds/text style/content mask parity with GPUI).
- “Inspector mode disables caching” semantics (GPUI does this; helps explainability and avoids unstable debug identity).

- [x] GPUI-MVP5-core-000 Define the “ephemeral prepaint items” contract and debug surfaces.
  - Goal: we can explain “why did the virtual window change” and “why did we rerender” in exported diagnostics bundles.
  - Touches: `crates/fret-ui/src/tree/prepaint.rs`, `crates/fret-ui/src/tree/mod.rs`, diagnostics export in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.
  - Contract: `docs/adr/0193-ephemeral-prepaint-items-v1.md` (Accepted).
  - Notes: ADR 0190 is now Accepted as the guiding contract; capture any new “hard-to-change” commitments as follow-up ADRs if needed.
  - Progress (v1):
    - Bundles can export VirtualList window telemetry via `UiTreeDebugSnapshotV1.virtual_list_windows` (debug-only, bounded) for postmortem analysis.
    - Bundles expose `debug.dirty_views[*].detail` to distinguish `scroll_handle_hit_test_only` vs `scroll_handle_layout`, making “why did this cache
      root rerender?” explainable for VirtualList scroll/scroll_to_item flows.
    - Bundles export `debug.prepaint_actions` (bounded) so prepaint-driven invalidations and scheduling requests are explainable without rerunning under a debugger.
      - Anchors: `crates/fret-ui/src/widget.rs` (`PrepaintCx`), `crates/fret-ui/src/tree/mod.rs` (`debug_prepaint_actions`), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.
    - `Widget::prepaint(PrepaintCx)` hook exists and is invoked for view-cache roots during the prepaint pass, even when the interaction cache replays.
      - Anchors: `crates/fret-ui/src/widget.rs` (`PrepaintCx`, `Widget::prepaint`),
        `crates/fret-ui/src/tree/prepaint.rs` (prepaint traversal),
        `crates/fret-ui/src/tree/tests/prepaint.rs` (`prepaint_hook_runs_for_view_cache_root_even_when_reusing_interaction_cache`).
    - Prepaint can stash per-cache-root ephemeral outputs keyed by the cache root's prepaint key.
      - This is a minimal “ephemeral prepaint items” substrate: it lets prepaint compute paint-only state without
        mutating the declarative structure, and makes reuse safe by clearing outputs when the cache root key changes.
      - Anchors: `crates/fret-ui/src/tree/mod.rs` (`PrepaintOutputs`), `crates/fret-ui/src/widget.rs` (`PrepaintCx::set_output`, `PrepaintCx::output`),
        `crates/fret-ui/src/tree/tests/prepaint.rs` (`prepaint_output_store_is_keyed_by_cache_root_prepaint_key`).
    - Paint-cache replay now keeps descendant bounds in sync when a cached subtree translates (required for correct hit-testing + semantics under caching).
      - Anchors: `crates/fret-ui/src/tree/paint.rs` (paint-cache replay translates descendant bounds),
        `crates/fret-ui/src/tree/tests/paint_cache.rs` (`paint_cache_replay_translates_descendant_bounds_for_descendants`).
  - Definition of done (v1; mark `[x]` when all are true):
    - [x] There is at least one end-to-end harness where a cache root stays clean (no rerender), but frame-local behavior still updates correctly via prepaint hooks and/or prepaint outputs
      (e.g. drag indicators, hover chrome, windowed surface telemetry).
    - [x] A single `bundle.json` contains enough evidence to explain:
      - why a prepaint output changed (inputs + key),
      - why a cache root rerendered (if it did), and
      - what prepaint requested (invalidate/redraw/RAF).
    - [x] There is a stable post-run gate for the chosen harness in `fretboard diag stats` (or `diag suite`) so regressions are caught without manual inspection.
  - Next steps (to close core-000, keep minimal and test-first):
    - [x] Pick the closing harness: `tools/diag-scripts/docking-demo-drag-indicators.json` (paint-only chrome under view-cache reuse)
    - [x] Add one small gate that asserts “prepaint acted” in the relevant frames (bounded) in addition to existing `--check-stale-paint` / view-cache gates.
      - Gate: `--check-prepaint-actions-min 1` (counts snapshots with non-empty `debug.prepaint_actions` after warmup).
    - [x] Record a fresh passing evidence bundle and link it here.
      - Command:
        - `cargo run -p fretboard -- diag run tools/diag-scripts/docking-demo-drag-indicators.json --warmup-frames 5 --check-prepaint-actions-min 1 --check-drag-cache-root-paint-only dock-demo-dock-space --check-stale-paint dock-demo-dock-space --env FRET_EXAMPLES_VIEW_CACHE=1 --launch -- cargo run -p fret-demo --bin docking_demo --release`
      - Evidence: `target/fret-diag/1769776050260-docking-demo-drag-indicators/bundle.json`
  - [~] GPUI-MVP5-virt-001 VirtualList: prepaint-driven visible-range window + overscan stability.
  - Goal: wheel scroll stays “transform-only” until the range window actually changes; avoid view-cache rerenders for small scroll deltas.
  - Reference: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs` (prepaint-driven range + reuse)
  - Touches: `ecosystem/fret-ui-kit/src/*`, `crates/fret-ui/src/tree/prepaint.rs`, `apps/fret-ui-gallery/src/*`
    - Current (v1): `VirtualList`’s `visible_items` are computed during declarative render (`crates/fret-ui/src/elements/cx.rs`), so changing the
      visible window requires a cache-root rerender to rebuild the item subtree. The v2 goal is to move “window derivation + ephemeral items”
      into prepaint (ADR 0190), so scroll-driven window updates do not necessarily imply a cache-root rerender.
    - Remaining work (non-retained v2; keep small and gate-driven):
      - Scope note: for *fully composable* row subtrees, “apply a window shift without rerendering the parent cache root” effectively requires a retained host boundary (ADR 0192 / `GPUI-MVP5-virt-003`).
        This track therefore focuses on: (1) prepaint deriving the *next desired window* (explainable), and (2) making escape ticks cheap and predictable (one-shot rerender, no extra current-frame churn).
      - [~] Define the non-retained v2 contract precisely in ADR 0190/0193 terms (what is allowed to change in prepaint without rerender, what must schedule a dirty-view rerender, and what is “retained-host only”).
        - [x] Make Track B scheduling responsibility explicit and prepaint-driven (avoid duplicated layout side effects).
          - ADR: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md` (“Scheduling responsibility” under Track B).
          - Anchors: `crates/fret-ui/src/tree/prepaint.rs` (non-retained schedules one-shot rerender on `window_shift_kind!=none`),
            `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (layout no longer duplicates rerender scheduling on `window_mismatch`).
          - Note: if `render_window_range` is missing (common when a cache-hit frame skips declarative render during the initial viewport bootstrap),
            prepaint must treat this as a mismatch so the next frame can mount the correct row subtree under view-cache reuse.
        - Specify the allowed "ephemeral prepaint updates" for the non-retained VirtualList path (e.g. window prediction, scroll-offset bookkeeping, one-shot escape scheduling), and explicitly forbid any attach/detach that would require remounting children without a rerender boundary.
        - Define the required explainability surface: every window change MUST correspond to a named reason and MUST be visible in one `bundle.json` (and in post-run gates).
        - References: ADR 0190 §3A + §4 (v2 explainability) and ADR 0193 §2A (“window plans” vs structural mutation).
      - [x] Make window shifts explainable from one bundle (kinds: `prefetch`/`escape`; reasons: `scroll_offset`/`viewport_resize`/`items_revision`/`scroll_to_item`), and ensure they line up with the invalidation detail that dirtied the cache root.
        - [x] Add a per-frame debug counter + samples for non-retained window shifts (kind + old/new range + triggering invalidation detail), similar to retained-host reconcile samples.
          - Exports:
            - `debug.stats.virtual_list_window_shifts_total` / `debug.stats.virtual_list_window_shifts_non_retained`
            - `debug.virtual_list_window_shift_samples` (bounded; only records non-retained rerender shifts)
          - Anchors:
            - `crates/fret-ui/src/tree/mod.rs` (`UiDebugFrameStats.virtual_list_window_shifts_*`, `UiDebugVirtualListWindowShiftSample`)
            - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiTreeDebugSnapshotV1.virtual_list_window_shift_samples`, `UiFrameStatsV1.virtual_list_window_shifts_*`)
        - [x] Ensure `debug.prepaint_actions` includes a `virtual_list_window_shift` action (kind + reason).
          - Anchors: `crates/fret-ui/src/tree/prepaint.rs` (records window shift action), `crates/fret-ui/src/tree/mod.rs` (`UiDebugPrepaintActionKind::VirtualListWindowShift`), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export).
          - Gate: `--check-vlist-window-shifts-have-prepaint-actions` (emits `check.vlist_window_shifts_have_prepaint_actions.json`).
        - Gate (retained-host safety): `--check-vlist-window-shifts-non-retained-max 0` (fails if a retained-host window shift falls back to `non_retained_rerender`; emits `check.vlist_window_shifts_non_retained_max.json`).
        - Evidence (suite; cache+shell, release): `target/fret-diag-suite-ui-gallery-vlist-window-boundary-smoke2/1770082913519-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
      - [x] Add a stable regression gate for the window-boundary harness:
        - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json`
        - Recommended env: `FRET_UI_GALLERY_VLIST_RETAINED=0`, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`
        - Gate target: “no rerender until escape” (dirty views budget) + stale-paint + prepaint actions.
        - Gate (new): `--check-vlist-visible-range-refreshes-min 1` (counts `debug.stats.virtual_list_visible_range_refreshes` after the first wheel event; emits `check.vlist_visible_range_refreshes_min.json`).
        - Gate (new): `--check-vlist-visible-range-refreshes-max 35` (counts `debug.stats.virtual_list_visible_range_refreshes` after the first wheel event; emits `check.vlist_visible_range_refreshes_max.json`).
        - Gate (new): `--check-vlist-window-shifts-explainable` (requires `debug.virtual_list_windows` to include `window_shift_kind` + `window_shift_reason` + `window_shift_apply_mode`; and for non-retained shifts, requires `window_shift_invalidation_detail` to match the shift reason when it is specific (`scroll_to_item`/`viewport_resize`/`items_revision`) or fall back to the shift kind (`prefetch`/`escape`); emits `check.vlist_window_shifts_explainable.json`).
        - Gate (new): `--check-vlist-window-shifts-have-prepaint-actions` (requires that every `source=prepaint` window shift has a matching `debug.prepaint_actions` entry; emits `check.vlist_window_shifts_have_prepaint_actions.json`).
        - Suite default: `fretboard diag suite ui-gallery-vlist-window-boundary` applies:
          `--check-stale-paint ui-gallery-virtual-list-root`, `--check-view-cache-reuse-min 1`,
          `--check-vlist-visible-range-refreshes-min 1`, `--check-vlist-visible-range-refreshes-max 35`,
          and `--check-vlist-window-shifts-explainable` + `--check-vlist-window-shifts-have-prepaint-actions` unless explicitly overridden.
          - Note: this suite intentionally does *not* default to `--check-wheel-scroll`, since VirtualList scroll can keep hit-test results within the root container.
  - Note: the paint-driven path (e.g. `windowed_rows_surface`) already satisfies ADR 0190 for fixed-height surfaces. For fully composable
     row subtrees, we need a retained host boundary so cache-hit frames can attach/detach items without rerendering the parent cache root
     (tracked in ADR 0192).
  - Stopgap (non-retained path; implemented): prefetch window shifts before `window_mismatch` so boundary work happens earlier / more often,
    reducing the likelihood of a single large “escape” tick.
    - This is intentionally *not* the final GPUI parity answer: it still relies on cache-root rerenders to rebuild `visible_items`.
    - Anchors:
      - `crates/fret-ui/src/tree/prepaint.rs` (non-retained prefetch; `window_shift_kind=prefetch`)
      - `crates/fret-ui/src/tree/mod.rs` (`UiDebugInvalidationDetail::ScrollHandlePrefetchWindowUpdate`)
      - `apps/fretboard/src/diag/stats.rs` (vlist visible-range refresh gates: `--check-vlist-visible-range-refreshes-min`, `--check-vlist-visible-range-refreshes-max`)
      - `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json` (tuned to trigger prefetch reliably)
    - Evidence (suite; cache+shell, release; prefetch-min + prefetch-dirty budget gated; non-retained default):
      - `target/fret-diag-suite-ui-gallery-vlist-window-boundary-nonretained0/1770084994647-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
  - Scope note: this item intentionally tracks **two** related tracks:
    - **Retained host windowed surfaces (ADR 0192)**: prepaint derives window shifts, and a retained reconcile attaches/detaches rows without
      dirtying the parent cache root. This track is now mostly about tuning and hardening (e.g. staged prefetch + budgets).
    - **Non-retained `VirtualList` path**: today, window changes still imply a cache-root rerender to rebuild `visible_items`. This is the main
      remaining gap to reach “GPUI-like scroll stays cheap even when the window shifts”.
  - Execution plan (v2 slices; keep incremental, avoid a big-bang rewrite):
    - [x] Pick one primary consumer surface (start with file tree, then inspector/table/tree).
      - Anchors: `ecosystem/fret-ui-kit/src/declarative/file_tree.rs` (`file_tree_view_retained_v0`),
        `apps/fret-ui-gallery/src/ui.rs` (`preview_file_tree_torture`).
      - Note: retained-host consumers MUST provide a meaningful `VirtualListOptions.items_revision` so window updates and key caches are explainable.
        - Evidence: `ecosystem/fret-ui-kit/src/declarative/file_tree.rs` folds tree + state revisions into `items_revision`.
    - [x] (Retained host) Move “window derivation” to prepaint (ADR 0190 / ADR 0193), keyed by: viewport + offset + overscan + items revision.
      - Current: prepaint updates `VirtualListState.window_range` from interaction records and scroll-handle state (no rerender required for retained hosts).
      - Next (retained host): lift/standardize the derived window into explicit prepaint outputs where needed, and keep the “why did the window change?” story fully explainable from one bundle.
      - Diagnostics: `debug.virtual_list_windows[*]` now include `policy_key` / `inputs_key` and the policy inputs (`estimate_row_height`, `gap`, `scroll_margin`, `content_extent`) so “policy changed vs. scroll changed” is distinguishable.
          - Evidence: `target/fret-diag-vlist-window-keys/1769858602068-components-gallery-file-tree-window-boundary-bounce/bundle.json`
        - Regression: prepaint detects when a render-derived window (from a smaller viewport) underestimates the final visible span while still
          appearing “within overscan”, and stages a one-shot `prefetch` shift attributed to `viewport_resize`.
          - Anchor: `crates/fret-ui/src/tree/prepaint.rs` (`prepaint_detects_render_window_insufficient_for_overscan_policy`)
      - Perf story (baseline): window-boundary frames are still layout-dominated mainly because retained-host reconcile can attach/detach multiple row subtrees in one tick.
        - Evidence (same bundle; warmup=5):
          - Worst layout frame: `frame_id=24` (`tick_id=25`) `layout_time_us=2560` with `retained_virtual_list_attached_items=9`, `detached_items=3` (delta=12) and `barrier_relayouts_performed=1`.
          - Worst attach/detach frame: `frame_id=28` (`tick_id=29`) `attached_items=10`, `detached_items=10` (delta=20) with `layout_time_us=1557`.
        - Takeaway: to reduce worst-tick layout time further, we either need to (a) reduce per-frame attach/detach delta (more frequent smaller shifts / staged prefetch), or (b) reduce the cost of attaching new rows (row recycling, cheaper row layout, or more effective keep-alive reuse).
    - [!] GPUI-MVP5-virt-004 (Optional; deferred) Non-retained window shifts without rerender.
      - Rationale: fully composable “window shifts without dirtying the parent cache root” require an explicit retained-host
        boundary (ADR 0192). Track B remains “plan-only”: it may derive a window plan in prepaint, but it must schedule a
        one-shot rerender to rebuild `visible_items` when the window actually changes.
      - If revisited, prefer implementing this as an explicit boundary/host (not structural mutation in prepaint) so the
        result remains diagnosable and compatible with view-cache reuse + GC invariants (ADR 0191/0193).
    - [x] Add staged prefetch (ADR 0190 v2 addendum): shift retained-host windows *before* `window_mismatch` and reconcile incrementally.
      - Idea: when the visible range approaches the prefetch boundary (but is still covered), shift the window by a small bounded step and request redraw.
      - Goal: turn “one big boundary tick” into a bounded stream of small reconciles, reducing worst-tick spikes.
      - Anchors:
        - `crates/fret-ui/src/tree/prepaint.rs` (compute `window_shift_kind` and schedule retained-host reconciles even when `window_mismatch == false`)
        - `crates/fret-ui/src/virtual_list.rs` (`prefetch_virtual_range_step` helper for bounded window shifting)
        - `crates/fret-ui/src/elements/runtime.rs` (pending retained-host reconcile queue carries `Prefetch` vs `Escape`)
        - `crates/fret-ui/src/declarative/mount.rs` (`reconcile_retained_virtual_list_hosts` records `reconcile_kind`)
        - `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`retained_virtual_list_prefetches_window_before_escape_without_rerendering_cache_root`)
      - Diagnostics + gates:
        - `debug.virtual_list_windows[*].window_shift_kind` (`none`/`prefetch`/`escape`) explains why a shift occurred.
        - `debug.retained_virtual_list_reconciles[*].reconcile_kind` (`prefetch`/`escape`) attributes attach/detach work to the right cause.
        - `debug.retained_virtual_list_reconciles[*].reconcile_time_us` provides coarse wall-time attribution for the retained-host reconcile work.
        - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` exports these fields to bundles.
        - Gate: `fretboard diag stats <bundle> --check-retained-vlist-prefetch-reconciles-min <n>` (use on `*window-boundary*` scripts).
        - Gate: `fretboard diag stats <bundle> --check-retained-vlist-prefetch-reconciles-max <n>` (guards against runaway steady-state prefetch).
      - ADR: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md` (v2 addendum; staged prefetch + budgets).
    - [x] Tune staged prefetch thresholds so steady-state overhead stays low.
      - Goal: avoid prefetching too early/often; prefetch should trigger mainly when approaching escape (or when predicted escape deltas would be large).
      - Evidence should show: fewer `window_shift_kind=prefetch` frames under slow scroll, while still bounding `attached_items+detached_items` deltas on boundary ticks.
      - Current tuning:
        - `prefetch_margin = (overscan / 6).max(1)`
        - `prefetch_step = (overscan * 3 / 2).max(prefetch_margin).max(1)`
        - Direction hint: when both ends are "near", prefer the last observed scroll direction to avoid prefetch oscillation.
      - Tree retained harness now fits a `prefetch_reconciles_max=30` budget.
        - Evidence: `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch30-directionState/1769921894057-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
        - Before (budget ~40): `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch9/1769913766043-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
        - After (budget 30): `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch30-directionState/1769921894057-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
    - [x] Drive attach/detach via retained host reconcile (ADR 0192) when the window shifts, without rerendering the parent cache root.
      - Anchors: `crates/fret-ui/src/tree/prepaint.rs` (marks retained hosts for reconcile),
        `crates/fret-ui/src/declarative/mount.rs` (`reconcile_retained_virtual_list_hosts`).
      - Note: retained-host reconcile now prefers the prepaint-derived `VirtualListState.window_range` (ADR 0190) rather than re-deriving the
        window from scroll state during reconcile. This keeps “why did the window change?” explainable from one bundle and reduces duplicated work.
        - Evidence: `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`retained_virtual_list_updates_visible_range_on_wheel_scroll_without_notifying_view_cache`)
    - [x] Add/keep a `window-boundary` script that deterministically crosses overscan boundaries and enforce gates:
      `--check-retained-vlist-reconcile-no-notify`, `--check-retained-vlist-reconcile-cache-reuse` (recommended: reconcile occurs on cache-hit),
      attach/detach bounds, `--check-retained-vlist-scroll-window-dirty-max`, plus `--check-wheel-scroll-hit-changes` (container targets) and
      `--check-stale-paint`.
      - Suite: `fretboard diag suite components-gallery-file-tree --launch -- cargo run -p fret-demo --bin components_gallery --release`
      - Scripts:
        - `tools/diag-scripts/components-gallery-file-tree-window-boundary-scroll.json`
        - `tools/diag-scripts/components-gallery-file-tree-toggle-and-scroll.json`
        - `tools/diag-scripts/components-gallery-file-tree-window-boundary-bounce.json`
      - Evidence (suite defaults; warmup=5; view-cache enabled):
        - `target/fret-diag-suite-components-gallery-file-tree-defaults6/1770080650850-components-gallery-file-tree-window-boundary-scroll/bundle.json`
        - `target/fret-diag-suite-components-gallery-file-tree-defaults6/1770080678326-components-gallery-file-tree-toggle-and-scroll/bundle.json`
        - `target/fret-diag-suite-components-gallery-file-tree-defaults6/1770080712385-components-gallery-file-tree-window-boundary-bounce/bundle.json`
      - Note: retained-vlist *window-boundary* gates are applied only to scripts named `*window-boundary*` when running multi-script suites
        (toggle/sort scripts still run, but are gated by stale-paint / wheel-scroll / view-cache reuse, etc.).
    - [x] Record before/after bundles and keep the “worst tick” attribution explainable (layout vs prepaint vs paint).
      - Note: the window-boundary scripts use `wait_until` to assert a row is mounted; the stable test id is
        `components-gallery-file-tree-node-<TreeItemId>` (e.g. `...-node-1`), not `...-node-0`.
      - Baseline recorded (warmup=5; prefetch not implemented / not enabled; attach/detach deltas are larger per shift):
        - window-boundary-scroll: max.us(total/layout/prepaint/paint)=2812/2119/15/678
          - Evidence: `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862343305-components-gallery-file-tree-window-boundary-scroll/bundle.json`
        - toggle+scroll: max.us(total/layout/prepaint/paint)=3339/2685/15/639
          - Evidence: `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862370728-components-gallery-file-tree-toggle-and-scroll/bundle.json`
        - window-boundary-bounce: max.us(total/layout/prepaint/paint)=2823/2142/19/662
          - Evidence: `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862397267-components-gallery-file-tree-window-boundary-bounce/bundle.json`
      - After staged prefetch (warmup=5; explicit gate: `--check-retained-vlist-prefetch-reconciles-min 1`):
        - window-boundary-scroll:
          - Prefetch reconciles observed: 19 (warmup excluded); max attach/detach delta bounded to 8 (down from 20 baseline).
          - max.us(total/layout/prepaint/paint)=2364/1809/19/536
          - Evidence: `target/fret-diag-perf-components-gallery-file-tree-suite-prefetch4/1769876796976-components-gallery-file-tree-window-boundary-scroll/bundle.json`
        - toggle+scroll:
          - Prefetch reconciles observed: 15 (warmup excluded).
          - max.us(total/layout/prepaint/paint)=2433/1869/22/542
          - Evidence: `target/fret-diag-perf-components-gallery-file-tree-suite-prefetch4/1769876825653-components-gallery-file-tree-toggle-and-scroll/bundle.json`
        - window-boundary-bounce:
          - Prefetch reconciles observed: 22 (warmup excluded); max attach/detach delta bounded to 8 (down from 20 baseline).
          - max.us(total/layout/prepaint/paint)=2332/1729/32/571
          - Evidence: `target/fret-diag-perf-components-gallery-file-tree-suite-prefetch4/1769876853802-components-gallery-file-tree-window-boundary-bounce/bundle.json`
      - Current assessment:
        - Correctness + explainability improved (prefetch is visible in bundles; deltas are bounded), and worst-tick time is reduced vs baseline
          for this surface in the current tuning.
        - Next steps:
          - Use `debug.retained_virtual_list_reconciles[*].reconcile_time_us` to identify whether time is dominated by row subtree mounting,
            keep-alive churn, or general layout/paint work.
          - Re-run a harness with known large spikes (e.g. ui-gallery vlist boundary) to confirm staged prefetch reduces peak tail latency.
          - Retained vlist harness (ui-gallery; minimal + known heights; cache+shell):
            - Baseline (no staged prefetch):
              - max.us(total/layout/prepaint/paint)=5181/4171/125/885
              - Evidence: `target/fret-diag-perf-vlist-retained-boundary-baseline/1769765204218-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
            - After tuned staged prefetch:
              - Prefetch reconciles observed: 23 (warmup excluded); shift_kinds: `none=111`, `prefetch=25`, `escape=3`
              - max.us(total/layout/prepaint/paint)=4435/3656/45/734; max `reconcile_time_us`=50
              - Evidence: `target/fret-diag-perf-vlist-retained-boundary-prefetch5/1769878053904-script-step-0048-wheel/bundle.json`
            - Suite gate (same harness, but with the full retained-vlist correctness gates enabled):
              - PASS: `fretboard diag suite ui-gallery-virt-retained --check-retained-vlist-prefetch-reconciles-min 1`
              - Evidence: `target/fret-diag-perf-ui-gallery-virt-retained-suite-prefetch7/1769911980127-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
      - Additional retained-host consumers (staged prefetch evidence + budgets):
        - Tree (default budget `--check-retained-vlist-prefetch-reconciles-max 30`):
          - `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch30-directionState/1769921894057-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
        - DataTable (default budget `--check-retained-vlist-prefetch-reconciles-max 30`):
          - `target/fret-diag-perf-ui-gallery-data-table-retained-suite-prefetch10/1769913856566-ui-gallery-data-table-window-boundary-scroll-retained/bundle.json`
  - Definition of done (v2; mark `[x]` when all are true):
    - [x] The primary surface’s `window-boundary` script shows reduced worst-tick layout time while preserving correctness gates.
    - [x] Window shifts do not force a cache-root rerender unless an explicit structural change requires it.
    - [x] The same substrate can be applied to at least one more surface (reused primitives, not a one-off hack).
  - Clarification: the legacy `virtual_list_keyed` API (frame-local `FnMut` row closures) cannot support “attach/detach on cache-hit frames”
    by construction. The v2 path for GPUI-like prepaint-driven window updates therefore focuses on retained-host surfaces (ADR 0192) and
    ergonomic ecosystem wrappers that adopt them by default.
  - Progress (v1):
    - VirtualList rerender frames now compute `render_window_range` against the latest scroll-handle offset (including out-of-band `set_offset`),
      reducing “window jump -> layout updates -> next frame rerender” one-frame lag.
      - Evidence: `crates/fret-ui/src/elements/cx.rs` (preview offset windowing),
        `crates/fret-ui/src/tree/tests/scroll_invalidation.rs` (`virtual_list_window_jump_rerender_uses_latest_handle_offset`).
    - Window mismatch gating now checks “visible range is outside the previously rendered overscan window” (containment),
      not “layout-derived window != rendered window”, avoiding unnecessary cache-root rerenders while still inside overscan.
      - Anchors: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`,
        `crates/fret-ui/src/tree/tests/scroll_invalidation.rs` (`virtual_list_out_of_band_scroll_upgrades_to_layout_after_overscan_window`).
    - Rerender frames now prefer `VirtualListState.render_window_range` (if valid) as the baseline window, falling back to layout-derived
      `VirtualListState.window_range`. This preserves subtree stability when unrelated rerenders happen during transform-only scrolling
      inside overscan.
      - Anchor: `crates/fret-ui/src/elements/cx.rs` (range baseline selection).
  - Evidence: `tools/diag-scripts/ui-gallery-virtual-list-torture.json` worst bundles show reduced `contained_relayout_time_us`.
  - Harness (window boundary scroll):
    - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json` (multiple small wheel deltas; should cross the overscan window boundary without a massive jump).
    - Evidence bundle (cache+shell, release): `target/fret-diag-perf-vlist-window-boundary-cache-shell2/1769171174767-script-step-0027-wheel/bundle.json`
    - Evidence bundle (cache+shell, release, post-range-baseline change): `target/fret-diag-perf-vlist-window-boundary-recenter/1769177486396-script-step-0027-wheel/bundle.json`
    - Notes: the current worst tick is layout-dominated when the wheel crosses the window boundary; this is the baseline we want to improve by moving window derivation toward prepaint (ADR 0190).
    - Tip: for a more stable baseline (avoid measure noise), run the harness with `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1` so the page uses `VirtualListOptions::known(...)`.
    - Note (v1): “sticky window shift by minimal delta” is harmful for the legacy VirtualList path because v1 still requires a cache-root rerender to rebuild `visible_items`.
    - Note (v2): the retained-host path can use “minimal shift” safely because the runtime can attach/detach rows during reconcile without rerendering the parent cache root.
      - Anchor: `crates/fret-ui/src/tree/prepaint.rs` (`shift_virtual_list_window_minimally`).
      - Evidence bundles (cache+shell, release): `target/fret-diag-perf-vlist-window-boundary-sticky/1769176834622-script-step-0027-wheel/bundle.json`,
        `target/fret-diag-perf-vlist-window-boundary-sticky2/1769177002575-script-step-0027-wheel/bundle.json`.
    - Known-heights evidence bundle (cache+shell, release, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `--warmup-frames 5`): `target/fret-diag-perf-vlist-window-boundary-known-cache-shell/1769174146628-script-step-0027-wheel/bundle.json`
      - Takeaway: the boundary tick remains layout-dominated even without measurement, so the dominant cost is rebuilding/layouting the row subtree, not measuring it.
    - Known-heights evidence bundle (cache+shell, release, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `--warmup-frames 5`, prepaint-driven scheduling): `target/fret-diag-perf-vlist-window-boundary-known-cache-shell-v2/1770040368919-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
      - Takeaway: Track B window-boundary scheduling is now fully prepaint-driven (including the “no render-derived window yet” case), avoiding the failure mode where a cached root never schedules the one-shot rerender and mounts zero rows.
    - Progress (v1.2): avoid triggering an extra contained relayout pass on window-mismatch frames.
      - Change: VirtualList now marks the nearest view-cache root as "needs rerender" (dirty view) and requests redraw, instead of issuing an `Invalidation::Layout` during layout.
      - Anchors: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`, `crates/fret-ui/src/tree/mod.rs` (`mark_nearest_view_cache_root_needs_rerender`).
      - Evidence bundle (cache+shell, release, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `--warmup-frames 5`): `target/fret-diag-perf-vlist-window-boundary-optin/1769349359414-script-step-0027-wheel/bundle.json`
      - Takeaway: the dominant cost is still the rerender frame that rebuilds new rows; this change removes avoidable current-frame work and keeps the contract GPUI-like ("mark dirty, rebuild next frame").
    - Progress (v1.3): keep scroll-handle invalidation HitTestOnly even when the visible range leaves the rendered overscan window; mark the nearest view-cache root dirty and request redraw instead of forcing a layout invalidation walk.
    - Progress (v1.4): make window-boundary scheduling prepaint-driven (Track B), and treat missing render-derived windows as an "escape" that schedules a one-shot rerender.
      - Anchors: `crates/fret-ui/src/tree/prepaint.rs` (window mismatch baseline uses `render_window_range` and schedules rerender when absent),
        `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (layout no longer duplicates rerender scheduling for `window_mismatch` under view-cache).
    - Progress (v2.0 retained host): reconcile now uses `render_window_range` as a baseline and shifts the window minimally only when the visible range leaves the overscanned window (aligns retained-host reconcile with prepaint window logic).
      - Anchors: `crates/fret-ui/src/declarative/mount.rs` (retained host reconcile window selection), `crates/fret-ui/src/virtual_list.rs` (`shift_virtual_range_minimally`), `crates/fret-ui/src/tree/prepaint.rs` (shared helper usage).
      - Evidence bundle (suite, cache+shell, release): `target/fret-diag-virt-retained-suite-local1/1769751016873-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
      - Change: `invalidate_scroll_handle_bindings_for_changed_handles` triggers `mark_nearest_view_cache_root_needs_rerender` with `scroll_handle_window_update` while keeping the node invalidation as hit-test-only.
      - Anchors: `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender`), `crates/fret-ui/src/tree/tests/scroll_invalidation.rs` (`virtual_list_out_of_band_scroll_avoids_layout_after_overscan_window`).
    - Progress (v2.1 retained host): add a bounded keep-alive bucket for detached item subtrees (Flutter sliver-style).
      - Mechanism: when items detach due to a window shift, keep up to `VirtualListOptions::keep_alive` item roots keyed by `ItemKey` for later reuse (no remount).
      - Liveness: keep-alive roots are included in the window's GC liveness roots (ADR 0191) so cache-hit frames cannot sweep kept-alive subtrees as “islands”.
      - State persistence: the keep-alive bucket is stored in element-local state; retained hosts must touch that state key during normal render so it survives between reconcile frames (and on view-cache hits).
      - Diagnostics: bundles report `reused_from_keep_alive_items` / `kept_alive_items` / `evicted_keep_alive_items` and the keep-alive bucket size (`keep_alive_pool_len_before` / `keep_alive_pool_len_after`).
      - Anchors:
        - `crates/fret-ui/src/element.rs` (`VirtualListOptions::keep_alive`, `VirtualListProps.keep_alive`)
        - `crates/fret-ui/src/declarative/mount.rs` (`reconcile_retained_virtual_list_hosts` keep-alive bucket)
        - `crates/fret-ui/src/elements/cx.rs` (touch keep-alive state key under retained hosts)
        - `crates/fret-ui/src/elements/runtime.rs` (keep-alive roots in window liveness bookkeeping)
        - `crates/fret-ui/src/windowed_surface_host.rs` (keep-alive state storage)
        - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export)
      - Evidence (monotonic scroll; expect `kept_alive_items > 0`, `reused_from_keep_alive_items == 0`):
        - `C:\fret-diag-perf-components-gallery-file-tree-boundary-keepalive\1769839663570-script-step-0022-wheel\bundle.json`
      - Evidence (bounce; expect `reused_from_keep_alive_items > 0`):
        - `target/fret-diag/1769851029699-components-gallery-file-tree-window-boundary-bounce/bundle.json`
        - Command: `cargo run -p fretboard -- diag run tools/diag-scripts/components-gallery-file-tree-window-boundary-bounce.json --env FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE=1 --env FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE_N=50000 --env FRET_COMPONENTS_GALLERY_FILE_TREE_KEEP_ALIVE=256 --env FRET_EXAMPLES_VIEW_CACHE=1 --warmup-frames 5 --check-retained-vlist-keep-alive-reuse-min 1 --launch -- cargo run -p fret-demo --bin components_gallery --release`
      - Note: keep-alive does not reduce the “first time we see new items” cost during one-direction boundary scroll; it targets oscillation/backtracking stability.
    - Validated (v1.1): per-row nested cache roots inside `VirtualList`.
      - Attempt: wrap each row in a nested `ViewCache` boundary (`FRET_UI_GALLERY_VLIST_ROW_CACHE=1`) to reuse row layout/paint across window rebuilds.
      - Fix: `ViewCacheProps::default().contained_layout` is now `false` (contained relayout is opt-in), so barrier-placed roots (VirtualList row placement) keep parent-provided bounds and do not get clobbered by out-of-band contained relayout.
      - Evidence bundle (PASS; cache+shell, release, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `FRET_UI_GALLERY_VLIST_ROW_CACHE=1`):
        `target/fret-diag-vlist-rowcached-fixed2/1769346674136-ui-gallery-virtual-list-edit-9000/bundle.json`
      - Unit regression: `crates/fret-ui/src/declarative/tests/view_cache.rs` (`view_cache_row_cached_virtual_list_keeps_semantics_in_viewport_space`)
      - Takeaway: nested row caches are viable again for v1.1 experiments, but they do not replace ADR 0190: v1 still requires rerender when the visible-item set changes (window derivation is still render-driven).
  - Next (v2 direction; ADR 0190):
    - Progress (v2.0): derive VirtualList window telemetry during prepaint (cache-hit safe).
      - Change: `UiTree::prepaint_virtual_list_window_from_interaction_record` updates `VirtualListState.{window_range,viewport_*,offset_*}` from interaction records, and can dirty the nearest cache root on overscan escape (no rerender required to compute the window).
      - Change: prepaint also refreshes `VirtualListScrollHandle` internal viewport/content sizes and clamps offset via `set_*_internal`, keeping scroll-state bookkeeping consistent under reuse.
      - Perf: avoid cloning `VirtualListProps` (which includes `visible_items`) in scroll-handle invalidation paths by adding a borrowed lookup helper (`with_element_record_for_node`) and using it for the fixed-mode scroll-to-item fast path.
      - Evidence bundle (cache+shell, release, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `--warmup-frames 5`): `target/fret-diag-perf-vlist-window-prepaint-v2/1769442109178-script-step-0027-wheel/bundle.json`
      - Evidence: `crates/fret-ui/src/tree/prepaint.rs` (`prepaint_updates_virtual_list_window_and_marks_cache_root_dirty_on_escape`), `crates/fret-ui/src/declarative/frame.rs` (`with_element_record_for_node`), `crates/fret-ui/src/tree/layout.rs` (borrowed vlist fast path).
    - Progress (v2.2): skip full layout engine work on layout-clean frames (prepaint-only fast path).
      - Change: `UiTree::layout_all_with_pass_kind` now early-exits when there are no layout invalidations and no pending barrier relayouts, while still refreshing semantics (if requested) and running prepaint.
      - Intent: keep scroll-only and cache-hit frames cheap without requiring every driver to grow a “layout_if_needed” wrapper.
      - Anchor: `crates/fret-ui/src/tree/layout.rs` (fast-path early return).
      - Perf note: the fast-path gate must be O(1). Use `UiTree.layout_invalidations_count` rather than scanning all nodes each frame.
      - Diagnostics: bundles report `debug.stats.layout_fast_path_taken` and `debug.stats.layout_invalidations_count` (helps validate the fast path is actually taking effect in scripted harnesses).
      - Gate: `fretboard diag stats <bundle> --check-layout-fast-path-min 1` (after warmup).
      - Evidence (smoke; `components_gallery` file-tree bounce): `target/fret-diag-smoke-layout-fastpath/1769855748827-components-gallery-file-tree-window-boundary-bounce/bundle.json`
      - Suite defaults (components gallery surfaces; override via CLI flags):
        - `--check-layout-fast-path-min 1`
        - `--check-view-cache-reuse-min 1`
        - `--check-stale-paint components-gallery-file-tree-root` / `components-gallery-table-root`
        - `--check-wheel-scroll-hit-changes components-gallery-file-tree-root` / `components-gallery-table-root`
        - Window-boundary scripts only:
          - `--check-vlist-window-shifts-non-retained-max 0`
          - `--check-vlist-policy-key-stable`
          - `--check-retained-vlist-reconcile-no-notify 1`
          - `--check-retained-vlist-attach-detach-max 64`
          - `--check-retained-vlist-keep-alive-reuse-min 1` (keep-alive bounce scripts)
    - Move “window derivation” into `prepaint` so window shifts can be applied while the view remains cache-reusable (no forced rerender).
    - Define (and gate via bundles) what data constitutes the VirtualList “window cache key” (viewport/offset/overscan/items revision) so reuse is explainable.
    - Add a regression gate for `ui-gallery-virtual-list-window-boundary-scroll` that flags boundary ticks that force cache-root rerenders too frequently under cache+shell mode:
      - `fretboard diag run tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll.json --warmup-frames 5 --check-vlist-scroll-window-dirty-max 2 ...`
      - Builtin suite: `fretboard diag suite ui-gallery-vlist-window-boundary` defaults to (non-retained path):
        - `--check-stale-paint ui-gallery-virtual-list-root`
        - `--check-view-cache-reuse-min 1`
        - `--check-vlist-visible-range-refreshes-min 1`
        - `--check-vlist-visible-range-refreshes-max 20`
        - `--check-vlist-window-shifts-prefetch-max 12`
        - `--check-vlist-window-shifts-escape-max 4`
        - Default launch env (overridable via `--env KEY=...`): `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`, `FRET_UI_GALLERY_VLIST_RETAINED=0`, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`.
        - Observed distribution (local, warmup=5, 3 runs):
          - `total_refreshes=15` (`check.vlist_visible_range_refreshes_max.json`)
          - `prefetch_shifts=7..8` (`check.vlist_window_shifts_prefetch_max.json`)
          - `escape_shifts=3` (`check.vlist_window_shifts_escape_max.json`)
          So the defaults above are tight-but-stable caps for churn regressions.
      - Builtin suite: `fretboard diag suite ui-gallery-vlist-window-boundary-retained` defaults to (retained-host path):
        - Same gates as the non-retained suite, plus:
          - `--check-retained-vlist-reconcile-no-notify 1`
          - `--check-retained-vlist-attach-detach-max 256`
          - `--check-vlist-window-shifts-non-retained-max 0` (no fallback)
        - Default launch env (overridable via `--env KEY=...`): `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`, `FRET_UI_GALLERY_VLIST_RETAINED=1`, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`.
        - Evidence bundle (suite; cache+shell, release): `target/fret-diag-suite-ui-gallery-vlist-window-boundary-retained0/1770086113873-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
      - Evidence bundle (suite; cache+shell, release; prefetch-min + prefetch-dirty budget gated; non-retained default):
        `target/fret-diag-suite-ui-gallery-vlist-window-boundary-nonretained0/1770084994647-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
      - Evidence bundle (suite; cache+shell, release; prefetch_step=overscan*8 for non-retained):
        `target/fret-diag-suite-ui-gallery-vlist-window-boundary-after-prefetch-step/1770047603074-ui-gallery-virtual-list-window-boundary-scroll/bundle.json`
        - Observed: `prefetch_shifts=20`, `escape_shifts=3`, `visible_range_refreshes=26` (budgeted by suite defaults).
      - Gate tightening ladder (post-warmup):
        - Current: `--check-vlist-visible-range-refreshes-max 20` (catches “too much window churn” regressions after wheel events, while allowing prefetch).
        - Next: aim for `10` once v2 window derivation reduces boundary work in a single tick and/or staged prefetch budgets are tightened.
        - Stretch: aim for `5` once the boundary harness is stable under the *default suite env* (cache+shell+known heights) and we have
          repeatable evidence that the prepaint-driven scheduling path no longer causes “extra settle refreshes” after the first wheel event.
          - Definition of done to tighten to `5` (run locally at least 3 times):
            - `fretboard diag suite ui-gallery-vlist-window-boundary --launch -- cargo run -p fret-ui-gallery --release` passes with `--check-vlist-visible-range-refreshes-max 5`,
            - `--check-stale-paint ui-gallery-virtual-list-root` still passes,
            - and the bundle shows at least one cache-hit tail streak (`--check-view-cache-reuse-min 1` stays meaningful).
        - Target: split budgets by cause once “window shift reasons” are exported (e.g. `prefetch` vs `escape`) and can be gated independently.
          - Gates: `--check-vlist-window-shifts-prefetch-max <n>`, `--check-vlist-window-shifts-escape-max <n>` (counts prepaint-sourced shifts after warmup).

- [x] GPUI-MVP5-virt-003 Retained windowed surface host for composable virtualization (ADR 0192).
  - Note: the existing `virtual_list_keyed` authoring API uses non-`'static` closures (`FnMut`), so v1 of virt-003 MUST be a new, opt-in surface that stores `'static` callbacks in element-local state (per ADR 0192) rather than retrofitting the existing helper.
  - Goal: allow scroll/window membership updates to attach/detach item subtrees without rerendering the parent cache root.
  - Contract: `docs/adr/0192-retained-windowed-surface-hosts.md` (Accepted).
  - Progress (v1 prototype; fixed/known baseline, measured variant gated):
    - Runtime host state: `crates/fret-ui/src/windowed_surface_host.rs` (`RetainedVirtualListHostMarker`, `RetainedVirtualListHostCallbacks`).
    - Scheduling: `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/tree/prepaint.rs` (`mark_retained_virtual_list_needs_reconcile`).
    - Input routing: `crates/fret-ui/src/declarative/host_widget/event/scroll.rs` schedules retained-host reconcile on overscan escape instead of forcing `notify()` (keeps parent cache roots reusable).
    - Prepaint window updates now shift retained-host windows minimally on overscan escape to reduce attach/detach churn (instead of snapping to the ideal visible range).
      - Anchor: `crates/fret-ui/src/tree/prepaint.rs` (`shift_virtual_list_window_minimally`).
    - Reconcile: `crates/fret-ui/src/declarative/mount.rs` (`reconcile_retained_virtual_list_hosts`).
    - Diagnostics: bundles export retained VirtualList reconcile deltas (`debug.retained_virtual_list_reconciles`) and frame counters (`debug.stats.retained_virtual_list_*`).
    - Tests: `crates/fret-ui/src/declarative/tests/virtual_list.rs` (`retained_virtual_list_host_updates_window_without_rerendering_view_cache_root`).
    - Constraint: the host must be a layout barrier (non-`Auto` main-axis size), otherwise children reattach is skipped to preserve mount invariants.
  - Harness (window-boundary scroll):
    - Script: `tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll-retained.json`
    - Run with: `FRET_UI_GALLERY_VLIST_RETAINED=1`, `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1`, `FRET_UI_GALLERY_VLIST_MINIMAL=1`, `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
    - Expectation: crossing the overscan window boundary does not force a parent cache-root rerender; instead the retained host reconciles (attach/detach delta only).
    - Variant (measured rows): set `FRET_UI_GALLERY_VLIST_VARIABLE_HEIGHT=1` (no fixed row height hints) to exercise `VirtualListMeasureMode::Measured` under the retained host path.
    - Gate (single script): `fretboard diag run tools/diag-scripts/ui-gallery-virtual-list-window-boundary-scroll-retained.json --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-max 64 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-virtual-list-row-0-label --check-stale-paint ui-gallery-virtual-list-row-0-label ...`
    - Gate (suite): `fretboard diag suite ui-gallery-virt-retained --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 64 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-virtual-list-row-0-label --check-stale-paint ui-gallery-virtual-list-row-0-label ...`
      - Defaults: `ui-gallery-virt-retained` sets `FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS=1` and `FRET_UI_GALLERY_VLIST_MINIMAL=1` to reduce measurement noise and focus the gate on retained window reconcile behavior.
    - Gate (suite, measured rows): `fretboard diag suite ui-gallery-virt-retained-measured --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 64 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-virtual-list-row-0-label --check-stale-paint ui-gallery-virtual-list-row-0-label ...`
      - Defaults: `ui-gallery-virt-retained-measured` sets `FRET_UI_GALLERY_VLIST_RETAINED=1`, `FRET_UI_GALLERY_VLIST_MINIMAL=1`, and `FRET_UI_GALLERY_VLIST_VARIABLE_HEIGHT=1`.
    - Re-verified (cache+shell, release; reconcile>=2 + attach/detach min enforced): `target/fret-diag-virt-retained-suite-stronger-gate/1769758544095-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
    - Note: `fretboard diag suite ui-gallery-virt-retained` now defaults to `--warmup-frames 5` plus the retained VirtualList gates above when not explicitly provided.
    - Evidence bundle (cache+shell, release, minimal harness; passes no-notify + bounded-delta + wheel-scroll + stale-paint gates): `target/fret-diag-vlist-virt-retained-suite2/1769511343500-script-step-0048-wheel/bundle.json`
    - Evidence bundle (measured rows; cache+shell, release; passes no-notify + bounded-delta + wheel-scroll + stale-paint gates): `target/fret-diag-vlist-virt-retained-measured-local1/1769676590792-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
    - Gate (suite, measured retained all-in-one): `fretboard diag suite ui-gallery-retained-measured --warmup-frames 5 --timeout-ms 240000 --poll-ms 200 --dir target/fret-diag-retained-measured-all-local1 --launch -- cargo run -p fret-ui-gallery --release`
      - Defaults: `ui-gallery-retained-measured` enables view-cache+shell plus the measured variants for VirtualList/Tree/DataTable/Table, and uses multi-test-id wheel-scroll + stale-paint gates.
      - Prefetch max gate: defaults to `--check-retained-vlist-prefetch-reconciles-max 30` for `*window-boundary*` scripts (tree drives the budget).
      - Note: retained-vlist window-boundary gates (reconcile/no-notify/attach-detach bounds) apply only to the boundary scripts in the suite (not to interaction-only scripts).
      - Evidence bundles (measured all-in-one; cache+shell, release):
        - `target/fret-diag-retained-measured-all-local1/1769680828211-ui-gallery-virtual-list-window-boundary-scroll-retained/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769680867856-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769680899431-ui-gallery-tree-retained-toggle-and-scroll/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769680928063-ui-gallery-data-table-window-boundary-scroll-retained/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769680957492-ui-gallery-data-table-retained-sort-select-scroll/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769680985681-ui-gallery-table-retained-window-boundary-scroll/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769681014581-ui-gallery-table-retained-sort-select-scroll/bundle.json`
        - `target/fret-diag-retained-measured-all-local1/1769681042494-ui-gallery-table-retained-keyboard-typeahead/bundle.json`
  - Tree harness (retained host consumer):
    - Script: `tools/diag-scripts/ui-gallery-tree-window-boundary-scroll-retained.json`
    - Script (toggle + scroll): `tools/diag-scripts/ui-gallery-tree-retained-toggle-and-scroll.json`
    - Run with: `FRET_UI_GALLERY_TREE_RETAINED=1`, `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
    - Variant (measured rows): set `FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT=1` to introduce multi-line labels and run the retained host with `VirtualListMeasureMode::Measured`.
    - Expectation: crossing the overscan boundary reconciles attach/detach deltas (no parent cache-root rerender), and remains stale-paint safe.
    - Gate (suite): `fretboard diag suite ui-gallery-tree-retained --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-tree-row-0 --check-stale-paint ui-gallery-tree-row-0 ...`
    - Gate (suite, measured rows): `fretboard diag suite ui-gallery-tree-retained-measured --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-tree-row-0 --check-stale-paint ui-gallery-tree-row-0 ...`
      - Note: in this multi-script suite, the retained-vlist window-boundary gates apply only to `ui-gallery-tree-window-boundary-scroll-retained.json` (the toggle+scroll script is still gated by wheel-scroll + stale-paint, etc.).
      - Defaults: `ui-gallery-tree-retained-measured` sets `FRET_UI_GALLERY_TREE_RETAINED=1` and `FRET_UI_GALLERY_TREE_VARIABLE_HEIGHT=1`.
      - Defaults (bundle size): retained suites cap semantics export by default (`FRET_DIAG_MAX_SEMANTICS_NODES=10000`, `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1`, `FRET_DIAG_SCRIPT_AUTO_DUMP=0`).
      - Prefetch max gate: `ui-gallery-tree-retained` and `ui-gallery-tree-retained-measured` default to `--check-retained-vlist-prefetch-reconciles-max 30` for `*window-boundary*` scripts (tree should stay under budget with `--warmup-frames 5`).
    - Note: the script uses the sidebar search input (`ui-gallery-nav-search`) to keep navigation stable as the page list grows.
    - Evidence bundles (cache+shell, release; pass no-notify + bounded-delta + wheel-scroll + stale-paint gates):
      - `target/fret-diag-tree-retained-suite-shell3/1769649443728-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
      - `target/fret-diag-tree-retained-suite-shell3/1769649473084-ui-gallery-tree-retained-toggle-and-scroll/bundle.json`
      - (With staged prefetch max gate) `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch9/1769913766043-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
      - (With staged prefetch max gate) `target/fret-diag-perf-ui-gallery-tree-retained-suite-prefetch9/1769913809155-ui-gallery-tree-retained-toggle-and-scroll/bundle.json`
      - (Measured rows) `target/fret-diag-tree-retained-measured-local1/1769678735646-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
      - (Measured rows) `target/fret-diag-tree-retained-measured-local1/1769678769241-ui-gallery-tree-retained-toggle-and-scroll/bundle.json`
  - DataTable harness (retained host consumer):
    - Script: `tools/diag-scripts/ui-gallery-data-table-window-boundary-scroll-retained.json`
    - Script (sort + select + scroll): `tools/diag-scripts/ui-gallery-data-table-retained-sort-select-scroll.json`
    - Run with: `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1`, `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
    - Keep-alive bounce (ADR 0192):
      - Env: `FRET_UI_GALLERY_DATA_TABLE_KEEP_ALIVE=256`
      - Script: `tools/diag-scripts/ui-gallery-data-table-window-boundary-bounce-keep-alive.json`
      - Suite: `fretboard diag suite ui-gallery-data-table-retained-keep-alive --launch -- cargo run -p fret-ui-gallery --release`
    - Variant (measured rows): set `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT=1` (enables `DataTable::measure_rows(true)` and introduces multi-line cell content).
    - Expectation: crossing the overscan boundary reconciles attach/detach deltas (no parent cache-root rerender), and remains stale-paint safe.
    - Gate (suite): `fretboard diag suite ui-gallery-data-table-retained --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-data-table-row-0 --check-stale-paint ui-gallery-data-table-row-0 ...`
    - Gate (suite, measured rows): `fretboard diag suite ui-gallery-data-table-retained-measured --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 2 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-data-table-row-0 --check-stale-paint ui-gallery-data-table-row-0 ...`
      - Note: in this multi-script suite, the retained-vlist window-boundary gates apply only to `ui-gallery-data-table-window-boundary-scroll-retained.json` (the sort+select+scroll script is still gated by wheel-scroll + stale-paint, etc.).
      - Defaults: `ui-gallery-data-table-retained-measured` sets `FRET_UI_GALLERY_DATA_TABLE_RETAINED=1` and `FRET_UI_GALLERY_DATA_TABLE_VARIABLE_HEIGHT=1`.
      - Prefetch max gate: `ui-gallery-data-table-retained` defaults to `--check-retained-vlist-prefetch-reconciles-max 30` for `*window-boundary*` scripts (observed ~15 prefetch reconciles with `--warmup-frames 5`).
    - Note: the script uses the sidebar search input (`ui-gallery-nav-search`) to keep navigation stable as the page list grows.
    - Implementation: `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_retained_v0`), via `ecosystem/fret-ui-shadcn/src/data_table.rs` (`DataTable::into_element_retained`).
    - Evidence bundles (cache+shell, release; pass no-notify + bounded-delta + wheel-scroll + stale-paint gates):
      - `target/fret-diag-data-table-retained-suite-shell6/1769651477808-ui-gallery-data-table-window-boundary-scroll-retained/bundle.json`
      - `target/fret-diag-data-table-retained-suite-shell6/1769651504240-ui-gallery-data-table-retained-sort-select-scroll/bundle.json`
      - (With staged prefetch max gate) `target/fret-diag-perf-ui-gallery-data-table-retained-suite-prefetch10/1769913856566-ui-gallery-data-table-window-boundary-scroll-retained/bundle.json`
      - (With staged prefetch max gate) `target/fret-diag-perf-ui-gallery-data-table-retained-suite-prefetch10/1769913891537-ui-gallery-data-table-retained-sort-select-scroll/bundle.json`
      - (Measured rows) `target/fret-diag-data-table-retained-measured-local1/1769679828598-ui-gallery-data-table-window-boundary-scroll-retained/bundle.json`
      - (Measured rows) `target/fret-diag-data-table-retained-measured-local1/1769679856618-ui-gallery-data-table-retained-sort-select-scroll/bundle.json`
      - (Keep-alive bounce) `target/fret-diag/1769961130938-ui-gallery-data-table-window-boundary-bounce-keep-alive/bundle.json`
  - UI kit list harness (retained host consumer; ADR 0192):
    - Page: `apps/fret-ui-gallery/src/ui.rs` (`preview_ui_kit_list_torture`), `apps/fret-ui-gallery/src/spec.rs` (`PAGE_UI_KIT_LIST_TORTURE`).
    - Script: `tools/diag-scripts/ui-gallery-ui-kit-list-window-boundary-scroll.json`
      - Note: the script targets `ui-gallery-ui-kit-list-row-0-label` for wheel events to ensure the wheel position remains within the visible viewport.
    - Suite: `fretboard diag suite ui-gallery-ui-kit-list-retained`
      - Defaults: `FRET_UI_GALLERY_VIEW_CACHE=1`, `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`, `FRET_UI_GALLERY_UI_KIT_LIST_LEN=2000`,
        `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=1`, and gates `--check-retained-vlist-scroll-window-dirty-max 0`, `--check-view-cache-reuse-min 5`,
        `--check-wheel-scroll ui-gallery-ui-kit-list-row-0-label`, `--check-stale-paint ui-gallery-ui-kit-list-row-0-label`.
    - Evidence bundle (smoke; cache+shell, release): `target/fret-diag-ui-kit-list-retained-smoke6/1769945002526-ui-gallery-ui-kit-list-window-boundary-scroll/bundle.json`
  - Implementation summary (v1):
    - A runtime-owned `WindowedSurfaceHost` boundary can attach/detach item subtrees during `prepaint` without re-running the parent render closure.
    - The opt-in authoring API stores `'static` callbacks in element-local state (item key + item render), plus window policy (overscan + keep-alive extent).
    - The retained host supports fixed/known heights and measured rows (via `VirtualListOptions.measure_mode` + `VirtualListKeyCache`), and is regression-gated with scripted UI Gallery suites (including measured variants).
  - Next:
    - Expand retained-table coverage from v0 to more of the existing UI Kit table surface (grouping/pinning/resizing), tracked in `GPUI-MVP5-eco-002`.
    - Keep tuning measured-mode churn (attach/detach deltas) under overscan-boundary scroll, while `GPUI-MVP5-virt-001` continues to target the default (non-retained) VirtualList path.

- [x] GPUI-MVP5-virt-002 VirtualList: add “known row heights” mode (skip runtime measurement).
  - Goal: support variable-but-deterministic row heights without `measure_in` on visible children.
  - Notes: this does not fix the `virtual_list_torture` worst tick because it is dominated by row subtree layout (shadcn-heavy row composition),
    but it is useful for fixed-height tables/trees with occasional deterministic height changes (group headers, separators).
  - Evidence:
    - API: `crates/fret-ui/src/element.rs` (`VirtualListMeasureMode::Known`, `VirtualListOptions::known`)
    - Metrics import: `crates/fret-ui/src/virtual_list.rs` (`rebuild_from_known_heights`)
    - Layout path: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
    - Unit test: `crates/fret-ui/src/virtual_list.rs` (`known_mode_can_import_fixed_per_index_heights`)
    - Diagnostics schema: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiVirtualListMeasureModeV1::Known`)

- [x] GPUI-MVP5-eco-007 Provide a “windowed rows surface” building block for simple lists/inspectors.
  - Goal: allow huge row surfaces to update the visible window via paint/prepaint without requiring per-row declarative subtrees.
  - Notes: this is the “single-node surface” escape hatch; composable rows still use `VirtualList` for semantics/focus correctness.
  - Evidence:
    - Core helper: `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` (`windowed_rows_surface`).
    - UI Gallery harness page: `apps/fret-ui-gallery/src/ui.rs` (`preview_windowed_rows_surface_torture`, `ui-gallery-windowed-rows-root`).
    - Scripted scroll capture: `tools/diag-scripts/ui-gallery-windowed-rows-surface-scroll-refresh.json`.
    - Interactive variant (v1): paint-only hover/selection chrome under view-cache reuse using pointer-hook `invalidate(Paint)`:
      - Helper: `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` (`windowed_rows_surface_with_pointer_region`)
      - UI Gallery page: `apps/fret-ui-gallery/src/ui.rs` (`preview_windowed_rows_surface_interactive_torture`, `ui-gallery-windowed-rows-interactive-canvas`)
      - Script: `tools/diag-scripts/ui-gallery-windowed-rows-interactive.json`
      - Evidence bundle (cache+shell, release): `target/fret-diag-windowed-rows-interactive/1769167932581-ui-gallery-windowed-rows-interactive-scroll-hover/bundle.json`
    - Bundle-based stale-paint check:
      - Generate: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-windowed-rows-surface-scroll-refresh.json --release`
      - Inspect: `cargo run -p fretboard -- diag stats <bundle.json> --check-stale-paint ui-gallery-windowed-rows-root`.
    - A/B perf sanity (one run, release, view-cache + shell enabled):
      - `windowed_rows_surface_scroll_refresh`: worst tick ~2.6ms (layout-dominated).
      - `virtual_list_torture`: worst tick ~29.8ms (layout-dominated).
      - Takeaway: large headroom remains for `GPUI-MVP5-virt-001` (prepaint-driven window to reduce scroll-time rerender/layout work).
- [x] GPUI-MVP5-eco-002 Migrate table/tree virtualization to the new VirtualList window model.
  - Touches: `ecosystem/fret-ui-kit/src/declarative/table.rs`, `ecosystem/fret-ui-kit/src/declarative/tree.rs`, gallery/demo callsites.
  - Done when: scroll-driven window updates do not mark the nearest cache root dirty (window boundary updates reconcile attach/detach deltas during prepaint under view-cache reuse), and the common interactions remain correct and stale-paint safe:
    - Tree: expand/collapse (toggle) + scrolling under the retained host path.
    - Table/DataTable: select + sort + keyboard navigation/typeahead + scrolling under the retained host path (including measured/variable-height variants).
  - Progress (v1):
    - UI Gallery harness page: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_DATA_TABLE_TORTURE`)
    - Harness implementation: `apps/fret-ui-gallery/src/ui.rs` (`preview_data_table_torture`, `ui-gallery-data-table-torture-root`)
    - Scripted scroll capture: `tools/diag-scripts/ui-gallery-data-table-torture-scroll-refresh.json`
    - Tree torture can opt into the retained host path (virt-003 consumer) via `FRET_UI_GALLERY_TREE_RETAINED=1`.
      - Script: `tools/diag-scripts/ui-gallery-tree-window-boundary-scroll-retained.json`
      - Script (toggle + scroll): `tools/diag-scripts/ui-gallery-tree-retained-toggle-and-scroll.json`
      - Debug affordance: retained tree rows expose deterministic toggle button IDs (`ui-gallery-tree-row-<id>-toggle`) for scripted expand/collapse gates.
      - Evidence bundles (cache+shell, release):
        - `target/fret-diag-tree-retained-suite-shell3/1769649443728-ui-gallery-tree-window-boundary-scroll-retained/bundle.json`
        - `target/fret-diag-tree-retained-suite-shell3/1769649473084-ui-gallery-tree-retained-toggle-and-scroll/bundle.json`
    - UI Kit table retained harness (virt-003 consumer):
      - UI Gallery page: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_TABLE_RETAINED_TORTURE`)
      - Scripts:
        - `tools/diag-scripts/ui-gallery-table-retained-window-boundary-scroll.json`
        - `tools/diag-scripts/ui-gallery-table-retained-sort-select-scroll.json`
        - `tools/diag-scripts/ui-gallery-table-retained-sort-desc.json`
        - `tools/diag-scripts/ui-gallery-table-retained-keyboard-typeahead.json`
      - Keyboard nav/typeahead:
        - `ecosystem/fret-ui-kit/src/declarative/table.rs` (`table_virtualized_retained_v0`, `RetainedTableKeyboardNavState`)
      - Gate (suite): `fretboard diag suite ui-gallery-table-retained --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-table-retained-row-0|ui-gallery-table-retained-row-9999 --check-stale-paint ui-gallery-table-retained-row-0|ui-gallery-table-retained-row-9999 ...`
        - Note: in this multi-script suite, the retained-vlist window-boundary gates apply only to `ui-gallery-table-retained-window-boundary-scroll.json` (the interaction scripts are still gated by wheel-scroll + stale-paint, etc.).
        - Prefetch max gate: `ui-gallery-table-retained` defaults to `--check-retained-vlist-prefetch-reconciles-max 30` for `*window-boundary*` scripts.
        - Note: `--check-wheel-scroll <test_id>` asserts that the target's semantics bounds move after the first wheel event (it does not require the debug hit-test node id to change).
      - Gate (suite, measured rows): `fretboard diag suite ui-gallery-table-retained-measured --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-max 128 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-table-retained-row-0|ui-gallery-table-retained-row-9999 --check-stale-paint ui-gallery-table-retained-row-0|ui-gallery-table-retained-row-9999 ...`
        - Note: in this multi-script suite, the retained-vlist window-boundary gates apply only to `ui-gallery-table-retained-window-boundary-scroll.json` (the interaction scripts are still gated by wheel-scroll + stale-paint, etc.).
        - Defaults: `ui-gallery-table-retained-measured` sets `FRET_UI_GALLERY_TABLE_VARIABLE_HEIGHT=1`.
        - Note: the measured-row suite currently excludes `ui-gallery-table-retained-sort-desc` until the sort-direction toggle is made deterministic in scripts (avoid flake from multi-click sort state transitions).
      - Evidence bundles (cache+shell, release):
        - `target/fret-diag-table-retained-suite-shell1/1769653530154-ui-gallery-table-retained-window-boundary-scroll/bundle.json`
        - `target/fret-diag-table-retained-suite-shell1/1769653557131-ui-gallery-table-retained-sort-select-scroll/bundle.json`
        - (Desc sort) `target/fret-diag-table-retained-suite-smoke3/1769698245411-ui-gallery-table-retained-sort-desc/bundle.json`
        - `target/fret-diag-table-retained-keyboard-local12/1769667088191-ui-gallery-table-retained-keyboard-typeahead/bundle.json`
        - (Measured rows) `target/fret-diag-table-retained-measured-local1/1769678819672-ui-gallery-table-retained-window-boundary-scroll/bundle.json`
        - (Measured rows) `target/fret-diag-table-retained-measured-local1/1769678848174-ui-gallery-table-retained-sort-select-scroll/bundle.json`
        - (Measured rows) `target/fret-diag-table-retained-measured-local1/1769678876972-ui-gallery-table-retained-keyboard-typeahead/bundle.json`
    - Bundle-based stale-paint check:
      - Generate (example): `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-data-table-torture-scroll-refresh.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --dir target/fret-diag-perf-data-table-torture --launch -- cargo run -p fret-ui-gallery --release`
      - Inspect: `cargo run -p fretboard -- diag stats <bundle.json> --check-stale-paint ui-gallery-data-table-torture-root`
    - Baseline perf (one run, release, view-cache + shell enabled):
      - `ui-gallery-data-table-torture-scroll-refresh`: worst tick ~19.3ms (layout-dominated; includes real 50k-row scroll).
      - Example bundle: `target/fret-diag-perf-data-table-torture5/1769150942029-script-step-0009-wheel/bundle.json`
      - Note: the harness pins `TableState.pagination.page_size = data.len()` so the table is not stuck at the default 10-row page.
    - Tree harness (v1):
      - UI Gallery harness page: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_TREE_TORTURE`)
      - Harness implementation: `apps/fret-ui-gallery/src/ui.rs` (`preview_tree_torture`, `ui-gallery-tree-torture-root`)
      - Scripted scroll capture: `tools/diag-scripts/ui-gallery-tree-torture-scroll-refresh.json`
      - Baseline perf (one run, release, view-cache + shell enabled):
        - `ui-gallery-tree-torture-scroll-refresh`: worst tick ~8.6ms (layout-dominated).
        - Example bundle: `target/fret-diag-perf-tree-torture/1769146889956-script-step-0009-wheel/bundle.json`
- [x] GPUI-MVP5-eco-003 Identify “code/text window” surfaces that should be prepaint-windowed.
  - Candidates:
    - `ecosystem/fret-code-view/src/*` (CodeBlock windowed lines; already has a harness).
    - `ecosystem/fret-markdown/src/*` (long scrolling documents; markdown/code blocks).
    - Diagnostics/inspectors in `ecosystem/fret-bootstrap/src/*` (large list-like surfaces).
  - Done when: we have an evidence-backed list + a first migration target (one component) with a perf/correctness harness.
  - Progress (v1):
    - UI Gallery now has a dedicated harness page: `code_view_torture` (large code block with vertical scroll).
    - Scripted scroll capture exists: `tools/diag-scripts/ui-gallery-code-view-scroll-refresh.json` (run with `fretboard diag run ...`).
    - Baseline variant exists (same steps, different label): `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json`.
    - Stale-paint check is wired: `cargo run -p fretboard -- diag stats <bundle.json> --check-stale-paint ui-gallery-code-view-root`.
    - First migration target complete: `ecosystem/fret-code-view` now supports `CodeBlockUiOptions.windowed_lines` and implements it via a retained
      VirtualList host (ADR 0192), so window shifts can reconcile without dirtying the parent cache root.
    - Next (v1):
      - First migration target: `ecosystem/fret-code-view` “CodeBlock -> windowed lines” (visible line window + overscan), with regression enforced by the harness above.
      - Run A/B: `FRET_UI_GALLERY_CODE_VIEW_WINDOWED=0|1` toggles the `code_view_torture` page path (default: `1`).
    - Evidence:
      - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_VIEW_TORTURE`)
      - `apps/fret-ui-gallery/src/ui.rs` (`preview_code_view_torture`, `ui-gallery-code-view-root`)
      - `tools/diag-scripts/ui-gallery-code-view-scroll-refresh.json`
      - `tools/diag-scripts/ui-gallery-code-view-scroll-refresh-baseline.json`
    - `ecosystem/fret-code-view/src/code_block.rs` (`render_code_block_windowed_lines`, retained VirtualList host)
    - A/B bundles (same script steps, different variant):
      - Baseline (`FRET_UI_GALLERY_CODE_VIEW_WINDOWED=0`): `target/fret-diag/1769092650269-ui-gallery-code-view-scroll-refresh-baseline/bundle.json`
        - `diag stats` time sum (us): total=70638041 layout=35427247 paint=35207014
      - Windowed (`FRET_UI_GALLERY_CODE_VIEW_WINDOWED=1`): `target/fret-diag/1769092702700-ui-gallery-code-view-scroll-refresh/bundle.json`
        - `diag stats` time sum (us): total=4976556 layout=4533404 paint=438751
      - Both pass stale-paint verification: `cargo run -p fretboard -- diag stats <bundle.json> --check-stale-paint ui-gallery-code-view-root`
- [~] GPUI-MVP5-eco-004 Identify “canvas/node graph culling” surfaces that should be prepaint-windowed.
  - Candidates:
    - `ecosystem/fret-node/src/*` (node graph viewport culling, edges/handles).
    - `ecosystem/fret-canvas/src/*` (large canvas surfaces).
    - `ecosystem/fret-viewport-tooling/src/*` (viewport overlays, gizmos).
  - Done when: we have an evidence-backed list + a first migration target (one component) with a perf/correctness harness.
  - Proposed harness plan (v1):
    - Add a UI Gallery page that renders a “large canvas scene” (thousands of nodes/edges or sprites) and supports pan/zoom.
    - Add a scripted capture that alternates between:
      - small pan deltas (should be paint-only or prepaint-only),
      - large pan jumps (should update the visible window deterministically),
      - zoom in/out (should update sampling/culling, but avoid full cache-root rerenders for small deltas).
    - Add stale-paint checks for pan/zoom (bounds/camera changed but scene fingerprint did not).
  - Likely first target:
    - `ecosystem/fret-node` (node graph) because it combines 2D culling + heavy paint ops and will stress both interaction routing and paint caching.
  - Progress (v1):
    - UI Gallery harness: `PAGE_CANVAS_CULL_TORTURE` with root test id `ui-gallery-canvas-cull-root` (pan/zoom canvas + viewport culling baseline).
    - Script: `tools/diag-scripts/ui-gallery-canvas-cull-torture-pan-zoom.json` (middle-drag + wheel).
    - Evidence bundle (cache+shell, release): `target/fret-diag-canvas-cull-torture/1769162100494-ui-gallery-canvas-cull-pan-zoom/bundle.json`
- [~] GPUI-MVP5-eco-005 Identify “chart/plot sampling” surfaces that should be prepaint-windowed.
  - Candidates:
    - `ecosystem/fret-chart/src/*` (timeseries/table-driven plots).
    - `ecosystem/fret-plot3d/src/*` (3D sampling + culling surfaces).
    - `ecosystem/delinea/src/*` (headless chart engine; windowed sampling).
  - Done when: we have an evidence-backed list + a first migration target (one component) with a perf/correctness harness.
  - Proposed harness plan (v1):
    - Add a UI Gallery page that renders a large timeseries (e.g. 1M points) with pan/zoom.
    - Define a deterministic sampling window contract:
      - pan -> shift visible x-window,
      - zoom -> adjust sampling density / visible x-window width.
    - Scripted capture should validate that small pans do not force cache-root rerenders, and that zoom changes are explainable in bundles.
  - Likely first target:
    - `ecosystem/delinea` (headless) + `ecosystem/fret-chart` (UI wrapper) because it cleanly separates “sampling math” from rendering.
  - Progress (v1):
    - UI Gallery harness: `PAGE_CHART_TORTURE` with root test id `ui-gallery-chart-torture-root` (large timeseries via `delinea` + `fret-chart`).
    - Script: `tools/diag-scripts/ui-gallery-chart-torture-pan-zoom.json` (drag + wheel).
    - Evidence bundle (cache+shell, release): `target/fret-diag-chart-torture/1769159171953-ui-gallery-chart-torture-pan-zoom/bundle.json`
    - Infrastructure: add `drag_pointer` to UI diagnostics steps (`ecosystem/fret-bootstrap/src/ui_diagnostics.rs`).
- [~] GPUI-MVP5-eco-006 Identify “paint-only chrome” surfaces that should not force rerender.
  - Candidates: caret/selection layers, hover/focus rings, drag/drop indicators, scrollbars, overlay arrows/anchors.
  - Done when: we have a first migration target (one component) with a regression harness that proves no cache-root rerender is needed for the effect.
  - Anchors:
    - `ecosystem/fret-code-view/tests/hover_is_paint_only.rs` (existing regression that hover does not force rerender).
  - Proposed harness plan (v1):
    - Add a UI Gallery “chrome torture” page that exercises hover/focus/pressed states across many widgets while view-cache + shell are enabled.
    - Scripted capture should include pointer-move sweeps, focus traversal, and repeated open/close of overlays.
    - Add a regression expectation: “hover-only” ticks should not list `notify_call` as a dirty-view source for the relevant cache roots.
  - Progress (v1):
    - UI Gallery harness: `PAGE_CHROME_TORTURE` with root test id `ui-gallery-chrome-torture-root`.
    - Script: `tools/diag-scripts/ui-gallery-chrome-torture.json` (pointer sweeps + focus traversal).
    - Evidence bundle (cache+shell, release): `target/fret-diag-chrome-torture/1769164619875-ui-gallery-chrome-torture/bundle.json`
    - Note: overlay open/close remains covered by `tools/diag-scripts/ui-gallery-overlay-torture.json` until the chrome harness grows stable overlay toggles.
    - Runtime support (v1): pointer hooks can request paint-only invalidation without rerender:
      - `crates/fret-ui/src/action.rs` (`UiPointerActionHost::invalidate`)
      - `crates/fret-ui/src/declarative/host_widget/event/pointer_region.rs`
      - `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`

- [x] GPUI-MVP5-eco-008 Docking: make drag/drop indicators paint-only under view-cache reuse.
  - Touches: `ecosystem/fret-docking/src/*`, `ecosystem/fret-ui-kit/src/*` (if shared chrome helpers are needed).
  - Done when: a harness can simulate “drag over docking targets” and confirm no cache-root rerender is needed
    for the indicator ticks (paint-only invalidation only), while still passing stale-paint checks.
  - Progress (v0):
    - Allow prepaint to kick paint-cache invalidations for cache roots:
      - `crates/fret-ui/src/widget.rs` (`PrepaintCx::{invalidate_self,invalidate}`)
      - `crates/fret-ui/src/retained_bridge.rs` (re-export `PrepaintCx` for retained widgets)
    - DockSpace kicks paint-cache replay on “start/stop frame-driven chrome” transitions:
      - `ecosystem/fret-docking/src/dock/space.rs` (`Widget::prepaint`, `PaintCx::request_animation_frame`)
    - Regression (unit): `ecosystem/fret-docking/src/dock/tests.rs` (`dock_space_kicks_paint_cache_on_drag_transition_for_cache_root`)
  - Progress (v1):
    - Scripted drags emit `InternalDrag` events so docking can be exercised without runner cursor routing:
      - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`drag_pointer` -> `InternalDrag::Over` + final `Drop`)
      - `docs/ui-diagnostics-and-scripted-tests.md` (step list + internal-drag note)
    - Docking demo exposes stable semantics anchors in the tab bar for scripted drags:
      - `apps/fret-examples/src/docking_demo.rs` (`dock-demo-tab-drag-anchor-{left,right}`)
    - Docking demo exposes the DockSpace cache root as a semantics `test_id` so `diag stats` can target it:
      - `ecosystem/fret-docking/src/dock/space.rs` (`DockSpace::with_semantics_test_id`)
      - `ecosystem/fret-docking/src/dock/mod.rs` (`create_dock_space_node_with_test_id`)
      - `apps/fret-examples/src/docking_demo.rs` (`dock-demo-dock-space`)
    - Script: `tools/diag-scripts/docking-demo-drag-indicators.json`
    - Gate: `fretboard diag stats <bundle.json> --check-drag-cache-root-paint-only dock-demo-dock-space`

- [x] GPUI-MVP5-eco-009 Workspace/inspectors: identify list/outline/file-tree surfaces that should be windowed.
  - Touches: `ecosystem/fret-workspace/src/*`, `apps/fret-editor/src/*`.
  - Done when: we have (1) an evidence-backed candidate list, (2) one migrated surface (windowed rows or VirtualList v2),
    and (3) a `diag` script that catches “looks stale / click hits correct but paint is stale” regressions.
  - Progress (v1):
    - Note: `apps/fret-editor` currently only contains the inspector protocol/services (no large inspector UI surface yet).
      To keep eco-009 moving, we exercise the “inspector-like property list” shape in UI Gallery as a stand-in harness.
    - UI Gallery harness page: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_INSPECTOR_TORTURE`), root test id `ui-gallery-inspector-root`.
    - Script: `tools/diag-scripts/ui-gallery-inspector-torture-scroll.json`.
    - Script (keep-alive bounce; ADR 0192): `tools/diag-scripts/ui-gallery-inspector-torture-bounce-keep-alive.json` (requires `FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE>0`).
    - Gate (cache+shell, retained host, release):
      - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-inspector-torture-scroll.json --warmup-frames 5 --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --check-view-cache-reuse-min 1 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 256 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-inspector-root --check-stale-paint ui-gallery-inspector-root --launch -- cargo run -p fret-ui-gallery --release`
    - Evidence bundle (cache+shell, release): `target/fret-diag-inspector-torture-local2/1769735532323-ui-gallery-inspector-torture-scroll/bundle.json`
    - Re-verified (cache+shell, release; retained gates enforced incl. attach/detach min): `target/fret-diag-inspector-suite-min-gate/1769756657266-ui-gallery-inspector-torture-scroll/bundle.json`
    - Evidence (keep-alive suite smoke; cache+shell, release; keep-alive gate enabled):
      - `target/fret-diag-inspector-keep-alive-smoke4/1769952707386-ui-gallery-inspector-torture-scroll/bundle.json`
      - `target/fret-diag-inspector-keep-alive-smoke4/1769952733603-ui-gallery-inspector-torture-bounce-keep-alive/bundle.json`
    - Builtin suite: `fretboard diag suite ui-gallery-inspector-torture --launch -- cargo run -p fret-ui-gallery --release` defaults to `--warmup-frames 5`, enables `cache+shell`, and enforces the retained VirtualList gates above.
      - Note (2026-01-30): retained VirtualList post-run checks are applied whenever configured (no per-script whitelist), so suite gates are effective.
    - Builtin suite (keep-alive): `fretboard diag suite ui-gallery-inspector-torture-keep-alive --launch -- cargo run -p fret-ui-gallery --release` defaults to `FRET_UI_GALLERY_INSPECTOR_KEEP_ALIVE=256` and enforces `--check-retained-vlist-keep-alive-reuse-min 1`.
    - UI Gallery harness page: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_FILE_TREE_TORTURE`), root test id `ui-gallery-file-tree-root`.
      - Row test ids are stable by identity: `ui-gallery-file-tree-node-{numeric_id}` (not row index).
    - Script: `tools/diag-scripts/ui-gallery-file-tree-torture-scroll.json`.
      - Gate (cache+shell, retained host, release):
        - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-file-tree-torture-scroll.json --warmup-frames 5 --timeout-ms 240000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --check-view-cache-reuse-min 1 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 256 --check-retained-vlist-scroll-window-dirty-max 0 --check-wheel-scroll ui-gallery-file-tree-node-0 --check-stale-paint ui-gallery-file-tree-node-0 --launch -- cargo run -p fret-ui-gallery --release`
    - Evidence bundle (cache+shell, release): `target/fret-diag-file-tree-suite-local3/1769748233062-ui-gallery-file-tree-torture-scroll/bundle.json`
    - Re-verified (cache+shell, release; retained gates enforced incl. attach/detach min): `target/fret-diag-file-tree-suite-min-gate/1769756694953-ui-gallery-file-tree-torture-scroll/bundle.json`
    - Builtin suite: `fretboard diag suite ui-gallery-file-tree-torture --launch -- cargo run -p fret-ui-gallery --release` defaults to `--warmup-frames 5`, enables `cache+shell`, and enforces the retained VirtualList gates above.
    - Interactive script (expand/collapse + selection + scroll): `tools/diag-scripts/ui-gallery-file-tree-torture-toggle.json`.
      - Builtin suite: `fretboard diag suite ui-gallery-file-tree-torture-interactive --launch -- cargo run -p fret-ui-gallery --release`.
      - Evidence bundle (cache+shell, release): `target/fret-diag-file-tree-suite-local4/1769748662066-ui-gallery-file-tree-torture-toggle/bundle.json`
      - Re-verified (cache+shell, release; retained gates enforced incl. attach/detach min): `target/fret-diag-file-tree-suite-min-gate/1769756742386-ui-gallery-file-tree-torture-toggle/bundle.json`
  - Next steps (eco-009 closure path; keep it “real UI surface” oriented):
    - [x] Promote the UI Gallery file-tree torture implementation into a reusable `ecosystem/fret-ui-kit` retained component (keep stable test ids),
      so future workspace adoption is a “swap the consumer” change, not a rewrite.
      - Implementation:
        - `ecosystem/fret-ui-kit/src/declarative/file_tree.rs` (`file_tree_view_retained_v0`)
        - `apps/fret-ui-gallery/src/ui.rs` (`preview_file_tree_torture` calling into ui-kit)
      - Closure gate (must stay green):
        - `fretboard diag suite ui-gallery-file-tree-torture --launch -- cargo run -p fret-ui-gallery --release`
        - `fretboard diag suite ui-gallery-file-tree-torture-interactive --launch -- cargo run -p fret-ui-gallery --release`
    - [x] Add a short “candidate surface list” with anchors to real code (as it appears).
      - Workspace shell surfaces (today):
        - Pane tree + split resize + tab drag/drop: `ecosystem/fret-workspace/src/panes.rs` (`workspace_pane_tree_element_with_resize`)
        - Tab strip (potentially large; needs stable cache roots + paint-only chrome for hover/drag indicators): `ecosystem/fret-workspace/src/tab_strip.rs` (`WorkspaceTabStrip::into_element`)
        - Tab MRU/dirty state model (feeds the tab strip): `ecosystem/fret-workspace/src/tabs.rs` (`WorkspaceTabs`)
        - Inspector/properties surfaces (future; `apps/fret-editor` currently only has protocol/services):
          - Property edit plumbing: `apps/fret-editor/src/property_edit.rs` (`PropertyEditService`), `apps/fret-editor/src/inspector_edit.rs` (`InspectorEditService`)
          - Policy recommendation when a real inspector UI lands: adopt retained-host windowed rows by default (ADR 0190/0192),
            and keep row hover/selection chrome paint-only (ADR 0181) so cache-hit frames remain correct.
    - [x] Add a “workspace shell” scripted demo that exercises real surfaces (pane tree + tab strip) rather than staying UI-Gallery-only.
      - Goal: an end-to-end `diag` bundle proves “cache-hit frames stay correct” (no stale paint) while still allowing paint-only chrome and retained windowed surfaces to update.
      - Implementation sketch:
        - New demo: `apps/fret-examples/src/workspace_shell_demo.rs` (wired through `fret-demo` like `docking_demo`).
        - Use real workspace surfaces: `ecosystem/fret-workspace/src/panes.rs` + `ecosystem/fret-workspace/src/tab_strip.rs`.
        - Script recording: drive + record via `UiDiagnosticsService` (see `apps/fret-examples/src/docking_demo.rs`).
        - Stable semantics anchors: tab drag handles + pane roots should expose `test_id` values for scripts.
      - Script: `tools/diag-scripts/workspace-shell-demo-tab-drag-and-scroll.json`
      - Builtin suite: `fretboard diag suite workspace-shell-demo --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`
        - Defaults: `--warmup-frames 5`, enables `cache+shell`, and enforces:
          - `--check-view-cache-reuse-min 1`
          - `--check-retained-vlist-reconcile-no-notify 1`
          - `--check-retained-vlist-reconcile-cache-reuse 1`
          - `--check-retained-vlist-scroll-window-dirty-max 0`
          - `--check-wheel-scroll workspace-shell-file-tree-node-0`
          - `--check-stale-paint workspace-shell-file-tree-node-0`
      - Evidence bundle (cache+shell, release): `target/fret-diag-workspace-shell-demo-tighten2/1769932026789-workspace-shell-demo-tab-drag-and-scroll/bundle.json`
      - Follow-up (keep-alive bounce; ADR 0192):
        - Env: `FRET_WORKSPACE_SHELL_FILE_TREE_KEEP_ALIVE=256`
        - Script: `tools/diag-scripts/workspace-shell-demo-file-tree-bounce-keep-alive.json`
        - Builtin suite: `fretboard diag suite workspace-shell-demo-file-tree-keep-alive --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`
        - Evidence bundle (cache+shell, release; keep-alive gate enabled): `target/fret-diag/1769956458243-workspace-shell-demo-file-tree-bounce-keep-alive/bundle.json`
    - [x] Adopt the ui-kit retained file tree component in the workspace shell (or a thin adapter), so eco-009 directly validates workspace integration.
      - Component: `ecosystem/fret-ui-kit/src/declarative/file_tree.rs` (`file_tree_view_retained_v0`).
      - Expectation: toggling nodes + scrolling should be handled by retained-host reconcile + paint-only chrome (no parent cache-root rerender).
    - [x] Capture and link evidence bundles for the workspace shell script under `cache+shell` (release), and mark eco-009 `[x]` when the gate stays green.
    - [x] Migrate exactly one real surface (not UI Gallery) onto the retained/windowed substrate and add a `diag` script for it.
      - Target (v0): `apps/fret-examples/src/components_gallery.rs` (file-tree panel).
      - Scripts:
        - `tools/diag-scripts/components-gallery-file-tree-window-boundary-scroll.json`
        - `tools/diag-scripts/components-gallery-file-tree-toggle-and-scroll.json`
      - Env (recommended): `FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE=1` (optional `…_N=50000`), `FRET_EXAMPLES_VIEW_CACHE=1`, `FRET_DIAG=1`.
      - Note: the torture surface includes an expandable `folder_1` at `TreeItemId=1` so the toggle harness can drive expand/collapse while keeping a large list.
      - Gate (example): `fretboard diag stats <bundle.json> --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-reconcile-cache-reuse 1 --check-retained-vlist-attach-detach-min 1 --check-retained-vlist-attach-detach-max 256 --check-retained-vlist-scroll-window-dirty-max 0 --check-stale-paint components-gallery-file-tree-root`
      - Builtin suite:
        - `cargo run -p fretboard -- diag suite components-gallery-file-tree --launch -- cargo run -p fret-demo --bin components_gallery --release`
      - Evidence bundles (suite, view-cache, release):
        - `C:\fret-diag-components-gallery-file-tree-suite-2scripts\1769829965598-components-gallery-file-tree-window-boundary-scroll/bundle.json`
        - `C:\fret-diag-components-gallery-file-tree-suite-2scripts\1769829992147-components-gallery-file-tree-toggle-and-scroll/bundle.json`
        - `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862343305-components-gallery-file-tree-window-boundary-scroll/bundle.json`
        - `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862370728-components-gallery-file-tree-toggle-and-scroll/bundle.json`
        - `target/fret-diag-smoke-components-gallery-file-tree-suite-attach64/1769862397267-components-gallery-file-tree-window-boundary-bounce/bundle.json`
      - Perf baselines (warmup=5, view-cache, release; worst tick max.us total/layout/prepaint/paint):
        - window-boundary: `C:\fret-diag-perf-components-gallery-file-tree-boundary\1769830674611-script-step-0022-wheel/bundle.json` (2897/2216/30/717)
        - toggle+scroll: `C:\fret-diag-perf-components-gallery-file-tree-toggle\1769830707477-script-step-0021-wheel/bundle.json` (2719/2035/26/768)
    - [x] Migrate one more real surface (table) onto the retained/windowed substrate and add a `diag` suite for it.
      - Target (v0): `apps/fret-examples/src/components_gallery.rs` (table torture mode).
      - Scripts:
        - `tools/diag-scripts/components-gallery-table-window-boundary-scroll.json`
        - `tools/diag-scripts/components-gallery-table-sort-and-scroll.json`
        - (keep-alive bounce; ADR 0192): `tools/diag-scripts/components-gallery-table-window-boundary-bounce.json`
      - Env (recommended): `FRET_COMPONENTS_GALLERY_TABLE_TORTURE=1` (optional `…_N=50000`), `FRET_EXAMPLES_VIEW_CACHE=1`.
      - Builtin suite:
        - `cargo run -p fretboard -- diag suite components-gallery-table --launch -- cargo run -p fret-demo --bin components_gallery --release`
        - Note: the builtin suite sets `FRET_COMPONENTS_GALLERY_TABLE_TORTURE=1` automatically (override with `--env`).
      - Keep-alive suite:
        - `cargo run -p fretboard -- diag suite components-gallery-table-keep-alive --launch -- cargo run -p fret-demo --bin components_gallery --release`
        - Default env: `FRET_COMPONENTS_GALLERY_TABLE_TORTURE=1` and `FRET_COMPONENTS_GALLERY_TABLE_KEEP_ALIVE=256` (override with `--env`).
      - Evidence bundles (suite, view-cache, release):
        - `C:\fret-diag-components-gallery-table-suite-2scripts3\1769833380478-components-gallery-table-window-boundary-scroll/bundle.json`
        - `C:\fret-diag-components-gallery-table-suite-2scripts3\1769833406244-components-gallery-table-sort-and-scroll/bundle.json`
        - `target/fret-diag-smoke-components-gallery-table-suite-attach64/1769862641667-components-gallery-table-window-boundary-scroll/bundle.json`
        - `target/fret-diag-smoke-components-gallery-table-suite-attach64/1769862668739-components-gallery-table-sort-and-scroll/bundle.json`
        - `target/fret-diag-perf-components-gallery-table-suite-prefetch6/1769878293896-components-gallery-table-window-boundary-scroll/bundle.json`
        - `target/fret-diag-perf-components-gallery-table-suite-prefetch6/1769878326558-components-gallery-table-sort-and-scroll/bundle.json`
        - `target/fret-diag-suite-components-gallery-table-nonretained0/1770032466340-components-gallery-table-window-boundary-scroll/bundle.json`
        - `target/fret-diag-suite-components-gallery-table-nonretained0/1770032494240-components-gallery-table-sort-and-scroll/bundle.json`
      - Gate evidence (suite-level, retained-vlist):
        - `target/fret-diag-suite-components-gallery-table-nonretained0/check.vlist_window_shifts_non_retained_max.json`
      - Evidence (keep-alive bounce; cache+shell, release; keep-alive gate enabled):
        - `target/fret-diag/1769958870151-components-gallery-table-window-boundary-bounce/bundle.json`
      - Note: in this multi-script suite, retained-vlist *window-boundary* gates apply only to
        `components-gallery-table-window-boundary-scroll.json` and `components-gallery-table-window-boundary-bounce.json`
        (the sort+scroll script is still gated by view-cache reuse + wheel-scroll + stale-paint, etc.).
      - Note: the suite also enables `--check-vlist-policy-key-stable`, applied only to the window-boundary scripts
        (policy changes are expected in sort/toggle style scripts).
      - Note: the suite sets `--check-retained-vlist-attach-detach-max 64` on the window-boundary scripts to catch extreme row churn regressions.
      - Perf baselines (warmup=5, view-cache, release; worst tick max.us total/layout/prepaint/paint):
        - window-boundary: `C:\fret-diag-perf-components-gallery-table-boundary\1769833617760-script-step-0018-wheel/bundle.json` (2757/1989/13/755)
        - sort+scroll: `C:\fret-diag-perf-components-gallery-table-sort\1769833651344-script-step-0011-wheel/bundle.json` (6155/4682/11/1462)
      - After tuned staged prefetch (warmup=5; `--check-retained-vlist-prefetch-reconciles-min 1`):
        - window-boundary: `target/fret-diag-perf-components-gallery-table-suite-prefetch6/1769878293896-components-gallery-table-window-boundary-scroll/bundle.json` (2201/1732/21/448)
        - sort+scroll: `target/fret-diag-perf-components-gallery-table-suite-prefetch6/1769878326558-components-gallery-table-sort-and-scroll/bundle.json` (4413/3249/11/1153)

- [~] GPUI-MVP5-eco-010 AI transcript surfaces: prepaint-windowed + paint-only selection/hover chrome.
  - Touches: `ecosystem/fret-ui-ai/src/*`, `apps/fret-ui-gallery/src/*`, `apps/fretboard/src/diag.rs`.
  - Done when: append-heavy transcript updates no longer rebuild/relayout the entire history while scrolling, and the harness proves stable paint
    under view-cache reuse.
  - Progress (v1):
    - UI Gallery harness: `PAGE_AI_TRANSCRIPT_TORTURE` with root test id `ui-gallery-ai-transcript-root`.
      - Note: the harness uses a bounded viewport (`h_px(Px(460.0))`) so VirtualList window telemetry is meaningful.
    - Script: `tools/diag-scripts/ui-gallery-ai-transcript-torture-scroll.json`.
    - Gate (suite): `fretboard diag suite ui-gallery-ai-transcript-retained --warmup-frames 5 --check-retained-vlist-reconcile-no-notify 1 --check-retained-vlist-attach-detach-max 256 --check-retained-vlist-scroll-window-dirty-max 0 --check-view-cache-reuse-min 10 --check-wheel-scroll ui-gallery-ai-transcript-row-0 --check-stale-paint ui-gallery-ai-transcript-row-0 ...`
      - Defaults: enables view-cache + shell and sets `FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT=1`.
    - Evidence bundle (cache+shell): `target/fret-diag/1769689580999-ui-gallery-ai-transcript-torture-scroll/bundle.json`.
- [~] GPUI-MVP5-perf-002 Reduce input-driven `notify_call` hotspots by narrowing cache roots or targeting dirtiness.
  - Goal: VirtualList torture no longer attributes the dominant `notify_call` hotspot to `pressable.rs:*` while preserving correctness.
  - Instrumentation (v2): bundles now export bounded `debug.notify_requests` with `notify()` callsite attribution
    (file/line/col) so notify hotspots are gateable.
    - Anchors: `crates/fret-ui/src/widget.rs` (`EventCx::notify`), `crates/fret-ui/src/tree/mod.rs` (`UiDebugNotifyRequest`),
      `crates/fret-ui/src/tree/dispatch.rs` (recording), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (bundle export).
    - Gate: `fretboard diag stats <bundle> --check-notify-hotspot-file-max <file> <max>` writes `check.notify_hotspots.json`
      next to the bundle (even on failure).
      - Anchors: `apps/fretboard/src/diag/stats.rs`, `apps/fretboard/src/diag/mod.rs`.
  - Baseline note (pre-v1): worst-tick bundles were layout-dominated and frequently attributed dirty views to
    `UiDebugInvalidationDetail::notify_call` from `crates/fret-ui/src/declarative/host_widget/event/pressable.rs:*`.
  - Progress (v1):
    - `Pressable` no longer implicitly calls `notify()` after invoking `on_activate`. If a hook mutates non-model state
      that must be reflected in declarative render under view-cache reuse, it should call `host.notify(action_cx)`
      explicitly.
      - Anchors: `crates/fret-ui/src/action.rs` (`UiActionHost::notify`), `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
      - Example adoption: `ecosystem/fret-code-view/src/copy_button.rs` (Copied feedback uses `host.notify(...)` from both activate + timer hooks).
      - Perf evidence (cache+shell, release): `target/fret-diag-perf-explicit-notify/1769155887844-script-step-0011-click/bundle.json`
        - Note: the worst “steady-state” tick in this bundle is layout-dominated, but `diag stats --sort time` no longer reports a dirty-view source
          attributed to `UiDebugInvalidationDetail::notify_call` from `pressable.rs:*`.
  - Done when:
    - Diagnostics bundles export bounded `debug.notify_requests` with file/line/col attribution for `notify()`.
    - `fretboard diag stats <bundle> --check-notify-hotspot-file-max crates/fret-ui/src/declarative/host_widget/event/pressable.rs 0`
      passes for the warmup-ranked worst bundles under the chosen harness/suite.
    - On failure, the gate writes `check.notify_hotspots.json` next to the bundle (with bounded offender samples and a stable aggregation key).
- [x] GPUI-MVP5-perf-003 Explain and de-risk `scroll_handle_layout` dirtiness when `window_mismatch=false`.
  - Goal: eliminate “looks stale / updates a frame late” and “unexpected relayout” classes of bugs by making scroll-handle invalidation explainable and minimal.
  - Hypothesis: some frames mark scroll-handle changes as `Layout` even when offset is unchanged (e.g. content size changes, viewport changes, or a too-eager upgrade path).
  - Proposed work:
    - Extend debug export to include the reason why a scroll-handle key was reported as `Layout` vs `HitTestOnly` (bounded debug-only).
    - Tighten classification so “no-op updates” (offset unchanged, content size unchanged) do not generate a change key.
    - Add a focused regression harness derived from `tools/diag-scripts/ui-gallery-virtual-list-torture.json` that targets the tick in
      `target/fret-diag/1769096169296-script-step-0011-click/bundle.json` and asserts that `scroll_handle_layout` implies a window mismatch,
      a content-size/viewport delta, or a deferred command consumption (observable in the bundle).
  - Progress (v1):
    - Diagnostics bundles can now export per-frame `debug.scroll_handle_changes` (bounded) with the exact deltas that drove scroll-handle invalidation.
      - Anchors: `crates/fret-ui/src/declarative/frame.rs` (`take_changed_scroll_handle_keys`), `crates/fret-ui/src/tree/layout.rs` (debug record),
        `crates/fret-ui/src/tree/mod.rs` (`UiDebugScrollHandleChange`), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiScrollHandleChangeV1`).
    - Scroll-handle revisions caused solely by viewport/content-size updates are now treated as `HitTestOnly` (repaint + hit-test), not `Layout`.
      This avoids view-cache rerenders/contained relayouts for scrollbars and other transform-only consumers.
    - Scroll-handle "revision-only" changes (revision changed, but offset/viewport/content unchanged) are treated as `HitTestOnly` by default,
      and are upgraded back to `Layout` only when a `VirtualList` must consume a deferred `scroll_to_item` request or the visible window leaves its overscan.
      This avoids false cache-root rerenders when a scroll handle is shared across multiple layout surfaces (e.g. table horizontal scroll sync).
      - Anchors: `crates/fret-ui/src/tree/layout.rs` (`invalidate_scroll_handle_bindings_for_changed_handles`),
        `crates/fret-ui/src/scroll.rs` (`VirtualListScrollHandle` request tracking).
  - Evidence (local bundles):
    - In `target/fret-diag-scroll-handle-repro/1769098640774-ui-gallery-virtual-list-edit-9000/bundle.json` at `tick_id=7`, the scroll handle bound to the
      VirtualList reports `content_changed=true` (280032 -> 280064) and was previously classified as `kind=layout`.
    - After the classification tightening, in `target/fret-diag-scroll-handle-repro2/1769099048813-ui-gallery-virtual-list-edit-9000/bundle.json` at `tick_id=7`,
      the same pattern reports `kind=hit_test_only` while still capturing the content/offset deltas in `debug.scroll_handle_changes`.
    - After fixing scroll-handle registry bookkeeping (so `prev_offset` matches internal layout updates), in
      `target/fret-diag-scroll-handle-after-fix/1769131324359-ui-gallery-virtual-list-edit-9000/bundle.json` at `tick_id=7`,
      the VirtualList scroll handle reports `prev_offset_y=252032` and `offset_changed=false` (content-only delta), avoiding spurious “jump” classification.
    - Perf improvement evidence (same script, cache+shell):
      - Before: `target/fret-diag/1769096169296-script-step-0011-click/bundle.json` top.us(total/layout/prepaint/paint)=503161/476991/241/25929
      - After: `target/fret-diag-perf-scroll-handle-after-fix/1769131393110-script-step-0011-click/bundle.json` top.us(total/layout/prepaint/paint)=244120/226780/165/17175
    - Table scroll baseline now shows the "revision-only" scroll-handle churn does not force `scroll_handle_layout` dirtiness on the cache root:
      - `target/fret-diag-perf-data-table-torture5/1769150942029-script-step-0009-wheel/bundle.json`
    - Mount invalidation overhead: reduce redundant invalidation propagation for newly mounted nodes.
      - Change: `declarative_instance_change_mask(None, _) -> 0` and a mount-only `UiTree::set_children_in_mount` path to avoid emitting
        per-node invalidation walks for freshly created nodes whose invalidation flags are already set.
      - Evidence (release perf; avoids Windows debug PDB limits): `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-virtual-list-torture.json --warmup-frames 5 --top 5 --sort time --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`
        produced `target/fret-diag-perf-mount-invalidation/1769133988059-script-step-0011-click/bundle.json` with warmup-ranked top.us(total/layout/prepaint/paint)=30365/29262/67/1036 and `debug.stats.invalidation_walk_calls=45`.
  - Progress: VirtualList torture rows are now wrapped in per-row view-cache roots (keyed by item key) so the shell can rerender without rebuilding heavy rows.

## Open Questions (Keep Short)

- Which interaction stream comes first for maximum “feel” payoff: hit regions vs cursor styles vs outside-press observers?
- Do we want to keep cache roots strictly opt-in only, or also provide guided defaults in demos (never in core)?
