# Fret Node Paint Prepaint Adapter v1 - Evidence And Gates

Status: Closed
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
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`

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

## NPA-030 Paint Root Scope Audit - 2026-05-25

Claim:

- Paint root scene emission is not one small adapter seam.
- A broad `PaintCx` adapter for `canvas.paint_root(cx)` would hide multiple retained-context
  operation families behind one large trait.
- The next paint follow-on should target cache-plan preparation first because it needs host access,
  bounds, and scale factor but does not directly mutate the scene.

Evidence:

- `retained_widget_runtime_paint.rs` only owns lifecycle theme sync and root paint dispatch.
- `paint_root/cached.rs` combines observation, view-state sync, frame prep, cache-plan prep,
  cached/immediate pass selection, and tail cleanup.
- `paint_root/frame.rs` mixes bounds, scene mutation, cache diagnostics, background, and grid paint.
- `paint_root/cache_plan.rs` is the best next slice: it uses `app`, `bounds`, and `scale_factor`
  for derived output and cache-plan preparation without directly emitting scene ops.
- `PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md` records the split table and follow-on recommendation.

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

## NPA-040 Closeout And Follow-On Split - 2026-05-25

Claim:

- This lane is closed.
- The shipped prepaint cull-window adapter seam remains the implementation proof.
- Paint-root cache-plan work is split into `fret-node-paint-root-cache-plan-adapter-v1`.

Evidence:

- `CLOSEOUT_AUDIT_2026-05-25.md` records final state and residual risks.
- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/` contains the follow-on scope,
  tasks, gates, and handoff.

Fresh validation:

- `python3 -m json.tool docs/workstreams/fret-node-paint-prepaint-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 446 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
