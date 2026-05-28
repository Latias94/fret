# ADR 0135: Node Graph Canvas Middleware (Tx Gate + Input Interception)

Status: Superseded
Scope: Ecosystem (`ecosystem/fret-node`) UI integration contract and guidance.

Status note (2026-05-28): this ADR is superseded by the retained node graph canvas exit and the
store-first/declarative-first surface. The retained `NodeGraphCanvas` and `NodeGraphCanvasMiddleware`
names below describe deleted historical design direction, not a current API. Current graph commit
interception lives in `NodeGraphStore` middleware/callbacks; any future UI interaction interception
must be designed as a declarative surface hook under
`docs/workstreams/fret-node-declarative-contract-closure-v1/`.

## Context

Historically, `ecosystem/fret-node` provided a retained `NodeGraphCanvas` widget that hosted a large
interaction surface (selection, panning/zooming, dragging, context menus/searchers, etc.). That
widget has since been deleted. The supported app-facing surface is now
`NodeGraphSurfaceBinding` + `node_graph_surface(...)` + `NodeGraphController`.

As the editor surface grows, we need a **single, non-bypassable gate** for all graph edits, and a
clean extension point for tool-mode and shortcut interception, without pushing editor policy into
`crates/fret-ui`.

## Goals

1. **Single tx gate**: any edit transaction produced by any entry point must pass the same
   validation/normalization path before it is applied and recorded for undo/redo.
2. **Tool interception**: allow optional tool-mode / shortcut logic to intercept events and
   commands without forking the canvas widget.
3. **Keep integration optional**: application-specific integrations (stores, overlays, docking,
   minimaps, etc.) must not become hard dependencies of the core middleware surface.
4. **Future compatibility**: align with the canvas guidance in ADR 0128 without introducing a
   second rendering subsystem.

Non-goals:

- Define a universal "canvas middleware" for all Fret canvases.
- Encode graph/domain rules inside `fret-canvas` or `crates/fret-ui`.

## Supersession Decision

Do not implement this ADR by reviving `NodeGraphCanvasMiddleware`.

The replacement direction is:

- Keep graph commit validation, normalization, history, and callbacks under `NodeGraphStore`.
- Keep declarative UI event/command interception behind a new explicit hook contract.
- Require any UI hook that commits graph edits to go through the store dispatch path.
- Treat retained middleware references below as historical design context only.

## Replacement Contract: Declarative Interaction Hooks

The supported replacement for retained canvas input interception is
`NodeGraphDeclarativeInteractionHook` on `NodeGraphSurfaceProps`.

The first shipped hook point is key-down capture for the declarative surface. This is intentionally
small: it covers tool-mode and shortcut interception without reopening retained widget authoring or
adding a second graph owner.

Hooks receive a `NodeGraphDeclarativeInteractionContext`, not a mutable `Graph` and not raw
`ModelStore` access. The context may expose:

- immutable graph/view snapshots for planning,
- binding/controller helpers that dispatch `GraphTransaction` through `NodeGraphStore`,
- view-state replacement helpers that preserve binding projection sync,
- focus/redraw/notify side-effect helpers for the current surface target.

The context must not expose `&mut Graph`, graph projection model mutation, or a retained
`NodeGraphCanvasMiddleware` chain. Any hook that wants to commit graph edits must submit a
`GraphTransaction` through the binding/controller/store dispatch path, so validation,
normalization, history, callbacks, and store events remain centralized.

Pointer, command, and observer hooks can be added later under the same contract. They should extend
`NodeGraphDeclarativeInteractionHook` rather than introduce another middleware family.

## Historical Decision (Superseded)

### 1) Introduce a `NodeGraphCanvasMiddleware` extension point (ecosystem)

`ecosystem/fret-node` may expose a middleware trait that can be composed as a chain:

- `handle_event(...) -> NotHandled | Handled`
- `handle_command(...) -> NotHandled | Handled`
- `before_commit(...) -> Continue | Reject { diagnostics }`

The middleware is **policy-light**: it may decide to intercept, rewrite, or reject, but it does
not own the graph model.

Reference implementation (wired into the canvas):

- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs` (`Widget::event`, `Widget::command`, `commit_transaction`)
- `ecosystem/fret-node/src/ui/canvas/widget/tests/middleware_conformance.rs`

### 2) A is the core: `before_commit` is the single edit gate

All graph edits that become history must pass `before_commit`:

- keyboard/menu command paths
- pointer-driven drags
- clipboard/paste helpers
- queued edits (if the canvas drains an edit queue)

Important constraint:

- `before_commit` should gate **new edits**. Undo/redo replays previously committed transactions and
  should not be re-gated, otherwise history can become invalid after rule upgrades.

### 3) B is the UI extension point: event/command interception is allowed but constrained

`handle_event` / `handle_command` are allowed to:

- intercept tool-mode input and dispatch commands,
- request redraw/invalidation,
- initiate/commit transactions by calling existing canvas helpers (preferred).

They should not:

- become the primary location for cross-model synchronization,
- depend on app-specific services directly (see next section).

### 4) C is not core: app integrations are an "add-on" layer

Application integration concerns (stores, overlays, docking bridges, minimaps, etc.) must not be
required by the middleware core interface.

Preferred pattern:

- keep the core middleware ctx minimal (graph, view state, bounds/pan/zoom, style),
- provide optional "adapter" middleware implementations in higher-level modules (e.g. `kit`) that
  close over app-owned models.

## Recommended first normalization rule (P0)

Implement one conservative, high-leverage rule first:

1. **Drop no-op ops**: remove any op whose `from == to` (e.g. `SetNodePos`, `SetNodeSize`,
   `SetEdgeKind`, etc.).
2. **Coalesce repeated setters (optional follow-up)**: within a single transaction, if the same
   `(op_kind, id)` appears multiple times for simple setters, merge into one op:
   - `from` taken from the first op
   - `to` taken from the last op

This improves determinism and reduces history noise without changing graph semantics.

## Relationship to `fret-canvas`

`fret-canvas` (ecosystem) is expected to host **canvas-generic** mechanisms (view transforms,
coordinate mapping, pixel policies, generic drag phases), but it must not absorb node-graph-specific
transaction semantics.

Node graph middleware remains domain-specific:

- `before_commit` is node-graph-only
- tool interception may produce graph transactions or view-state changes

## Open Questions

1. Do we need an `after_commit` hook for app-side side effects, or should that remain entirely
   outside the middleware surface (callbacks/subscriptions)?
2. Should the chain semantics support "always-run" observers (telemetry) distinct from interceptors?

## References

- Canvas guidance: `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- Undo/redo direction: `docs/adr/0024-undo-redo-and-edit-transactions.md`
