# ADR 0135: Node Graph Editor and Typed Connections (`fret-node`)

Status: Proposed  
Date: 2026-01-06

## Context

Fret aims to be an editor-grade UI framework. A general-purpose node graph editor is a foundational
editor primitive that we want to reuse across multiple domains:

- dataflow graphs (generic pipelines and “Dify-like” flows),
- blueprint-style exec graphs,
- shader graphs (a stricter, compiler-backed specialization).

Local upstream references show three complementary strengths:

- `repo-ref/imgui-node-editor`: mature interaction protocol (“draw your content, we do the rest”),
  including link creation/deletion queries, selection, navigation, persisted editor state hooks, and
  a **canvas coordinate escape hatch** (`Suspend/Resume`) for screen-space popups/menus while the
  graph is pan/zoom transformed.
- `repo-ref/Graphics/Packages/com.unity.shadergraph`: asset-first graph model, strong slot compatibility
  rules, graph validation/diagnostics, dynamic slot concretization, unknown-node survival, and migration.
- `repo-ref/egui-snarl`: small data model + separate UI state + a “viewer” trait that externalizes
  behavior, including multi-connection interactions.
- `repo-ref/xyflow` (React Flow / Svelte Flow): production-grade interaction and geometry contracts:
  handle IDs, strict vs loose connection modes, reconnection flows, edge hit slop (`interactionWidth`),
  parent/child subflows with movement extents, and the separation of user-authored nodes from derived
  internals (measured sizes, absolute positions, cached handle bounds).

We want a Fret-native node editor that:

- avoids future large refactors by locking the “hard-to-change” boundaries now,
- stays consistent with Fret’s layering rules (ADR 0074: component-owned interaction policy),
- remains extensible enough to support future shader/blueprint/workflow specializations.

Related ADRs:

