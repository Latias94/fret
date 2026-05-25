# Fret Node Paint Root Frame Diagnostics Adapter v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter
```

## Gate Set

### Scope Gate

```bash
python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

### Package Gate

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter
cargo check -p fret-node
cargo check -p fret-node --features compat-retained-canvas
```

### Boundary Gate

```bash
python3 tools/check_layering.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Initial Scope Evidence - 2026-05-25

Claim:

- Path-cache diagnostics recording is the smallest next frame retained-context seam.
- The seam should not absorb cache begin, viewport, clip, background paint, grid paint, tail
  cleanup, or cached/immediate passes.

Fresh validation:

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter` -
  passed; 1 test passed, 1165 filtered out.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 449 dedicated directories and 47
  standalone markdown files.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.
