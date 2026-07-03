---
type: Progress
title: Phase 3 U1 contract freeze
tags: fret,architecture,phase3,contracts,retained-bridges
timestamp: 2026-07-03
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Phase 3 U1 Contract Freeze

Phase 3 U1 freezes the retained-bridge deletion contract before runtime code changes.

Updated contract surfaces:

- `docs/runtime-contract-matrix.md` now points to the Phase 3 plan and distinguishes normal runtime behavior from debug/parity oracles, compatibility alias readers, and explicit advanced/raw app seams.
- `docs/ui-closure-map.md` now treats Phase 3 as the active closure target and requires remaining bridge matches to be classified as normal path, debug/parity oracle, compatibility alias/reader, or explicit advanced/raw seam.
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md` now interprets ADR 0066, ADR 0165, and ADR 0327 through the Phase 3 retained-bridge deletion plan.
- `docs/knowledge/engineering/current-state.md` now makes Phase 3 U2 the next action.

Key decisions captured:

- `Node.parent` is retained storage/debug evidence after frame/boundary topology covers live queries.
- Parent repair deletion needs a non-mutating would-repair shadow oracle, not only zero normal repair calls.
- `FlatCompat` is explicit debug/parity oracle only; supported chunk-launch fixtures need zero normal-path flat usage.
- Text closure must preserve shaping cluster/run facts in WGPU `TextShape` residency metadata before full-blob text helpers are retired.
- Partial upload expansion needs per-stream fallback reasons and write-count/byte budgets.

Next action: implement U2 retained identity/liveness pressure gates.

