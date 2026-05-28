# `fret-node` Declarative Contract Closure v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

This workstream is closed. FNDC-010 through FNDC-070 are complete. Current retained guidance is
binding/controller/declarative-first, `NodeGraphStore` dispatch shares one internal commit path for
store-profile and external-profile transaction dispatch, binding graph/view/config app model handles
are documented as store-derived projections, declarative input interception has a store-first
key-down hook contract, and paint-only orchestration has one pure interaction frame plan extracted
from host-bound frame assembly.

## Last Completed Task

- Task ID: FNDC-070
- Owner: codex
- Files:
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/DESIGN.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/TODO.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/MILESTONES.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/HANDOFF.md`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
  - `docs/workstreams/fret-node-declarative-contract-closure-v1/CLOSEOUT_AUDIT_2026-05-28.md`
- Validation:
  - `cargo fmt --check`
  - `cargo nextest run -p fret-node --no-default-features`
  - `cargo nextest run -p fret-node`
  - `cargo nextest run -p fret-canvas`
  - `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  - `git diff --check`
- Status: DONE
- Review: DONE
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Opened a new lane instead of reopening closed retained or architecture lanes.
- Kept broad ReactFlow hook facade and full semantic focus tree work out of the first task.
- Marked ADR 0135 superseded instead of preserving obsolete retained middleware as current policy.
- Added a source-policy test so standalone/ADR drift cannot silently reintroduce current retained
  `NodeGraphCanvas` guidance.
- Collapsed store-profile and external-profile dispatch into one private dispatch implementation
  while preserving public `NodeGraphStore` methods and error types.
- Added an external-profile dispatch regression test covering middleware order, committed patch
  shape, history visibility, and store event publication.
- Reclassified binding graph/view/editor-config model handles from mirrors to store-derived
  projections for observation and advanced sync.
- Added a regression test proving projection-model graph edits do not mutate the authoritative
  `NodeGraphStore` graph and are overwritten by `sync_from_store`.
- Updated public node graph guides to keep mutations flowing through binding helpers,
  `NodeGraphController`, or `NodeGraphStore`.
- Added `NodeGraphDeclarativeInteractionHook` as the declarative replacement seam for retained
  canvas middleware, with key-down capture as the first implemented hook point.
- Kept the hook context store-first: it exposes snapshots and binding/controller helper methods, not
  raw graph/model-store mutation.
- Updated ADR 0135 and public node graph guides to describe the declarative hook replacement path.
- Extracted `plan_paint_only_interaction_frame` so per-frame transient interaction paint/semantics
  decisions are pure snapshot derivation before host-bound cache and internals sync.
- Kept the extraction inside `fret-node`; the helper is node-graph-specific and does not yet justify
  a `fret-canvas` move.
- Closed the lane with fresh `fret-node`, `fret-canvas`, format, layering, catalog, JSON, and diff
  gates.

## Blockers

- None.

## Follow-ons

- Add pointer, command, and observer hook methods to `NodeGraphDeclarativeInteractionHook` when a
  concrete editor workflow proves the need.
- Start a separate ReactFlow/XyFlow facade lane if app authors need a broader `useReactFlow`-style
  hook bundle.
- Start a semantic focus/a11y lane for nodes, ports, minimap, and controls if richer keyboard
  navigation or screen-reader evidence requires it.
- Split a cache/scene-plan adapter lane only when a domain-neutral helper has a non-node consumer or
  a clear `fret-canvas` contract.
