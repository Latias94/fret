# Fret Node Paint Prepaint Adapter v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo check -p fret-node --features compat-retained-canvas
```

## Gate Set

### Scope Gate

```bash
python3 -m json.tool docs/workstreams/fret-node-paint-prepaint-adapter-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

### Targeted Iteration Gate

```bash
cargo test -p fret-node --features compat-retained-canvas paint_prepaint_adapter
```

### Package Gates

```bash
cargo check -p fret-node
cargo check -p fret-node --features compat-retained-canvas
```

### Boundary Gate

```bash
python3 tools/check_layering.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/DESIGN.md`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/prepaint_cull_window_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root_helpers.rs`
- `ecosystem/fret-node/src/lib.rs`

## Initial Scope Evidence - 2026-05-25

Claim:

- Paint/prepaint is too broad for the low-level adapter closeout lane.
- Prepaint cull-window behavior is the smallest likely first proof because it is much narrower than
  the retained paint tree.

Fresh validation:

- Pending first implementation slice.

Notes:

- This lane intentionally starts after `fret-node-low-level-adapter-v1`; do not reopen that lane for
  paint/prepaint adapter work.

## NPA-020 Implementation Evidence - 2026-05-25

Claim:

- Prepaint cull-window route preparation now sits behind a named retained-agnostic adapter seam.
- The retained `PrepaintCx` binding is isolated to `retained_widget_cull_window.rs`.
- Cull-window key-shift debug recording goes through the adapter contract instead of directly
  calling retained debug APIs.

Evidence:

- `prepaint_cull_window_adapter.rs` defines `PrepaintCullWindowCx` and owns view-state sync, bounds
  lookup, cull-window key calculation, and key-shift dispatch.
- `retained_widget_cull_window.rs` implements the adapter for `PrepaintCx` and delegates to
  `sync_prepaint_cull_window`.
- `retained_widget_cull_window_shift.rs` records key-shift debug output through
  `record_node_graph_cull_window_shift`.
- `surface_policy_tests::paint_prepaint_adapter_keeps_cull_window_route_preparation_off_retained_cx`
  locks the adapter/helper source away from retained Cx names and keeps retained route preparation
  out of the lifecycle entrypoint.

Fresh validation:

- `cargo test -p fret-node --features compat-retained-canvas paint_prepaint_adapter` - passed; 1
  test passed, 1161 filtered out.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-prepaint-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
