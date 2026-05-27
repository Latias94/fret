# `fret-node` Retained Canvas Mirror Cleanup (v1) - Evidence And Gates

Status: complete
Last updated: 2026-05-27

## Baseline

This lane follows the closed runtime/store contract workstream. Its closeout split retained
`NodeGraphCanvas` graph/view/editor-config mirror cleanup as a compatibility follow-on.

Baseline anchors:

- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/CLOSEOUT_AUDIT_2026-05-27.md`
- `docs/workstreams/fret-node-runtime-store-contract-closure-v1/UI_MIRROR_INVENTORY_2026-05-26.md`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner
cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers
```

### Feature And Compatibility Gates

```bash
cargo check -p fret-node --features compat-retained-canvas
cargo check -p fret-node --no-default-features
cargo nextest run -p fret-node --no-default-features runtime
```

### Closeout Gates

```bash
cargo fmt --check
cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner
cargo check -p fret-node --features compat-retained-canvas
cargo check -p fret-node --no-default-features
python3 tools/check_layering.py
```

Use narrower retained compat tests during iteration because full retained canvas coverage is large.
Record any skipped broader gate with a concrete reason.

## Evidence Anchors

- `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/DESIGN.md`
- `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/TODO.md`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/construct.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/sync.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`

## Evidence Log

### 2026-05-27 - NCM-010 completed

Workstream opened from the closed runtime/store follow-on. No implementation gates were run during
document creation.

### 2026-05-27 - NCM-020 completed

Changes:

- Added `NodeGraphCanvasMirrors` as the private retained canvas mirror owner.
- Moved `NodeGraphCanvasWith` graph/view/editor-config model fields behind that mirror owner.
- Updated retained canvas internals and crate-internal tests to cross the explicit mirror boundary.
- Added `retained_canvas_mirror_owner_quarantines_external_models` source-policy coverage.

Red/green evidence:

- Initial `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`:
  failed because `NodeGraphCanvasMirrors` did not exist.
- Final `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_mirror_owner`:
  passed, 1 test.

Fresh gates:

- `cargo fmt -p fret-node --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers`:
  passed, 1 test.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.

Evidence anchors:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/construct.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/view_state/sync.rs`
- `ecosystem/fret-node/src/surface_policy_tests.rs`

### 2026-05-27 - NCM-030 completed

Audit result:

- `commit_legacy` was an unused duplicate retained transaction pipeline.
- It repeated the same store-backed dispatch/sync and fallback mirror write path as the current
  `commit` module.
- No call sites referenced `commit_ops_legacy`, `commit_transaction_legacy`, or
  `apply_transaction_result_legacy`.

Changes:

- Removed `mod commit_legacy` from retained canvas widget registration.
- Deleted `ui/canvas/widget/commit_legacy/*`.
- Added `retained_canvas_commit_pipeline_has_no_legacy_mirror_writer` source-policy coverage.

Red/green evidence:

- Initial `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_commit_pipeline_has_no_legacy_mirror_writer`:
  failed because `mod commit_legacy;` still existed.
- Final `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_commit_pipeline_has_no_legacy_mirror_writer retained_canvas_mirror_owner`:
  passed, 2 tests.

Fresh gates:

- `cargo fmt -p fret-node --check`: passed.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.
- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.

Evidence anchors:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/commit/`
- `ecosystem/fret-node/src/surface_policy_tests.rs`

### 2026-05-27 - NCM-040 closeout completed

Claim:

- The scoped retained canvas mirror cleanup lane is complete.
- Retained canvas mirrors are quarantined and the duplicate legacy retained commit writer is gone.
- No required follow-on remains inside this lane's scope.

Fresh closeout gates:

- `cargo fmt --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_commit_pipeline_has_no_legacy_mirror_writer retained_canvas_mirror_owner`:
  passed, 2 tests.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.
- `python3 tools/check_layering.py`: passed.

Evidence anchors:

- `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/CLOSEOUT_AUDIT_2026-05-27.md`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/commit/`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
