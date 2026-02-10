---
title: "Delinea: Mutation Cache Invalidation and Resume Policy (v1)"
status: Proposed
date: 2026-02-09
scope: ecosystem/delinea, ecosystem/fret-chart
---

# Delinea: Mutation Cache Invalidation and Resume Policy (v1)

## Context

`delinea` uses multiple caches/stages to keep large-data interactions bounded under `WorkBudget`.
With append-only data streams, we need a clear policy for:

- which caches may **resume** scanning when the dataset grows,
- which caches must **invalidate and rebuild**,
- and how those decisions relate to `DataTable.revision` and stable raw-index identity.

This ADR is the v1 policy companion to:

- ADR 1198 (mutation surface + revisioning rules)
- ADR 1149 (appendData + incremental caches)

## Goals

- Provide an explicit v1 matrix: mutation type -> cache behavior.
- Define stable cache keys and the minimal invalidation boundaries.
- Keep behavior deterministic across refactors and compatible with derived dataset lineage (ADR 1195).

## Non-goals (v1)

- Proving optimal incremental recompute for every node in the pipeline.
- Fine-grained dependency tracking across arbitrary transform graphs.

## Mutation classes (v1)

1. **Append rows**: `row_count` increases; existing rows unchanged.
2. **Append columns**: schema grows; existing rows unchanged; new columns may be used by downstream encodes.
3. **Update values** (if supported): values change in-place; `row_count` stable.

## v1 Policy Matrix

### Append rows

#### Output continuity under `WorkBudget` (v1 invariant)

For append-only row growth, the engine must preserve previously emitted outputs while it incrementally
extends/rebuilds under budget.

Invariants:

- The engine must **not clear** `output.marks` solely due to append-only dataset growth.
- While an append-only rebuild is **unfinished**, `output.marks.nodes` must remain non-empty and must
  continue to represent the last completed geometry for each series that previously produced marks.
  (Incremental refinement is allowed; clearing/removing the entire geometry is not.)
- Per-series mark identity must remain stable across append-only rebuilds (mark ids remain associated
  with the same `SeriesId`).
- Once the rebuild completes, the marks output must reflect the newly appended raw indices where
  applicable (e.g. LOD / sampling output includes the appended raw index when it is within the
  series’ participation contract).

Evidence target (implementation seam):

- Append-only rebuild path keeps the previous `MarkTree` visible and calls `begin_append_rebuild`
  instead of clearing marks: `ecosystem/delinea/src/engine/mod.rs`.

Allowed to resume (v1 target):

- Nodes that scan a monotonic prefix and can safely extend to `new_row_count` without revisiting earlier
  rows, while producing deterministic output:
  - nearest-X index building (prefix-based),
  - ordinal/category index building (prefix-based),
  - indices views that are built by scanning raw indices and are safe to extend.

May rebuild (acceptable in v1):

- Derived dataset transform mappings (filter/sort) may rebuild on revision/row_count change until a
  resume strategy is implemented, but must remain bounded and step-able under `WorkBudget`.

Must invalidate:

- Any cache keyed to absolute row ranges that become out-of-bounds without clamping.

### Append columns

Must invalidate:

- Any cache that reads the appended column (directly or via `FieldId -> column` mapping), including:
  - marks for series that encode the new field,
  - dataset transform filters/sorts that reference the new field,
  - data extent caches for axes reading the new field.

May retain:

- Caches unrelated to the new column (e.g. indices built on independent columns), provided keys include
  the correct field/column dependencies.

### Update values

v1 default policy: **invalidate and rebuild** for all caches that read updated values.
If updates are introduced later, the update API must include enough information to decide what can be
resumed safely.

## Cache key requirements (v1)

For any cached node, keys must include (as applicable):

- `model.revs.*` for spec/model dependencies,
- dataset identity + **lineage root dataset id** (ADR 1195),
- `DataTable.revision` for table-dependent computations,
- `row_count` and any view/window inputs used to define the scanned range,
- transform parameters (filter/sort configs, filter modes, etc.).

## Consequences

- We can land v1 with conservative invalidation while keeping correctness deterministic.
- Resume is a performance optimization that must be explicitly proven and regression-gated where it
  matters (append-only workloads).

## Evidence targets (current and planned)

- Append-only APIs and revision bumps: `ecosystem/delinea/src/data/mod.rs`
- Incremental caches (existing): `ecosystem/delinea/src/transform_graph/data_view.rs`,
  `ecosystem/delinea/src/engine/stages/{nearest_x_index,ordinal_index}.rs`
- Derived dataset lineage mapping (current): `ecosystem/delinea/src/transform_graph/dataset_transform.rs`
- Regression gates: `ecosystem/delinea/src/engine/tests.rs` (append-only continuity + update invalidation), headless goldens under `goldens/echarts-headless/v1/`
