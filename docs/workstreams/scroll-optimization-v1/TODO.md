# Scroll Optimization Workstream (v1) — TODO

Date: 2026-05-16
Status: Active

## Candidate perf slice — Resize-jitter ScrollArea layout root attribution

- [x] Reproduce the local resize-jitter post-row-fragment attribution with scroll/layout profiling enabled.
  - Seed evidence:
    `target/fret-diag/local-next-editor-paint-20260516-retained-row-fragment-resize-jitter-r3/worst.stats.json`
  - Current worst-frame shape: `total=1464us`, `layout=796us`, `layout_roots=538us`,
    `layout_engine_solve=395us`, `paint=450us`.
  - Top layout hotspot: gallery content `ScrollArea` / `Scroll` with `263us` exclusive and `388us`
    inclusive layout time.
  - Profiled rerun:
    `target/fret-diag/local-next-scroll-layout-resize-jitter-20260516-r1/worst.stats.json`
  - Triage:
    `target/fret-diag/local-next-scroll-layout-resize-jitter-20260516-r1/triage.json`
  - Profiled worst-frame shape: `total=1362us`, `layout=805us`, `layout_roots=574us`,
    `layout_engine_solve=373us`, `paint=397us`.
- [x] Explain why the gallery content view-cache root is `reuse_reason=needs_rerender` during the resize-jitter
  probe even though the editor row-fragment path is stable.
- [x] Classify the `ScrollArea` cost more narrowly.
  - Current profile rules out child measure as the dominant owner (`measure_children_us=0` on the
    worst content viewport profile).
  - Current profile rules out dirty-descendant escalation in that worst frame
    (`direct_children_layout_invalidated=false`, `descendant_subtree_layout_dirty=false`).
  - Fresh local evidence first reported `layout_dirty_source=notify` /
    `layout_dirty_detail=animation_frame_request`; after converting code-editor torture autoscroll
    to paint-only RAF, the dirty source is the legitimate `scroll_handle_window_update`.
  - Remaining candidate: windowed-paint scroll-handle updates force the parent content view-cache
    root to rerender even when retained row fragments already cover the current visible row window.
- [x] Only after classification, choose the implementation owner:
  - `crates/fret-ui` for scroll mechanism or layout-root scheduling,
  - `ecosystem/fret-ui-shadcn` for recipe/chrome policy,
  - or `ui-perf-zed-smoothness-v1` for a broader resize solve batching contract.
  - Decision: keep the next implementation owner in `crates/fret-ui` / `ecosystem/fret-ui-kit`
    windowed-paint scroll reuse semantics, not in shadcn `ScrollArea` recipe policy.
- [x] Add one focused gate before changing behavior, preferably a single resize-jitter diag script plus
  `diag stats --sort time --top 15 --json` evidence.
- [x] Investigate a retained-windowed-paint scroll update path that can avoid marking the parent
  content view-cache root `needs_rerender` when the current retained row-fragment cache already
  covers the visible row window.
  - Guardrail: do not weaken `Scroll.windowed_paint=true` globally. The existing rule is still
    correct for non-retained windowed paint and virtualized surfaces that need declarative window
    updates.
  - Required proof: a focused Rust regression for the retained/windowed case plus the same
    resize-jitter diag sample showing row replay/store invariants stay stable.
  - Implemented proof: `Scroll.windowed_paint` offset-only updates now mark the cache root paint
    dirty with `ScrollHandleWindowUpdate` instead of forcing the nearest view-cache root to
    rerender. Non-retained `VirtualList` window updates still keep the rerender escape hatch.
  - Post-merge local evidence:
    `target/fret-diag/local-next-no4090-windowed-scroll-paint-only-post-merge-20260517-r3/worst.stats.json`.
  - Result: repeat=3 p95 total/layout/solve/prepaint/paint is `1697/1048/346/260/408us`;
    `top_view_cache_roots_needs_rerender=0`, `top_view_cache_roots_reused=1`, row replay/store
    stays `289/0`, and row-scene replay hit rate stays `100%`.
- [x] Attribute the remaining changing-bounds content `Scroll` layout cost after the view-cache
  root rerender escape is gone.
  - Current owner: worst-bundle `layout_roots_time_us=812us` and `layout_engine_solve_time_us=346us`;
    the content `Scroll` hotspot is still visible (`214us` exclusive, `533us` inclusive) while
    renderer text and code-editor row replay remain bounded.
  - The top solves are all `new_frame_key_changed` small-width-delta solves with `measure_time_us=0`,
    so the first follow-up should target bounds-change solve/apply work before any text-measure cache
    or renderer work.
  - Candidate: a narrower contained-root changing-bounds apply/solve path, not a shadcn
    `ScrollArea` recipe rewrite and not another renderer/text slice.
  - Fresh local no-4090 attribution:
    `target/fret-diag/local-next-root-solve-attrib-20260517-r1/worst.stats.json`.
  - Result: p95 total/layout/layout-roots/solve/prepaint/paint is
    `1242/682/468/321/264/379us`; the worst frame is
    `1242/618/404/305/251/373us` with `4` layout-engine solves,
    `top_view_cache_roots_needs_rerender=0`, `top_view_cache_roots_reused=1`,
    rows replayed/stored `289/0`, and renderer text prepare `64us`.
  - Top solves remain small-width-delta `new_frame_key_changed` roots with no measured widget/text
    time: content `Semantics` `172us` (`available_w_delta=-4`, `subtree_nodes=136`), root
    `Stack` `128us` (`available_w_delta=-4`, `subtree_nodes=102`), and editor
    `PointerRegion` `3us` (`available_w_delta=-4`, `subtree_nodes=2`).
