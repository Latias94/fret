---
type: Subagent Finding
title: UI framework architecture audit findings
tags: fret,architecture,performance,dx,gpui
timestamp: 2026-06-30
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
---

# Finding

Four audit lanes converged on the same target architecture.
Fret should not replace its current direction; it should make the existing direction enforceable through contracts and gates.

# Evidence

- Repository boundary audit: `crates/fret-ui` has clean dependency posture, but public vocabulary and example paths risk turning the mechanism layer into a policy/component layer.
- GPUI/Zed comparison: copy per-frame element trees, externalized state, `notify`, dirty views, prepaint frame products, dispatch snapshots, and cached frame-product reuse; avoid GPUI platform coupling and keep policy outside `fret-ui`.
- Framework consumer audit: the user journey reaches todo but lacks a promoted second-hour app ladder for settings, command palette, data table, workspace shell, and canvas/node graph.
- Performance audit: Fret already has retained layout, dispatch snapshots, bounds tree, renderer cache, and text atlas infrastructure, but identity, dirty graph, scene encoding, GPU upload, and text/glyph cache budgets are not yet unified enough for editor scale.

# Recommendation

Use `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md` as the next execution contract.
Start with contract alignment and source-policy gates, then add architecture metrics before stable handle, dirty graph, scene chunk, and text/glyph budget migrations.

# Disposition

Integrated into the plan as U1 through U9.
No code was changed by the subagents.

## Follow-up Audits During Execution

Two older read-only audit agents were closed during U3 execution:

- `019f1440-5447-75e2-b450-6700e61df8c6` found performance hot spots to feed U4/U7/U8: layout fast paths still pay root/dirty scans, subtree invalidation clones child vectors, paint-cache replay rebases descendants after movement, text layout keys clone/compare large objects, text input copies full strings, atlas revision invalidates scene encoding too broadly, text residency scans whole scenes, virtual list range allocation and rerender spikes need gates, managed-surface unplaced-child checks are quadratic, and diagnostics/HUD can pollute perf observations.
- `019f1440-7fc5-7330-83d1-b51dae3f56f3` found ecosystem refactor risks for U6/U9: shadcn public/raw surface is too wide, dropdown/select are giant policy files, select scroll buttons should become reusable policy, table code and tests freeze implementation shape, headless table row model is too monolithic, docking-specific diagnostics leak into runtime, docking tests are oversized, chart/node/plot state and panels need narrower controllers and fixtures.

U4 explorer `019f1627-edfc-7001-be3e-8ef1d98c076e` recommends starting with counters before identity handle migration:

- Add `UiDebugFrameStats` fields for identity seeded hit/stale, fallback scans/nodes/hit/miss, parent pointer repair, GC reachability/stale removals, dirty frontier breadth, dispatch snapshot cache hit/miss/build/invalidation, and model/global observation churn.
- Instrument first in `tree/layout/state.rs`, `declarative/mount.rs`, `tree/dispatch_snapshot.rs`, `tree/observation.rs`, and debug stats export paths.
- Do not remove fallback scans or introduce `StableNodeHandle` in the first U4 slice. Keep fallback scans as counted temporary correctness paths, then gate them toward zero after warmup.

U4 follow-up explorer `019f167a-2cb3-7dd0-bffe-889f9be0f2ff` narrowed the remaining observability gap:

- Dirty frontier breadth should be recorded at `view_boundary.rs::mark_boundary_layout_dirty`, layout start after detached followups are pruned, and contained view-cache candidate collection. It should not duplicate `layout_subtree_dirty_agg_*`, which measures parent-chain maintenance cost rather than frontier width.
- Model/global observation churn should be reported by `ObservationIndex::record()` / `GlobalObservationIndex::record()` deltas at layout/paint recording points, without changing the existing `model_change_*` and `global_change_*` propagation baseline semantics.
- The minimum useful fields are dirty boundary max/layout-start/contained-candidate counts and model/global observation edge added/removed/mask-changed counts.

U5 explorer `019f167a-817e-7ba3-9dbd-35815bc40c4b` recommends making the next implementation slice a `DirtyViewFrontier` ownership wrapper:

- Replace raw ownership of `UiTree::dirty_boundaries: HashSet<NodeId>` with a frontier type that stores `ViewId`/boundary vocabulary and exposes small NodeId bridge methods for v1 retained-node compatibility.
- Start with `view_boundary.rs` write points, then update reads in `ui_tree_subtree_layout_dirty.rs`, `layout/entrypoints.rs`, and `ui_tree_debug/frame.rs`.
- Do not start U5 by moving dispatch snapshots, interaction/prepaint caches, paint recording, or scene chunks; those are larger follow-on slices for U5/U7 and would touch focus/overlay/renderer contracts.
- After this slice, update ADR 0165 and ADR 0327 implementation alignment evidence but keep the status partially aligned until entity-first `ViewId` and frame products are fully migrated.

U5 implementation-risk explorer `019f16b7-57cb-70f2-9232-a5e7a760e802` reviewed the dirty-frontier call sites during the first implementation slice:

- Dirty frontier writes are already concentrated through `mark_boundary_layout_dirty`, `clear_boundary_layout_dirty`, and `remove_view_boundary_state`; the fragile reads are detached pruning, contained relayout candidate collection, subtree-dirty coverage checks, and debug dirty-view snapshot/export.
- The recommended first-slice API is `DirtyViewFrontier { views: HashSet<ViewId> }` with ViewId-native iteration plus explicitly named v1 boundary-node bridge methods such as `mark_boundary_node_v1`, `clear_boundary_node_v1`, and `iter_boundary_nodes_v1`.
- The highest-risk behavior is frontier/state synchronization after initial mount, main-pass layout consumption, detached boundary pruning, contained relayout, notify propagation, and hover-edge dirtying. Focused tests should cover view-cache contained relayout, detached dirty roots, notify ancestor propagation, hover edges, and layout dirty harness schema metrics.
- ADR 0165 should remain `Partially aligned`; ADR 0327 should remain `Aligned (with known gaps)`. This slice moves dirty frontier ownership toward ViewId/boundary vocabulary but does not land entity-first `ViewId`, dispatch snapshot ownership, paint recording ownership, or scene chunks.

U5 input/dispatch snapshot audit subagent `019f181e-fe0a-72c0-ba7c-79cf3a60ccf3` concluded `UiInputArbitrationSnapshot`, `UiDispatchSnapshot`, focus/capture state, command availability caches, and layer policy flags should stay window/layer-forest owned for now. They can span multiple layer roots, so forcing them into a single boundary product would hide the real invalidation scope.

U5 semantics audit subagent `019f181f-9af6-7741-93ae-7c5dd821f1fb` concluded the final `SemanticsSnapshot` and AccessKit update must remain window-owned because roots, barriers, focus/capture, and relation normalization are global. The safe boundary-owned cut is only reusable clean semantics subtree products/ranges.

U5 hit-test path cache audit subagent `019f1853-0640-79d2-8e19-8411b52ef258` concluded `hit_test_path_cache` is not boundary-owned:

- The cache payload is keyed by a `layer_root`, but validity depends on the window's active layer roots, modal barriers, pointer occlusion policy, event type, and whether the query is a primary pointer route or a secondary hover/occlusion probe.
- A boundary product would make the owner look more local than the invalidation contract really is, increasing the risk that future routing changes reuse a path across the wrong layer forest.
- The recommended cut is a window-owned `HitTestPathRoutingCacheState` with explicit clear, suspend, and update APIs. Secondary hit tests should suspend publication so they cannot replace the primary pointer-route cache entry.
