# Diag Fearless Refactor v2 — Crate and Module Map

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

## 0) Why this note exists

The v2 umbrella doc explains *what* diagnostics should become as a platform.
This note answers the more operational question contributors ask first:

- which crate should I edit,
- which module should own the change,
- which layer should stay untouched,
- and when DevTools GUI is in scope vs out of scope.

This is intentionally a routing map, not a contract document. ADRs and protocol docs remain the source of truth
for hard-to-change surfaces.

## 1) Layered routing map

### 1.1 Protocol and serializable contracts

Primary crate:

- `crates/fret-diag-protocol`

Owns:

- transport envelopes,
- script schemas,
- selectors and predicates,
- script results and reason-bearing payloads,
- serde-friendly types shared across tooling and runtime.

Edit here when:

- a new script step or predicate needs a versioned JSON shape,
- a transport message gains a new payload type,
- a result artifact needs a new stable field shared by runtime and tooling.

Do not put here:

- file I/O,
- CLI parsing,
- runtime-only logic,
- GUI-only browsing state.

Concrete anchors:

- `crates/fret-diag-protocol/src/lib.rs`
- `crates/fret-diag-protocol/src/builder.rs`

### 1.2 In-app runtime diagnostics service

Primary crate / module root:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/`

Owns:

- capture/export of runtime snapshots,
- script execution inside the app,
- inspect/pick flows,
- filesystem trigger handling,
- runtime-side WS bridge hooks,
- bundle/index/sidecar writing,
- runtime extension slots.

Edit here when:

- a new snapshot/debug surface is exported from the running app,
- selector resolution must change at runtime,
- a script step interacts with live UI state,
- a new sidecar is produced at capture time,
- runtime capabilities or pick/inspect behavior changes.

Important sub-areas today:

- service/root state:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- snapshot and bundle export:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/snapshot_recording.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_index.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_sidecars.rs`
- inspect/pick:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/pick.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/pick_flow.rs`
- script engine and step implementations:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_*.rs`
- transport-specific runtime glue:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/devtools_ws_helpers.rs`
- extensibility and domain diagnostics:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/docking_diagnostics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/virtual_list_diagnostics.rs`

Do not put here:

- repo-level suite orchestration,
- zip packing policies for host-side sharing,
- CLI-only argument resolution,
- DevTools screen layout decisions.

### 1.3 Transport layer

Primary crates / modules:

- `crates/fret-diag-ws`
- runtime bridge hooks in `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- tooling-side connection logic in `crates/fret-diag/src/devtools.rs`

Owns:

- moving diagnostics messages between processes or runtimes,
- session attachment and transport-specific connection details,
- WS-specific protocol framing implementation.

Edit here when:

- the transport handshake changes,
- session routing or reconnect behavior changes,
- a transport adds a capability that still uses existing protocol payloads.

Do not put here:

- gate logic,
- triage heuristics,
- GUI-only state,
- business rules that belong to runtime/tooling.

### 1.4 Tooling engine and host-side orchestration

Primary crate:

- `crates/fret-diag`

Thin CLI entrypoint:

- `apps/fretboard/src/diag.rs`

Owns:

- `run`, `suite`, `repeat`, `shrink`, `repro`, `matrix`, `perf`,
- bundle/path resolution,
- artifact indexing/lint/doctor/triage,
- compare/stats/query/slice,
- pack/zip/repro packaging,
- post-run checks and reason-code friendly failures,
- transport-agnostic orchestration.

Edit here when:

- the host decides how runs are launched or grouped,
- a new post-run gate is introduced,
- bundle triage/query/compare behavior changes,
- a new report or artifact pack is generated,
- a summary or regression-oriented command is added.

Important sub-areas today:

- command facade:
  - `crates/fret-diag/src/commands/`
- orchestration flows:
  - `crates/fret-diag/src/diag_run.rs`
  - `crates/fret-diag/src/diag_suite.rs`
  - `crates/fret-diag/src/diag_repeat.rs`
  - `crates/fret-diag/src/diag_repro.rs`
  - `crates/fret-diag/src/diag_matrix.rs`
  - `crates/fret-diag/src/diag_perf.rs`
- artifact and bundle handling:
  - `crates/fret-diag/src/artifact_store.rs`
  - `crates/fret-diag/src/artifacts.rs`
  - `crates/fret-diag/src/bundle_index.rs`
  - `crates/fret-diag/src/json_bundle.rs`
  - `crates/fret-diag/src/paths.rs`
- checks and registries:
  - `crates/fret-diag/src/registry/suites.rs`
  - `crates/fret-diag/src/registry/checks/mod.rs`
  - `crates/fret-diag/src/registry/checks/builtin_post_run/`
  - `crates/fret-diag/src/post_run_checks.rs`
- sharing and bounded evidence:
  - `crates/fret-diag/src/pack_zip.rs`
  - `crates/fret-diag/src/evidence_index.rs`
  - `crates/fret-diag/src/commands/ai_packet/`
- live-tooling transport bridge:
  - `crates/fret-diag/src/devtools.rs`

Do not put here:

- per-frame runtime capture logic,
- GUI view composition,
- portable protocol schema definitions that belong in `fret-diag-protocol`.

### 1.5 Presentation surfaces

Primary apps / tools:

- CLI wrapper: `apps/fretboard/src/diag.rs`
- MCP adapter: `apps/fret-devtools-mcp/src/main.rs`, `apps/fret-devtools-mcp/src/native.rs`
- script generation helper: `apps/fret-diag-scriptgen/src/main.rs`
- export helper: `apps/fret-diag-export/src/main.rs`
- offline viewer: `tools/fret-bundle-viewer`
- DevTools GUI workstream: `docs/workstreams/diag-devtools-gui-v1.md`

Owns:

- user-facing invocation UX,
- browsing and editing affordances,
- resource subscriptions and consumer-side flows,
- surface-specific integration glue.

Edit here when:

- the change is about how humans or external tools consume diagnostics,
- MCP resource/tool naming or response shape changes at the adapter layer,
- viewer/GUI UX changes without changing the underlying diagnostics contract.

Do not put here:

- canonical artifact semantics,
- runtime capture logic,
- host-side orchestration policies that should remain available to non-GUI consumers.

## 2) Practical routing recipes

### 2.1 "I need a new script step"

Typical path:

1. Add or extend the schema in `crates/fret-diag-protocol`.
2. Implement runtime execution in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps*.rs`.
3. If host-side validation/lint/help text is needed, update `crates/fret-diag`.
4. Only then update GUI/editor affordances if applicable.

