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
- `request_animation_frame()` parity note (Zed/GPUI): in GPUI, RAF requested from within a view effectively `notify`s
  that view on the next frame. In Fret today, RAF guarantees a paint pass (blocks paint replay) but does not
  necessarily force a declarative rerender for cache roots unless the caller explicitly `notify()`s or triggers a
  layout invalidation. Track this gap explicitly (see MVP2 tasks below).

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
- [x] GPUI-MVP0-perf-006 Avoid false global-change churn from stable “service globals”.
  - Touches: `ecosystem/fret-ui-kit/src/dnd/service.rs`
  - Notes: use `with_global_mut_untracked` for lazy init + stable read paths (prevents global-change tracking from firing on every frame).
- [x] GPUI-MVP0-perf-007 Avoid false global-change churn from frame-local overlay registries.
  - Touches: `ecosystem/fret-ui-shadcn/src/a11y_modal.rs`
  - Notes: `ModalA11yRegistry` is a frame-local registry used during modal overlay construction; it should not participate in host global-change tracking.

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
- [x] GPUI-MVP2-rt-003 Make `request_animation_frame()` dirty the current view (GPUI-aligned).
  - Touches: `crates/fret-ui/src/elements/cx.rs` (`ElementContext::request_animation_frame`), plus dirty-view scheduling if needed.
  - Goal: if a subtree relies on frame-driven updates (animations), `request_animation_frame()` must not allow a cache-hit frame to replay stale output indefinitely.
  - Reference: `repo-ref/zed/crates/gpui/src/window.rs` (`request_animation_frame` -> notify view / dirty views).
  - Notes: v1 implements this as `request_animation_frame()` implying `notify()` on the nearest cache root.
  - Evidence:
    - `crates/fret-ui/src/elements/cx.rs` (`ElementContext::request_animation_frame`)
    - `crates/fret-ui/src/declarative/mount.rs` (drains animation-frame notify requests and invalidates with `UiDebugInvalidationDetail::AnimationFrameRequest`)
    - `crates/fret-ui/src/declarative/tests/view_cache.rs` (`request_animation_frame_marks_view_cache_root_dirty`)
