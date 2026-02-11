# ADR 0127: Undo/Redo Infrastructure Boundary (Framework Hooks vs App Policy)

Status: Proposed
Scope: Framework-level infrastructure (portable); undo history policy remains app-owned.

## Context

Several existing Fret surfaces assume “editor-grade” interaction:

- Docking operations are expressed as explicit ops (ADR 0013).
- Viewport tools require interaction phases suitable for undo coalescing (ADR 0049).
- The node graph editor is op-based and transaction-friendly (ADR 0126).
- Inspector/property editing is designed to emit begin/update/commit phases (ADR 0048).

ADR 0024 intentionally scoped undo/redo as **example editor application** work (Deferred), but we
still want to lock the **framework-level contracts** that make undo/redo feasible without forcing
every app to invent incompatible plumbing.

This ADR answers a narrow question:

> What should the Fret framework provide to enable robust undo/redo across complex interactive
> tools (node graphs, viewport drags, inspector edits) while keeping history policy app-owned?

## Goals

1. Keep undo/redo **policy and storage app-owned** (ADR 0027), while providing stable integration
   hooks so advanced tools do not require later refactors.
2. Support editor workflows that require:
   - explicit reversible operations,
   - explicit transaction boundaries (begin/commit/cancel),
   - coalescing during continuous interaction (drag/slider scrub).
3. Integrate cleanly with Fret’s command routing and focus model (ADR 0020).

## Non-goals

- A “one true” undo stack for all domains (scene, assets, node graph, text) shipped inside `fret-ui`.
- Persisting undo history to disk.
- Cross-process or OS-level undo integration.
- A framework-mandated history topology (linear vs branching). Linear history is assumed for v1,
  but implementations may choose to support branching.

## Terminology

- **Edit**: a user-visible change that should be undoable (may span multiple models/domains).
- **Transaction**: a grouping boundary for edits, with lifecycle `Begin → (Update)* → Commit|Cancel`.
- **Undo entry**: one history item created by a committed transaction.
- **Coalescing**: combining multiple intermediate updates into a single undo entry.
- **Document**: an app-defined unit of history (e.g. graph document, scene, asset). A window may
  host multiple documents across panels/tabs.

## Decision

### 1) Undo history is not a `fret-ui` responsibility

The UI runtime (`fret-ui`) must not own a global undo stack. It may implement **local undo** for
internal widget state (e.g. text input buffers), but cross-domain undo history remains app-owned.

Rationale:

- Undo/redo policy is inherently domain-specific (scene vs assets vs graph vs layout).
- Persisting identity, diffs, and “dirty” rules are editor/engine responsibilities (ADR 0027).
- Keeping this out of `fret-ui` avoids backend leakage and makes embedding (ADR 0052) easier.

### 2) The framework provides *hooks*, not policy

The framework should provide a small, portable infrastructure surface that apps can adopt:

#### a) Transaction boundary vocabulary

We standardize the concept of an edit transaction:

- `Begin` (start an interaction that should become one undo entry),
- `Update` (optional; intermediate updates during the interaction),
- `Commit` (finalize and push a single history entry),
- `Cancel` (revert to the pre-begin state and push nothing).

This vocabulary must remain compatible with:

- pointer capture lifetimes (ADR 0020),
- viewport tool phases (ADR 0049),
- inspector begin/update/commit patterns (ADR 0048),
- node graph drag coalescing (ADR 0126).

#### b) Coalescing keys (continuous interaction)

The infrastructure must support coalescing for “continuous” edits:

- A drag/move/scrub is one entry even if state updates happen every frame.
- Coalescing is keyed by app-defined data, typically:
  - tool kind + stable target id (e.g. `NodeId`, `EntityId`, `PropertyPath`).

The framework does not define the key schema; it only requires that the infrastructure can accept
an optional `coalesce_key` and apply “last-wins within a transaction” semantics.

Cancellation semantics:

