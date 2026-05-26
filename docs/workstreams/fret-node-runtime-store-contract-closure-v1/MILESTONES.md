# `fret-node` Runtime/Store Contract Closure (v1) - Milestones

Status: active
Last updated: 2026-05-26

## Global Success Criteria

- Controlled-mode callbacks and external synchronization see every observable committed graph edit.
- `store.lookups()` is fresh immediately after dispatch for all supported runtime operations.
- Store dispatch is the obvious single commit pipeline for graph document, changes, lookups,
  history, subscribers, and controller/binding sync.
- UI cleanup removes or quarantines duplicate state mirrors only after runtime/store gates prove the
  source of truth is reliable.
- Feature and dependency-boundary docs match the supported Cargo feature matrix.
- The crate root no longer carries large policy scans that can live in focused integration tests or
  audit helpers.

## M0 - Workstream Opened And Baseline Captured

Status target: documentation closure

Done criteria:

- `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and
  `HANDOFF.md` exist.
- The lane explicitly references the older declarative-first workstream without absorbing it.
- The first executable task is ready and bounded.

Evidence anchors:

- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/DESIGN.md`
- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/TODO.md`
- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/EVIDENCE_AND_GATES.md`

## M1 - Runtime Change Semantics Closed

Status target: FNRS-010 complete

Done criteria:

- Every `GraphOp` variant is accounted for by change emission or an explicit non-observable
  decision.
- No catch-all branch silently drops future observable operations.
- Focused tests fail against the old behavior and pass after the implementation.
- Controlled-mode semantics have enough change information to keep external state synchronized.

Evidence anchors:

- `ecosystem/fret-node/src/ops/mod.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- focused runtime tests added or updated by FNRS-010

## M2 - Lookup Cache Correctness Closed

Status target: FNRS-020 complete

Done criteria:

- Lookup-affecting `GraphOp` variants update `NodeGraphLookups` incrementally or trigger an
  explicit rebuild.
- Hidden state, reconnectability, endpoint, port, and geometry cache paths have focused coverage
  where relevant.
- Store dispatch does not expose stale lookup snapshots after a committed transaction.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- focused lookup/store tests added or updated by FNRS-020

## M3 - Store Dispatch Pipeline Hardened

Status target: FNRS-030 complete

Done criteria:

- Store dispatch order is locally documented and test-proven.
- Graph document mutation, change emission, lookup maintenance, history, subscribers, and controlled
  sync stay coherent for representative transactions.
- Any remaining bypass paths are either removed or explicitly marked compatibility-only with tests.

Evidence anchors:

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`

## M4 - UI Mirror Cleanup Started Safely

Status target: FNRS-040 complete

Done criteria:

- Remaining long-lived UI mirrors have an owner and reason.
- At least one unnecessary mirror or sync path is removed or quarantined.
- Retained compatibility gates still pass for the touched surface.
- Local interaction state versus committed graph state is documented in the touched code or tests.

Evidence anchors:

- `ecosystem/fret-node/src/ui/binding*.rs`
- `ecosystem/fret-node/src/ui/canvas/`
- focused retained/declarative tests

## M5 - Feature And Documentation Contracts Aligned

Status target: FNRS-050 complete

Done criteria:

- The Cargo feature matrix is documented and validated by commands.
- `headless` behavior is either renamed, documented precisely, or otherwise made non-misleading.
- The `fret-ui-kit` dependency boundary is resolved in code or docs.
- Large policy scans are moved out of the crate root where practical.

Evidence anchors:

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/tests/`
- `docs/node-graph-roadmap.md`

## M6 - Closeout

Status target: workstream complete or split follow-ons

Done criteria:

- Fresh command evidence is recorded in `EVIDENCE_AND_GATES.md`.
- `HANDOFF.md` describes no unresolved blocker for the closed scope.
- Remaining work is either out of scope or split into a follow-on workstream/task ledger.
- Reviewer can verify the lane without relying on chat context.