- [~] GPUI-MVP2-cache-003 Gate view-cache reuse on dirty views.
  - Touches: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/declarative/mount.rs`, `crates/fret-ui/src/elements/runtime.rs`
  - Done when: a notified view never reuses cached ranges; a clean view reliably reuses them.
  - Progress: `notify` marks the nearest cache root as `view_cache_needs_rerender`, which disables view-cache reuse for that root.
  - Progress: model/global observation invalidation also marks cache roots dirty (`view_cache_needs_rerender`) so reuse is disabled on data changes.
  - Progress: cache-hit frames still uplift element-recorded observations to cache roots (prevents stale cache-hit when an input event changes model state but the subtree is reused).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`should_reuse_view_cache_node`, `invalidation_source_marks_view_dirty`), `crates/fret-ui/src/widget.rs` (`EventCx::notify`), `crates/fret-ui/src/elements/runtime.rs`,
    `crates/fret-ui/src/tree/tests/view_cache.rs` (`view_cache_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots`).

- [x] GPUI-MVP2-cache-004 Stabilize overlay interactions under `ViewCache` shell reuse.
  - Touches: `crates/fret-ui/src/declarative/mount.rs`
  - Goal: `tools/diag-scripts/ui-gallery-overlay-torture.json` completes with `FRET_UI_GALLERY_VIEW_CACHE=1` and `FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`.
  - Root cause: the declarative element GC ("stale nodes after gc lag frames") is keyed off `last_seen_frame`, but view-cache reuse intentionally skips re-mounting cached subtrees.
    This caused live overlay subtree nodes (e.g. `ui-gallery-overlay-reset`, `ui-gallery-dialog-trigger`) to be swept as soon as shell caching started reusing roots.
  - Fix (temporary liveness rule): skip stale-node sweeping when `UiTree::view_cache_enabled()` is on, to prevent deleting live cached subtrees.
  - Hardening: replay cached tooltip/hover-overlay requests when a cache-hit frame skips the subtree that emits them (prevents transient unmounts under shell reuse).
    - Touches: `ecosystem/fret-ui-kit/src/window_overlays/frame.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-kit/src/window_overlays/state.rs`
  - Evidence (pass): `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-overlay-torture.json --timeout-ms 120000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`
  - Evidence (perf): `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`
  - Follow-up: reintroduce GC with GPUI-aligned "cache root liveness" (dirty views + notify) so cached subtrees can be skipped without leaking detached nodes.

- [~] GPUI-MVP2-cache-005 Reintroduce declarative node GC with explicit cache-root liveness.
  - Touches: `crates/fret-ui/src/declarative/mount.rs` (GC), plus cache-hit frame liveness bookkeeping.
  - Goal: collect truly-detached nodes without deleting live cached subtrees (keep `ui-gallery-overlay-torture.json` green under shell reuse).
  - Notes: a naive "detached + stale" sweep is not sufficient on its own; cache-hit frames can still cause overlay/demo semantics to disappear if layer/overlay presence
    depends on rerendered outputs. We likely need a GPUI-style "effect output replay" (or equivalent) for cache-hit frames (layers/overlays/semantics roots).
  - Progress: add a diagnostics capture when script-driven clicks cannot resolve a selector in the current semantics snapshot.
    - Touches: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`script-step-XXXX-click-no-semantics-match` forced dump label).
    - Motivation: cache-hit correctness regressions can present as “step N stuck” without producing a per-step bundle; this ensures we always capture a bundle at the first missing target.
  - Progress: make cache-hit subtree walks use `UiTree::children` (avoid stale/partial `window_frame.children` tables).
    - Touches: `crates/fret-ui/src/declarative/mount.rs` (cache-hit helpers).
  - Progress: introduce cache-root subtree bookkeeping for future liveness/GC work.
    - Touches: `crates/fret-ui/src/elements/runtime.rs` (`view_cache_subtree_elements`).
  - Current state: keep stale sweeping disabled for the main window root when `UiTree::view_cache_enabled()` is on.
    - Rationale: enabling sweeping for the main root still causes `ui-gallery-overlay-torture.json` to get stuck at step 10
      (`ui-gallery-dialog-trigger` disappears from the captured semantics snapshot; bundle label `script-step-0010-click-no-semantics-match`).
    - Symptom details: when the overlay page subtree is wrapped in `cached_subtree_with(...contained_layout(true))` (nested view-cache root),
      the semantics snapshot drops 10 overlay demo test IDs once the outer content cache root becomes a cache hit:
      `ui-gallery-overlay-reset`, `ui-gallery-dialog-trigger`, `ui-gallery-popover-trigger`, `ui-gallery-tooltip-trigger`,
      `ui-gallery-dropdown-trigger`, `ui-gallery-context-trigger`, `ui-gallery-hovercard-trigger`,
      plus transient overlay IDs like `ui-gallery-overlay-underlay`, `ui-gallery-popover-close`, `ui-gallery-overlay-last-action`.
      This leaves the script stuck retrying the click until timeout (auto-dump captured at the first missing selector).
    - We still sweep detached nodes for dismissible overlay roots via `render_dismissible_root_impl`.
  - Evidence:
    - `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --warmup-frames 5 --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`
    - Failure bundles (when main-root sweeping is enabled):
      - `target/fret-diag/1768831095473-script-step-0010-click-no-semantics-match/bundle.json`
      - `target/fret-diag/1768828347887-script-step-0010-click-no-semantics-match/bundle.json`
    - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json --dir target/fret-diag-sidebar-scroll --timeout-ms 300000 --poll-ms 200 --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release`

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

- [ ] GPUI-MVP4-eco-001 Add an ecosystem-facing “cached subtree” helper API (policy-free).
  - Touches: `ecosystem/fret-ui-kit/src/*`
- [ ] GPUI-MVP4-demo-002 Migrate `fret-ui-gallery` hotspots to the new patterns (hover chrome, scrollbars, code views).
  - Touches: `apps/fret-ui-gallery/src/*`, selected `ecosystem/*` components

## Open Questions (Keep Short)

- Which interaction stream comes first for maximum “feel” payoff: hit regions vs cursor styles vs outside-press observers?
- Do we want to keep cache roots strictly opt-in only, or also provide guided defaults in demos (never in core)?
