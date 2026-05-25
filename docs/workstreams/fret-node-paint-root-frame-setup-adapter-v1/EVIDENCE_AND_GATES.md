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
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md`
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

## FSA-010/FSA-020 Scope Audit Evidence - 2026-05-25

Claim:

- Frame setup is not one operation family.
- Bounds/viewport/render-cull computation is the smallest next frame seam because it needs only
  paint bounds at the retained boundary and does not touch scene mutation.
- Cache stats diagnostics, clip emission, background paint, and grid paint should remain outside the
  first frame seam.

Evidence:

- `FRAME_SETUP_SCOPE_AUDIT_2026-05-25.md` records the operation-family split table.
- `paint_root/frame.rs` shows frame setup currently combines cache begin, path-cache diagnostics,
  viewport/render-cull math, clip emission, background, and grid paint.
- `paint_root/frame/cache.rs` shows diagnostics needs window/node/app/global-registry access.
- `paint_root/frame/background.rs` combines skin graph reads through `cx.app` with scene emission.

Fresh validation:

- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed; validated 447 dedicated directories and 47
  standalone markdown files.
- `git diff --check` - passed.
