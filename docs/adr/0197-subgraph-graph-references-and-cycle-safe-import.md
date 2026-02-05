# ADR 0197: Subgraph Graph References and Cycle-Safe Imports (`fret-node`)

Status: Proposed  
Date: 2026-02-05

## Context

Editor-grade node graphs frequently need **composition**:

- a graph can contain a node that represents another graph (“subgraph node”),
- a graph can depend on other graphs for shared logic, reusable functions, or domain libraries,
- graphs can be split into multiple assets/modules over time.

The highest risk is not the UI; it is the **dependency semantics**:

- missing references must fail deterministically (no partial imports),
- import order must be deterministic (stable derived state, stable patch units),
- cycles must be detected with a stable error surface (no recursion/infinite loops),
- dependency closure must be explicit and refactor-friendly (fearless internal refactors).

This intersects with but does not depend on an app-owned asset database (ADR 0026). `fret-node`
still needs a stable, serializable contract for “graph depends on graph”.

Related:

- ADR 0135: Node Graph Editor and Typed Connections (`fret-node`)
- ADR 0026: Asset Database and Import Pipeline (app-owned, deferred)
- ADR 0024: Undo/redo and edit transactions (patch unit determinism)

## Goals

- Provide a **stable, serializable dependency declaration** for cross-graph references.
- Provide a **cycle-safe import closure** algorithm with deterministic results.
- Keep the contract **headless-safe** (no UI types, no `fret-ui` dependency).
- Make it possible to add UI affordances later (subgraph nodes, library browsers) without changing
  the core dependency semantics.

## Non-Goals

- Designing a full asset database or file system layout (app-owned).
- Defining subgraph UI semantics (presentation/interaction) in this ADR.
- Automatically inferring dependencies by scanning arbitrary node payloads.

## Decision

### 1) Graphs declare imports explicitly

`core::Graph` carries an explicit imports table:

- `Graph.imports: BTreeMap<GraphId, GraphImport>`

Contract:

- The **key** is the imported graph’s `GraphId`.
- The map value is reserved for future metadata (aliasing/namespacing, version pins, etc.).
- Import declarations are **data-only** and must remain stable under UI refactors.

Rationale:

- Cross-graph references should not depend on the presence of a particular node kind.
- Explicit imports enable deterministic dependency tracking and cycle detection without requiring
  schema-specific payload scanning.

### 2) Cycle-safe, deterministic import closure

We define a headless helper:

- `resolve_import_closure(root_graph, resolver) -> Result<GraphImportClosure, GraphImportError>`

Contract:

- Missing imports return `GraphImportError::MissingGraph { from, to }`.
- Cycles return `GraphImportError::Cycle { cycle }` where `cycle` is a stable path that ends at the
  repeated node (e.g. `A -> B -> C -> A`).
- The traversal order is deterministic:
  - imports are visited in the sorted order of `GraphId` (the `BTreeMap` key order),
  - closure order is a deterministic DFS postorder (dependencies appear before dependents).

Rationale:

- Determinism is a prerequisite for stable derived internals and for future collaboration patch units.
- A stable error surface prevents “sometimes works” behavior when cycles appear.

### 3) Asset GUIDs remain app-owned (for now)

This ADR uses `GraphId` as the import reference key. An app-owned asset database may choose to
manage a separate stable GUID per file (ADR 0026) and map it to a `GraphId` by reading the asset.

Open question (tracked in ADR 0135):

- whether node graph assets should also carry an app-level GUID alongside `GraphId`.

### 4) Subgraph nodes reference imports (they do not declare them)

We reserve a node kind for subgraph nodes:

- `SUBGRAPH_NODE_KIND = "fret.subgraph"`

Contract:

- A subgraph node's `Node.data` must be a JSON object with a `graph_id` string (UUID).
- The referenced `graph_id` **must be declared** in `Graph.imports`.

Rationale:

- `Graph.imports` remains the explicit dependency declaration (contract-level).
- Subgraph nodes are a consumer of that contract; this avoids implicit dependency inference from
  arbitrary domain payloads while still enabling domain UX ("subgraph nodes") to be standardized.

## Consequences

- Graph dependency semantics are explicit, deterministic, and testable.
- UI implementations can evolve without breaking import closure semantics.
- Future work (typed symbol references, subgraph node UX, collaboration patches) has a stable base.

## Evidence / Conformance Gates

- Unit tests:
  - `ecosystem/fret-node/src/core/imports.rs`
  - `ecosystem/fret-node/src/core/tests.rs` (import closure tests)
  - `ecosystem/fret-node/src/core/subgraph.rs`
  - `ecosystem/fret-node/src/core/tests.rs` (subgraph binding tests)
