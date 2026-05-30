# `fret-node` Retained Exit And Parity Seams (v1) - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

The lane is closed after completing four follow-on fearless refactors:

1. delete the retained canvas compatibility island,
2. clean old retained `NodeGraphCanvas` tutorial/API docs,
3. extract one more generic canvas mechanism to `fret-canvas`,
4. close one bounded XyFlow hook/focus parity seam.

## Next Task

Start a separate follow-on only if the next scope is full semantic focus nodes, port-focus authoring,
minimap/controls focus, or a broader ReactFlow hook facade. Do not reopen this lane for that work.

The retained feature/source island is deleted, public node-graph docs now describe the supported
binding/controller/declarative surface, generic resize handle vocabulary now lives in `fret-canvas`,
and `disableKeyboardA11y` now gates the declarative active-descendant/a11y internals path with
focused conformance coverage.

## Continuation Notes

- The prior `fret-node-architecture-fearless-refactor-v2` lane is closed and should not be reopened.
- The prior retained mirror cleanup lane is closed and explicitly says broader retained-surface
  removal needs a separate lane.
- If retained code cannot be deleted cleanly, replace only the behavior that matters with supported
  seam tests; do not reintroduce retained public authoring.
- `NRP-020` removed `compat-retained-canvas`, `ui/compat_transport.rs`, feature-gated retained
  canvas modules, and old retained source-policy shape tests. Do not add a replacement feature gate
  unless a new ADR/workstream explicitly reintroduces a low-level adapter.
- `NRP-030` removed public `NodeGraphCanvas` tutorial/API guidance from node-graph docs and rewrote
  ADR alignment rows around `NodeGraphSurfaceBinding`, `node_graph_surface(...)`,
  `NodeGraphController`, and declarative paint-only cache paths.
- `NRP-040` moved generic resize handle vocabulary into
  `fret_canvas::interaction::{ResizeHandle2D, ResizeHandleSet2D}` and left `fret-node` with
  node-named aliases only.
- `NRP-050` landed the XyFlow focus seam on the supported declarative surface:
  `NodeGraphInteractionState.disable_keyboard_a11y` suppresses active-descendant semantics at
  `sync_binding_internals_for_surface` without exposing retained widget contexts. Broader semantic
  focus nodes, port-focus authoring, and minimap/controls focus remain follow-on parity gaps.

## Last Known Gates

- `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
- `python3 tools/check_layering.py`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 130 tests after retained exit.
- `cargo nextest run -p fret-node`: passed, 442 tests after retained exit.
- `cargo nextest run -p fret-node --no-default-features public_node_graph_guides`: passed.
- `! rg -n "NodeGraphCanvas::|compat-retained-canvas|retained canvas" docs/node-graph*.md ecosystem/fret-node/README.md`: passed.
- `cargo nextest run -p fret-canvas handle_`: passed, 2 tests.
- `cargo nextest run -p fret-canvas`: passed, 72 tests.
- `cargo nextest run -p fret-node --no-default-features resize_handle_vocabulary_lives_in_fret_canvas`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests after resize extraction.
- `cargo nextest run -p fret-node node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant`: passed.
- `cargo nextest run -p fret-node node_graph_surface_active_descendant_points_to_focused_port_semantics_node`: passed.
- `cargo fmt --check`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests at closeout.
- `cargo nextest run -p fret-node`: passed, 444 tests at closeout.
- `cargo nextest run -p fret-canvas`: passed, 72 tests at closeout.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.
- `python3 tools/check_layering.py`: passed.
- `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `git diff --check`: passed.
