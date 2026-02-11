# ADR 0183: Deterministic Graph Diff and Patch Units (`fret-node`)

Status: Accepted  
Date: 2026-02-05

## Implementation Status (as of 2026-02-06)

The deterministic diff and patch unit are implemented:

- Deterministic `graph_diff(from, to) -> GraphTransaction`: `ecosystem/fret-node/src/ops/diff.rs`
- Apply/invert/normalize correctness is covered by conformance tests:
  - Determinism + roundtrip: `ecosystem/fret-node/src/ops/tests.rs` (`graph_diff_is_deterministic_and_roundtrips`)
  - Cascading removals (node/port): `ecosystem/fret-node/src/ops/tests.rs`
  - Group removal detaches nodes deterministically: `ecosystem/fret-node/src/ops/tests.rs`
  - Edge endpoint changes preserve identity (`SetEdgeEndpoints`): `ecosystem/fret-node/src/ops/tests.rs`

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
3) Groups (add/remove + setters, including color)
4) Nodes (add/remove + setters)
5) Ports (add/remove + setters)
6) Edges (add/remove + setters)
7) Sticky notes (add/remove + setters)

Rationale:

- This order reduces "referenced id missing" apply failures during forward apply.
- Groups are emitted before nodes because nodes may reference a parent group.
- Edges are emitted last because they reference ports.

### 4) Setter strategy

For MVP, setters are emitted when a field differs (when a setter op exists and keeps identity):

- node: kind, kind_version, pos, parent, size, collapsed, hidden, ports ordering, data
- port: soft fields (`connectable*`, `ty`, `data`)
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

- Port-level setters exist for the most common "soft" fields:
  - `connectable`, `connectable_start`, `connectable_end`, `ty`, and `data`.
- Sticky-note setters exist for common edit fields:
  - `text`, `rect`, and `color`.
- Group setters include common edit fields:
  - `title`, `rect`, and `color`.
- Structural port changes (owner node, key, dir, kind, capacity) are represented as remove+add
  (the owning node `ports` ordering is restored and incident edges are re-added afterwards).
  This preserves correctness and determinism but is not minimal.
- Group removals detach child nodes as part of `RemoveGroup`:
  - Diffs must not emit redundant `SetNodeParent(Some(group), None)` ops.
  - If a node is re-parented away from a removed group, the diff emits `SetNodeParent(None, Some(new))`
    (because the intermediate state after `RemoveGroup` is detached).
