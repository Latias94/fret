---
type: Work Progress
title: Phase 3 U5.5 retained parent query bridges
tags: fret,phase3,u5-5,retained-parent,topology,subagent
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagents:
  - 019f27e8-69ae-77e2-8886-9c5a1ab16ebc
  - 019f27e8-b218-7bc3-b070-ee0df54a3246
---

# Summary

This slice inserted U5.5 into the Phase 3 plan and closed the remaining normal-path retained-parent
query bridges found after U5.

Changes:

- `set_children_in_mount` now treats the redundant initial mount walk skip as a layer-root-only
  fast path derived from current topology, not from retained `Node.parent == None`.
- Runtime/debug parent-chain callers in scroll telemetry, shadcn command/sheet/drawer tests, and
  the gallery hit-chain debug helper now use `node_parent_in_layer_tree`.
- `node_parent_in_layer_tree` is public child-edge topology API; retained `node_parent` is
  test-only storage inspection.
- The stale GC comment that described `node_layer` as parent-pointer based was updated to name the
  actual invariant: GC liveness must be child-edge reachability from liveness roots.

# Subagent Findings Applied

- Explorer `019f27e8-69ae-77e2-8886-9c5a1ab16ebc` found two remaining risks: the
  `set_children_in_mount` initial-skip predicate read retained `Node.parent`, and a normal scroll
  debug ancestry path still called `node_parent()`. It also confirmed child-edge topology helpers
  were acceptable and that direct `Node.parent` writes remain storage sync, not query authority.
- Explorer `019f27e8-b218-7bc3-b070-ee0df54a3246` confirmed the U6 text direction: cluster/run
  facts already exist in `fret-render-text`, while WGPU residency remains glyph-only. U6 should add
  CPU-side `TextShape` cluster metadata and keep atlas pinning glyph-based.

# Verification

Passed:

- `cargo check -p fret-ui`
- `cargo check -p fret-ui-shadcn`
- `cargo check -p fret-ui-gallery`
- `cargo nextest run -p fret-ui set_children_in_mount --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`

# Deletion Gate

Static search for `.node_parent(` in `crates`, `ecosystem`, and `apps` now finds only test paths in
`crates/fret-ui`. Runtime callers must use `node_parent_in_layer_tree`, which follows layer-root
child edges.

# Next

Proceed to U6 text shaping metadata:

- Add CPU-side `TextGlyphCluster` metadata to WGPU `TextShape`.
- Add `cluster_index` to CPU `GlyphInstance`.
- Make visible text residency select complete clusters, then expand to glyph keys for atlas pinning.
- Preserve full-blob helpers until U7.
