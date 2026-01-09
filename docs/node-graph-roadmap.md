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
  - multi-connection “bundle” interactions (Shift/Ctrl variants),
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

### Demo harness (present)

- `apps/fret-examples/src/node_graph_demo.rs` demonstrates:
  - typed ports/wires + connect/reconnect,
  - conversion picker insertion flow,
  - portal number editing (stepper + drag threshold).

## Missing Pieces (Gap Analysis)

This section lists “editor-grade” capabilities that we still need to reach XyFlow maturity and
ShaderGraph-level workflows.

### P0 gaps (block scaling the surface area)

- Conformance harness for node graph interactions (like `docs/docking-arbitration-checklist.md`):
  - connect/reconnect determinism,
  - selection invariants,
  - undo granularity (drag sessions commit once),
  - modifier routing invariants (IME/shortcuts boundary).
- Contract hardening around portal measurement injection ordering (frame-order hazards).
- View-state persistence contract for multi-view graphs:
  - stable view keys,
  - what persists per-view vs per-asset.

### P1 gaps (XyFlow-level editor usability parity)

- Minimap / overview navigation.
- Built-in canvas controls: zoom in/out, fit view, reset view.
- Auto-pan while connecting or dragging near canvas edges (configurable).
- Better handle resolution policy:
  - connection radius / strict vs loose mode tunables,
  - deterministic tie-breakers when multiple handles are in range.
- Selection UX parity:
  - keyboard nudge,
  - align/distribute,
  - “frame selection / frame all”.
- Clipboard UX:
  - copy/paste selection as `GraphFragment`,
  - deterministic paste offsets,
  - cross-view paste.
- Edge rendering parity:
  - per-edge style overrides (interaction width, color, label),
  - reroute nodes / edge splitting affordances.
- Accessibility semantics baseline for the canvas and embedded controls.

### P2 gaps (ShaderGraph / Blueprint readiness)

- Blackboard/symbol workflows:
  - variables as first-class, type-safe references,
  - local vs graph-scoped symbol tables.
- Subgraphs:
  - explicit graph references + cycle safety,
  - subgraph instance parameters.
- Dynamic ports & concretization end-to-end (domain-driven):
  - port add/remove flows with stable keys,
  - validation + conversion insertion policies.
- Domain toolchains:
  - shader slot typing (vector/matrix/precision/color space),
  - preview nodes (GPU-backed thumbnails),
  - compile/validate pipelines with diagnostics and “quick fix” insertion.

### P3 gaps (Large-graph performance & collaboration)

- Large graph rendering:
  - culling / incremental scene updates,
  - cache invalidation strategy for derived geometry.
- Collaboration readiness:
  - deterministic diffs and patch sets,
  - stable ordering and conflict handling.

## Milestones and Exit Criteria

### NG0 — Contracts + Harness (P0)

Exit criteria:

- A repeatable interaction checklist + at least one automated conformance test set.
- Portal command routing semantics locked (ADR 0135) and implemented consistently across editors.
- View-state persistence contract decided (ADR 0135 open question resolved or deferred explicitly).

### NG1 — Editor Usability Parity (P1)

Exit criteria:

- Minimap + controls shipped and wired through the canonical pan/zoom ops.
- Auto-pan during connect/drag implemented with tunables.
- Clipboard copy/paste for selection works with deterministic offsets and undo/redo.

### NG2 — Domain Readiness (P2)

Exit criteria:

- Domain profile APIs proven by a “real” domain demo (shader-ish or blueprint-ish) without
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
- [ ] Implement minimap overlay consuming derived geometry.
- [ ] Implement canvas controls (zoom/fit/reset) and bind to commands.
- [ ] Implement auto-pan during connect/drag near edges.
- [ ] Add “drag handle tooltip/help” in demo (components-layer tooltip; do not add `fret-ui-kit` dep to `fret-node`).

### Medium-term

- [ ] Clipboard copy/paste for selection with `GraphFragment` + deterministic paste offset.
- [ ] Edge label rendering + per-edge style overrides.
- [ ] Reroute node + edge split UX.
- [ ] Selection align/distribute + keyboard nudge ops.

### Long-term

- [ ] Subgraph graph references + cycle-safe import.
- [ ] Blackboard variables + typed symbol references (domain-ready).
- [ ] Large-graph culling + incremental updates.
- [ ] Deterministic graph diff/patch set for collaboration.
