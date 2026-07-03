---
type: Work Progress
title: Phase 3 U3 input ancestry topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,input,shortcuts,focus
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Input Ancestry Topology Cleanup

Phase 3 U3 input-routing slice removes retained parent ancestry from shortcut key-context and
default pointer-focus ancestor queries.

Implemented topology changes:

- `UiTree::shortcut_key_context_stack` now walks focused barrier descendants through layer-forest
  child edges instead of retained `Node.parent`.
- `UiTree::focus_chain_contains` now checks barrier containment through layer-forest child edges, so
  a stale retained parent cannot force shortcut routing to fall back to the barrier root.
- `UiTree::first_focusable_ancestor_including_declarative` now resolves focusable ancestors through
  layer-forest child edges, so pointer-down default focus cannot select a stale retained sibling.

Regression coverage:

- `shortcut_key_context_stack_uses_child_edges_for_focused_barrier_descendant` failed on the retained
  parent implementation with only the overlay-root key context and now passes with the focused leaf
  plus overlay stack.
- `default_focus_ancestor_uses_child_edges_under_stale_parent_pointers` failed on the retained
  parent implementation by selecting the stale focusable sibling and now selects the current-frame
  child-edge parent.
- Existing `key_dispatch_barrier_root` and `prevent_default` focused tests still pass.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1191 tests.

Verification:

- `cargo nextest run -p fret-ui default_focus_ancestor_uses_child_edges_under_stale_parent_pointers shortcut_key_context_stack_uses_child_edges_for_focused_barrier_descendant --no-fail-fast`
- `cargo nextest run -p fret-ui key_dispatch_barrier_root prevent_default --no-fail-fast`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-ui --no-fail-fast`

Next action: continue U3 with hit-test path cache, semantics snapshot parent assignment, widget
coordinate mapping, propagation depth, and remaining invalidation or debug-only retained parent
classifications.