- [x] Design and implement the next root-solve / geometry-propagation slice.
  - Do not simply skip `Scroll` child layout or extend the current safe-subset whitelist by name.
    `Scroll` still owns handle viewport/content updates, deferred probe state, overflow observation,
    and child transform side effects.
  - Start from `crates/fret-ui/src/tree/layout/node.rs::try_propagate_clean_engine_layout` and
    `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs::layout_scroll_impl`.
  - Implemented answer: engine-backed clean roots can skip the barrier/root Taffy solve only during
    small-step interactive width-only resize when child bounds can be derived from previous clean
    geometry.
  - `Scroll` is a boundary, not a skipped node: parent geometry can be propagated to it, but its
    own layout still runs so viewport/content handles, deferred probes, overflow observation, and
    child transforms remain authoritative.
  - The actual no-layout propagation path remains narrow (`Stack`, no-wrap vertical
    `Flex`/`SemanticFlex`/`RovingFlex`, and already-safe fixed-size leaves/text cases). The
    root-solve preflight may walk selected pass-through geometry wrappers only to prove child
    bounds; unsupported/layout-side-effect nodes still run their layout bodies.
  - `ViewCache` and `VirtualList` stay out of this fast path. `ViewCache` needs retained/render
    semantics proof before it can participate, and `VirtualList` must keep visible/render-window
    resize semantics authoritative.
  - Keep this in `scroll-optimization-v1`. Split a new workstream only if a later patch changes the
    layout data model or introduces a durable cross-crate layout-side-effect contract.
  - RTX4090 validation is intentionally deferred as follow-up evidence; it is not a completion gate
    for this local root-solve slice.
- [x] Classify the remaining `Semantics` / `Stack` / `PointerRegion` small-width-delta solves
  before widening the clean-geometry proof.
  - `Semantics` (`177us`) maps to the code-editor preview panel and still contains a wrap-flex
    descendant. Do not skip this by name; wrapped flex needs a line-break stability proof or a
    more explicit layout side-effect contract before it can participate.
  - Root `Stack` (`140us`) maps to the gallery root stack and mixes app chrome, workspace command
    scopes, view-cache/sidebar/content boundaries, containers, overlays, and custom frame widgets.
    This should not be solved by adding more wrapper names to the proof.
  - Editor `PointerRegion` (`3us`) is effectively the windowed rows `PointerRegion -> Canvas`
    root. A dedicated `Canvas` leaf proof is possible, but its measured benefit is too small to
    justify widening the current root-solve slice.
  - Decision: stop expanding clean-geometry proof ad hoc. The next meaningful optimization is an
    explicit layout side-effect / geometry-propagation contract, plus debug rejection reasons for
    why a clean root cannot skip its solve.
- [x] Draft the layout side-effect / geometry-propagation contract before attempting another root
  solve skip expansion.
  - Minimum contract questions: which element kinds are pure geometry, which are side-effectful
    layout boundaries, which may be safe leaves only, and what data model records this without
    relying on an ever-growing local whitelist.
  - Required guardrails: wrapped flex line-break stability, view-cache retained semantics,
    Canvas prepaint/paint bounds dependency, layout-query snapshots, focus/semantics/overlay
    geometry, and virtual-list visible ranges.
  - Add low-noise diagnostics that report the first unsupported kind/reason in
    `can_skip_clean_geometry_engine_solve_for_resize(...)` before adding more fast-path coverage.
  - Implemented minimum slice: `CleanGeometryNodeContract` now classifies the current supported
    proof as pure pass-through geometry, no-wrap vertical flex, safe leaf, or side-effect boundary,
    and `CleanGeometrySolveSkipRejectionReason` records the first rejected reason/kind in
    `UiDebugFrameStats` / diagnostics bundles.
  - Guardrails locked: `ViewCache` remains `unsupported_kind`, wrapped flex reports `flex_wrap`,
    and successful clean skips keep the rejection counter at `0`. This intentionally does not widen
    the fast path to `ViewCache`, `VirtualList`, wrap flex, or `Canvas`.
- [x] Attach clean-geometry solve-skip rejections to individual layout-engine solve records.
  - Fresh local no-4090 resize-jitter evidence before per-solve attribution:
    `target/fret-diag/local-next-clean-geometry-rejections-20260517-r1/1778978609337/bundle.schema2.json`.
  - Fresh local no-4090 resize-jitter evidence after per-solve attribution:
    `target/fret-diag/local-next-clean-geometry-rejections-20260517-r2/1778979436452/bundle.schema2.json`.
  - Result: the worst changing-bounds solves now carry their own rejection reason/kind instead of
    relying on the frame-level first rejection only. The large `Semantics` and root `Stack` solves
    are blocked first by `unsupported_kind=Container`; the editor `PointerRegion -> Canvas` solve is
    only `3-4us`, and root `Scroll` rejection is a `side_effect_boundary` with `0us` solve time.
  - Decision: the next optimization candidate is a conservative `Container` geometry contract. Do
    not start with `Canvas` or a root `Scroll` skip from this evidence.
