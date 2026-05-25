# Fret Node Paint Root Pass Clip Adapter v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter
```

## Gate Set

```bash
cargo fmt --package fret-node
cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter
cargo check -p fret-node
cargo check -p fret-node --features compat-retained-canvas
python3 tools/check_workstream_catalog.py
python3 tools/check_layering.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/DESIGN.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/immediate_pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/pass_scene_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/pass_scene_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Validation

Initial validation:

- `python3 -m json.tool docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/WORKSTREAM.json` -
  passed.

Implementation validation:

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.
