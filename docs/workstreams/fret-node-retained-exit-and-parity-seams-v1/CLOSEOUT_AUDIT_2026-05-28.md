# `fret-node` Retained Exit And Parity Seams v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-28

## Verdict

This lane is closed.

It completed the four scoped fearless refactors without preserving the retained canvas compatibility
island as a hidden public authoring path. The remaining XyFlow parity work is intentionally split as
future scope rather than reopened here.

## Shipped State

- `compat-retained-canvas` was removed from `fret-node`, along with the retained widget source
  island, retained transport adapters, and retained-only policy tests.
- Public node graph docs now teach `NodeGraphSurfaceBinding`, `node_graph_surface(...)`,
  `NodeGraphController`, store/callback composition, and declarative paint-only internals instead of
  the deleted retained `NodeGraphCanvas` API.
- Generic 2D resize handle vocabulary now lives in
  `fret_canvas::interaction::{ResizeHandle2D, ResizeHandleSet2D}`. `fret-node` keeps only
  graph-named aliases at the node boundary.
- `NodeGraphInteractionState.disable_keyboard_a11y` now suppresses the declarative
  active-descendant/a11y internals path at `sync_binding_internals_for_surface`, with a focused test
  proving the disabled behavior and a companion test proving the default active-descendant path.

## Deferred Follow-ons

- Full semantic focus nodes for nodes, edges, and ports.
- Port-focus authoring semantics beyond the current active-descendant support.
- Minimap and controls focus parity.
- A broader ReactFlow hook facade or declarative UI interception middleware.

These are not regressions in this lane; they are the next scope boundary.

## Closeout Evidence

- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/DESIGN.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/TODO.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/HANDOFF.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/node-graph-xyflow-parity.md`
- `ecosystem/fret-canvas/src/interaction/resize.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`

## Fresh Gates

- `cargo fmt --check` - passed.
- `cargo nextest run -p fret-node --no-default-features` - passed, 131 tests.
- `cargo nextest run -p fret-node` - passed, 444 tests.
- `cargo nextest run -p fret-canvas` - passed, 72 tests.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings` - passed.
- `cargo clippy -p fret-canvas --all-targets -- -D warnings` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- The lane deliberately did not build a full accessibility tree; active-descendant support is now
  gated correctly, but richer per-element focus semantics need their own design.
- The lane removed retained UI middleware. Store-level callbacks and middleware remain supported,
  while first-class declarative UI interception hooks should be designed as a new surface rather than
  copied from the deleted retained contexts.
