---
title: Delinea Engine Contract Closure v1 — Milestones
status: draft
date: 2026-02-09
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

Status: In progress (engine + retained adapter landed; global controllers landed; brush+link scaffolds landed; remaining: broader linking semantics beyond brush X exports).

Linking follow-ups (now tracked as contracts-first):

- ADR 1172: axisPointer/cursor link events (crosshair sync).
- ADR 1173: domain window link events (sync zoom/pan).

## M2 — Transform lineage contract (derived datasets/columns)

Scope:

- Represent derived datasets/columns as first-class engine nodes with stable raw-index identity.
- Allow adapter/translator to describe transforms declaratively, without eager table cloning.

Acceptance criteria:

- A minimal “dataset transform graph v0” exists (filter + sort + simple derived columns).
- Raw-index identity is stable and testable across transform chains.
- Headless golden coverage includes at least one chained transform scenario.

Status: Planned.

## M3 — Incremental mutation semantics (append/update)

Scope:

- Define explicit mutation surfaces and their cache invalidation boundaries:
  - append-only streaming (fast path),
  - limited updates (v1: constrained by contract; not arbitrary editing).

Acceptance criteria:

- Append-only behavior is regression-gated and remains bounded under budget.
- Update semantics are explicit (what is supported, what is not) and covered by at least one invariant test.

Status: Planned.

## M4 — Conformance harnesses (keep refactors safe)

Scope:

- Expand headless goldens for semantic drift prevention.
- Add at least one interactive/scripted gate for multi-grid + linking if/when the UI adapter surface stabilizes.

Acceptance criteria:

- Headless goldens cover: multi-axis, multi-grid, transforms lineage, filter-mode edge cases.
- Interactive demo(s) exist with a stable “what to validate” checklist.

Status: Planned.
