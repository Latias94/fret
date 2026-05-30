# `fret-node` Retained Exit And Parity Seams (v1) - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `NRP` for node retained/parity.

## Cross-cutting Guardrails

- [x] Prefer deletion over compatibility shims when a retained-only path conflicts with the target
  architecture.
- [x] Preserve supported behavior with headless/default conformance tests before deleting retained
  coverage.
- [x] Keep reusable canvas mechanics in `fret-canvas`; keep graph-specific policy in `fret-node`.
- [x] Keep docs aligned with binding-first/declarative-first app authoring.
- [x] Update ADR alignment/evidence when ADR 0330 retained-compat facts change.

## M0 - Scope And Evidence Freeze

- [x] NRP-010 [owner=planner] [deps=none] [scope=docs/workstreams/fret-node-retained-exit-and-parity-seams-v1]
  Goal: Freeze the four-refactor scope, target state, deletion plan, and validation gates.
  Validation:
  - `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`
  - `cargo nextest run -p fret-node --no-default-features`
  - `python3 tools/check_layering.py`
  Review: Planner checks that this is a new follow-on, not a reopened closed lane.
  Evidence:
  - `DESIGN.md`
  - `EVIDENCE_AND_GATES.md`
  Fresh gates:
  - `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
  - `python3 tools/check_layering.py`: passed.
  Handoff: DONE. Start implementation with `NRP-020`.

## M1 - Retained Compatibility Island Exit

- [x] NRP-020 [owner=codex] [deps=NRP-010] [scope=ecosystem/fret-node/Cargo.toml,ecosystem/fret-node/src/ui,ecosystem/fret-node/src/surface_policy_tests.rs,docs/adr/0330-retained-runtime-internal-and-compat-surface.md]
  Goal: Delete or fully quarantine the `compat-retained-canvas` feature and retained-only widget
  island so supported builds no longer carry a retained canvas path.
  Validation:
  - `cargo nextest run -p fret-node --no-default-features`
  - `cargo nextest run -p fret-node`
  - focused retained source-policy replacement test
  Review: Ensure retained-only behavior is either deleted or replaced by supported seam coverage.
  Evidence:
  - `ecosystem/fret-node/Cargo.toml`
  - `ecosystem/fret-node/src/ui/canvas`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  Fresh gates:
  - `cargo nextest run -p fret-node --no-default-features`: passed, 130 tests.
  - `cargo nextest run -p fret-node`: passed, 442 tests.
  - `cargo nextest run -p fret-node --no-default-features ui_sources_do_not_use_retained_canvas_compatibility`: passed.
  Review: DONE. The retained feature, raw transport queue, feature-gated canvas modules, retained
  widget source island, and retained-only policy tests were removed; ADR 0330 and implementation
  alignment now record the exit.
  Handoff: Continue with `NRP-030`.

## M2 - Public Node Graph Docs And API Narrative Cleanup

- [x] NRP-030 [owner=codex] [deps=NRP-020] [scope=docs/node-graph-*.md,ecosystem/fret-node/README.md,docs/adr/IMPLEMENTATION_ALIGNMENT.md]
  Goal: Remove old retained `NodeGraphCanvas` tutorial/API guidance and rewrite docs around
  store/controller/declarative composition.
  Validation:
  - `cargo nextest run -p fret-node --no-default-features public_node_graph_guides`
  - `rg -n "NodeGraphCanvas::|compat-retained-canvas|retained canvas" docs/node-graph*.md ecosystem/fret-node/README.md`
  Review: Docs should describe retained history only when the text explicitly says it was removed.
  Evidence:
  - `docs/node-graph-controlled-mode.md`
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-xyflow-parity.md`
  Fresh gates:
  - `cargo nextest run -p fret-node --no-default-features public_node_graph_guides`: passed.
  - `! rg -n "NodeGraphCanvas::|compat-retained-canvas|retained canvas" docs/node-graph*.md ecosystem/fret-node/README.md`: passed.
  Review: DONE. Public node-graph docs now teach `NodeGraphSurfaceBinding`,
  `node_graph_surface(...)`, `NodeGraphController`, and store/callback composition instead of the
  deleted retained canvas API. ADR alignment rows for the affected node/canvas contracts were
  updated to the declarative surface and current `fret-canvas` boundary.
  Handoff: Continue with `NRP-040`.

