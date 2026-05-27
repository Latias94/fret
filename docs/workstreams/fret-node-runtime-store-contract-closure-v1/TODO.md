# `fret-node` Runtime/Store Contract Closure (v1) - TODO

Status: complete
Last updated: 2026-05-27

Task IDs use `FNRS` for `fret-node runtime/store`.

## Guardrails

- [x] Runtime correctness lands before UI mirror deletion.
- [x] Every task has a focused validation command before it is marked done.
- [x] Compatibility retained behavior remains covered while internals move toward store-first.
- [x] Feature-contract changes update docs and compile-matrix gates in the same slice.
- [x] Workstream decisions stay in `DESIGN.md`, `MILESTONES.md`, or ADRs, not only in chat.

## FNRS-010 - Close `GraphOp` to `NodeGraphChanges` semantics

Status: done
Owner: planner/worker
Dependencies: none

Problem:

`GraphOp` covers more graph edits than `runtime/changes.rs` reports. The current mapping silently
drops unmatched operations, which makes controlled-mode callbacks and external synchronization
incomplete.

Scope:

- `ecosystem/fret-node/src/ops/mod.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- focused runtime tests near the existing change-mapping coverage
- controlled-mode example only if the public callback contract needs a small adjustment

Deliverables:

- A small change-coverage matrix in code or tests that names every `GraphOp` variant.
- Red tests for at least two currently dropped observable operations.
- An implementation that emits deliberate `NodeGraphChange` values or explicit non-observable
  outcomes for every `GraphOp`.
- No catch-all silent drop for future observable operations.

Validation:

- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`

Review notes:

- Review should focus on semantic intent, not just line coverage.
- If a `GraphOp` is intentionally non-observable, the reason must be documented in the mapping
  test or helper name.

Completion notes:

- `NodeChange` now covers node selectable, draggable, connectable, deletable, parent, extent,
  expand-parent, hidden, and port-order changes.
- `EdgeChange` now covers edge selectable, deletable, and reconnectable changes.
- `RemoveNode` and `RemovePort` now report cascaded edge removals in `NodeGraphChanges`.
- `NodeGraphChanges::from_transaction` no longer uses a catch-all arm; graph-resource operations
  outside the XyFlow-style node/edge change-array contract are explicitly listed as requiring the
  committed `GraphTransaction` for full-fidelity controlled integrations.
- Fresh validation on 2026-05-26:
  - `cargo nextest run -p fret-node --no-default-features runtime`: 41 passed.
  - `cargo check -p fret-node --no-default-features`: passed.

## FNRS-020 - Make lookup cache updates exhaustive and stale-safe

Status: done
Owner: planner/worker
Dependencies: FNRS-010

Problem:

`NodeGraphLookups` caches derived node/edge fields, but incremental dispatch does not cover every
operation that can affect cached hidden state, reconnectability, endpoints, ports, or geometry.

Scope:

- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/runtime/store.rs`
- focused lookup/store tests

Deliverables:

- Tests proving `store.lookups()` is fresh immediately after dispatch for operations that mutate
  hidden state and reconnectability.
- Exhaustive incremental application for lookup-affecting operations, or an explicit rebuild path
  for operations that cannot be incrementally updated safely.
- A guard against adding future lookup-affecting operations without updating lookup handling.

Validation:

- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`

Review notes:

- Prefer precise incremental updates where the operation payload has enough information.
- Prefer a deliberate rebuild over partial incremental handling when precision would be fragile.

Completion notes:

- Added store-level regression coverage for stale lookup updates after `SetNodeHidden` and
  `SetEdgeReconnectable`.
- Added regression coverage for `RemovePort` updating node port lookup state and incident edge
  lookup state.
- Added regression coverage for `RemoveGroup` clearing detached node parent lookup state.
- `NodeGraphLookups::apply_op` no longer has a catch-all success arm; lookup-unaffected operations
  are explicitly listed.
- Fresh validation on 2026-05-26:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: 45 passed.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

## FNRS-030 - Harden store dispatch as the single runtime commit pipeline

Status: done
Owner: planner/worker
Dependencies: FNRS-010, FNRS-020

Problem:

Store dispatch, change emission, lookup maintenance, history, subscribers, and controlled sync must
derive from one coherent commit pipeline. If these are maintained independently, the UI cleanup
phase will reintroduce drift.

Scope:

- `ecosystem/fret-node/src/runtime/store.rs`
- `ecosystem/fret-node/src/runtime/changes.rs`
- `ecosystem/fret-node/src/runtime/lookups.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`

Deliverables:

- A documented dispatch order inside the store implementation.
- Tests proving a transaction updates graph document, changes, lookups, and subscribers coherently.
- Removal or quarantine of duplicate commit paths that bypass store dispatch for committed edits.

Validation:

- `cargo nextest run -p fret-node --no-default-features runtime`
- targeted default-feature tests for controller/binding store sync, as discovered during the task

Review notes:

- This task may split if a large bypass surface is found.
- Do not remove retained compatibility transport until equivalent store-first evidence exists.

Completion notes:

- Added a store dispatch coherency test proving graph state, `NodeGraphChanges`, lookups, history,
  and subscribers observe the same committed metadata update.
- Extracted common store commit finalization helpers for installing committed graph state,
  advancing revision, updating lookups, deriving changes, emitting `GraphCommitted`, and notifying
  selectors.
