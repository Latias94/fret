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
