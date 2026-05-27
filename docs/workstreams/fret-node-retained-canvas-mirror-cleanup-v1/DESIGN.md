# `fret-node` Retained Canvas Mirror Cleanup (v1)

Status: active
Last updated: 2026-05-27

## Why This Lane Exists

The runtime/store contract lane closed the correctness hazards that made UI mirror cleanup risky.
Its closeout deliberately split retained `NodeGraphCanvas` mirror cleanup into this follow-on lane
because retained canvas compatibility has a separate review and test surface.

## Relevant Authority

- ADRs:
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
  - `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- Existing docs:
  - `docs/node-graph-roadmap.md`
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1/CLOSEOUT_AUDIT_2026-05-27.md`
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1/UI_MIRROR_INVENTORY_2026-05-26.md`
- Related workstreams:
  - `docs/workstreams/fret-node-runtime-store-contract-closure-v1/`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`

## Problem

Retained `NodeGraphCanvasWith` still stores external graph, view-state, and editor-config model
mirrors as top-level widget fields. That makes the retained compatibility island look like another
authoritative state owner even though the store/controller path is now the safer source of truth.

## Target State

When this lane closes:

- retained canvas graph/view/editor-config mirrors are explicitly owned by a private compatibility
  mirror container,
- new retained canvas code has to cross a named mirror boundary before reading or updating external
  models,
- store-backed retained canvas sync keeps compatibility behavior while making drift risk visible,
- source-policy coverage prevents reintroducing top-level retained mirror fields,
- retained compatibility gates prove the cleanup did not regress existing retained canvas behavior.

## In Scope

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/*.rs`
- retained canvas model sync helpers under `ecosystem/fret-node/src/ui/canvas/widget/view_state/`
- focused retained canvas tests and source-policy tests
- this workstream's evidence and closeout docs

## Out Of Scope

- Removing retained `NodeGraphCanvas` public compatibility constructors.
- Changing the declarative-first public recommendation.
- Reopening the closed runtime/store contract lane.
- Fixing unrelated `fret-ui` clippy findings.
- Renaming public Cargo features.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Retained canvas mirrors are still needed for compatibility during this lane. | High | Existing retained canvas constructors and tests pass explicit graph/view/editor-config models. | The lane can delete more than planned, but only after proving compatibility gates. |
| A private mirror owner is a safe first slice. | High | `NodeGraphSurfaceBinding` already uses the same quarantine pattern. | If retained code relies on direct fields semantically, the first slice must add narrower access helpers. |
| Store-backed retained sync should remain intact while mirrors shrink. | High | Runtime/store closeout requires retained compatibility to stay covered. | A failing compat gate means the cleanup must stop before deletion. |

## Architecture Direction

Treat retained `NodeGraphCanvas` as a compatibility island. The authoritative runtime document is
the store when a controller/store is attached; external graph/view/editor-config models are retained
mirrors used for compatibility and tests. The first slice should make that ownership explicit before
attempting deletion.

## Closeout Condition

This lane can close when:

- retained canvas mirror ownership is explicit and source-policy guarded,
- at least one mirror path is deleted or quarantined with tests,
- retained compatibility and feature-matrix gates pass,
- remaining retained work is split into a follow-on or documented as intentionally deferred.