- `Cancel` must revert the user-visible state to the pre-`Begin` baseline.
- The framework does not prescribe how cancel is implemented, but practical implementations require
  capturing “before” state at `Begin` (snapshot/patch seed), or staging changes in a preview layer.

Transaction metadata (recommended, not mandatory):

- Transactions may carry optional metadata for UX and policy:
  - user-facing label/title (for menu/history UI),
  - significance (affects “dirty” tracking or not),
  - document identity (which history stack to target),
  - tool identity (for analytics/debugging).

#### c) Command routing integration

Undo/redo is invoked via commands routed through the existing focus model (ADR 0020):

- The framework recommends providing commands that can be handled by the focused surface:
  - Example: a node graph widget can handle `node_graph.undo` / `node_graph.redo` (ADR 0126).
  - Example: a text input widget can handle `text.undo` / `text.redo` (local history).
- A window-level fallback may handle “document undo” when focus is not inside a specialized widget.

This ADR does not lock a single global command id; it only requires that command routing supports:

- focused-surface undo (widget scope),
- window/document undo (window scope),
- app-wide undo (app scope) only where an app explicitly chooses it.

Command naming guidance (non-normative, recommended):

- Reserve `edit.undo` / `edit.redo` for window/document-level undo targets.
- Use domain-specific namespaces for specialized surfaces:
  - `node_graph.undo` / `node_graph.redo` (ADR 0126),
  - `text.undo` / `text.redo` for text editing surfaces (local history),
  - tool-specific commands may exist but should generally delegate to one of the above.

The key requirement is not the string id, but that the focus/router can resolve whether undo/redo
targets the focused surface’s local history or the active document history.

#### d) Integration with model observation and invalidation (ADR 0051)

Undo/redo is just another model mutation. Therefore:

- Undo/redo apply must run through the same app-owned model update mechanisms (ADR 0031).
- Resulting model changes must participate in the same “changed ids → invalidate observers” path
  across all windows (ADR 0051), so undo reliably refreshes all dependent panels.

This ADR does not require special-case invalidation behavior for undo; it requires that undo uses
the same mutation pathways as normal edits.

#### e) Correctness invariants (framework-facing)

To avoid late rewrites, undo infrastructure should follow a small set of invariants:

- **Atomicity at the user level**: a committed transaction corresponds to exactly one undo entry.
  Intermediate `Update`s must not leak as separate history items.
- **Redo clearing**: applying a new committed edit after an undo should clear redo history (linear
  history baseline). If an app chooses branching history, it must define explicit UI for it; the
  framework does not assume branching.
- **Deterministic identity**: operations and coalescing keys must use stable ids (not UI pointers or
  transient indices) (aligned with ADR 0024, ADR 0126, ADR 0048).
- **Begin/Commit pairing**: a `Commit` without a corresponding active `Begin` is a logic error; a
  `Begin` must eventually `Commit` or `Cancel` (even if routed through error handling).
- **Cancel is lossless**: cancel must restore the exact pre-begin baseline for the affected domain
  state, not “best effort”.
- **No re-entrancy requirements**: undo application should run through normal event/tick boundaries
  and must not require re-entrant UI rebuilding inside model update closures (aligned with ADR 0020
  and ADR 0031).

#### f) Multi-window and multi-document behavior (recommended)

- A document history should be scoped to an app-defined document identity, not to a specific
  window. Multiple windows/panels may edit the same document.
- Undo/redo availability (enabled/disabled) should be derived from the active document + focused
  surface, and reflected consistently in menus/palette.
- Because undo/redo can change many models at once, apps should assume broad invalidation and rely
  on ADR 0051 observation propagation rather than manual “refresh all panels” glue.

### 3) Recommended implementation placement

The default undo infrastructure should live in portable app-runtime crates, not in UI/runtime:

- A default `UndoService` implementation may live in `fret-app` (integrated apps), or in an
  ecosystem crate for reuse across editor projects.
