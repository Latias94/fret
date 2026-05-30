# `fret-node` Retained Canvas Mirror Cleanup (v1) - Closeout Audit

Date: 2026-05-27
Status: closed

## Closed Scope

This workstream completed the retained `NodeGraphCanvas` follow-on split from the runtime/store
contract closure lane:

- Added the private `NodeGraphCanvasMirrors` owner for retained canvas graph/view/editor-config
  external model mirrors.
- Moved retained canvas internals and crate-internal tests to cross that named mirror boundary.
- Added source-policy coverage that prevents top-level mirror fields from returning to
  `NodeGraphCanvasWith`.
- Deleted the unused duplicate `commit_legacy` retained transaction pipeline.
- Added source-policy coverage that retained canvas keeps one current commit pipeline and no legacy
  mirror writer.

## Fresh Closeout Gates

- `cargo fmt --check`: passed.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_commit_pipeline_has_no_legacy_mirror_writer retained_canvas_mirror_owner`: passed, 2 tests.
- `cargo check -p fret-node --features compat-retained-canvas`: passed.
- `cargo check -p fret-node --no-default-features`: passed.
- `python3 tools/check_layering.py`: passed.

Additional task evidence:

- `cargo nextest run -p fret-node --no-default-features runtime`: passed, 46 tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas binding_surface_covers_instance_style_sync_and_history_helpers`: passed, 1 test.

## Follow-Ons

- No follow-on is required for this scoped mirror cleanup lane.
- Broader public retained-surface removal remains a separate compatibility-exit concern and should
  not be folded into this closed workstream.

## Evidence Anchors

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/construct.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/commit/`
- `ecosystem/fret-node/src/surface_policy_tests.rs`
- `docs/workstreams/fret-node-retained-canvas-mirror-cleanup-v1/EVIDENCE_AND_GATES.md`
