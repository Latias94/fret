---
type: Work Progress
title: View boundary cache architecture research checkpoint
tags: fret,architecture,research,view-cache,gpui,phase3
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagents:
  - 019f2756-0736-7e23-a9cb-37cd1d27536c
  - 019f2755-c241-78e3-8c16-dcb51aea6ce9
  - 019f2756-4675-77f1-9f87-5cf903f7c540
---

# Verdict

The optimization direction is correct, but the current implementation must still be treated as a
bridge state rather than the final architecture.

Fret should keep the `ViewId -> ViewBoundary -> dirty frontier -> boundary frame products ->
scene fragments -> renderer reuse` direction. This is the right foundation for an editor-grade,
GPU-first UI framework because cached work must be tied to stable view or boundary identity and to
explicit frame products, not to transient retained storage nodes.

The main risk is not the existence of a view/boundary cache. The main risk is allowing retained
`NodeId`, retained `Node.parent`, retained subtree scans, or GC reachability to remain live topology
or liveness authorities. Phase 3 should continue deleting and demoting those bridges.

# Research Summary

GPUI/Zed has the same class of architecture, although with different public names. `AnyView::cached`
stores previous-frame prepaint and paint ranges in element state, skips reuse when the entity is
dirty, and marks dirty views plus ancestors through the dispatch tree. The important lesson is that
the cache is a runtime frame-product reuse boundary, not a component-library memoization feature.

Broader UI prior art points in the same direction:

- Flutter `RepaintBoundary` separates display-list repaint work and can let the engine raster-cache
  stable subtrees.
- Jetpack Compose `drawWithCache` and graphics layers cache expensive draw setup behind explicit
  dependency invalidation.
- React `memo` is useful for parent-prop churn, but React's own docs make clear that state, context,
  and identity rules still drive re-rendering; memoization is not a liveness model.
- egui `Id`, React keys, Xilem/Masonry tree state, Iced tree state, Slint bindings, Unreal Slate
  invalidation, Qt Quick scene graph, WebRender picture caching, and Chromium RenderingNG all
  separate stable identity/current-frame dependency tracking from retained or renderer storage.

# Fret-Specific Judgment

Keep:

- `ViewBoundaryStore` as the owner of independent `BoundaryId` / `ViewId` allocation.
- `ViewBoundaryKey::Element(GlobalElementId)` for declarative boundary identity.
- `DirtyViewFrontier` as the dirty-view aggregation vocabulary.
- `BoundaryFrameProducts` as the boundary-local container for prepaint, hit-test, semantics,
  interaction, scene, and paint products.
- `StableNodeHandle` / `ElementNodeIndex` style validation for retained placement when a retained
  node still has to be referenced.
- Inspection/debug cache bypasses.

Change:

- Move normal topology queries off retained `Node.parent` and onto frame or boundary topology with
  an explicit epoch/freeze lifecycle.
- Move cache-hit membership and liveness bookkeeping off retained subtree scans and into
  boundary-owned recorded membership.
- Move `view_cache_needs_rerender` out of retained node storage toward boundary dirty/rerender
  pressure state.
- Rename or split `PaintCacheKey` where it is really a boundary product key for prepaint or scene
  fragments.
- Reframe raw `cx.view_cache`, `ViewCacheProps`, and `set_node_view_cache_flags` as advanced
  mechanism APIs; app-facing recipes should expose typed dependency keys and safe wrappers.

Delete or demote:

- Normal-path `repair_parent_pointers_from_layer_roots`.
- Cache-hit retained subtree touch/scan helpers used for liveness reconstruction.
- GC reachability as a live-query proof.
- Long-term reliance on flat paint replay as production cache semantics once chunk-native renderer
  inputs have parity gates.
- Hard-coded action-hook type scans for cache-hit liveness; use recorded state keys or a registry.

# Plan Corrections

The Phase 3 plan remains directionally right. The research suggests these additions should be made
explicit while implementing U3/U4/U5:

- Add a phase-scope invalidation matrix: layout, prepaint, hit-test, semantics, command routing,
  accessibility, paint, scene chunks, GPU resources.
- Add explicit cache-key and dependency tracking rules per phase. Bounds/text style alone are not
  enough for a general UI framework.
- Add a boundary promotion policy so only expensive or independently dirty subtrees become cache
  boundaries by default.
- Add GPU resource budget and eviction rules for cached scene chunks, text blobs, glyph pages, and
  prepared shapes.
- Add stale-product tests covering hit-test, focus restore, semantics, a11y, text layout,
  interaction handlers, and command availability under stale retained nodes and stale parents.

# Current Implementation Risk

Recent U3 work moving normal queries from retained parent pointers to child-edge topology is the
right intermediate step, but repeated `parent_in_layer_forest_via_children` scans are not the final
shape. The target should be a frozen `FrameTopology` / `BoundaryTopology` index with clear build,
freeze, invalidate, and consume rules.

# Sources

- GPUI local source: `repo-ref/zed/crates/gpui/src/view.rs`
- GPUI local source: `repo-ref/zed/crates/gpui/src/window.rs`
- GPUI local source: `repo-ref/zed/crates/gpui/src/key_dispatch.rs`
- Fret contract: `docs/runtime-contract-matrix.md`
- Fret target vocabulary: `docs/golden-architecture.md`
- Fret Phase 3 plan: `docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md`
- Official GPUI docs: https://docs.rs/gpui/latest/gpui/struct.AnyView.html
- Official Flutter `RepaintBoundary` docs: https://api.flutter.dev/flutter/widgets/RepaintBoundary-class.html
- Official React `memo` docs: https://react.dev/reference/react/memo
- Official React state identity docs: https://react.dev/learn/preserving-and-resetting-state
- Official Jetpack Compose graphics docs: https://developer.android.com/develop/ui/compose/graphics/draw/overview

# Disposition

Continue Phase 3 U3/U4/U5. Do not abandon view/boundary caching. Do not treat the current retained
bridge implementation as the final architecture.
