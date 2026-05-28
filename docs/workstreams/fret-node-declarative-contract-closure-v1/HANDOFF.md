# `fret-node` Declarative Contract Closure v1 - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

The workstream has been opened as a follow-on to the completed architecture, runtime-store, and
retained-exit lanes. FNDC-020, FNDC-030, and FNDC-040 are complete. Current retained guidance is
binding/controller/declarative-first, `NodeGraphStore` dispatch now shares one internal commit path
for store-profile and external-profile transaction dispatch, and binding graph/view/config app model
handles are documented as store-derived projections instead of a second authority.

## Last Completed Task

- Task ID: FNDC-040
- Owner: codex
- Files:
  - `ecosystem/fret-node/src/ui/binding.rs`
  - `ecosystem/fret-node/src/ui/binding_store_sync.rs`
  - `ecosystem/fret-node/src/ui/binding_viewport.rs`
  - `ecosystem/fret-node/src/ui/controller.rs`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-controlled-mode.md`
- Validation:
  - `cargo nextest run -p fret-node view_projection_model graph_projection_model_is_not_the_authoritative_store_graph binding_surface controller_surface public_node_graph_guides`
  - `cargo nextest run -p fret-node`
  - `cargo fmt --check`
  - `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
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

## Blockers

- None currently.

## Next Recommended Action

- Start FNDC-050: replace the obsolete retained `NodeGraphCanvasMiddleware` direction with a
  declarative interaction hook contract that cannot bypass store commits.
