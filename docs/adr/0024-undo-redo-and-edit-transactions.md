# ADR 0024: Undo/Redo and Edit Transactions

Status: Deferred
Scope: Example editor application (out of scope for the Fret UI framework)

Note: this ADR remains the “editor application” direction. Framework-level *hooks* that enable
undo/redo without committing to history policy are tracked separately:

- `docs/adr/0127-undo-redo-infrastructure-boundary.md`

## Context

A game engine editor must support reliable undo/redo across many domains:

- scene edits (create/delete/move entities, component changes),
- asset operations (rename/move, import settings),
- viewport tool interactions (dragging gizmos, snapping),
- UI layout operations (docking changes, tab moves).

If undo/redo is bolted on later, the codebase usually suffers large rewrites because state changes
are not expressed as explicit, reversible operations.

This ADR is intentionally **not** a Fret framework commitment. Fret should provide the UI
infrastructure that makes undo/redo *possible* (commands, focus routing, docking ops as data),
but the undo/redo history and policies belong to the editor application / engine layer.

References:

- Docking operations as transactions:
  - `docs/adr/0013-docking-ops-and-persistence.md`
- App-owned models and “lease” update pattern (design inspiration):
  - https://zed.dev/blog/gpui-ownership

## Decision

### 1) All user-visible edits are represented as explicit operations

Any action that should be undoable is represented as an explicit “edit operation” (conceptually `EditOp`):

- apply: mutates app/models/engine state
- undo: reverses the mutation

Operations are data-first (serializable where feasible) and do not store direct references into UI trees.

### 2) Transactions define coalescing boundaries

Edits are grouped into transactions:

- a single click action → one transaction
- a drag (gizmo move) → one transaction that may coalesce many intermediate updates

Transaction boundaries are explicit:

- begin on input capture start
- commit on capture end
- cancel on escape / tool abort

### 3) Multi-domain edits are supported

Transactions may span:

- app models (editor state),
- engine scene state (entities/components),
- asset database state.

This enables “move entity + update selection + mark dirty” to undo as a single unit.

### 4) Determinism and identity are required

Undo/redo requires stable identities:

- scene entities/components must have stable IDs (engine responsibility),
- editor-side objects use stable handles (`Model<T>`, `PanelKind`, asset GUIDs).

Operations store IDs, not pointers.

## Consequences

- Undo/redo becomes a core capability rather than a fragile patch.
- Complex editor tools can be implemented without inventing bespoke “revert” logic.
- Docking persistence and docking edits can share the same transaction machinery.

## Future Work

- Define the concrete `EditOp` trait and storage strategy (stack, branching history, per-document histories).
- Define “dirty state” rules (when a project is considered modified).
- Decide how to serialize edit history (likely not persisted in v1, but model should allow it).