- [x] Prototype a conservative `Container` clean-geometry contract.
  - Accept only a provable subset: static children, px spacing/insets, nonnegative border widths,
    definite/fill width behavior whose child content rect can be derived from previous geometry, and
    height semantics that do not depend on reflow.
  - Reject absolute children, non-px/fractional insets without a stable basis, auto-height cases
    that can change child flow, and any side-effectful descendant boundary not already covered by
    the existing proof.
  - Required proof: focused Rust layout tests for padding/border child bounds and rejection cases,
    then rerun the no-4090 resize-jitter script to see whether the next blocker becomes wrap flex or
    another app-shell boundary.
  - Implemented proof: `Container` participates only when px padding/border insets and static child
    geometry allow manual child-bound derivation. `Container` is manual-bounds-only in the clean
    propagation path, so unsupported variants do not silently fall back to stale engine local rects.
  - Focused guardrails: px padding/border `Container` skips the small width-delta solve and updates
    fill child content bounds; fraction padding rejects with `non_px_spacing / Container`.
  - Local no-4090 evidence:
    `target/fret-diag/local-next-container-clean-geometry-20260517-r1/1778981585274/bundle.schema2.json`.
  - Result: top frame total/layout/solve is `1242/623/297us`, `layout_engine_solves=4`,
    view-cache remains reused, and row replay/store remains `289/0`.
  - New blockers: `auto_child_height` under `Container` for the content `Semantics` solve, and
    `auto_child_height` under `Flex` for the root `Stack` solve. The content `Semantics` root still
    reports one wrap-flex node, so the next step must classify auto-height/line-break stability
    before widening the proof.
- [x] Classify the `auto_child_height` blockers before another geometry-proof expansion.
  - Separate fixed-height-but-auto-style wrappers from genuine width-dependent height/reflow.
  - Treat wrap flex as a hard blocker until a line-break stability proof exists.
  - Use per-solve rejection attribution rather than broad element-name whitelisting.
  - Implemented classification: stable auto-height `Container` wrappers and stable auto-height
    children in vertical no-wrap `Flex` can participate only when recursive geometry proof keeps
    descendant sizes stable; text leaves whose computed box size changes reject with `text_reflow`.
  - Focused guardrails:
    `clean_geometry_small_resize_skips_stable_auto_height_container_wrapper`,
    `clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child`, and
    `clean_geometry_small_resize_rejects_auto_height_text_reflow`.
  - Current classification is acceptable as a local clean-geometry contract, but should not grow
    indefinitely as one enum whitelist. If the next slice covers `Grid`, horizontal flex rows,
    `ViewCache`, `VirtualList`, `Canvas`, layout queries, or transforms, split the model into
    explicit axes: layout side effects, parent-derived child-bounds strategy, and width-delta size
    stability.
  - Local no-4090 evidence:
    `target/fret-diag/local-next-auto-height-classification-20260517-r1/1778984176582/bundle.schema2.json`.
  - Result: `auto_child_height` is no longer the first blocker. Remaining top solve blockers are
    content `Semantics` -> `unsupported_kind=Grid` with `wrap_nodes=1`, root `Stack` ->
    `flex_direction=Flex`, editor `PointerRegion` -> `unsupported_kind=Canvas`, and root `Scroll`
    remains a `side_effect_boundary`.
  - Decision: do not start a broad node-classification rewrite yet. The next optimization candidate
    is a proof-first `Grid` / horizontal `Flex` geometry contract or a formal classification-model
    refactor if that proof cannot stay local and reviewable.
- [x] Refactor the clean-geometry node classification model before widening to `Grid` or
  horizontal `Flex`.
  - Audit conclusion: no broad layout-architecture rewrite is needed from the current evidence.
    The problematic part was narrower: `CleanGeometryNodeContract` was beginning to mix node
    categories, layout side-effect policy, child-bound derivation, and leaf size stability into a
    single enum.
  - Implemented answer: keep the same supported node set and rejection behavior, but split the
    internal contract into explicit axes: `layout_effect`, `child_bounds`, and `size_stability`.
  - `Scroll` remains a side-effect boundary, but boundary detection now reads from the same
    `CleanGeometryNodeContract` instead of a separate `Scroll` special-case. Future boundaries
    should therefore be added in one classification surface.
  - Guardrails: this intentionally does not add `Grid`, horizontal `Flex`, `ViewCache`,
    `VirtualList`, `Canvas`, layout-query, or transform participation. Those still require their
    own proof and evidence.
  - Verified gates:
    `cargo nextest run -p fret-ui clean_geometry_small_resize_skips_px_container_and_updates_child_bounds clean_geometry_small_resize_rejects_container_fraction_padding clean_geometry_small_resize_skips_stable_auto_height_container_wrapper clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`,
    `cargo nextest run -p fret-ui layout_engine --no-fail-fast`,
    `cargo nextest run -p fret-ui scroll --no-fail-fast`,
    `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`,
    `cargo fmt`, and `python3 tools/check_layering.py`.

