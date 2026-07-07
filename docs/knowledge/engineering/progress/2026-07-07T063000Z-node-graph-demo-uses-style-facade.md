---
type: "Work Progress"
title: "Node graph demo uses style facade"
description: "Work Progress for Node graph demo uses style facade."
timestamp: 2026-07-07T06:30:00Z
tags: ["ui-surface", "examples", "node-graph", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/node-graph-surface-facade"
---

# Summary

Promoted `apps/fret-examples/src/node_graph_demo.rs` from advanced/manual quarantine to the default
app authoring surface by routing its remaining paint vocabulary through `fret::style`.

# Details

Changed files:

- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/tests/node_graph_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Use `fret::style::{Color, DashPatternV1}` in the demo source.
- Rely on the app prelude for `Px`.
- Classify `node_graph_demo.rs` as default-clean because it now uses the app view surface plus
  node-graph facade APIs, not raw `fret_core` paint types.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test node_graph_demo_surface --features node-graph-demos --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples --features node-graph-demos`

# Next Action

Continue shrinking advanced/manual quarantine only where the remaining seam is a missing facade or
misplaced harness. Keep renderer/effect/docking/runtime proofs explicit.

# Citations

- `apps/fret-examples/src/node_graph_demo.rs`
- `tools/check_surface_policy.py`
