# Fret Node Paint Root Frame Setup Adapter v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```bash
cargo check -p fret-node --features compat-retained-canvas
```

## Gate Set

### Scope Gate

```bash
python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

### Package Gate

```bash
cargo check -p fret-node --features compat-retained-canvas
```

### Boundary Gate

```bash
git diff --check
```

## Evidence Anchors

- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`

## Initial Scope Evidence - 2026-05-25

Claim:

- Frame setup is too broad to wrap without an operation-family audit.
- The first likely implementation candidate is bounds/viewport route inputs, not scene emission.

Fresh validation:

- Pending first audit slice.

Notes:

- Keep static layer replay/store, cached/immediate passes, and tail cleanup out of this lane.