- `dispatch_transaction`, `dispatch_transaction_with_profile`, `undo`, `undo_with_profile`,
  `redo`, and `redo_with_profile` now share the same graph-state install/publish path.
- Fresh validation on 2026-05-26:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: 46 passed.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

## FNRS-040 - Reduce UI state mirrors after runtime/store gates are green

Status: done
Owner: planner/worker
Dependencies: FNRS-030

Problem:

Declarative and retained UI code still carries multiple graph/view/editor-config mirrors and sync
helpers. Once the runtime/store contract is reliable, these mirrors should either become short-lived
snapshots or be deleted.

Scope:

- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/controller_store_sync.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- focused retained/declarative compatibility tests

Deliverables:

- A short inventory of remaining long-lived UI mirrors and their owner.
- At least one concrete mirror-removal or quarantine slice with tests.
- A rule for when a UI surface may hold transient local interaction state versus committed graph
  state.

Validation:

- `cargo nextest run -p fret-node --features compat-retained-canvas`
- focused default-feature tests for the changed UI surfaces

Review notes:

- This task should not become a broad rewrite. Split follow-ups if more than one compatibility
  surface needs independent review.

Completion notes:

- Added `UI_MIRROR_INVENTORY_2026-05-26.md` with the first long-lived UI mirror map.
- Quarantined `NodeGraphSurfaceBinding` graph/view/editor-config mirrors behind a private
  `NodeGraphSurfaceMirrors` container while preserving public accessors and sync behavior.
- Updated surface-policy coverage so future binding fields must preserve the explicit mirror
  boundary.
- Left retained `NodeGraphCanvas` model ownership untouched; it needs a separate retained
  compatibility slice.
- Fresh validation on 2026-05-26:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers new_binding_seeds_graph_view_and_store_models from_store_clones_initial_store_state_into_surface_models`: 3 passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo clippy -p fret-node --features compat-retained-canvas --all-targets -- -D warnings`:
    failed in unrelated `crates/fret-ui/src/tree/layout/clean_geometry.rs` lints; no `fret-node`
    lint was reached.

## FNRS-050 - Clean feature, dependency-boundary, and policy-test contracts

Status: done
Owner: planner/worker
Dependencies: FNRS-010, FNRS-020, FNRS-030

Problem:

The `headless` feature name is misleading with default features enabled. The `fret-ui-kit`
dependency and roadmap wording disagree. The crate root contains large string-scanning surface
policy tests that should live closer to the contract they protect.

Scope:

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/tests/`
- `docs/node-graph-roadmap.md`
- this workstream and related public-posture docs

Deliverables:

- A documented feature matrix that states default, headless, and compatibility-retained modes.
- Either a code change or documentation update that resolves the `fret-ui-kit` boundary tension.
- Migration of large surface-policy tests out of `src/lib.rs` where practical.
- Compile/test gates for the supported feature matrix.

Validation:

- `cargo check -p fret-node --no-default-features`
- `cargo check -p fret-node --no-default-features --features headless`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo nextest run -p fret-node --no-default-features runtime`

Review notes:

- Renaming or deprecating features is a public contract change. Update docs and consider ADR
  alignment if the impact crosses crate boundaries.

Completion notes:

- Clarified `headless` as a documented marker for `--no-default-features` builds, not a feature
  that disables defaults.
- Added a feature contract section to `docs/node-graph-roadmap.md`.
- Updated the roadmap wording to acknowledge the current `fret-ui` -> `fret-ui-kit` integration
  dependency while preserving the headless runtime/store policy boundary.
- Moved the large crate-root `surface_policy_tests` module from `src/lib.rs` to
  `src/surface_policy_tests.rs`; `src/lib.rs` is now back to the crate surface.
- Fresh validation on 2026-05-26:
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers first_party_node_graph_demos_stay_declarative_only`: 2 passed.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo check -p fret-node --no-default-features --features headless`: passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: 46 passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.

## FNRS-060 - Closeout verification and follow-on split

Status: done
Owner: planner/reviewer
Dependencies: FNRS-010, FNRS-020, FNRS-030, FNRS-040, FNRS-050

Problem:

This lane is complete only when the runtime/store contract is strong enough to support future
fearless cleanup without relying on chat memory.

Scope:

- workstream docs
- closeout audit
- validation command evidence
- related docs or ADR alignment rows when changed

Deliverables:

- Fresh evidence for all required gates.
- Updated `HANDOFF.md` with no hidden blockers.
- A closeout audit or split follow-on list.
- Clear statement of which remaining retained/declarative cleanup belongs to other workstreams.

Validation:

- `cargo fmt --check`
- `cargo nextest run -p fret-node --no-default-features runtime`
- `cargo check -p fret-node --no-default-features`
- `cargo check -p fret-node --no-default-features --features headless`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`

Completion notes:

- Closeout audit added in `CLOSEOUT_AUDIT_2026-05-27.md`.
- Fresh closeout gates passed on 2026-05-27:
  - `cargo fmt --check`: passed.
  - `cargo fmt -p fret-node --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features runtime`: 46 passed.
  - `cargo check -p fret-node --no-default-features`: passed.
  - `cargo check -p fret-node --no-default-features --features headless`: passed.
  - `cargo check -p fret-node --features compat-retained-canvas`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- Remaining retained `NodeGraphCanvas` mirror cleanup is split as follow-on work because it has a
  separate compatibility review surface from the runtime/store contract closure.
