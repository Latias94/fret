# ADR 0198: Deterministic Graph Diff and Patch Units (`fret-node`)

Status: Proposed  
Date: 2026-02-05

## Context

`fret-node` uses `GraphOp` + `GraphTransaction` as the canonical reversible edit unit. For:

- collaboration (OT/CRDT integration later),
- deterministic serialization and syncing,
- large refactors without semantic drift,

we need a stable, deterministic way to derive a patch set between two graph snapshots.

The hard part is not "diffing JSON"; it is locking:

- the patch unit shape (what edits exist),
- deterministic ordering (no hash iteration leakage),
- normalization rules (coalescing setter chains, removing no-ops),
- roundtrip properties (apply diff recreates the target state).

## Goals

- Provide a deterministic diff from `Graph` → `Graph` expressed as `GraphTransaction`.
- Guarantee stable ordering and stable normalization semantics.
- Keep this headless-safe (no UI deps) and resilient to internal refactors.

## Non-Goals

- Solving conflict resolution / merging semantics (OT/CRDT comes later).
- Inferring domain semantics from opaque payloads (node/port/symbol `data` is treated as opaque JSON).
- Producing minimal diffs at all costs (determinism and correctness first).

## Decision

### 1) Patch unit is `GraphTransaction`

`graph_diff(from, to)` returns a `GraphTransaction` of `GraphOp` variants.

Normalization:

- always call `ops::normalize_transaction` on the produced transaction.

### 2) Deterministic ordering

The diff algorithm MUST:

- iterate maps in stable key order (`BTreeMap` order),
- emit ops in a stable phase order (see below),
- never depend on insertion order or hash iteration.

### 3) Phase order (apply-safe)

The diff emits ops in the following high-level order:

1) Imports (add/remove + alias)
2) Symbols (add/remove + setters)
3) Nodes (add/remove + setters)
4) Ports (add/remove + setters)
5) Edges (add/remove + setters)
6) Groups (add/remove + setters; color changes via remove+add)
7) Sticky notes (add/remove; changes via remove+add)

Rationale:

- This order reduces "referenced id missing" apply failures during forward apply.
- Edges are emitted last because they reference ports.

### 4) Setter strategy

For MVP, setters are emitted when a field differs:

- node: kind, kind_version, pos, parent, size, collapsed, hidden, ports ordering, data
- port: all fields including `ty` and `data`
- edge: kind, endpoints, selectable/deletable/reconnectable
- import: alias
- symbol: name, ty, default_value, meta

The diff does not attempt to synthesize "remove+add" when a setter exists; it prefers setters to
preserve identity when possible.

### 5) Roundtrip contract

For any `from`, `to` that are individually structurally valid:

- applying `graph_diff(from, to)` to a clone of `from` must yield a graph that is JSON-equal to `to`.

Additionally:

- running `graph_diff(from, to)` multiple times must yield identical JSON for `tx.ops`.

## Evidence / Conformance Gates

- `ecosystem/fret-node/src/ops/diff.rs`
- `ecosystem/fret-node/src/ops/tests.rs` (diff determinism + roundtrip)
Note (MVP compromise):

- Port-level setters are not defined yet, so port changes are represented as remove+add (and edges
  are re-added afterwards). This preserves correctness and determinism but is not minimal.

