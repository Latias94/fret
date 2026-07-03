---
type: Progress
title: Phase 3 U3 root and descendant topology authority
tags: fret,architecture,phase3,retained-bridges,topology,liveness
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Root and Descendant Topology Authority

Phase 3 U3 third slice changes the core `UiTree` root/descendant queries from retained parent
pointers to layer-forest child-edge topology.

Implemented topology changes:

- `UiTree::node_root` now resolves the containing layer root by testing current layer roots and
  authoritative `children` reachability.
- `UiTree::is_descendant` now delegates to child-edge reachability, making the public query name
  match current-frame topology authority.
- `node_is_attached_to_layer_tree`, `node_layer`, stable element handle validation, viewport owner
  filtering, and other callers that depend on `node_root` now reject stale retained parents as live
  attachment proof.

Regression coverage:

- `descendant_via_children_ignores_stale_parent_pointers` now asserts both `is_descendant` and
  `is_descendant_via_children` stay true when only retained parents are corrupted.
- `gc_retention_rejects_stale_parent_pointer_layer_membership` now proves stale retained parents
  cannot make `node_layer` succeed.
- Full `fret-ui` nextest passed after the migration.

Next action: continue U3 by classifying the remaining parent uses. Normal dispatch paths should use
dispatch snapshots or child-edge topology; retained storage mutation, debug diagnostics, and U5
parent repair/dirty-count migration remain separate.