- Portable “hook types” (transaction meta, coalesce key, handle IDs) may live in `fret-runtime` if
  needed for host-generic embedding (ADR 0052).

#### Implementation status (2026-01-10)

This ADR intentionally does not *require* a single canonical implementation. However, a reusable
reference substrate now exists for editor apps:

- Ecosystem crate: `ecosystem/fret-undo` (`UndoHistory`, `UndoService`, `UndoRecord`, `CoalesceKey`).
- Value-based invertible transactions: `ValueTx<T>` implements `InvertibleTransaction`.
- Example wiring: `apps/fret-examples/src/gizmo3d_demo.rs` routes gizmo commits through
  `UndoService<ValueTx<Transform3d>>` and applies `edit.undo/edit.redo` at the window/document level.

Known gaps (not yet implemented by the framework):

- Focus-chain-based undo target resolution (ADR 0020) that prefers widget-local undo targets before
  falling back to the window's active document history.
- A standardized cross-model transaction composition story (multi-model edits, batching, grouping).

### 4) Targeting and scoping (multiple histories)

Most editor apps require multiple undo targets:

- a document-scoped history (e.g. the active graph/scene),
- specialized local histories (e.g. text fields),
- sometimes a global app history (rare; typically discouraged).

The framework does not mandate a topology, but it should ensure the routing story is stable:

- focused-surface undo should win when the focused surface declares a local undo target,
- otherwise, fall back to the active document history for the window,
- apps may optionally provide an app/global fallback.

This is analogous to existing command scoping (ADR 0020).

Practical interpretation of “scope” (what users typically mean):

- “Undo only works in this region/panel” usually means that region owns a **local history** (e.g.
  a text field, a node-graph canvas with its own document).
- “Undo works for the active document in the window” means a **window-level document target** is
  selected based on focus/active panel.
- “Undo is global” means an app chooses to provide a global fallback (uncommon for editor-grade
  apps because it makes intent ambiguous).

Recommended resolution algorithm (non-normative, but strongly suggested for consistency):

1. Determine the focused widget chain (focused node + bubbling chain) per ADR 0020.
2. If any widget in that chain declares an undo target, route to the closest such target:
   - e.g. text input local history, node-graph document history.
3. Otherwise route to the window’s active document history (app-defined “active document”).
4. Otherwise (optional) route to an app-global history.

UI enable/disable and menu labels:

- Whether undo/redo is enabled should be computed from the same target resolution (not from ad-hoc
  `cfg` or platform branches).
- Menus/palette may display target-specific labels (e.g. “Undo Move Nodes”, “Undo Set X”) by asking
  the resolved target for its next undo/redo entry metadata.

Note (Zed reference, non-normative):

- Zed distinguishes multiple undo-like actions in its editor layer (e.g. “undo selection” vs
  general undo) via separate commands (`editor::UndoSelection` / `editor::RedoSelection` in
  `repo-ref/zed`). This reinforces the need for explicit target resolution rather than assuming a
  single global history.
- Zed’s text buffers implement explicit transactions with time-based grouping/coalescing and
  transaction composition/merging (e.g. `group_interval`, `push_empty_transaction`,
  `merge_transactions`) (`repo-ref/zed/crates/text/src/text.rs`). This is a concrete example of
  domain-local history that stays outside the UI runtime while still requiring stable begin/end
  boundaries and coalescing hooks.

### 5) Patterns from existing editors (examples)

These examples motivate the hooks above; they are non-normative.

#### Unreal Engine

- Uses `FScopedTransaction` to define explicit begin/commit boundaries (RAII style).
- Objects participate by recording “before” state (`Modify()`), enabling stable undo across many
  domains while keeping policy app-owned.

#### Unity

- Uses grouped undo records (`Undo.RecordObject`, group id + collapse) so continuous drags coalesce
  into a single history entry.
- Many tools mutate state continuously but are committed as one undo step.

#### Godot

- Uses an `UndoRedo` manager where each action registers do/undo callbacks and is committed as a
  unit (`create_action` / `commit_action`), matching the transaction vocabulary directly.

