---
title: "Delinea: Mutation Surface and Data Revisioning (v1)"
status: Proposed
date: 2026-02-09
scope: ecosystem/delinea
---

# Delinea: Mutation Surface and Data Revisioning (v1)

## Context

`delinea` is designed to support editor-grade, long-lived charts where data arrives incrementally.
If mutation semantics are left implicit, downstream caches and participation/output contracts will drift
as we refactor (especially under `WorkBudget` stepping).

This ADR defines the **v1 mutation surface** and the **revisioning rules** that all engine stages must
observe.

Related ADRs / docs:

- ADR 1149 (appendData + incremental caches): `docs/adr/1149-delinea-appenddata-and-incremental-caches.md`
- Workstream: `docs/workstreams/delinea-engine-contract-closure-v1.md` (M3)

## Goals

- Define a small set of supported mutation operations in v1.
- Make revision bumps and identity rules explicit (so caches can key correctly).
- Ensure append-only streaming remains deterministic and bounded under `WorkBudget`.

## Non-goals (v1)

- Arbitrary in-place edits (random row updates) with guaranteed incremental recompute.
- Row deletions / reindexing while preserving previously emitted raw-index identity.

## Definitions

- **Dataset root**: The physical `DataTable` stored in `DatasetStore` for a dataset id that has a table.
  Derived datasets (lineage) do not require a table in the store.
- **Raw index**: A stable, 0-based row identity within a dataset root table. Existing raw indices never
  change meaning under append-only mutation.
- **Data revision**: A monotonically increasing `Revision` on `DataTable` representing any mutation to
  table content or shape.

## v1 Mutation Surface

### 1) Append-only row/column growth (Required)

Supported operations (current API surface):

- Append rows to an existing table (row count increases).
- Append new columns (schema grows) when explicitly requested by the adapter/host.

Contract:

- Existing rows keep their raw indices.
- New rows receive increasing raw indices at the end (`old_row_count..new_row_count`).
- The table `revision` must bump when append occurs.

### 2) Updates (Explicit + constrained) (Optional)

v1 may support updates, but only if they are explicit and constrained by contract.
Until implemented, v1 treats any update-like change as requiring full invalidation of affected caches
(see ADR 1179).

If/when implemented, update constraints must be documented as:

- allowed column types / shapes,
- allowed row ranges,
- whether the mutation preserves row count,
- and which outputs must be considered invalid.

### 3) Deletions / reindexing (Not supported in v1)

Deleting rows or reindexing breaks the raw-index identity contract and is out of scope for v1.

## Revisioning Rules

### DataTable revision

- Any mutation to a `DataTable` (append rows, append columns, value updates) bumps `DataTable.revision`.
- The `row_count` and `columns.len()` are part of the observable table shape and must be consistent with
  the revision.

### Participation/output contracts

Consumers (marks, hit-test, tooltip/axisPointer sampling, link events) must treat `DataTable.revision`
as the canonical invalidation signal for any cached computation that depends on table contents.

## Consequences

- Append-only can be optimized via incremental caches, but correctness always keys off `DataTable.revision`
  and stable raw-index identity.
- v1 does not promise update-in-place performance; explicit invalidation is acceptable until contracts
  and caches are expanded.

## Implementation notes (evidence targets)

- Data append APIs: `ecosystem/delinea/src/data/mod.rs`
- Cache carriers / stepping: `ecosystem/delinea/src/transform_graph/*`, `ecosystem/delinea/src/engine/stages/*`
- Participation contract: `ecosystem/delinea/src/engine/stages/filter_processor.rs`