## Current slice — Deferred probe seed vs authoritative extent

- [x] Make deferred probe policy read retained seed state before deciding to skip a deep probe.
- [x] Allow deferred invalidation frames to consume `intrinsic_measure_cache` as the seed extent
  when retained child measured sizes are absent.
- [x] Centralize authoritative extent commits so pending probe clearing only happens on explicit
  probe / authoritative observation paths.
- [x] Make authoritative extent commits end deferred probe state entirely, instead of clearing only
  the pending invalidation bit and leaving deferred mode armed for later frames.
- [x] Ensure unchanged authoritative post-layout observation still clears deferred invalidation
  pending state instead of forcing an extra at-edge probe on the next frame.
- [x] Ensure unchanged authoritative observation on resize-deferred frames also clears deferred
  resize state instead of arming a redundant follow-up relayout/redraw on the first stable frame.
- [x] Record the dedicated verification results for the seed/authority regression gates in
  `EVIDENCE_AND_GATES.md`.

## Current perf slice — Engine-solved apply-path side-effect audit

- [x] Audit layout side effects before adding any engine-solved subtree apply fast path.
- [x] Confirm `Scroll` and `VirtualList` remain non-pure layout nodes because they update scroll
  handles, viewport/content extents, deferred-scroll state, and visible ranges during layout.
- [x] Confirm text and text-input widgets remain non-pure layout nodes because they observe global
  font state, refresh text caches, and/or update IME / selection / platform snapshot data during
  layout.
- [x] Confirm `LayoutQueryRegion`, `RenderTransform`, `FractionalRenderTransform`, and `Anchored`
  remain non-pure layout nodes because they write query/layout output state or compute transforms
  during layout.
- [x] Keep `Canvas` and `ViewportSurface` provisional rather than whitelisted by default, even
  though they currently look like leaf-like geometry nodes.
- [x] Prototype a narrower dirty-frontier scroll relayout path instead of a broad
  `widget.layout` skip.
- [x] Keep the proof bounded to one repro, one gate, and one evidence bundle.
- [x] Lock mixed direct-child / descendant-only post-layout shrink observation so synthetic scroll
  content roots cannot keep stale pinned extents authoritative after child frontier contraction.
- [x] Keep non-retained `VirtualList` visible-range escapes authoritative for view-cache rerender
  while preserving retained-list reconcile semantics.
- [x] Profile the remaining direct-child-invalidated / resize-measure path separately; do not fold
  it into the contained view-cache dirty-frontier proof.
- [x] Repair or replace the stale prewarm command form for local resize-stress samples so future
  p95 comparisons use the same normalization surface.

## Follow-on slice — Command and event focus targets resolve authoritative live attached nodes

- [x] Replace command dispatch source-element resolution so pending command metadata falls back from
  stale detached `node_entry` seeds to the live attached node.
- [x] Replace command-hook `request_focus(element)` resolution so the hook host no longer trusts
  `window_state.node_entry(target)` directly.
- [x] Add event-side `requested_focus_target` plumbing so key/pointer focus hooks defer element
  target resolution until dispatch regains access to `UiTree`.
- [x] Lock the command/event live-node resolution contract with focused stale-detached regression
  gates in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Declarative rebuild and invalidation element paths resolve authoritative live nodes

- [x] Replace element-runtime model/global invalidation target resolution so stale detached
  `node_entry` seeds fall back to the live attached node.
- [x] Replace rebuild-time `notify_for_animation_frame` invalidation target resolution so it no
  longer trusts `window_state.node_entry(element)` directly.
- [x] Replace declarative mount/root reuse node resolution so rebuild prefers the live attached
  node and only reuses a retained seed when no live attached node exists for the element.
- [x] Replace view-cache GC / retained virtual-list reconcile root resolution so detached stale
  `node_entry` seeds do not become authoritative keep-alive roots.
- [x] Lock the declarative rebuild/invalidation live-node contract with focused stale-detached
  regression gates in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — GC liveness must ignore parent-pointer-derived layer membership

- [x] Replace GC retain-time `UiTree::node_layer(...)` keepalive shortcuts so stale detached nodes
  are not preserved solely because a stale parent pointer still resolves to a layer.
- [x] Make GC liveness authoritative on retained liveness roots plus reachable child edges
  (`UiTree` children and `WindowFrame` children union), while keeping view-cache reuse memberships
  and retained keep-alive roots as explicit inputs.
- [x] Prune dead retained keep-alive root `NodeId`s before they participate in GC liveness, and
  make the reachability walk ignore nonexistent root ids as a guardrail.
- [x] Lock the contract with a focused regression that proves a stale parent path can keep
  `node_layer(...)` non-`None` without keeping the detached node alive.

## Follow-on slice — Runtime-owned subtree updates must refresh cache-root membership

- [x] Refresh ancestor cache-root membership lists when retained virtual-list reconcile mutates a
  subtree under a reused cache root without rerendering the cache-root closure.
