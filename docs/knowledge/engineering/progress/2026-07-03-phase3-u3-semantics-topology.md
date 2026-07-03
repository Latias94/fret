---
type: Work Progress
title: Phase 3 U3 semantics topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,semantics
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Semantics Topology Cleanup

Phase 3 U3 semantics slice removes retained parent ancestry from semantics snapshot parent links.

Implemented topology changes:

- Semantics traversal stack entries now carry the current-frame parent derived from child-edge
  traversal.
- `SemanticsNode.parent` is now written from that traversal parent instead of `Node.parent`.
- The semantics scratch stack and frame-arena capacity accounting now include the carried parent.

Regression coverage:

- `semantics_snapshot_parent_uses_child_edges_under_stale_parent_pointers` failed on the retained
  parent implementation by publishing the stale focusable sibling as the leaf's semantics parent.
  It now publishes the actual child-edge parent.
- Existing semantics subtree reuse, focus shortcut, scroll-into-view, and slider set-value gates
  continue to pass.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1193 tests.

Verification:

- `cargo nextest run -p fret-ui semantics_snapshot_parent_uses_child_edges_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui semantics_focus_shortcuts semantics_slider_set_value_gate scroll_into_view --no-fail-fast`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-ui --no-fail-fast`

Next action: continue U3 with widget coordinate mapping, propagation depth ordering, bounds-tree
prepaint parent reconstruction, and remaining invalidation or debug-only retained parent
classifications.
