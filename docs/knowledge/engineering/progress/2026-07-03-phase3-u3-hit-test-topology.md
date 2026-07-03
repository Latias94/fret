---
type: Work Progress
title: Phase 3 U3 hit-test topology cleanup
tags: fret,architecture,phase3,retained-bridges,topology,hit-test
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U3 Hit-Test Topology Cleanup

Phase 3 U3 hit-test slice removes retained parent ancestry from normal hit-test path validation and
path-cache publication.

Implemented topology changes:

- Bounds-tree candidate reachability now builds the root-to-candidate path from child edges rooted at
  the queried layer root.
- Hit-test path-cache publication now stores a child-edge path, so stale retained parent pointers
  cannot poison the reusable pointer route.
- `crates/fret-ui/src/tree/hit_test.rs` has no remaining direct `Node.parent` reads.

Regression coverage:

- `hit_test_path_cache_uses_child_edges_under_stale_parent_pointers` failed on the retained-parent
  implementation because the second query missed the cached path and fell back. It now passes by
  reusing the child-edge path cache.
- Existing `tree::tests::hit_test` coverage still passes for layer order, stale path rejection,
  transformed siblings, rounded clips, modal fallback coordinates, and cache suspension.
- Full `cargo nextest run -p fret-ui --no-fail-fast` passed with 1192 tests.

Verification:

- `cargo nextest run -p fret-ui hit_test_path_cache_uses_child_edges_under_stale_parent_pointers --no-fail-fast`
- `cargo nextest run -p fret-ui tree::tests::hit_test --no-fail-fast`
- `rg -n "\\.parent|node_parent\\(" crates/fret-ui/src/tree/hit_test.rs`
- `cargo check -p fret-ui`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `cargo nextest run -p fret-ui --no-fail-fast`

Next action: continue U3 with semantics snapshot parent assignment, widget coordinate mapping,
propagation depth ordering, bounds-tree prepaint parent reconstruction, and remaining invalidation
or debug-only retained parent classifications.