- [x] Lock the contract with a regression asserting the cache-root membership list includes the new
  visible retained rows after the runtime-owned window update.

## Follow-on slice — Recursive cache-root keep-alive closure must ignore stale nested roots

- [x] Make the recursive keep-alive closure for `view_cache_reuse_roots -> view_cache_elements_for_root`
  accept only elements that still resolve to a live attached node.
- [x] Prevent stale nested cache-root membership lists from recursively widening the keep-alive
  closure after structural removal or root replacement.
- [x] Lock the contract with a focused regression proving stale nested cache roots no longer keep
  detached descendants inside the keep-alive closure.

## Follow-on slice — Reuse-frame membership touch must revalidate recorded members

- [x] Treat recorded `view_cache_elements_for_root(...)` membership as reusable only after every
  recorded member resolves to a live attached node for the current frame.
- [x] If any recorded member is stale/detached, invalidate the whole recorded membership list for
  that reuse frame and fall back to an authoritative retained-subtree walk plus membership
  re-record.
- [x] Lock explicit `view_cache_keep_alive(...)` reuse with a regression proving stale detached
  recorded members are neither touched nor retained in the refreshed authoritative membership list.

## Follow-on slice — Overlay owner pruning must use authoring-identity liveness

- [x] Track current-frame declarative authoring identities in `fret-ui` even when a `scope` /
  `keyed` root does not map to a mounted node, and restore those identities on view-cache reuse
  frames where the producer subtree skips rerender.
- [x] Expose a public current-frame identity-liveness query distinct from live-node resolution, so
  ecosystem policy code can reason about scope-only producers without treating `node_entry` or
  attached-node existence as authoritative.
- [x] Prune owned cached overlay requests/layers only when the cached owner identity disappears
  from the current frame's authoring pass, not merely when that identity lacks a mounted node.
- [x] Lock the contract with focused `fret-ui` and `fret-ui-kit` regressions in
  `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Interaction targets resolve authoritative live attached nodes

- [x] Replace hover/pressed interaction target bookkeeping so runtime state stores element identity
  separately from the current authoritative live node instead of resolving `node_entry(element)`
  at mutation time.
- [x] Sync retained hover/pressed/hover-region target nodes against the live attached tree during
  final layout-frame commit so same-element rebuild/remount does not keep clearing a stale node.
- [x] Replace timer element-target dispatch so `Event::Timer` resolves the live attached node in
  `UiTree` / dispatch instead of trusting `window_state.node_entry(element)` directly.
- [x] Replace selectable-text active-selection dispatch so `Event::SetTextSelection` keeps routing
  through the live attached selectable-text node even if retained runtime state or `node_entry`
  was seeded with a stale detached node.
- [x] Lock the interaction-target live-node contract with focused stale-detached regression gates
  in `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Final-layout / dispatch / anchored queries resolve authoritative live attached nodes

- [x] Replace render-time `focus-within` containment and focused-node-to-element sync so they read
  live window-frame nodes before falling back to retained `node_entry` / `element_for_node`
  mappings.
- [x] Replace final-layout focus repair so the canonical focus node resolves from the live attached
  tree instead of the last-known `elements::node_for_element(...)` mapping.
- [x] Replace touch-drag locked target and wheel scroll-dismiss element lookups so dispatch uses
  live attached nodes instead of retained stale `node_entry` mappings.
- [x] Replace anchored layout `anchor_element` resolution so layout uses the live attached anchor
  node rather than the last-known `elements::node_for_element(...)` mapping.
- [x] Add a dedicated wheel scroll-dismiss regression so this dispatch path is locked directly
  instead of only being covered by the broader stale-detached live-node suite.
- [x] Record the new regression gates and the non-test mechanism-path audit result in
  `EVIDENCE_AND_GATES.md`.

## Follow-on slice — Ecosystem runtime paths use explicit live-node query surfaces

- [x] Expose public `fret-ui` query surfaces that distinguish authoritative current/live node
  resolution from last-known retained mappings:
  - `UiTree::live_attached_node_for_element(...)`
  - `elements::live_node_for_element(...)`
  - `ElementContext::live_node_for_element(...)`
- [x] Keep `elements::node_for_element(...)` / `ElementContext::node_for_element(...)` as explicit
  last-known query surfaces instead of silently widening their semantics.
- [x] Tighten the public live query contract so current-frame liveness comes from
  `WindowElementState::node_entry(...).last_seen_frame`, not from
  `ElementFrame::window_frame.instances`, which may retain stale records until subtree GC.
- [x] Replace ecosystem authoritative runtime call sites with the new live query surfaces:
  - overlay focus request / restore paths in `window_overlays/render.rs`,
  - focus-scope initial-focus / restore helpers,
  - dismissable-layer branch resolution,
  - live active-descendant helpers in `fret-ui-kit`.
- [x] Keep render-time semantics authoring surfaces declarative when the parent relationship is
  known before the current frame's child nodes are mounted:
  - `fret-ui-kit/declarative/table.rs`,
  - `fret-ui-shadcn/select.rs`,
  - use `SemanticsDecoration::active_descendant_element(...)` instead of forcing a current-frame
    `NodeId` lookup.