- Action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Commands and palette: `docs/adr/0023-command-metadata-menus-and-palette.md`
- Shortcut normalization: `docs/adr/0018-key-codes-and-shortcuts.md`
- Undo/redo transactions (editor scope): `docs/adr/0024-undo-redo-and-edit-transactions.md`
- Clipboard / drag-and-drop sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Inspector protocol (editor scope): `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- Render-transform hit-testing: `docs/adr/0083-render-transform-hit-testing.md`
- Theme tokens: `docs/adr/0032-style-tokens-and-theme-resolution.md`
- Crate boundaries: `docs/adr/0093-crate-structure-core-backends-apps.md`

## Goals

- Provide an editor-grade **node graph UI** for Fret with stable, reusable contracts.
- Support **typed connections** and **constraint-driven graph validation** without coupling to a
  specific domain (shader/blueprint/workflow).
- Support long-lived assets: stable IDs, serialization, and an explicit migration pipeline.
- Keep `crates/fret-ui` mechanism-only: the node editor is a policy-heavy component in `ecosystem/`.

## Non-Goals

- Shipping a full ShaderGraph clone in the framework layer.
- Baking a specific type system (shader precision/space, JSON schema, etc.) into the core model.
- Guaranteeing compile-time typing for runtime node payloads (pluginability and migration take priority).

## Decision

### 1) Crate placement and feature policy

We introduce a new workspace crate:

- `ecosystem/fret-node`

Feature policy:

- Default features include `fret-ui` integration and a convenience kit (`default = ["fret-ui", "kit"]`).
- A `headless` feature exists for non-UI usage (graph model + schema + rules only).
  `headless` must not depend on `crates/fret-ui`.
- A `kit` feature provides common profiles and recipes on top of the substrate. It is headless-safe
  and must not depend on `crates/fret-ui`.

Rationale:

- This is a policy-heavy editor component (ADR 0074), not a mechanism-only runtime feature.
- Default-on UI integration optimizes for Fret’s primary use case while keeping an escape hatch for
  CLI tools, server-side validation, and alternative front-ends.
- Keeping a kit inside the same crate avoids crate proliferation while still separating
  **mechanism-only substrate contracts** (XyFlow-like) from **optional convenience policies**.

### 2) Hard boundary: Model vs Rules vs UI

`fret-node` is organized as three conceptual layers (not necessarily three crates initially):

1. **Graph Model (data + ops)**: pure graph state, stable IDs, serialization, and undoable edits.
2. **Schema + Type/Constraint System**: node definitions, port/type descriptors, connection planning,
   and graph validation diagnostics.
3. **Editor UI** (optional via `fret-ui` feature): interaction state machine, rendering, input routing,
   and action hooks.

The UI layer must not encode domain semantics. All “can connect?” decisions and all automatic fixes
flow through the rules layer.

Additionally, within a single crate we distinguish:

- **Substrate (mechanism)**: stable contracts for graph editing (similar to XyFlow’s philosophy).
  This includes `core/`, `ops/`, `rules/`, `schema/`, and `profile/` *interfaces/pipeline*.
- **Kit (convenience)**: optional reusable policies and recipes (e.g. a permissive `DataflowProfile`,
  demo/utility node kinds). This lives under `kit/` and is feature-gated.

### 3) Graph primitives and stable IDs

The graph model defines:

- `Graph`: the whole document/asset.
- `GraphId`: stable graph document identifier used for cross-file references and editor-state lookup.
- `NodeId`, `PortId`, `EdgeId`: stable identifiers persisted in serialized assets.
- `Node`: `{ kind: NodeKindKey, pos, collapsed, data, ports }` (exact fields may evolve).
- `Port`: `{ node: NodeId, dir: In|Out, kind: Data|Exec, capacity, type_desc?, ui_hints? }`.
- `Edge`: `{ id, from: PortId, to: PortId, kind: EdgeKind }`.
- `Group` and `StickyNote` as first-class model elements (ShaderGraph parity).

ID policy:

- IDs are stable across sessions and independent of in-memory storage order.
- Copy/paste and graph merges must remap IDs deterministically (no collisions).
- The graph file includes a schema/version number to support migrations.
- Ports must not be identified only by positional indices; `PortId` stability is required to preserve
  connections across port reorderings and schema evolution.
- Serialization must be deterministic (stable ordering) to support diffs, reviews, and merges.

Concrete encoding (locked):

- `GraphId`, `NodeId`, `PortId`, `EdgeId`, and `SymbolId` are UUIDs (`uuid::Uuid`) serialized via serde as
  standard UUID strings.
- `NodeKindKey` is a stable, namespaced string identifier (same convention as `PanelKind` in docking):
  e.g. `core.math.add`, `core.flow.if`, `plugin.acme.http_request`.
- `PortKey` is a stable string identifier used for schema-declared ports (and for dynamic ports when needed).

Port key rules (locked):

- `PortKey` identity is stable across releases; do not reuse a removed `PortKey` for a different meaning.
- Renames are expressed via migrations (old `PortKey` -> new `PortKey` mapping).
- Dynamic/variadic ports must allocate stable keys that survive reorderings; the recommended strategy is
  to generate `PortKey` values as `dyn.<uuid>` and store them in the node instance data.

ID generation and remapping (locked):

- New IDs are generated with `Uuid::new_v4()`.
- ID remapping (copy/paste, duplicate, import/merge) must be deterministic with respect to input order:
  - build remap tables by iterating source IDs in a stable sort order (UUID bytes),
  - never rely on hash map iteration order when allocating new IDs.
- Remapping never changes referenced external identities (e.g. `GraphId` references for subgraphs).

Rationale:

- Mature editors treat graphs as long-lived assets (ShaderGraph).
- Stable IDs are prerequisite for migration, diffing, diagnostics references, and incremental caches.

### 4) Edge kinds and default connection policies

We explicitly model edge kinds from day one:

- `EdgeKind::Data` (typed value flow)
- `EdgeKind::Exec` (control flow; no value typing by default)

Default capacity policy (overridable per port via schema/rules):

- `DataIn`: single connection
- `DataOut`: multi connection
- `ExecIn`: single connection
- `ExecOut`: multi connection

Cycle policy is rule-driven:

- The model can represent cycles.
- Graph rules may forbid cycles by `EdgeKind` (typical: forbid `Data`, allow `Exec`), and must report
  diagnostics when violated.

Rationale:

- This avoids a later structural rewrite when introducing blueprint/workflow control flow.
- Different domains need different cycle and capacity rules; this must be a policy decision.

### 5) Schema registry and unknown-node survival

Node definitions are externalized via a registry:

- `NodeKindKey`: stable, namespaced string identifier.
- `NodeRegistry`: maps `NodeKindKey -> NodeSchema`.
- `NodeSchema` declares ports, default node data, UI metadata (title/category/keywords/icon), and
  dynamic port behavior.
- A stable per-node-kind `PortKey` exists for schema-declared ports so that migrations can rename,
  reorder, or regroup ports without breaking connections. (Instance ports can still carry generated
  `PortId`, but schema evolution needs a durable mapping.)

Unknown-node policy:

- Graph deserialization must succeed even when a `NodeKindKey` is missing from the registry.
- Unknown nodes are preserved as `UnknownNode` with raw payload retained so that saving does not
  destroy data.
- The UI must present unknown nodes in a survivable way (read-only or limited edits), and diagnostics
  must surface “missing node kind” errors.

Rationale:

- This is required for plugin ecosystems, cross-team projects, and forward compatibility.

Reserved builtin kinds (locked):

- `fret.reroute`: a schema-less "wire reroute" node used for routing and interaction affordances.
  - It may be inserted by editor gestures (split-edge) even when the domain registry does not
    contain a schema for it.
  - Its ports are implementation-defined by `fret-node` and must remain stable (one data in, one
    data out; no domain payload semantics).

Node kind versioning and migration (locked):

- Each node instance stores `{ kind: NodeKindKey, kind_version: u32, data: ... }`.
- Each `NodeSchema` declares `latest_kind_version: u32` and a pure-data migrator that can upgrade
  `data` (and port keys if needed) from older versions to the latest.
- A registry may declare `kind_aliases: Vec<NodeKindKey>` to support renames:
  - loading a node with an alias kind rewrites it to the canonical `NodeKindKey` via migration.
- When a migrator is missing, the node must remain survivable (UnknownNode behavior), and diagnostics
  must explain what is missing.

### 6) Typed connections via `TypeDesc` + `ConnectPlan`

We standardize on a runtime, serializable type descriptor:

- `TypeDesc`: a structured description (builtins + parameters + type variables + constraints).

`TypeDesc` MVP builtins (locked):

- Top/special:
  - `Any`: top type (“accepts anything”).
  - `Unknown`: inference placeholder (used during partial graphs / unresolved generics).
  - `Never` (optional): bottom type for “no value can exist” / unreachable.
  - `Null`: explicit null value (required for JSON/workflow domains).
- Scalars:
  - `Bool`, `Int`, `Float`, `String`, `Bytes`.
- Containers:
  - `List(T)`.
  - `Map(K, V)` (MVP may restrict `K = String`, but the descriptor supports general keys).
- Objects:
  - `Object { fields: BTreeMap<String, TypeDesc>, open: bool }`.
    - `open = true` means “this object may contain additional unknown fields”.
    - `open = false` means “closed record” with an exact field set.
- Unions:
  - `Union(Vec<TypeDesc>)`.
  - `Option(T)` is a syntax sugar for `Union([T, Null])`.
- Type variables:
  - `Var(TypeVarId)` with optional constraints (e.g. “must be numeric”, “must be JSON-like”).
- Extensions:
  - `Opaque { key: String, params: Vec<TypeDesc> }` for domain-specific types (shader, tools, schemas).

Rationale:

- Dataflow/workflow graphs require structural types (object/list/option/union) to avoid future rewrites.
- Shader/blueprint specializations must be able to add rich domains without expanding the core enum.

Connection is mediated through a planner:

- `plan_connect(graph, registry, from, to) -> ConnectPlan`

`ConnectPlan` includes:

- decision: `Accept | Reject`
- diagnostics: reasons and severity
- optional edits: a small `Vec<GraphOp>` representing auto-fixes, such as:
  - disconnecting prior edges when connecting into a single-capacity port,
  - inserting conversion nodes (cast, pack/unpack, enum mapping),
  - inserting reroute nodes.

Rationale:

- ShaderGraph-level maturity comes from “compatibility + concretization + diagnostics + autofix,” not
  from the UI alone.
- `ConnectPlan` aligns with the query/accept flow in `imgui-node-editor` while remaining rules-driven.

Conversion discovery and disambiguation (locked):

- The system must support “this connection is rejected, but could be made valid via explicit conversion”.
- To avoid embedding UI decisions into rules, conversion insertion is treated as a separate, UI-driven
  workflow:
  - `plan_connect` answers the direct-connect question and may include deterministic auto-fixes
    (e.g. disconnecting conflicting edges), but it must not silently change runtime semantics.
  - The presenter may additionally expose a set of conversion candidates via
    `NodeGraphPresenter::list_conversions(graph, from, to) -> Vec<InsertNodeTemplate>`.
  - The UI uses these candidates to:
    - show “convertible” affordances during hover/preview,
    - auto-insert a conversion node when there is exactly one unambiguous candidate (optional policy),
    - or open a conversion picker (screen-space overlay) when multiple candidates exist.
- Conversion candidates are inserted as explicit graph edits (ops), producing a concrete conversion
  node in the serialized graph. No implicit casts are applied.
- Presenter hooks control UX without changing semantics:
  - `NodeGraphPresenter::conversion_label(...) -> Arc<str>` defines the label shown in the picker/UI,
  - `NodeGraphPresenter::conversion_insert_position(...) -> CanvasPoint` defines where the conversion
    node is placed (e.g. midpoint between endpoints for ShaderGraph-like readability).

Type inference and conversion boundary (locked):

- `TypeDesc` is a data model, not a policy engine:
  - It does not embed implicit cast tables or “auto-convert” logic.
  - All compatibility decisions live in the rules layer and are surfaced through `ConnectPlan`.
- Unification/inference is rules-driven and profile-scoped:
  - A profile defines how `Var` is solved and how `Unknown` propagates.
  - A profile defines the “compatibility lattice” for `Any`, `Union`, and open objects.
- Auto-conversion is expressed as explicit edits:
  - If a connection requires a conversion, `ConnectPlan` returns `GraphOp` edits that insert a
    conversion node (or performs a safe rewrite), rather than silently changing runtime semantics.
- Deterministic shape:
  - Object fields use stable ordering (`BTreeMap`) to keep serialization and diffing deterministic.
  - `Union` normalization (dedup/sort/flatten) is performed in a deterministic way by the rules layer.

### 7) Graph validation and diagnostics are first-class

The rules layer exposes:

- `validate_graph(graph, registry) -> Vec<Diagnostic>`

Diagnostics:

- include stable references to `NodeId/PortId/EdgeId`,
- have severity (`Error|Warning|Info`) and a machine key for filtering/suppression,
- support “quick fix” actions expressed as `GraphOp` transactions where possible.

Rationale:

- Diagnostics are a core part of the editing experience and a foundation for future compilers/runtimes.

### 8) Undo/redo is op-based and transaction-friendly

All edits flow through `GraphOp`:

- `GraphOp` is the minimal reversible edit unit.
- Multi-step operations (dragging, paste, connect with autofix) are grouped into transactions and
  support coalescing (ADR 0024).
- Reconnection-friendly shape: endpoint moves that should preserve edge identity (and metadata like
  selection, inspection state, or per-edge UI) should be representable as a dedicated reversible op
  (e.g. `SetEdgeEndpoints`) rather than only as remove+add.

Rationale:

- Without op-level correctness, every advanced editor feature (autofix, paste, multi-select moves,
  batch edits) becomes brittle.

### 9) UI integration: retained canvas + action hooks

The `fret-ui` integration uses:

- `Widget::render_transform` for pan/zoom to keep layout bounds authoritative while transforming
  paint and hit-testing (ADR 0083).
- Action hooks for policy:
  - pointer down/move/up streams for drag interactions,
  - key handlers for shortcuts and command routing,
  - context menu triggers and node creation flows (ADR 0074).

Canvas coordinate escape hatch (locked):

- The editor must be able to render and interact with **screen-space** UI (context menus, typeahead
  search, tooltips, toasts) while the graph content is under a pan/zoom transform.
- This is the Fret equivalent of `imgui-node-editor`'s `Suspend/Resume`:
  - "Graph content" lives under the canvas transform.
  - "Overlays" are rendered outside that transform, using window/screen coordinates, but can be
    anchored to graph elements via explicit `canvas_to_screen` geometry conversions.
- This is the preferred way to implement “floating” editor affordances (conversion pickers, node
  searchers, tooltips) without requiring node content to be implemented as independent floating
  windows.

Rendering model (locked):

- The node graph editor is a single canvas widget embedded in panels/tabs (docking/multi-view), not a
  collection of native floating windows.
- The long-term target is that node content (header/body/ports) is authored as regular retained
  `fret-ui` subtrees provided by the presenter/viewer surface, and hosted by the canvas via a
  dedicated **Canvas Portal** (see below). The editor owns interaction and layout framing.
- MVP implementations may paint simple labels/diagnostics directly in the canvas without embedding
  a full subtree, but must not lock the API surface such that later portal-based composition
  requires a breaking refactor.
- Wires, background patterns, selection rectangles, and other canvas-level visuals are drawn by the
  editor widget using Fret’s renderer primitives (paths, strokes, fills) under the canvas transform.
- Screen-space popups/menus (including conversion pickers) use overlays rendered outside the canvas
  transform (see above), avoiding the need for a separate “floating window” UI subsystem.

Embedded node content: Canvas Portal (locked boundary, staged implementation):

- Motivation: editor graphs frequently need real widgets inside nodes (text input, sliders, toggles,
  images, previews). Re-implementing a parallel widget toolkit inside the node graph is not an
  acceptable long-term strategy.
- Requirement: node-embedded widgets must use the same focus, IME, and accessibility contracts as
  the rest of Fret UI (see ADR 0012, ADR 0067, ADR 0069).
- The canvas hosts embedded content using a portal mechanism, conceptually similar to “absolute
  layout + transform”:
  - the presenter provides an element subtree per node (and optionally per port row),
  - the canvas positions the subtree in screen-space, anchored to the node’s canvas geometry,
  - input events are routed through the normal UI tree, not via custom per-node event code.
- Measurement/geometry flow:
  - the portal host measures subtrees and emits **derived geometry hints** (node bounds, handle
    bounds, anchor bounds) into the internals store, keyed by stable IDs (`NodeId`, `PortId`),
  - the canvas uses those hints for wire routing and hit-testing without depending on a specific
    layout engine (taffy or otherwise).
- Staging:
  - Stage 1 (MVP): text-only labels/hints via `NodeGraphPresenter::node_body_label` and optional size
    hints; no embedded widgets.
  - Stage 2: portal host for node header/body subtrees (enables IME-backed renames and constants).
  - Stage 3: optional per-port-row subtrees and richer semantics (inline controls in pin rows).

Portal command routing (locked):

- Embedded widgets must not mutate graph state directly. They emit commands and the host/controller
  decides whether to commit a `GraphTransaction`, reject, or surface inline errors.
- The portal host maintains UI-only editor session state keyed by view context (at least `(window,
  portal_root_name, node_id)`), e.g. the current text buffer and last error message.
- Standard command shapes (subject to revision, but should remain stable once shipped):
  - `fret_node.portal.submit_text:<node_uuid>`
  - `fret_node.portal.cancel_text:<node_uuid>`
  - `fret_node.portal.step_text:<node_uuid>:<delta>:<mode>` where `mode` ∈ `fine|normal|coarse`.
- Modifier policy (recommended default):
  - Shift → `coarse`
  - Ctrl/Cmd → `fine`
  - otherwise → `normal`
- Modifier lock (recommended default):
  - For drag-style continuous edits (sliders, number drags), the modifier-derived mode should be
    captured on pointer down and remain stable for the duration of the drag session to avoid
    mid-drag discontinuities.
- Submit semantics:
  - Parse/validate in the domain spec.
  - On success, emit a `GraphTransaction` commit so undo/redo works uniformly (ADR 0024).
  - On failure, keep the buffer and show an inline error string (UI-only state).
- Drag semantics (recommended default):
  - Use a small movement threshold (e.g. 1–3 px) before starting a value drag to avoid accidental
    edits on click.
  - During drag, update only the widget buffer (preview). Commit once on pointer up to keep undo
    granularity sane.

Interaction protocol target (inspired by `imgui-node-editor`):

- The widget produces “pending requests” (e.g. `PendingConnect`, `PendingDelete`).
- The host/controller applies `ConnectPlan`/`GraphOp` transactions and feeds the updated graph back.

Node creation protocol:

- “Create Node” is a command/palette surface (ADR 0023) that can be invoked from:
  - keyboard (e.g. Space) without hard-coding keys at the widget layer (ADR 0018),
  - background context menu,
  - dropping a wire on empty space (wire-drop menu / searcher).
- When invoked from a wire-drop, the creation surface is provided a context:
  - source pins (one or many; multi-connect),
  - desired direction (creating a node to connect to an input vs output),
  - a type/edge-kind filter derived from `ConnectPlan` so the search results are relevant.
- The output of the creation surface is a transaction:
  - add node,
  - position node near drop point,
  - connect selected source pins to chosen node ports (with `ConnectPlan` autofix where applicable).
- Cancel semantics:
  - A wire drag can be canceled without opening any menu (e.g. secondary button while dragging),
    to avoid accidental menu opens and to match common editor UX.

Baseline UI capabilities (MVP parity targets):

- selection (single/multi, rectangle selection),
- pan/zoom + “frame selection” navigation,
- link creation/deletion with accept/reject feedback,
- node moving with transaction coalescing,
- context menus (background/node/port/edge/group),
- copy/paste of subgraphs (ADR 0041 alignment).

Planned advanced interaction features (parity with `egui-snarl` / editor expectations):

- multi-connect gestures (bundle connect, yank-reconnect),
- edge reconnection (yank and reattach one endpoint while preserving edge identity when possible),
- reroute nodes and wire hit-testing with large-graph performance constraints,
- node collapse/expand and group dragging,
- movement constraints: graph-wide translate extents, per-node extents, and optional “expand parent”
  behaviors for frame-like parent nodes (ReactFlow parity, future extension),
- deterministic draw order (z-order) and explicit “bring to front” interactions,
- wire styling hooks for “execution flow” visualization (animated markers / highlight),
- configurable wire layer (render behind nodes vs above nodes),
- configurable background patterns (grid/dots/custom),
- persisted editor view state (camera, selection, fold states) in a separate editor-state file or
  per-asset metadata, without mixing with core graph semantics.

Node layout contract:

- Nodes have standard regions (header, inputs, body, outputs, footer) to ensure consistent styling,
  hit-testing, and extensibility (inspired by `egui-snarl`).
- Port placement is configurable (inside frame / on edge / outside) as a style/UX choice.
- Wire hit-testing uses a larger, configurable interaction width independent of the visual stroke
  (touch-friendly, ReactFlow parity).

Port geometry contract (handles vs measurement):

- Port anchors are a UI concept used for wire routing and hit-testing.
- Implementations must support two sources of truth for port anchors:
  1) **Measured anchors**: derived from the rendered node subtree (cached per frame / invalidated on resize),
  2) **Declared anchors**: provided by schema/presenter for non-DOM backends or highly optimized nodes.
- The UI is allowed to cache per-node “handle bounds” or anchor maps in editor view state to avoid
  per-frame recomputation (ReactFlow’s `internals.handleBounds` pattern), but these caches must be
  treated as derived data and never as graph semantics.
- `fret-node` provides an explicit derived-geometry output (`CanvasGeometry`) for the canvas:
  - per-node rects in canvas space (`node_rect`),
  - per-port handle bounds and centers (`handle_bounds` / `port_center`).
  This output is UI-only, depends on style + zoom + node layout, and must never be serialized into the
  graph asset (it may be cached in editor view state as derived internals).

Derived geometry and internals (locked):

- The editor must maintain a clear separation between **user-authored graph state** and **derived UI internals**.
  Inspired by ReactFlow / XyFlow's "internal node" model, the following fields are considered derived:
  - measured node size (`measured.width/height`),
  - absolute/cached node position in the current parent/extent context (`positionAbsolute`),
  - handle/port bounds (`handleBounds` / anchor rects),
  - z-index / stacking hints derived from selection policy.
- Derived internals:
  - may be cached for performance,
  - must be invalidated deterministically (node resize, zoom changes, node template changes, port layout changes),
  - must not be serialized into the graph asset.
- API clarity constraint:
  - avoid overloading `Node.width/height` or other user-facing fields with measured values;
    measured geometry must have its own namespace in editor state to prevent "who owns size?" confusion.

Minimap and overview navigation (locked):

- The editor provides an optional minimap/overview surface (ReactFlow parity) that is:
  - purely a view over derived geometry (node rects + viewport rect),
  - not serialized into the graph asset (view state only),
  - implemented as a separate widget/overlay that consumes `CanvasGeometry`/internals stores.
- Minimap interactions (drag viewport, click-to-pan) must produce the same canonical navigation ops as
  normal canvas navigation:
  - set pan/zoom,
  - frame a rect (selection / all nodes),
  - fit view (optional command surface).
- The minimap must not require a "floating window" subsystem:
  it is rendered as an overlay outside the canvas transform using the coordinate escape hatch.

Connection modes and handle resolution (locked):

- We standardize a `connection_mode` concept (ReactFlow parity):
  - `Strict`: connections can only be created when the pointer is over a concrete compatible handle/port.
  - `Loose`: connections may "snap" to a compatible handle within a radius, even if the pointer is not
    exactly over the handle (useful for dense graphs / touch).
- Handle resolution is a UI concern but must be deterministic:
  - The UI chooses a candidate `(from_port, to_port)` pair based on the pointer position, connection radius,
    and `connection_mode`, and then delegates final acceptance to the rules layer via `ConnectPlan`.
  - When multiple handles are within range, the UI must use a deterministic tie-breaker (closest distance,
    then stable port ordering) to avoid flicker.
- The UI exposes tunables in editor view state (not graph semantics):
  - `connection_radius` (for loose mode),
  - `edge_interaction_width` (wire hit slop independent from stroke thickness),
  - `reconnect_radius` (see below),
  - `auto_pan` parameters for drag/connect.

Reconnection protocol and anchors (locked):

- Edge reconnection is a first-class workflow:
  - dragging from an existing edge endpoint or from a dedicated reconnection handle triggers a
    reconnection interaction,
  - the rules layer decides via `plan_reconnect_edge` (preserving `EdgeId` when possible).
- Edges may be configured as reconnectable or not (ReactFlow parity):
  - reconnectability is a UI policy flag and may be controlled per edge kind or per edge instance.
- Custom reconnection anchors are supported:
  - an edge may expose one or more "reconnect anchors" (small hit targets) that start a reconnection drag,
    similar to ReactFlow's `EdgeReconnectAnchor` concept for custom edges,
  - anchors must have a configurable hit radius (`reconnect_radius`) independent of wire stroke thickness.

Parent/child subflows and movement extents (locked):

- The editor must support a parent/child relationship between nodes (ReactFlow parity: `parentId`):
  - a child node's `pos` is interpreted as **relative to its parent**,
  - a node's derived absolute position is computed from the parent chain + per-node origin rules
    (derived internals; see "Derived geometry and internals").
- Parent/child is a **layout/interaction contract**:
  - dragging a parent moves its children as a group,
  - selection, hit-testing, and z-order must behave deterministically when parents overlap children.
- Movement extents must support both global and per-node constraints (ReactFlow parity: `nodeExtent`):
  - graph-wide extent ("global node extent") clamps nodes from leaving a defined canvas region,
  - per-node extent may override the global extent for that node.
- Parent extent modes:
  - a child may declare its extent as `"parent"` (clamp within the parent bounds),
  - alternatively, a child may opt into `"expand_parent"` behavior: moving/resizing the child can
    expand the parent bounds rather than clamping the child.
- Deterministic clamping during multi-select drag:
  - when dragging multiple nodes under a global extent, clamping must be applied based on the
    bounding box of the dragged set, not per-node independently, to avoid jitter and drift
    (matches ReactFlow's multi-drag extent adjustments).
- These constraints are **editor interaction policy**, but their inputs must be persistable:
  - parent/child relationships are part of the graph document (asset semantics),
  - extents may be stored as graph semantics (for graphs that require it) or as profile policy defaults.

Auto-pan, snapping, and drag thresholds (locked):

- The editor supports snapping to a grid for node move and resize interactions (ReactFlow parity):
  - `snap_to_grid: bool`
  - `snap_grid: (x, y)`
  - snapping must be applied consistently in all coordinate conversion helpers (screen <-> canvas)
    and in all interactive edits (drag, paste, align tools).
- Drag threshold must be stable under zoom:
  - the minimal pointer movement before a drag starts is measured in screen pixels and must not
    implicitly scale with zoom (matches XyFlow's `nodeDragThreshold` fixes).
- Auto-pan is supported for editor-grade UX:
  - auto-pan while dragging nodes near viewport edges,
  - auto-pan while connecting/reconnecting edges near viewport edges,
  - optional auto-pan when focusing a node via keyboard navigation.
- Auto-pan tunables are part of editor view state (not graph semantics):
  - enable flags per workflow (node drag / connect / focus),
  - speed and edge margin thresholds.

Resizable nodes and node origins (locked):

- The editor supports optional node resize interactions (XyFlow parity: `NodeResizer`) for node kinds
  that opt into it (frames/comments, domain nodes with variable UI, etc.).
- We explicitly separate three size concepts:
  - **persisted size** (user intent / graph semantics): stored only when a node kind declares it,
  - **measured size** (derived internals): computed from the rendered node subtree each frame or via
    cached measurement,
  - **minimum size** (policy): derived from style + port layout + node content constraints.
- Resize edits are undoable and expressed as deterministic transactions (ops or domain-owned payload
  updates). Resizing must not mutate derived internals directly.
- We standardize a `node_origin` concept (XyFlow parity: `nodeOrigin`):
  - node `pos` is interpreted as the canvas position of an origin point inside the node rect,
  - default is top-left (`[0, 0]`), but profiles/components may choose other origins,
  - origin affects selection/hit-testing, fit-view framing, parent/child extents, and resizer math.

Node presentation contract (Viewer-style):

- The node graph widget does not own domain UI. Instead, a presenter/viewer surface is provided
  (conceptually similar to `egui-snarl`’s `SnarlViewer`) to render:
  - node title / header / body / footer content,
  - per-port content (labels, inline editors, icons),
  - per-port visuals (pin shape, pin rect/handle size, wire color/style),
  - context menus (graph background, node, port, edge),
  - optional on-hover popups (node/pin/edge),
  - optional wire widgets (small UI at or near the wire midpoint).
- The presenter may hold extra, non-serialized UI data (e.g. cached measurements, inline editor state),
  but graph semantics must remain in the serialized `Graph` model.

Zoom and UI scaling policy (locked):

- Viewport zoom is applied via render transforms (ADR 0083) to keep input and paint consistent.
- The node editor supports a style policy that can choose how different primitives scale with zoom:
  - wire thickness / arrow markers,
  - pin size and hit target padding,
  - background pattern density and line thickness,
  - optional “clamped” scaling for readability (e.g. do not let text/pins become too small).
- A “semantic zoom” option is supported via deterministic LOD rules:
  - hide or simplify node body content below a threshold,
  - replace rich pin content with compact labels/icons,
  - keep hit-testing aligned with what is actually rendered.

### 10) Graph profiles (domain specializations without rewrites)

We standardize on an explicit “profile” concept:

- `GraphProfile` selects a registry subset (allowed node kinds) and a ruleset (type/constraints).

Examples:

- `DataflowProfile`: permissive types, optional cycles, no exec edges (initial MVP).
- `BlueprintProfile`: exec edges + control-flow constraints, optional data typing rules.
- `ShaderProfile`: strict data types, forbid data cycles, shader-stage constraints, precision/space rules.

Rationale:

- Profiles prevent the “one graph tries to satisfy every domain simultaneously” failure mode.
- Profiles provide a stable extension seam for future shader/blueprint/workflow projects.

### 11) Extension data hooks (graph- and node-scoped)

We reserve extension hooks similar to ShaderGraph’s graph sub-data pattern:

- Graph-scoped extension data for future needs (blackboard-like inputs, compiler settings, domain
  metadata), keyed by a stable `ExtKey`.
- Node-scoped extension data for plugin nodes that need to attach auxiliary metadata without forcing
  a core schema change.

These extensions must be serializable and survive unknown-kind scenarios.

### 12) Large-graph performance: caches are derived, not serialized

The model layer may maintain derived indexes (e.g. per-node adjacency lists, per-port edge lookups,
spatial indexes for hit-testing), but:

- derived caches are rebuildable from the serialized canonical state,
- caches are not part of the stable asset format unless explicitly versioned and justified.

The UI layer additionally maintains **editor internals** (derived geometry) similar to ReactFlow/XyFlow:

- node rectangles (in canvas and screen space),
- port handle bounds and anchor points (screen px),
- edge routing samples and edge hit-testing slop,
- spatial indexes (grid/R-tree) to keep hit-testing and selection rectangles fast on 10k+ graphs.

Hard boundary (locked):

- Internals are **never** serialized into the graph asset.
- Internals may be persisted as editor-state only when they are user intent (e.g. node z-order),
  not when they are derived measurement (e.g. handle bounds).

Invalidation contract (locked):

- Derived geometry caches must be invalidated by a small set of monotonic revision keys:
  - graph model revision,
  - view state revision (camera/zoom/draw order),
  - presenter geometry revision (custom anchor hints / sizing heuristics),
  - optional measured geometry revision (host-provided layout measurements).

Measured geometry injection (locked):

- Hosts may feed measured node sizes and handle bounds into the canvas without mutating the graph model.
- Measured values are expressed in screen-space logical pixels (px), consistent with the style system.
- The injection surface is a small, thread-safe store (`MeasuredGeometryStore`) + a presenter wrapper
  (`MeasuredNodeGraphPresenter`) that consults the store before delegating to the inner presenter.

Rationale:

- This mirrors XyFlow’s separation of user-authored node data from derived internals (`handleBounds`),
  and prevents graph assets from accumulating render/layout-specific noise.

### 13) Graph symbols (blackboard/variables) are first-class

Many graph domains require “graph-scoped inputs”:

- shader properties/keywords (ShaderGraph),
- blueprint variables,
- workflow parameters and environment inputs (Dify-like).

We standardize on a symbol concept:

- `SymbolId`: stable per-symbol ID.
- `Symbol`: `{ id, name, type_desc, default_value?, category?, metadata? }`.

Rules:

- Symbols live at graph scope and are referenced by nodes via `SymbolId` (not by name).
- Symbols are serializable and participate in migrations (rename, type changes).
- The UI supports a “blackboard” surface:
  - add/remove/rename/reorder symbols,
  - drag a symbol into the graph to create a reference node at a location,
  - copy/paste across graphs copies symbols into the destination graph when needed.

Reserved node kind (baseline contract):

- `fret.symbol_ref`: a built-in "symbol reference" node.
  - `Node.data` must be an object with a `symbol_id` string (UUID).
  - The referenced `symbol_id` must exist in `Graph.symbols`.

Implementation note:

- Symbols are a **built-in** graph section (not an optional extension), because they are required
  across multiple graph domains and must interoperate with copy/paste and node creation flows.

### 14) Dynamic ports and concretization

Some node kinds have ports that depend on node data or graph context:

- variadic nodes (N inputs),
- polymorphic nodes (ports change type after connections),
- “function” nodes where ports are derived from a referenced signature,
- subgraph nodes whose port list mirrors the subgraph interface.

We require a concretization protocol:

- A node kind may declare ports as dynamic and provide a deterministic “concretize” routine.
- Concretization produces a sequence of `GraphOp`s (add/remove/remap ports, retag types) so it is:
  - undoable,
  - serializable as part of edits,
  - testable.
- Port identity must remain stable across concretization:
  - schema-level `PortKey` is used to preserve connections when ports are reordered/renamed,
- when a port is removed, connected edges must be dropped explicitly with diagnostics.

Concretization scheduling (locked):

- Concretization runs as part of the edit pipeline, not as an ad-hoc UI side effect.
- Each user action produces a single committed transaction:
  1) apply direct `GraphOp`s,
  2) run concretization and validation incrementally to a fixed point (bounded),
  3) apply derived `GraphOp`s (port changes, edge drops, inserts) within the same transaction.
- If fixed-point resolution does not converge within a small bound, the transaction is rejected with
  diagnostics (to prevent infinite oscillation between rules).
- UI must render from the post-concretized state to avoid one-frame “wrong ports then snap” jitter.

### 15) Editor state persistence is separate from graph semantics

We separate:

- **graph asset state** (the canonical graph document), and
- **editor view state** (per-user/per-machine).

Editor view state includes:

- camera (pan/zoom), visible rect,
- selection and focused element (optional),
- per-node UI state (collapsed/open, z-order),
- custom zoom levels or input scheme overrides (mouse buttons / smooth zoom).
- optional interaction settings: snap grid, selection mode (partial vs full), connection mode
  (strict vs loose), and auto-pan tuning for drag/connect.

Multi-view (docking) integration:

- Multiple node-graph canvases may view the same `GraphId` simultaneously (split views, multiple tabs).
- Each canvas instance must maintain its own in-memory `NodeGraphViewState` so camera/selection can
  differ per view. The persistence layer must not assume a single canonical view state per graph.
- Persisted state MAY store:
  - a single `"state"` (the “last active view”), and/or
  - a `"views"` map keyed by a host-provided stable view key (e.g. docking panel instance id),
    allowing restoring multiple views when reopening a workspace.

Persistence:

- Editor state is stored outside the graph asset by default and keyed by `GraphId`:
  - project scope (recommended): `./.fret/node_graph/view_state/<graph_id>.json`
  - user scope (optional): OS config directory per ADR 0014.
- The persistence API supports save reasons / dirty flags (navigation/selection/position/etc.) to
  allow throttling and reduce churn (inspired by `imgui-node-editor`).
- `fret-node` provides optional IO helpers for the default JSON shape:
  - `NodeGraphViewStateFileV1` and `default_project_view_state_path` in `ecosystem/fret-node/src/io/mod.rs`.

On-disk shape (locked, v1):

- JSON file stored at one of the default locations above.
- Wrapper object to allow future evolution without breaking older files:
  - `{ "graph_id": "<uuid>", "state_version": 1, "state": { ... } }`
- Backward compatibility: loaders may also accept a plain `state` root object when `graph_id` is
  supplied out-of-band by the caller (mirrors `DockLayoutFileV1`’s wrapper tolerance).

View-state schema (locked, v1):

- All fields are optional unless stated otherwise; missing fields must default to the behavior
  described below.
- Unknown fields must be ignored to allow forward-compatible additions.
- `state.pan`: `{ "x": f32, "y": f32 }` (canvas units).
- `state.zoom`: `f32` (default `1.0`).
- `state.selected_nodes`: `Vec<NodeId>` (optional).
- `state.selected_edges`: `Vec<EdgeId>` (optional).
- `state.draw_order`: `Vec<NodeId>` (optional).
- `state.interaction` (optional): editor interaction tuning and policy overrides (per-user/per-project):
  - `connection_mode`: `"strict" | "loose"` (default `"strict"`).
  - `connection_radius`: `f32` (screen px; only used for `"loose"`; default `16`).
  - `reconnect_radius`: `f32` (screen px; default `10`).
  - `edge_interaction_width`: `f32` (screen px; default `12`).
  - `snap_to_grid`: `bool` (default `false`).
  - `snap_grid`: `{ "width": f32, "height": f32 }` (canvas units; default `16x16`).
  - `node_drag_threshold`: `f32` (screen px; default `1`).
  - `auto_pan` (optional):
    - `on_node_drag`: `bool` (default `true`).
    - `on_connect`: `bool` (default `true`).
    - `on_node_focus`: `bool` (default `false`).
    - `speed`: `f32` (screen px/s; default `900`).
    - `margin`: `f32` (screen px; default `24`).

Graph asset on-disk shape (locked, v1):

- Canonical format is JSON via `serde_json` (aligns with docking persistence patterns).
- Wrapper object is used to allow schema evolution:
  - `{ "graph_id": "<uuid>", "graph_version": 1, "graph": { ... } }`.
- Backward compatibility: loaders may also accept a plain `graph` root object (legacy format) and
  derive `graph_id` from the embedded `GraphId` field.

### 16) Selection is an integration surface (Inspector and commands)

Selection is editor-owned policy but must be exposed as data:

- The node graph widget reports selection changes as explicit events or model updates.
- Selection can feed an inspector/property panel via the editor-layer protocol (ADR 0048), enabling
  node/edge properties to be edited without coupling the UI widget to domain logic.
- Core editing commands are command IDs (ADR 0023), not hard-coded key handlers:
  - `node_graph.undo`
  - `node_graph.redo`
  - `node_graph.open_insert_node` (background picker / palette)
  - `node_graph.open_split_edge_insert_node` (edge “insert node” picker)
  - `node_graph.insert_reroute`
  - `node_graph.open_conversion_picker` (re-open last conversion candidates)
  - `node_graph.copy`
  - `node_graph.cut`
  - `node_graph.paste`
  - `node_graph.delete_selection`
  - `node_graph.duplicate`
  - `node_graph.frame_selection`
  - `node_graph.select_all`
  - `node_graph.nudge_left|right|up|down` (repeatable)
  - `node_graph.nudge_left_fast|right_fast|up_fast|down_fast` (repeatable)
  - `node_graph.align_left|right|top|bottom|center_x|center_y`
  - `node_graph.distribute_x|distribute_y`
  - `node_graph.focus_next|focus_prev`
  - `node_graph.focus_next_edge|focus_prev_edge`
  - `node_graph.focus_next_port|focus_prev_port`
  - `node_graph.focus_port_left|right|up|down` (spatial port focus)
  - `node_graph.activate` (keyboard click-connect)

Semantics baseline:

- The canvas provides a minimal semantics node (`SemanticsRole::Viewport`) so assistive tech can identify the editor surface.
- The presenter may override accessible labels via `NodeGraphPresenter::{a11y_canvas_label,a11y_node_label,a11y_port_label,a11y_edge_label}`.
- The editor may optionally mount semantics-only child nodes so the canvas can set `active_descendant` to a real semantics node (e.g. `NodeGraphA11yFocusedPort|Edge|Node`).

### 17) Clipboard and drag payloads have stable formats

The node graph editor must support:

- copy/paste of a subgraph fragment (nodes + edges + groups + notes + referenced symbols),
- internal drag payloads for moving nodes and creating connections,
- compatibility with cross-window internal drag sessions (ADR 0041) when the graph is in a torn-off window.

Clipboard format rules:

- A graph fragment is a deterministic, self-contained serialization that can be pasted into another graph.
- Pasting remaps `NodeId/PortId/EdgeId` to fresh IDs, and remaps `SymbolId` by copy-in when required.

### 18) Graph references (subgraphs) are explicit and cycle-safe

Reusable subgraphs are a core scaling mechanism (shader subgraphs, workflow subflows, blueprint macros).

Contract:

- A node kind may reference another graph document via `GraphId` (or asset GUID when an asset system exists).
- Referenced graph interfaces are treated as dynamic ports and must be concretized deterministically.
- Recursive dependencies must be detected and surfaced as diagnostics, and profiles may forbid them.
- Copy/paste of subgraph nodes preserves the reference (it does not inline by default).

### 19) Collaboration readiness: deterministic diffs and patchability

Even in single-user editors, graphs end up in Git and require workable diffs/merges.

We require:

- Deterministic serialization shape:
  - stable ID types (already),
  - stable map ordering for canonical formats (e.g. `BTreeMap` in the model),
  - stable normalization for derived structures (e.g. unions, port ordering).
- A stable patch unit for integration:
  - `GraphTransaction` is the canonical reversible patch unit and should be usable for:
    - local undo/redo,
    - scripted refactors/migrations,
    - future collaborative edit streams (CRDT/OT is explicitly out of scope for v1).
- Diff hygiene:
  - derived internals are never serialized into the graph asset,
  - editor view state is stored separately and can be excluded from VCS if desired.

### 20) Editor interaction contract checklist (implementation-oriented)

This section is intentionally implementation-oriented and exists to prevent "death by missing small
details" when we claim parity with mature editors (XyFlow / ImGui Node Editor / ShaderGraph / Snarl).
It is a checklist of **hard-to-change interaction contracts**. We expect to implement these
incrementally, but we want to lock the semantics early to avoid later rewrites.

MVP (v1) must provide:

- **Two-phase connect handshake** (ImGui `BeginCreate/QueryNewLink/AcceptNewItem` mental model):
  - during drag: expose a stable "in-progress connection" preview (from, to, candidate, validity),
  - on release: emit a single "commit decision point" that either applies a `GraphTransaction` or
    cancels without side effects.
- **Reconnect as first-class** (XyFlow `reconnectEdge` mental model):
  - reconnection is distinct from new connection creation,
  - reconnection preserves `EdgeId` when the domain allows it (see rules `ConnectPlan`),
  - reconnection supports both “single edge” and “yank many” flows (Snarl multi-connection).
- **Connection modes** (XyFlow `ConnectionMode`):
  - `Strict`: source->target only,
  - `Loose`: allow same-side connections, resolved by closest compatible port within a configurable
    screen-space radius.
- **Edge interaction width** (XyFlow `interactionWidth`):
  - edge hit-testing must use a separate "interaction width" (screen-space) independent of visual
    stroke width to keep selection usable at all zoom levels.
- **Auto-pan while connecting**:
  - while dragging a wire near viewport edges, pan the canvas (speed and threshold are style knobs).
- **Viewport helpers**:
  - `fit_view` / `frame_selection` must exist and be deterministic given derived geometry.
- **Derived geometry is authoritative for hit-testing**:
  - node bounds, port anchors, and edge paths must come from the measured geometry store or
    presenter-provided fallback, never from ad-hoc per-frame layout guesses.

Soon (parity targets we should design for now):

- **Parent/child (subflow) constraints** (XyFlow `parentId` + `nodeExtent` mental model):
  - define whether we model this as `Group` bounds, a "frame node kind", or both,
  - enforce movement extents and prevent overlap across unrelated parents.
- **Node resizing** (XyFlow `NodeResizer` mental model):
  - decide where "explicit size" lives (graph vs extension vs view-state) and how it composes with
    parent extents and snapping.
- **Edge markers and routing policies**:
  - arrowheads/markers must not be an afterthought (ties into execution graphs),
  - routing must be swappable (bezier, step, orthogonal, custom).
- **Minimap / overview navigation**:
  - consumes derived geometry and writes only view-state.
- **Z-order policy**:
  - define how edges layer relative to nodes and parent frames (XyFlow elevates some edges above
    parents; Fret must define this for group/subflow parity).

Notes:

- Items above are contracts; the *visual* design (theme tokens) is intentionally orthogonal.
- Features like "conversion node insertion" and "domain-specific auto nodes" remain profile-level
  policy and must not leak into the substrate.

### 21) Groups (container frames) and explicit node size are graph semantics

We standardize the "subflow / parent container" concept early (XyFlow `parentId` mental model):

- `Graph.groups` stores group frames as `Group { rect: CanvasRect, ... }` in **canvas space**.
- `Node.parent: Option<GroupId>` assigns a node to a container frame.
- Node positions remain **absolute canvas positions** (`Node.pos` is not made relative to the parent).
  - Moving a group in UI should translate its child nodes explicitly (policy), rather than relying on
    a relative-position encoding in the core model.

Group removal semantics (locked):

- Removing a group must detach its child nodes (`Node.parent = None`) as part of the same reversible
  edit transaction.
- `GraphOp::RemoveGroup` carries a `detached` list so undo/redo can restore the original parents.

Node explicit size semantics (locked):

- `Node.size: Option<CanvasSize>` is stored in the graph asset (not view-state).
- `Node.size` is interpreted as a **semantic size in logical px at zoom=1**.
  - Geometry conversion divides by `zoom` so the node remains readable under semantic zoom.
  - When `None`, the editor derives size from measured geometry or style defaults.
- The node resize interaction (NodeResizer) writes `Node.size` via `GraphOp::SetNodeSize`.

## Consequences

Pros:

- Strongly reduces the chance of later structural rewrites when adding shader/blueprint/workflow
  specializations (the model is edge-kind aware; rules are pluggable).
- Enables “mature editor” behaviors early: diagnostics, unknown node survival, migrations, op-based
  undo/redo, and rules-driven connect planning.
- Keeps `crates/fret-ui` stable by using action hooks and a component-layer policy architecture.

Cons:

- Requires upfront design effort (registry, diagnostics, ops, versioning) before “pretty UI” work.
- Runtime typing is validated by rules and tests, not by Rust’s compile-time generics.

## Alternatives Considered

1) Put the node graph editor into `crates/fret-ui`:
   rejected (violates ADR 0074; the editor is policy-heavy).

2) Make the graph type-safe via Rust generics (compile-time typing):
   rejected as the primary model (pluginability, unknown-node survival, and migrations become hard).

3) Bake a single domain type system (shader slots) into core:
   rejected (we need cross-domain reuse).

## Appendix: Upstream parity map (non-normative)

This appendix is intentionally implementation-oriented: it helps keep naming and responsibilities
aligned with upstream mental models while preserving Fret’s own layering.

XyFlow (`@xyflow/system`) concept map:

- Pan/zoom (`XYPanZoom`) → `NodeGraphCanvasTransform` + view-state stored pan/zoom.
- Dragging (`XYDrag`) → node/selection drag interactions in the canvas state machine.
- Handles/connections (`XYHandle`) → port anchor resolution + connection/reconnection flows.
- Connection mode (`ConnectionMode`) → `NodeGraphConnectionMode` (strict/loose).
- Edge hit slop (`EdgeBase.interactionWidth`) → `NodeGraphStyle::wire_interaction_width` (screen-space).
- Edge updates (`reconnectEdge`) → `ConnectPlan::Reconnect` + `GraphTransaction` apply.
- Derived internals (`internals.handleBounds`, `positionAbsolute`, `measured`) → `CanvasGeometry` /
  `MeasuredGeometryStore` / `NodeGraphInternalsStore` (output-only, never serialized into the graph asset).
- Minimap (`XYMiniMap`) → optional overlay widget consuming derived geometry (see “Minimap and overview navigation”).
- Change sets (`NodeChange` / `EdgeChange`) → `GraphOp` + `GraphTransaction` as the canonical reversible patch unit.

ImGui Node Editor concept map:

- “Draw your content, we do the rest” → presenter/viewer surface: the canvas owns interaction; the
  caller owns node/pin/wire UI.
- `Suspend/Resume` → canvas coordinate escape hatch for screen-space overlays (menus/searchers).

egui-snarl concept map:

- Typed node parameter (`Snarl<T>`) → data-only `Graph` + typed connections via `TypeDesc` (the node
  data itself remains domain-owned extension data).
- Viewer (`SnarlViewer<T>`) → `NodeGraphPresenter`/`NodeGraphProfile` split:
  - presenter provides UI-facing descriptions (titles, port rows, inline content),
  - profile/rules provide validation, compatibility, and connection policy.
- Five spaces (header/inputs/body/outputs/footer) → standardized node regions in the presenter API
  to keep layout, hit-testing, and styling stable across domains.
- “User controlled responses for wire connections” → connection/reconnection is always mediated by
  `ConnectPlan` + `GraphTransaction`, never by direct edge mutation from the UI.
- Multiconnections → canvas interaction supports bundled connect/reconnect flows (view policy; graph
  semantics remain single-edge records).

Unity ShaderGraph concept map:

- Graph validation pipeline → `GraphProfile::validate_graph` + `Diagnostic` reporting.
- Concretization (dynamic slots) → `GraphProfile::concretize` executed in the apply pipeline.
- Unknown node survival + migrations → schema versioning + explicit migrations (see earlier sections).

## Open Questions

- Whether to standardize on a single on-disk format (JSON/RON) for node assets, or support multiple
  formats behind a stable serde model.
- Whether to support additional “compat read” formats (e.g. RON) while keeping JSON as canonical.
- Whether node graph assets should additionally carry an asset-database GUID alongside `GraphId` (ADR 0026).
- Whether to lock a stable "view key" contract for multi-view persistence (dock panel id, explicit `ViewId`,
  or per-asset “named views”), and how to migrate between them when layouts change.
- Whether node resizing is a first-class interaction (ReactFlow/XyFlow `NodeResizer` parity), and if so:
  - where explicit sizes live (graph vs extension data vs editor-state),
  - how it composes with parent/child extents and group frames.
- Whether to expose a viewer hook to adjust the canvas transform (inspired by `egui-snarl`’s
  `SnarlViewer::current_transform`) to support “UI scaling” modes where text remains readable at extreme zoom.
- How to connect `crates/fret-ui`’s post-layout measurement (node bounds / handle bounds) to the measured-geometry
  injection surface without introducing frame-order hazards or accidental coupling to a specific layout engine.
- Whether edge hit-testing should be strictly "interaction-width only", or additionally allow a
  per-edge override (XyFlow allows per-edge `interactionWidth`).
- Whether to standardize a portal-based “node view” presenter API now (element subtrees + measured
  geometry), or to keep MVP text-only rendering longer and accept a larger later migration.
- How the portal host composes with semantic zoom and “UI scaling” modes (text remains readable at
  extreme zoom) without breaking input hit-testing and cursor positions.
