---
title: Fret Mechanism Harness v1 Coverage Map
status: active
date: 2026-05-11
---

# Coverage Map

| Domain | Current coverage | Artifact | Runtime gate | Current gap |
| --- | --- | --- | --- | --- |
| Layout primitives | auto size, fill, flex stretch, transparent wrapper, chrome outer/inner sizing, grid fr/auto, layout/visual/hit transform spaces | `crates/fret-ui/src/declarative/tests/fixtures/layout_primitives_v1.json` | shadcn seed diagnostics for ButtonGroupText alignment | Add text wrapping, min/max constraints, percent sizing, scroll roots, and RTL/writing-mode cases. |
| Layout dirty invalidation | child dirty aggregation, suppressed child boundary, subtree removal, underflow repair, visible sibling preservation, contained view-cache relayout, scroll/direct-child dirty frontier coverage, detached dirty cache-root pruning, view-cache layout-dirty expansion attribution | `crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json` | `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`; focused gates in `crates/fret-ui/src/tree/tests/view_cache.rs`, `crates/fret-ui/src/declarative/tests/layout/scroll.rs`, and `crates/fret-ui/src/tree/tests/interactive_resize_flow_rebuild.rs` | Add declarative cache-hit lifecycle cases for state/model/layout-query dependencies. |
| Environment view-cache invalidation | viewport-size, reduced-motion, color-scheme, contrast, forced-colors, text-scale, reduced-transparency, accent-color, safe-area, and occlusion changes rerender only dependent cache roots through the real `WindowMetricsService` path | `crates/fret-ui/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json` | focused gates in `crates/fret-ui/src/declarative/tests/environment_queries.rs` | Add UI Gallery diagnostics for runtime platform preference changes when a stable demo page exists. |
| Scroll-handle invalidation and virtualization | scroll-handle registry change classification, windowed-paint cache-root dirtying, revision-only internal offset baseline behavior, virtual-list window escape, detached stale binding filtering | `crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json` | focused gates in `crates/fret-ui/src/tree/tests/view_cache.rs`, `crates/fret-ui/src/declarative/tests/virtual_list/retained.rs`, and `crates/fret-ui/src/declarative/frame.rs` | Add retained-host reconcile fixture metrics, prepaint prefetch window-update cases, viewport/content/items-revision window-update detail cases, and UI Gallery scroll/virtual-list diagnostics gates. |
| Hit-test routing | transform spaces, clipping, transparent wrappers, disabled hit-test gates, overlay roots, modal barrier roots | `crates/fret-ui/src/declarative/tests/fixtures/hit_test_routing_v1.json` | pointer traces in UI Gallery scripts | Promote pointer occlusion and captured-pointer routing into fixture-driven matrices. |
| Overlay placement/sizing | shell sizing locks for drawer and popover command shell | `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json` | shadcn overlay diagnostics scripts | Add anchor-space, collision, viewport clamp, arrow, nested modal, and cross-root cases. |
| Focus | basic observed focus predicates in harness | `crates/fret-mechanism-harness/src/oracle.rs` | existing focus/overlay diag scripts | Add fixture-driven focus trap, focus restore, roving focus, and active-descendant suites. |
| Semantics | selector predicates over observed semantics nodes | `fret_diag_protocol::UiPredicateV1` reuse | diagnostics bundle and sidecar selectors | Add semantics-only invariants for roles, labels, hidden state, controls relation, and active descendant. |

## Coverage Rule

A mechanism invariant is covered only when it has at least one of these:

- a synthetic fixture case with stable case id and oracle predicates;
- a focused Rust test when the scenario is too procedural for fixtures;
- a UI Gallery diagnostics gate when the invariant has real runtime risk;
- an evidence note that states the layer owner and next uncovered slice.

Passing diagnostics alone is not enough. Passing synthetic fixtures alone is not enough for a
high-risk app path. The goal is controlled mechanism proof plus one real runtime lock where the
mechanism can visibly fail.