- [x] Let semantics-time declarative relation resolution fall back from the local mounted element
  map to the authoritative current-frame live mapping so retained / virtualized child subtrees can
  still resolve `active_descendant`, `controls`, and related element relations.
- [x] Lock the public-surface contract with stale-last-known regressions in
  `fret-ui-kit` and record the gates in `EVIDENCE_AND_GATES.md`.

## Gates-first checklist

- [x] Confirm baseline scripts pass:
  - [x] `ui-gallery-scroll-area-wheel-scroll` (bundle: `target/fret-diag/1772468071457-scroll-area-wheel-scroll`, 2026-03-02)
  - [x] `ui-gallery-scrollbar-drag-baseline-content-growth` (bundle: `target/fret-diag/1772498133742-scrollbar-drag-baseline-content-growth`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-wheel-torture` (bundle: `target/fret-diag/1772498149599-scroll-area-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-nested-scroll-routing` (bundle: `target/fret-diag-scroll-nested-debug6/sessions/1772508480737-75452/1772508483614-scroll-area-nested-scroll-routing`, 2026-03-03)
  - [x] `ui-gallery-wheel-burst-coalescing` (new gate: wheel events per frame ≤ 1; suite: `diag-hardening-smoke`)
    - `diag run` evidence: `target/fret-diag-runs/1772530803405-wheel-burst/check.wheel_events_max_per_frame.json` (2026-03-03)
  - [x] `ui-gallery-virtual-list-wheel-torture` (bundle: `target/fret-diag-vlist-wheel/sessions/1772508526189-62940/1772508528623-virtual-list-wheel-torture`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-toggle-code-tabs` (bundle: `target/fret-diag-underflow-check/sessions/1772500876247-61448/1772500879851-scroll-area-toggle-code-tabs`, 2026-03-03)
  - [x] `ui-gallery-scroll-area-expand-at-bottom` (bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-scroll-area` (bundle: `target/fret-perf-scroll-area/sessions/1772501734226-65632/1772501741770`, 2026-03-03)
  - [x] `diag perf perf-ui-gallery-virtual-list` (bundle: `target/fret-perf-vlist/1772508561962`, 2026-03-03)
- [x] Promote nested scroll routing into `diag-hardening-smoke`:
  - suite manifest: `tools/diag-scripts/suites/diag-hardening-smoke/suite.json`
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-nested-scroll-routing.json`

## Mechanism hardening

- [x] Fix view-cache contained relayout bookkeeping (layout invalidation clears must keep subtree aggregation in sync):
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
- [x] Keep layout-only contained relayout from forcing next-frame cache-root rerender / dirty-view carry-over:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
- [x] Prune detached roots from layout follow-up scheduling before final layout:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/tests/view_cache.rs`
  - `crates/fret-ui/src/tree/tests/barrier_subtree_layout_dirty_aggregation.rs`
- [x] Audit all barrier-related paths that can affect scroll surfaces:
  - [x] `set_children_barrier(...same_children...)` schedules authoritative follow-up relayout when
    descendant layout work is still pending,
  - [x] `set_children(...same_children...)` and `set_children_in_mount(...same_children...)`
    reconnect authoritative layout when they repaired stale parent pointers under pending
    descendant layout work,
  - [x] `set_root(...)` / `set_base_root(...)` now prune detached focus/capture immediately when a
    layer-root replacement rebases the active layer roots, while preserving still-active overlay
    interaction state,
  - [x] pending multi-stroke shortcut continuation now revalidates the authoritative key-context
    stack before matching the next chord, so root replacement cannot keep stale shortcut contexts
    alive,
  - [x] publishing command/action availability snapshots now refreshes the authoritative
    window-level key-context stack, so cross-surface gating cannot keep stale `keyctx.*` state
    alive after rebuild,
  - [x] declarative rebuild commit points (`render_root(...)` / `render_dismissible_root_with_hooks(...)`)
    now republish authoritative window input/key-context/action-availability snapshots after tree
    GC/root reuse, so later same-frame surfaces cannot keep consuming stale window services,
  - [x] raw imperative `UiTree` mutations now have an explicit authoritative window-snapshot
    commit surface, so same-frame consumers can republish input/key-context/action-availability
    after retained-state changes without waiting for rebuild/dispatch/paint,
  - [x] best-effort `WindowInputContextService` consumers now overlay the authoritative
    `WindowCommandAvailabilityService` before using `edit.can_*` / `router.can_*`, so stale
    published input snapshots cannot suppress cross-surface gating or shortcut lookup,
  - [x] add a source-policy guardrail so future first-party code cannot reintroduce raw
    `WindowInputContextService` command-availability drift outside the runtime owner files,
  - [x] audit remaining raw `WindowInputContextService` readers and confirm they are limited to
    runtime ownership, diagnostics/debug surfaces, or text-boundary/IME reads,
  - [x] remaining child-list mutation helpers now route through the same authoritative structural
    contract; `add_child(...)` reparents by severing old parent edges, avoiding duplicate child
    edges, and delegating the structural write to `set_children(...)`,
  - [x] reparent cleanup now respects the old parent's structural child-write policy
    (`Standard` vs `Barrier`) instead of guessing from the new write path, so stale child edges
    from cached/runtime wrapper transitions cannot force the wrong detach semantics,
  - [x] contained cache-root dirty markers now align with main-pass layout consumption and
    descendant-truncated contained relayout scheduling,
  - [x] subtree dirty aggregation bookkeeping.
- [x] Add/extend unit tests to cover:
  - [x] barrier relayout sets `subtree_layout_dirty_count` consistently,
  - [x] barrier same-children clean remount stays no-op,
  - [x] barrier same-children dirty descendant converges via contained relayout,
  - [x] descendant layout invalidation under a contained cache root stays layout-only but still
    schedules contained relayout,
  - [x] same-children parent repair reconnects detached descendant layout for normal and mount-time
    child-list mutation helpers,
  - [x] layer-root replacement clears detached interaction state without clearing still-active
    overlay interaction state,
  - [x] pending shortcut continuation drops stale key-contexts after root replacement,
  - [x] cross-surface command gating refreshes stale key-context snapshots when action
    availability is republished,
  - [x] declarative rebuild republished window input snapshots before the next paint,
  - [x] declarative rebuild republished window key-context snapshots before later same-frame
    consumers read them,
  - [x] declarative rebuild republished widget command availability before the next explicit
    publish/dispatch boundary,
  - [x] detached dismissible roots only finish the declarative snapshot commit after their parent
    or overlay attachment becomes authoritative,
  - [x] imperative raw tree mutation only refreshes window input/key-context/action-availability
    after an explicit window snapshot commit,
  - [x] `layout_all()` after an imperative raw tree mutation still requires that same explicit
    window snapshot commit instead of acting as an implicit authoritative publish boundary,
  - [x] raw rebuilds may leave stale declarative frame records behind, but internal-drag target
    promotion still ignores detached stale regions and only walks the live hit ancestry,
  - [x] best-effort input-context readers inherit authoritative command availability over stale or
    fallback published snapshots,
  - [x] diagnostics inspect overlays stay input-transparent and non-authoritative for window
    runtime snapshots across a full rebuild/layout/publish frame sequence,
  - [x] `add_child(...)` reparents without stale child edges and no-ops when already attached once,
  - [x] barrier-parent reparent cleanup keeps the old parent on contained-relayout semantics,
  - [x] barrier-to-barrier reparent cleanup removes stale edges without bubbling ancestor relayout,
  - [x] scroll handle revision-only bumps stay classified correctly,
  - [x] scroll handle invalidation ignores detached same-frame stale bindings,
  - [x] scroll handle registry dedupes same-frame duplicate element bindings,
  - [x] event-time scroll handle invalidation resolves authoritative live bindings across layers,
  - [x] explicit scroll-target invalidation resolves authoritative live target nodes.

## Wheel/trackpad delta coalescing

- [ ] Decide coalescing layer:
  - [ ] runner/platform (preferred),
  - [ ] UI core (fallback).
- [x] Implement behind a runtime knob (opt-in) with a clear default.
  - [x] Native (winit): `FRET_WINIT_COALESCE_WHEEL=1` (coalesce consecutive wheel events).
- [x] Add a max-abs delta guardrail for a single coalesced wheel event (still needs perf validation on VirtualList):
  - `FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX` (default: `120`)
- [x] Implement frame-boundary buffering in the desktop runner (deliver ≤ 1 wheel per frame when enabled):
  - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
- [x] Add a runner-level “same-frame wheel burst” regression gate:
  - Script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-wheel-burst-coalescing.json`
  - Suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-wheel-burst-coalescing.json`
  - Post-run check: `check.wheel_events_max_per_frame.json`
- [x] Collect repeatable perf evidence (repeat=11, warmup=10):
  - `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `30777/46060` / `29402/43910` / `3072/4510`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-off-r11/1772509265134/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `28134/29352` / `26956/28203` / `2859/3036`
      - worst bundle: `target/fret-perf-scroll-area-coalesce-on-r11/1772509316761/bundle.json`
      - log: `target/perf-logs/scroll-area-coalesce-on-r11.log`
  - `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
    - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
      - p50/p95 `total/layout/solve` us: `10910/11393` / `10180/10595` / `2996/3363`
      - worst bundle: `target/fret-perf-vlist-coalesce-off-r11/1772509365874/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-off-r11.log`
    - ON (`FRET_WINIT_COALESCE_WHEEL=1`):
      - p50/p95 `total/layout/solve` us: `11870/18175` / `11185/17468` / `3437/5012`
      - worst bundle: `target/fret-perf-vlist-coalesce-on-r11/1772509420507/bundle.json`
      - log: `target/perf-logs/virtual-list-coalesce-on-r11.log`
- [x] Re-run repeat=11 perf after adding the max-abs cap (2026-03-03):
  - Goal: keep `perf-ui-gallery-scroll-area` improved, remove `perf-ui-gallery-virtual-list` p95 regression.
  - Result (pre frame-boundary buffering): `cap=120` still shows high variance for VirtualList; see the “Full rerun (pre frame-boundary buffering)” section below.
  - Result (after frame-boundary buffering): `cap=120` is stable for both scripts; see the “Frame-boundary buffering rerun” section below.

### Rerun (2026-03-03) — max-abs cap default (`120`)

Short rerun (repeat=5, warmup=10) to sanity-check the new default cap behavior:

- `perf-ui-gallery-virtual-list`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `10963/11109`
    - worst bundle: `target/fret-diag/1772514040891/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `10424/11102`
    - worst bundle: `target/fret-diag/1772513954382/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r5.log`

- `perf-ui-gallery-scroll-area`
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total` us: `30567/49376`
    - worst bundle: `target/fret-diag/1772513830516/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-current-r5.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, default cap `120`):
    - p50/p95 `total` us: `28054/29181`
    - worst bundle: `target/fret-diag/1772513987367/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r5.log`
- [ ] Add diag evidence:
  - [x] stress wheel in a scroll area (`ui-gallery-scroll-area-wheel-torture`),
  - [x] stress wheel in a virtual list (`ui-gallery-virtual-list-wheel-torture`),
  - [x] nested scrollable case (inner X should not consume Y wheel: `ui-gallery-scroll-area-nested-scroll-routing`).

Full rerun (repeat=11, warmup=10) with explicit env overrides (2026-03-03, pre frame-boundary buffering):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10619/12213` / `9945/11573` / `2987/3319`
    - worst bundle: `target/fret-diag/1772517019308/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `11611/24223` / `10978/22757` / `3258/5983`
    - worst bundle: `target/fret-diag/1772517054654/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `10872/12343` / `10231/11625` / `3042/3281`
    - worst bundle: `target/fret-diag/1772517987201/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-cap60-r11-20260303c.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `27674/28643` / `26613/27521` / `2844/3019`
    - worst bundle: `target/fret-diag/1772517184852/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `27873/28904` / `26766/27801` / `2859/3254`
    - worst bundle: `target/fret-diag/1772517215826/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap120-r11-20260303c.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `60`):
    - p50/p95 `total/layout/solve` us: `29862/32033` / `28487/30812` / `2965/3549`
    - worst bundle: `target/fret-diag/1772518038237/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-cap60-r11-20260303c.log`

