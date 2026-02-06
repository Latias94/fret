# Node Graph Roadmap (fret-node)

This is a living implementation roadmap (not an ADR). Contracts are defined by ADRs; this file
tracks staged work, prioritization, and concrete TODOs for the node graph substrate and editor UI.

Authoritative contracts:

- Node graph editor + typed connections: `docs/adr/0135-node-graph-editor-and-typed-connections.md`
- Undo/redo + transactions: `docs/adr/0024-undo-redo-and-edit-transactions.md`
- Action hooks / policy boundary: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Pan/zoom transform + hit-testing: `docs/adr/0083-render-transform-hit-testing.md`

Conformance:

- Interaction checklist (manual script + invariants): `docs/node-graph-interaction-checklist.md`
- XyFlow parity matrix (detailed feature map): `docs/node-graph-xyflow-parity.md`

Upstream references (pinned locally):

- XyFlow: `repo-ref/xyflow` (esp. `packages/system/src/*`)
- ImGui Node Editor: `repo-ref/imgui-node-editor`
- egui-snarl: `repo-ref/egui-snarl`
- Unity ShaderGraph: `repo-ref/Graphics/Packages/com.unity.shadergraph`

## Scope

`ecosystem/fret-node` is the reusable node graph crate that targets multiple domains:

- shader graphs (ShaderGraph-like)
- blueprints / behavior graphs
- dataflow pipelines (Dify-like workflows)

Design constraints:

- The graph model is long-lived, serializable, and migration-friendly.
- UI-only derived state must not leak into graph assets.
- UI integration is optional (behind the default `fret-ui` feature); a headless build must remain
  usable for validation, tooling, and server-side workflows.

## Current Status (Snapshot)

Now / Next / Later (high level):

- **Now**: lock refactor invariants (derived internals + invalidation discipline) via conformance tests (workstream M0).
  - Detailed contract checklist: `docs/workstreams/fret-node-internals-m0.md`
- **Next**: stabilize built-in add-ons API (minimap/controls/background theming) without policy bleed (workstream M2).
  - Workstream: `docs/workstreams/fret-node-addons-api-m2.md`
- **Later**: scale targets (5k–20k) + deterministic patch units for collaboration (NG3/workstream M6 + future milestones).

### Headless substrate (present)

- Serializable graph model with stable IDs: `Graph`, `Node`, `Port`, `Edge`, `Group`, `StickyNote`.
- Typed connections and planning: `TypeDesc` + `ConnectPlan` + rules layer.
- Reversible edits: `GraphOp` + `GraphTransaction` + history (undo/redo).
- Schema and profiles:
  - schema registry / port declarations,
  - profile pipeline hooks for domain specialization (validation, concretization).
- Diagnostics are first-class (`Diagnostic`, severity, source).

### Editor UI substrate (present, WIP-stabilization)

- Single-canvas node graph widget (`NodeGraphCanvas`) with:
  - pan/zoom,
  - node selection + marquee,
  - node dragging, group dragging/resizing (where applicable),
  - edge connect/reconnect flows,
  - multi-connection bundle interactions (Shift/Ctrl variants),
  - snaplines,
  - context menus + conversion picker/searcher flows (staged).
- Derived internals separation:
  - `MeasuredGeometryStore` / `NodeGraphInternalsStore` are UI-only and not serialized.
- Canvas portal (Stage 2 from ADR 0135):
  - `NodeGraphPortalHost` mounts `fret-ui` subtrees per node in screen space.
  - Portal commands are mediated through a handler and commit `GraphTransaction` for undo/redo.
  - Editors:
    - `PortalTextEditor` (text input + error, stepper)
    - `PortalNumberEditor` (text input + error, stepper, drag-adjust)
- Minimap + controls overlays (present, WIP):
  - minimap consumes derived geometry only (`NodeGraphInternalsStore`),
  - controls dispatch canonical view ops (zoom/fit/reset) via `node_graph.*` commands.

### Demo harness (present)

- `apps/fret-examples/src/node_graph_demo.rs` demonstrates:
  - typed ports/wires + connect/reconnect,
  - conversion picker insertion flow,
  - portal number editing (stepper + drag threshold).
- The demo is now **store-driven** (B-layer): `NodeGraphStore` is the source of truth; `Graph` and `NodeGraphViewState` models are kept in sync by the canvas.

To run:

- `cargo run -p fret-demo --features node-graph-demos --bin node_graph_demo`
- `cargo run -p fret-demo --features node-graph-demos --bin node_graph_domain_demo`
- preferred runner: `cargo run -p fretboard -- dev native --bin node_graph_demo`

## Missing Pieces (Gap Analysis)

See `docs/node-graph-xyflow-parity.md` for the detailed parity and gap map (mechanism-first).

## Refactor tracking (workstreams)

When doing larger refactors, treat `docs/workstreams/fret-node-xyflow-parity.md` as the active
execution plan (milestones + gates + evidence anchors). This roadmap remains a “what/why” view and
should not accumulate ambiguous TODOs that aren’t tied to an exit criterion.

Suggested mapping (high level):

- NG0 (Contracts + Harness) ↔ workstream M0/M1
- NG1 (Editor Usability) ↔ workstream M2 (add-ons stabilization) + selected A-layer fixes
- NG2 (Domain Readiness) ↔ typed connections + profile pipeline hardening (domain demos)
- NG3 (Scale + Collaboration) ↔ derived geometry invalidation + culling + deterministic patch units (workstream M6)