## M3 - Additional Generic Canvas Mechanism Extraction

- [x] NRP-040 [owner=codex] [deps=NRP-020] [scope=ecosystem/fret-canvas,ecosystem/fret-node/src/ui/canvas]
  Goal: Move one additional reusable canvas mechanism below `fret-node` and leave a thin graph-specific
  adapter in the node crate.
  Validation:
  - `cargo nextest run -p fret-canvas`
  - `cargo nextest run -p fret-node --no-default-features`
  - focused extraction conformance test
  Review: Reject extraction if it introduces graph vocabulary into `fret-canvas`.
  Evidence:
  - `ecosystem/fret-canvas/src`
  - `ecosystem/fret-node/src/ui/canvas`
  Fresh gates:
  - `cargo nextest run -p fret-canvas handle_`: passed, 2 tests.
  - `cargo nextest run -p fret-canvas`: passed, 72 tests.
  - `cargo nextest run -p fret-node --no-default-features resize_handle_vocabulary_lives_in_fret_canvas`: passed.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests.
  Review: DONE. Generic 2D resize handle vocabulary and bitset moved to
  `fret_canvas::interaction::{ResizeHandle2D, ResizeHandleSet2D}`. `fret-node` keeps only
  node-named aliases at the presenter/canvas boundary, with a source-policy test preventing the
  enum/bitset from drifting back into the graph crate.
  Handoff: Continue with `NRP-050`.

## M4 - XyFlow Hook/Focus Parity Seam

- [x] NRP-050 [owner=codex] [deps=NRP-020,NRP-030] [scope=ecosystem/fret-node/src/ui/declarative/paint_only,docs/node-graph-xyflow-parity.md]
  Goal: Land one bounded XyFlow-style extension/focus seam with tests and update the parity document
  from TODO/partial to current behavior where justified.
  Validation:
  - `cargo nextest run -p fret-node node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant`
  - `cargo nextest run -p fret-node node_graph_surface_active_descendant_points_to_focused_port_semantics_node`
  - `cargo nextest run -p fret-node`
  Review: The seam must not expose retained widget contexts or require downstream retained authoring.
  Evidence:
  - `docs/node-graph-xyflow-parity.md`
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
  - focused conformance tests in `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  Fresh gates:
  - `cargo nextest run -p fret-node node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant`: passed.
  - `cargo nextest run -p fret-node node_graph_surface_active_descendant_points_to_focused_port_semantics_node`: passed.
  Review: DONE. `disableKeyboardA11y` now gates the declarative active-descendant/a11y internals
  path directly, while the default active-descendant path remains covered. The parity document
  records the shipped behavior and leaves broader semantic focus nodes/minimap controls as a
  follow-on gap.
  Handoff: Continue with `NRP-060`.

## M5 - Closeout

- [x] NRP-060 [owner=planner] [deps=NRP-050] [scope=docs/workstreams/fret-node-retained-exit-and-parity-seams-v1,docs/node-graph-*.md]
  Goal: Verify the lane, update evidence, mark follow-ons, and close or split remaining parity work.
  Validation:
  - `cargo fmt --check`
  - `cargo nextest run -p fret-node --no-default-features`
  - `cargo nextest run -p fret-node`
  - `cargo nextest run -p fret-canvas`
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`
  - `cargo clippy -p fret-canvas --all-targets -- -D warnings`
  - `python3 tools/check_layering.py`
  Review: Use `verify-rust-workstream` before marking the goal complete.
  Evidence:
  - `EVIDENCE_AND_GATES.md`
  - `WORKSTREAM.json`
  - `HANDOFF.md`
  Fresh gates:
  - `cargo fmt --check`: passed.
  - `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests.
  - `cargo nextest run -p fret-node`: passed, 444 tests.
  - `cargo nextest run -p fret-canvas`: passed, 72 tests.
  - `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
  - `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.
  - `python3 tools/check_layering.py`: passed.
  - `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
  - `python3 tools/check_workstream_catalog.py`: passed.
  - `git diff --check`: passed.
  Review: DONE. The four scoped fearless refactors are complete and broader XyFlow/a11y parity is
  deferred to a separate follow-on boundary.
  Handoff: CLOSED. Do not reopen this lane for semantic focus nodes, minimap focus, or a broad
  ReactFlow hook facade.
