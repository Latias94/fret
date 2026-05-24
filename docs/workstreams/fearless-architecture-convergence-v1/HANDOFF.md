# Fearless Architecture Convergence v1 - Handoff

Updated: 2026-05-25

## Current State

This coordinator lane is active but should now be read mostly as the first-open owner map. It has
mapped the six fearless cuts, landed the retained public-surface first slice, and opened the five
narrow follow-on lanes for the remaining cuts.

## Owner Lanes

- Retained public surface exit: `docs/workstreams/retained-public-surface-exit-v1/`
- Node low-level adapter: `docs/workstreams/fret-node-low-level-adapter-v1/`
- Kit taxonomy boundaries: `docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1/`
- Overlay/focus/dismissal oracle: `docs/workstreams/ui-overlay-focus-dismissal-oracle-v1/`
- Frame Pipeline v2 phase contract follow-on:
  `docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1/`
- Launch root-surface convergence:
  `docs/workstreams/fret-launch-root-surface-convergence-v1/`

## Verified First Slice

Recorded gates:

- `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
- `cargo check -p fret-ui`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

The `cargo check -p fret-ui` gate passed with existing warnings in
`crates/fret-ui/src/tree/layout/clean_geometry.rs` and `current_effective_opacity`; these warnings
were not introduced by the retained public-surface slice.

## Next Step

Either close out `docs/workstreams/retained-public-surface-exit-v1/` after final review, or continue
with `NLA-010` in `docs/workstreams/fret-node-low-level-adapter-v1/` to introduce the first named
canvas/viewport adapter seam.