### 2.2 "I need a new runtime debug field or sidecar"

Typical path:

1. Decide whether it belongs in the main snapshot/bundle or an extension/sidecar.
2. Implement export in `ecosystem/fret-bootstrap/src/ui_diagnostics/`.
3. Teach `crates/fret-diag` how to read/index/query it if needed.
4. Teach GUI/viewer how to browse it only if it becomes user-facing.

### 2.3 "I need a new regression gate"

Typical path:

1. Prefer host-side implementation in `crates/fret-diag`.
2. Reuse existing runtime artifacts if possible.
3. Add runtime export only when the evidence truly does not exist yet.
4. Add GUI affordances later as a consumer feature, not as the gate definition.

### 2.4 "I need a new transport capability"

Typical path:

1. Keep payload types in `crates/fret-diag-protocol`.
2. Implement transport mechanics in `crates/fret-diag-ws` and runtime/tooling bridges.
3. Avoid embedding gate or artifact policy in transport code.

### 2.5 "I need a new DevTools panel"

Typical path:

1. Confirm the underlying data is already exported or queryable.
2. If not, decide whether the missing piece is runtime, tooling, or adapter-level.
3. Keep the panel as a consumer of existing contracts whenever possible.

## 3) Recommended refactor seams by crate

### 3.1 `ecosystem/fret-bootstrap`

High-ROI seams to keep strengthening:

- service state and scheduling,
- snapshot/bundle export pipeline,
- inspect/pick flows,
- script step domains,
- transport bridges,
- extension registry and bounded domain payloads.

Preferred direction:

- continue pushing feature-specific logic into focused modules under `ui_diagnostics/`,
- keep `service.rs` as orchestration/state holder rather than a dumping ground for every new behavior.

### 3.2 `crates/fret-diag`

High-ROI seams to keep strengthening:

- command facade vs engine internals,
- orchestration flows vs reusable helpers,
- artifact resolution/materialization,
- registries and post-run checks,
- pack/triage/evidence reporting,
- transport adapters vs run semantics.

Preferred direction:

- add new behavior via registries or domain modules first,
- avoid re-growing `lib.rs` or giant match-driven orchestration paths.

### 3.3 Presentation apps

High-ROI seams to protect:

- `fretboard` stays thin,
- MCP stays an adapter over existing contracts,
- viewer/GUI stay consumers of bundle/artifact/report models.

Preferred direction:

- if presentation code needs new core semantics, add them to the lower layer first instead of inventing local workarounds.

## 4) DevTools GUI: included, but where exactly?

DevTools GUI is in scope for the v2 refactor, but its ownership line should stay explicit.

GUI should own:

- panel layout,
- tree browsing UX,
- script editing UX,
- artifact browser UX,
- live subscriptions and user interaction polish.

GUI should not own:

- the canonical meaning of bundle fields,
- the canonical definition of a gate,
- transport-only special cases that bypass shared tooling,
- alternative artifact models that only the GUI understands.

A good test:

- if the same behavior should also work in CLI, CI, MCP, or the viewer, it probably belongs below the GUI layer.

## 5) Current "do not split yet" stance

For now, v2 should still prefer **clearer module seams before new crate splits**.

That means:

- prefer cleaning boundaries inside `ecosystem/fret-bootstrap` and `crates/fret-diag`,
- keep `apps/fretboard` and adapters thin,
- only consider new crates after internal seams are proven stable and ownership/dependency pressure justifies a split.

## 6) Definition of done for this map

This routing map is doing its job when:

- a contributor can identify the target layer before editing code,
- runtime/tooling/presentation changes stop bleeding into each other by default,
- DevTools GUI can be discussed as "in scope" without becoming the center of every diagnostics decision,
- new diagnostics workstreams can reference this file instead of re-explaining the same crate boundaries.
