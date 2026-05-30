# `fret-node` Declarative Contract Closure v1 - Closeout Audit

Status: Closed
Date: 2026-05-28

## Result

This lane is complete. It closed the remaining store-first/declarative-first contract risks after
retained node graph authoring was removed:

- stale retained `NodeGraphCanvas` guidance is superseded and source-policy guarded,
- `NodeGraphStore` dispatch now uses one internal commit path,
- binding graph/view/config app models are named and documented as store-derived projections,
- declarative interaction interception has a key-down hook contract that cannot receive `&mut Graph`
  or raw model-store access,
- paint-only frame assembly has a pure interaction frame-plan extraction.

## Fresh Closeout Gates

- PASS `cargo fmt --check`
- PASS `cargo nextest run -p fret-node --no-default-features` (135 tests)
- PASS `cargo nextest run -p fret-node` (451 tests)
- PASS `cargo nextest run -p fret-canvas` (72 tests)
- PASS `python3 -m json.tool docs/workstreams/fret-node-declarative-contract-closure-v1/WORKSTREAM.json`
- PASS `python3 tools/check_layering.py`
- PASS `python3 tools/check_workstream_catalog.py`
- PASS `git diff --check`

## Follow-ons

- Add pointer, command, or observer methods to `NodeGraphDeclarativeInteractionHook` only when a
  concrete editor workflow proves the need.
- Open a separate ReactFlow/XyFlow facade lane if authors need a broader `useReactFlow`-style hook
  bundle.
- Open a semantic focus/a11y lane for nodes, ports, minimap, and controls if richer keyboard or
  screen-reader behavior needs it.
- Split deeper cache/scene-plan extraction only when a domain-neutral helper has a non-node consumer
  or a clear `fret-canvas` contract.

## Residual Risk

- The shipped hook contract currently proves key-down interception only; pointer and command hooks
  remain intentionally deferred.
- The paint-only frame-plan extraction is node-graph-specific. Moving it to `fret-canvas` would be
  premature without another consumer.