Notes:

- Current evidence suggests the cap is workload-sensitive:
  - `cap=120` is acceptable for `scroll-area` but shows high variance/regression in `virtual-list`.
  - `cap=60` removes the `virtual-list` spikes but regresses `scroll-area` in this torture script.
- Follow-up (partially resolved): frame-boundary buffering makes `cap=120` stable in repeat=11 for both scripts; next step is deciding if/when this becomes default-on across platforms.

Frame-boundary buffering rerun (repeat=11, warmup=10) (2026-03-03):

- `perf-ui-gallery-virtual-list` (script: `ui-gallery-virtual-list-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `10927/12140` / `10263/11451` / `2986/3307`
    - worst bundle: `target/fret-diag/1772519046872/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `10729/11614` / `10099/10922` / `2985/3187`
    - worst bundle: `target/fret-diag/1772519094741/bundle.json`
    - log: `target/perf-logs/virtual-list-coalesce-on-frame-cap120-r11-20260303.log`

- `perf-ui-gallery-scroll-area` (script: `ui-gallery-scroll-area-wheel-torture`)
  - OFF (`FRET_WINIT_COALESCE_WHEEL=0`):
    - p50/p95 `total/layout/solve` us: `28544/52680` / `27404/50496` / `2855/5707`
    - worst bundle: `target/fret-diag/1772519164488/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-off-frame-r11-20260303.log`
  - ON (`FRET_WINIT_COALESCE_WHEEL=1`, cap `120`):
    - p50/p95 `total/layout/solve` us: `29282/31195` / `28203/30055` / `2957/3434`
    - worst bundle: `target/fret-diag/1772519183814/bundle.json`
    - log: `target/perf-logs/scroll-area-coalesce-on-frame-cap120-r11-20260303.log`

## Perf harness plumbing

- [x] Allow `fretboard-dev diag perf perf-ui-gallery-scroll-area` to resolve via the promoted scripts registry:
  - `crates/fret-diag/src/perf_seed_policy.rs`

## Scrollbar drag stability

- [x] Add “drag baseline” to `ScrollbarState` (mechanism-only).
- [x] Update thumb math while dragging to use baseline.
- [x] Add diag script + semantics assertions (`ui-gallery-scrollbar-drag-baseline-content-growth`).

## Extents probing / observation

- [x] Add diag script for “expand at bottom” (pinned extents regression):
  - script: `tools/diag-scripts/ui-gallery/scroll-area/ui-gallery-scroll-area-expand-at-bottom.json`
  - suite redirect: `tools/diag-scripts/suites/diag-hardening-smoke/ui-gallery-scroll-area-expand-at-bottom.json`
  - bundle: `target/fret-diag-scroll-expand-at-bottom-v4/sessions/1772539486117-27536/1772539488297` (2026-03-03)
- [x] Validate post-layout observation budgets:
  - [x] wrapper peel budget hit triggers a probe next frame,
  - [x] deep scan budget hit triggers a probe next frame.
  - test: `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` (`scroll_post_layout_observation_budget_hit_schedules_probe_next_frame`)
