---
title: Delinea Engine Contract Closure v1 — Milestones
status: draft
date: 2026-02-10
scope: ecosystem/delinea, ecosystem/fret-chart, contracts, tests, demos
---

# Delinea Engine Contract Closure v1 — Milestones

This document is a one-screen milestone board for engine-level contract closure in `delinea`.

Source of truth for detailed TODOs: `docs/workstreams/delinea-engine-contract-closure-v1-todo.md`.
Narrative + scope: `docs/workstreams/delinea-engine-contract-closure-v1.md`.

## M0 — Docs honesty + audit closure

Acceptance criteria:

- ADRs and alignment docs accurately describe the shipped v1 subset:
  - multi-dimensional `WeakFilter` subset and size caps,
  - Y dataZoom “mapping-first” boundary and indices materialization constraints,
  - current multi-grid posture (single engine + per-grid plot viewports) and the follow-up target
    (global controllers + opt-in cross-grid linking).
- Implementation alignment table reflects current reality for the above ADRs.

Status: Done (2026-02-09).

## M-1 — Main sync (keep worktree green)

Scope:

- Regularly merge local `main` into the workstream branch.
- Keep the worktree compiling as core contract surfaces evolve (notably:
  - `UiServices` gaining `MaterialService`,
  - `Paint`-based `SceneOp::Quad` fields (`background`, `border_paint`),
  - component-level prop additions like `PressableProps.key_activation`).

Acceptance criteria:

- `cargo check --workspace --all-targets` passes after each sync.
- A minimal set of representative tests can still run (at least one nextest gate in `fret-node` or `fret-chart`).

Status: Done (2026-02-10).
Evidence: `5cad446f`, `fcc14780`, merges `e9c13385` + `8f174420`.

## M1 — Single-engine multi-grid viewport/layout contract

Scope:

- Add a per-grid viewport/layout carrier to the engine contract surface (spec/model/output).
- Define deterministic routing rules for:
  - axisPointer crosshair sampling,
  - dataZoom / window writes,
  - brush selection output,
  - link groups across grids (opt-in).

Acceptance criteria:

- A single `ChartEngine` can render multiple grids without splitting the spec.
- The existing multi-axis harness can be extended into a multi-grid harness with deterministic routing outcomes.
- At least one headless regression gate covers multi-grid window + marks outputs.

Status: Done (2026-02-10).

Linking follow-ups (now tracked as contracts-first):

- ADR 0249: axisPointer/cursor link events (crosshair sync).
- ADR 0250: domain window link events (sync zoom/pan).
- ADR 0251: host-level link mapping policy (LinkAxisKey) for cross-spec linking.

Implementation follow-ups:

- Implement a key-based chart link coordinator (`LinkedChartGroup`) and allow explicit axis map overrides for ambiguous specs.

## M2 — Transform lineage contract (derived datasets/columns)

Scope:

- Represent derived datasets/columns as first-class engine nodes with stable raw-index identity.
- Allow adapter/translator to describe transforms declaratively, without eager table cloning.

Acceptance criteria:

- A minimal “dataset transform graph v0” exists (filter + sort for derived datasets; derived columns are a follow-up).
- Raw-index identity is stable and testable across transform chains.
- Headless golden coverage includes at least one chained transform scenario.

Contracts (v1 subset):

- ADR 0252: transform lineage + raw-index identity.
- ADR 0253: minimal dataset transform node set (filter + sort + derived dataset chaining).
- ADR 0254: cache keys + invalidation boundaries for dataset transforms.

Status: Done (2026-02-10) for the v1 subset (derived datasets with filter/sort transforms + stable raw-index identity).

## M3 — Incremental mutation semantics (append/update)

Scope:

- Define explicit mutation surfaces and their cache invalidation boundaries:
  - append-only streaming (fast path),
  - limited updates (v1: constrained by contract; not arbitrary editing).

Acceptance criteria:

- Append-only behavior is regression-gated and remains bounded under budget.
- Update semantics are explicit (what is supported, what is not) and covered by at least one invariant test.

Contracts (v1 subset):

- ADR 0255: mutation surface + data revisioning rules.
- ADR 0256: cache invalidation + resume policy matrix.

Status: Done (2026-02-09).

Progress notes:

- Append-only under `WorkBudget` is regression-gated and preserves previously emitted marks while unfinished:
  `ecosystem/delinea/src/engine/tests.rs` (`append_only_marks_rebuild_preserves_geometry_while_unfinished_multi_series`) (2026-02-09).
- Update semantics are explicit via `DataTable` update APIs and are regression-gated (forces deterministic invalidation):
  `ecosystem/delinea/src/engine/tests.rs` (`update_mutation_clears_marks_and_forces_rebuild`) (2026-02-09).

## M4 — Conformance harnesses (keep refactors safe)

Scope:

- Expand headless goldens for semantic drift prevention.
- Add at least one interactive/scripted gate for multi-grid + linking if/when the UI adapter surface stabilizes.

Acceptance criteria:

- Headless goldens cover: multi-axis, multi-grid, transforms lineage, filter-mode edge cases.
- Interactive demo(s) exist with a stable “what to validate” checklist.

Status: Done (2026-02-10).

Known remaining closure items:

- document a “fast” nextest subset for local + CI (done via `tools/gates_delinea_fast.ps1`).
- ensure the linking `fretboard diag` script remains runnable after refactors and `main` syncs
  (confirmed 2026-02-10 via `tools/diag-scripts/chart-multi-axis-linking-domain-window-pixels-changed.json`).
