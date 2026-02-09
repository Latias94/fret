# ADR 1177: `delinea` Dataset Transform Cache Keys + Invalidation (v1)

Status: Proposed

## Context

`delinea` is revision-driven and budget-aware. To keep derived dataset transforms scalable, we must
define:

- what caches exist for dataset transform nodes,
- what uniquely identifies a cached output (cache key),
- and when caches must resume vs invalidate (mutation boundaries).

This ADR complements:

- ADR 1140 / ADR 1149 (dataset storage + append semantics),
- ADR 1175 (raw-index identity),
- ADR 1176 (v1 transform node set).

## Decision

### 1) Dataset transform nodes are cached by deterministic signatures

Each dataset transform node output is cached by a signature that includes:

- upstream lineage root dataset id,
- upstream dataset revision (from `DataTable.revision`),
- transform chain hash (type + parameters in order),
- and any execution-relevant gating parameters (e.g. max view length caps).

The output of a dataset transform node is a `RowSelection` (typically indices) plus any auxiliary
state required for incremental stepping.

### 2) Append-only vs replace invalidation (v1)

When the lineage root dataset revision changes:

- **Append-only mutation** (row_count increases, historical rows unchanged):
  - filter caches may resume scanning from the previous completion point,
  - sort caches may extend their key vectors with only the appended rows, but must re-establish the
    global order before publishing the final mapping.
- **Replace mutation** (table replaced or historical rows may have changed):
  - all dataset-transform caches for that root must invalidate and rebuild from scratch.

### 3) Budgeted stepping rules

Dataset transform nodes must support `WorkBudget` stepping:

- Filter nodes may materialize indices incrementally and publish partial progress internally.
- Sort nodes may scan keys incrementally, but must only publish a final stable order once the
  required inputs are available.

The engine must remain deterministic: given the same inputs and a “run to completion” budget, the
final mapping must be identical.

### 4) Chain composition

For chained transforms, each node’s signature includes the upstream node’s signature (or an
equivalent hash of its output identity), so intermediate caches can be reused when only later nodes
change.

## Consequences

- Transform chains can be incrementally recomputed without cloning tables.
- Streaming workloads can reuse work on append-only mutations where safe.
- Cache invalidation remains explicit and testable.

## Follow-ups

- Add unit tests for cache reuse on append-only mutations across a filtered derived dataset.
- Add headless goldens to assert raw-index identity under chained transforms.

