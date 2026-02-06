# fret-node Deterministic Patch Units M6 (Collaboration Readiness)

This workstream locks the **collaboration patch unit** for `ecosystem/fret-node`:
`GraphOp` + `GraphTransaction`, and the deterministic diff generator `ops::graph_diff(from, to)`.

The goal is not "minimal diffs at all costs". The goal is:

- deterministic, stable patch unit shape,
- apply-safe ordering (no missing references),
- roundtrip correctness (apply diff reproduces `to`),
- refactor safety (conformance tests that prevent semantic drift).

Related contract ADR:

- `docs/adr/0198-deterministic-graph-diff-and-patch-units.md`

## Why this exists

For large-graph scale (NG3) and future collaboration (OT/CRDT later), we need a reversible,
deterministic patch format that can be:

- serialized and synced without hash-order drift,
- used as a refactor gate ("this edit stream still means the same thing"),
- applied safely even when destructive ops cascade (e.g. node/port removal removes edges).

## Scope

In scope:

- `GraphOp` and `GraphTransaction` as the canonical reversible patch unit.
- Deterministic diff: `graph_diff(from, to) -> GraphTransaction`.
- Normalization semantics (coalescing setters, dropping no-ops).
- Apply/invert correctness for all diff-emitted ops.
- Conformance tests that lock determinism + roundtrip.

Out of scope:

- Conflict resolution / merging semantics (OT/CRDT integration is explicitly deferred).
- Domain-specific semantics derived from opaque JSON payloads (`node.data`, `port.data`, `symbol.meta`).
- UI internals contracts (those are covered by workstream M0).

## Quality gates (definition of done)

- `cargo fmt`
- `cargo nextest run -p fret-node`

## Hard contracts

### 1) Deterministic ordering (no hash iteration leaks)

- `graph_diff(from, to)` must emit ops in stable, deterministic order.
- Running the diff multiple times must yield identical JSON for `tx.ops`.

Evidence:

- `docs/adr/0198-deterministic-graph-diff-and-patch-units.md`
- `ecosystem/fret-node/src/ops/diff.rs`
- `ecosystem/fret-node/src/ops/tests.rs` (`graph_diff_is_deterministic_and_roundtrips`)

### 2) Apply-safe ordering (no missing references)

- The diff must be apply-safe in forward order (no "referenced id missing" errors).
- Destructive ops that cascade (RemoveNode / RemovePort) must not cause follow-up double-removals.

Evidence:

- `ecosystem/fret-node/src/ops/diff.rs`
- `ecosystem/fret-node/src/ops/tests.rs`
  - `graph_diff_roundtrips_when_deleting_a_node_with_ports_and_edges`
  - `graph_diff_roundtrips_when_deleting_a_port_with_incident_edges`

### 3) Roundtrip correctness (JSON-equal target)

For any `from`, `to` that are individually structurally valid:

- Applying `graph_diff(from, to)` to a clone of `from` must yield a graph JSON-equal to `to`.

Evidence:

- `ecosystem/fret-node/src/ops/tests.rs` (`graph_diff_is_deterministic_and_roundtrips`)

## MVP compromises (explicitly locked)

- Ports use setter ops for soft fields (`connectable*`, `ty`, `data`).
- Structural port changes (owner node, key, dir, kind, capacity) are represented as remove+add.
  - The owning node `ports` ordering is restored (`SetNodePorts`).
  - Incident edges removed by `RemovePort` are re-added when they still exist in `to`.
- Groups and sticky notes prefer setters for common edits to preserve identity.

## Work items (M6)

### M6A — Lock the patch unit shape (docs + tests)

- [x] ADR 0198: patch unit shape + determinism contract.
- [x] Conformance tests for determinism + roundtrip.
- [x] Conformance tests for destructive cascades (node/port deletion).
- [x] Conformance tests for structural port change apply-safety (remove+add + restore).

### M6B — Close remaining gaps (backlog)

These are "nice to have" improvements once the MVP contracts are stable:

- [ ] Consider port structural setter ops (key/dir/kind/capacity) if/when we need more minimal
  collaboration diffs (ensure apply-safe detaching/reattaching semantics stay explicit).
- [ ] Add dedicated conformance tests for:
  - group removal detaches nodes deterministically,
  - edge endpoint changes preserve identity (`SetEdgeEndpoints`) across reconnect flows.