## Milestones and Exit Criteria

### NG0 — Contracts + Harness (P0)

Exit criteria:

- A repeatable interaction checklist + at least one automated conformance test set.
- Portal command routing semantics locked (ADR 0135) and implemented consistently across editors.
- View-state persistence contract decided (ADR 0135 open question resolved or deferred explicitly).

### NG1 — Editor Usability (P1)

Exit criteria:

- Minimap + controls shipped and wired through the canonical pan/zoom ops.
- Auto-pan during connect/drag implemented with tunables.
- Clipboard copy/paste for selection works with deterministic offsets and undo/redo.

### NG2 — Domain Readiness (P2)

Exit criteria:

- Domain profile APIs proven by a "real" domain demo (shader-ish or blueprint-ish) without
  modifying core UI contracts.
- Subgraph + symbol workflows implemented in headless substrate and surfaced in UI.

### NG3 — Scale + Collaboration (P3)

Exit criteria:

- Demonstrate a 5k–20k node graph with acceptable frame times (target TBD) via culling/caching.
- Deterministic patch format for collaboration (OT/CRDT integration deferred, but the patch unit is locked).

## TODO List (Actionable)

Legend:

- `[ ]` TODO
- `[~]` in progress
- `[x]` done

### Near-term (next 1–2 sprints)

- [x] Add node graph interaction checklist doc (like docking arbitration): `docs/node-graph-interaction-checklist.md`.
- [x] Implement minimap overlay consuming derived geometry.
- [x] Implement canvas controls (zoom/fit/reset) and bind to commands.
- [x] Implement auto-pan during connect/drag near edges.
- [x] Add "drag handle tooltip/help" in demo (components-layer tooltip; do not add `fret-ui-kit` dep to `fret-node`).

### Medium-term

- [x] Clipboard copy/paste for selection with `GraphFragment` + deterministic paste offset.
- [x] Edge label rendering + per-edge style overrides.
- [x] Edge markers (end arrow) + routing kinds (straight/step/bezier) polish.
- [x] Reroute node + edge split UX (double-click reroute; Alt+double-click and Alt-drag open insert picker).
- [x] Interaction presets (XyFlow vs ShaderGraph) as kit helpers (`NodeGraphInteractionPreset`).
- [x] Keyboard nudge ops.
- [x] Selection align/distribute ops.

### Long-term

- [~] Subgraph graph references + cycle-safe import.
  - ADR: `docs/adr/0197-subgraph-graph-references-and-cycle-safe-import.md`
  - Core closure + tests: `ecosystem/fret-node/src/core/imports.rs`, `ecosystem/fret-node/src/core/tests.rs`
  - Subgraph node contract + binding tests: `ecosystem/fret-node/src/core/subgraph.rs`, `ecosystem/fret-node/src/core/tests.rs`
  - Import edit ops (Add/Remove/Alias): `ecosystem/fret-node/src/ops/mod.rs`, `ecosystem/fret-node/src/ops/apply.rs`, `ecosystem/fret-node/src/ops/history.rs`
  - Tests: `ecosystem/fret-node/src/ops/tests.rs`
- [~] Blackboard variables + typed symbol references (domain-ready).
  - Symbol edit ops (name/type/default/meta): `ecosystem/fret-node/src/ops/mod.rs`, `ecosystem/fret-node/src/ops/apply.rs`, `ecosystem/fret-node/src/ops/history.rs`
  - Tests: `ecosystem/fret-node/src/ops/tests.rs`
  - Symbol reference nodes (baseline contract + validation):
    - Contract helpers: `ecosystem/fret-node/src/core/symbol_ref.rs` (`SYMBOL_REF_NODE_KIND`)
    - Structural validation: `ecosystem/fret-node/src/core/validate.rs`
    - Tests: `ecosystem/fret-node/src/core/tests.rs`
    - Copy/paste includes referenced symbols: `ecosystem/fret-node/src/ops/fragment.rs` + `ecosystem/fret-node/src/ops/tests.rs`
- [~] Large-graph culling + incremental updates.
  - [x] Portal subtree culling for offscreen nodes (`NodeGraphPortalHost::layout`).
  - [x] Canvas paint culling for offscreen nodes/edges (`NodeGraphCanvas::paint`).
- [x] Deterministic graph diff/patch set for collaboration (MVP).
  - ADR: `docs/adr/0198-deterministic-graph-diff-and-patch-units.md`
  - Minimal deterministic diff: `ecosystem/fret-node/src/ops/diff.rs` (`graph_diff`)
  - Patch units:
    - Ports: setter ops for soft fields (`connectable*`, `ty`, `data`); structural changes use remove+add (and restore `SetNodePorts` + re-add incident edges when needed).
    - Groups: setters for common edits (`title`, `rect`, `color`) to preserve identity.
    - Sticky notes: setters for common edits (`text`, `rect`, `color`) to preserve identity.
  - Tests: `ecosystem/fret-node/src/ops/tests.rs` (`graph_diff_is_deterministic_and_roundtrips`, `graph_diff_roundtrips_when_a_port_changes_structurally`, `graph_diff_roundtrips_when_deleting_a_port_with_incident_edges`)
- [ ] Patch unit minimality follow-ups (optional, deferred).
  - Consider port structural setter ops (key/dir/kind/capacity) if we need more minimal collaboration diffs.
  - Workstream: `docs/workstreams/fret-node-deterministic-patch-units-m6.md`
