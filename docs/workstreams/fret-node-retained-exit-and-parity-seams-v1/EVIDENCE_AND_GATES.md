# `fret-node` Retained Exit And Parity Seams (v1) - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p fret-node --no-default-features
```

This proves the supported headless/binding/controller surface compiles and passes without retained
compatibility.

## Gate Set

### Baseline Gate

```bash
cargo nextest run -p fret-node --no-default-features
python3 tools/check_layering.py
```

Use this before deleting retained code to confirm the supported surface starts clean.

### Retained Exit Gate

```bash
cargo nextest run -p fret-node --no-default-features
cargo nextest run -p fret-node
```

After `compat-retained-canvas` is removed, there should be no retained feature gate to run. Any
behavioral coverage that mattered must move to supported seams.

### Canvas Extraction Gate

```bash
cargo nextest run -p fret-canvas
cargo nextest run -p fret-node --no-default-features
```

This proves the generic helper remains reusable below `fret-node` and its graph adapter still works.

### Closeout Gate

```bash
cargo fmt --check
cargo nextest run -p fret-node --no-default-features
cargo nextest run -p fret-node
cargo nextest run -p fret-canvas
cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings
cargo clippy -p fret-canvas --all-targets -- -D warnings
python3 tools/check_layering.py
```

Use narrower workspace gates because the lane touches `fret-node`, `fret-canvas`, and docs, not the
full workspace.

### Review Gate

Run `review-workstream` semantics before accepting task or lane completion: check workstream
compliance, code quality, deletion scope, and whether docs/tests prove the shipped behavior.

## Evidence Anchors

- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/DESIGN.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/TODO.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/MILESTONES.md`
- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/ui/canvas`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `ecosystem/fret-canvas/src`
- `docs/node-graph-xyflow-parity.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`

## Fresh Evidence

### NRP-010 Baseline - 2026-05-28

- `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 134 tests.
- `python3 tools/check_layering.py`: passed.

### NRP-020 Retained Compatibility Island Exit - 2026-05-28

- `cargo nextest run -p fret-node --no-default-features`: passed, 130 tests.
- `cargo nextest run -p fret-node`: passed, 442 tests.
- `cargo nextest run -p fret-node --no-default-features ui_sources_do_not_use_retained_canvas_compatibility`: passed.
- Evidence anchors:
  - `ecosystem/fret-node/Cargo.toml`
  - `ecosystem/fret-node/src/ui/canvas/mod.rs`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

### NRP-030 Public Node Graph Docs And API Narrative Cleanup - 2026-05-28

- `cargo nextest run -p fret-node --no-default-features public_node_graph_guides`: passed.
- `! rg -n "NodeGraphCanvas::|compat-retained-canvas|retained canvas" docs/node-graph*.md ecosystem/fret-node/README.md`: passed.
- Evidence anchors:
  - `docs/node-graph-controlled-mode.md`
  - `docs/node-graph-how-to-build-like-xyflow.md`
  - `docs/node-graph-xyflow-parity.md`
  - `docs/node-graph-roadmap.md`
  - `docs/node-graph-addons-theming.md`
  - `docs/node-graph-addons-minimap-controls.md`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

### NRP-040 Additional Generic Canvas Mechanism Extraction - 2026-05-28

- `cargo nextest run -p fret-canvas handle_`: passed, 2 tests.
- `cargo nextest run -p fret-canvas`: passed, 72 tests.
- `cargo nextest run -p fret-node --no-default-features resize_handle_vocabulary_lives_in_fret_canvas`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests.
- Evidence anchors:
  - `ecosystem/fret-canvas/src/interaction/resize.rs`
  - `ecosystem/fret-canvas/src/interaction/mod.rs`
  - `ecosystem/fret-node/src/ui/canvas/resize_handle.rs`
  - `ecosystem/fret-node/src/ui/presenter.rs`
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

### NRP-050 XyFlow Hook/Focus Parity Seam - 2026-05-28

- `cargo nextest run -p fret-node node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant`: passed.
- `cargo nextest run -p fret-node node_graph_surface_active_descendant_points_to_focused_port_semantics_node`: passed.
- Evidence anchors:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  - `docs/node-graph-xyflow-parity.md`

### NRP-060 Closeout - 2026-05-28

- Claim verified: the four scoped refactors are complete, documented, and covered by fresh targeted
  gates.
- `cargo fmt --check`: passed.
- `cargo nextest run -p fret-node --no-default-features`: passed, 131 tests.
- `cargo nextest run -p fret-node`: passed, 444 tests.
- `cargo nextest run -p fret-canvas`: passed, 72 tests.
- `cargo clippy -p fret-node --no-default-features --all-targets -- -D warnings`: passed.
- `cargo clippy -p fret-canvas --all-targets -- -D warnings`: passed.
- `python3 tools/check_layering.py`: passed.
- `python3 -m json.tool docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`: passed.
- `python3 tools/check_workstream_catalog.py`: passed, 477 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: passed.
- Broader workspace tests were not run because this lane touched `fret-node`, `fret-canvas`, and
  docs; the lane-specific closeout gates cover those surfaces directly.
- Evidence anchors:
  - `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/CLOSEOUT_AUDIT_2026-05-28.md`
  - `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/TODO.md`
  - `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/WORKSTREAM.json`
  - `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/HANDOFF.md`

## Notes

Do not preserve retained compatibility only because old tests depended on retained widget contexts.
Move meaningful assertions to supported seams and delete implementation-shape tests that only
describe the historical island.
