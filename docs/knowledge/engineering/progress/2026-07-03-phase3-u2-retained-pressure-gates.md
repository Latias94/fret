---
type: Progress
title: Phase 3 U2 retained identity and liveness pressure gates
tags: fret,architecture,phase3,diagnostics,retained-bridges
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U2 Retained Identity and Liveness Pressure Gates

Phase 3 U2 adds deletion evidence before moving live queries away from retained bridge paths.

Implemented runtime signals:

- `UiDebugFrameStats` now records shadow parent repair passes/nodes, retained subtree membership
  scan roots/nodes, and GC stale-liveness offenders.
- `UiTree::parent_pointers_would_repair_from_layer_roots` counts the same reachable parent
  inconsistencies as `repair_parent_pointers_from_layer_roots` without mutating `Node.parent`.
- Existing normal repair call sites record the shadow oracle before repair, so U5 can require zero
  would-repair nodes before demoting or deleting normal repair.
- View-cache retained subtree membership fallbacks now record scan pressure without changing the
  fallback behavior; U4 owns the migration away from those scans.
- GC remove-subtree diagnostics now also increment `gc_stale_liveness_offenders` when a stale
  removal still looks live according to layer/view-cache liveness evidence.

Implemented diagnostics gates:

- `fret-diag` registers the new frame stats as perf keys and exposes an opt-in `diag stats` gate
  through:
  - `--check-parent-pointer-would-repair-max`
  - `--check-gc-stale-liveness-offenders-max`
  - `--check-retained-subtree-membership-scan-nodes-max`
- Historical bundles remain compatible because missing new fields default to zero in the pressure
  gate and in `UiFrameStatsV1` serde.
- `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json` was
  regenerated from `fretboard diag stats --perf-keys-json`.

Subagent input:

- Explorer `019f262e-eba0-7460-be7f-4d360328913b` audited existing `fret-diag` registry/gate
  patterns and recommended keeping U2 to minimal opt-in max gates, leaving expanded taxonomy,
  streaming support, and suite defaults to later units.

Next action: U3 should introduce frame/boundary topology as the normal query authority while using
these U2 counters as the deletion safety net.