#### Blender / DCC tools

- “Modal operators” preview changes during drag but only push an undo step on confirm; cancel
  reverts to the initial state.

#### Figma / document editors

- Edits are expressed as data patches applied to a document model; transactions are committed on
  interaction boundaries; coalescing keys avoid creating a new entry per frame.

### 6) How this satisfies Fret’s editor surfaces

#### Docking

- Docking already has explicit `DockOp` operations (ADR 0013). These are undo-friendly by design.
- The undo infrastructure can treat a dock move/close as a transaction entry without requiring UI
  runtime changes.

#### Viewport tools

- Viewport tools should define begin/update/commit/cancel around capture lifetimes (ADR 0049),
  producing one undo entry per user gesture.

#### Node graph

- Node graph edits are already op-based (`GraphOp`) (ADR 0126).
- A drag of N nodes can be represented as either:
  - one transaction with one op that stores before+after positions, or
  - a transaction with multiple ops that coalesce by `(tool, node_id)` into a single entry.

#### Inspector/property editing

- Numeric “scrub” edits naturally map to begin/update/commit (ADR 0048).
- Coalescing keys should include a stable property path, not UI row indices.

### 7) Implementation sketch (non-normative)

This section is intentionally concrete to reduce “everyone reinvents it differently” drift.

#### a) Continuous drag (node move, gizmo, slider scrub)

1. On pointer down (capture begin): start a transaction:
   - capture “before” state (or a reversible seed op),
   - set `coalesce_key = (tool_kind, target_id)` (or a broader key for multi-select).
2. On pointer move (capture update): apply preview updates to the document model.
   - do not push history entries.
3. On capture end:
   - commit: store an undo entry representing (before → after) and clear redo.
   - cancel: restore “before” and store nothing.

#### b) Multi-model transaction (typical editor reality)

A single gesture often updates multiple models:

- node position changes (graph model),
- selection/hover changes (selection model),
- dirty flags (project/document model).

Recommendation:

- Only user-visible document edits belong in the undo entry payload.
- Selection/hover changes are typically *not* undoable and should not be recorded unless an app
  explicitly wants “selection undo”.
- Dirty tracking is derived from the document model or from explicit transaction metadata.

#### c) Undo routing (local vs document)

When `edit.undo` is invoked:

1. If the focused surface declares a local undo target (e.g. text input), it handles undo locally.
2. Otherwise, the window resolves the active document history and performs document undo.
3. App-level undo is optional and should be explicit.

The important property is that command routing follows the same scope precedence as ADR 0020.

### 8) Observability requirements (recommended)

Undo/redo issues are notoriously hard to debug. A default undo infrastructure should expose:

- the active transaction state (begun? pending? coalesce key?),
- the top-of-stack entry label + document target,
- whether redo history is present/cleared after new edits.

This should integrate with existing observability/inspector hooks (ADR 0036), but the exact UI is
app/editor-owned.

## Consequences

Pros:

- Locks the minimal contracts needed for editor-grade undo/redo without forcing policy into `fret-ui`.
- Aligns with existing ADRs that already assume transactions/coalescing in tools.
- Works with multi-window and docking because identity and ops remain data-first.

Cons:

- Apps must still implement their own history stacks and domain ops.
- Without a default `UndoService`, different apps may diverge; an ecosystem “golden” implementation
  is still recommended.

## References

- Deferred editor-level undo: `docs/adr/0024-undo-redo-and-edit-transactions.md`
- Framework scope boundary: `docs/adr/0027-framework-scope-and-responsibilities.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- App-owned models: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- Observation + invalidation: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
- Docking ops as data: `docs/adr/0013-docking-ops-and-persistence.md`
- Viewport tool phases: `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`
- Inspector protocol: `docs/adr/0048-inspector-property-protocol-and-editor-registry.md`
- Node graph editor: `docs/adr/0126-node-graph-editor-and-typed-connections.md`
