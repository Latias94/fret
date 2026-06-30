# Fearless Architecture Convergence v1 - Handoff

Updated: 2026-06-30

## Current State

This coordinator lane is active but should now be read mostly as the first-open owner map. It has
mapped the six fearless cuts, landed the retained public-surface first slice, and opened the five
narrow follow-on lanes for the remaining cuts.

The 2026 UI framework convergence plan now extends this coordinator as the current owner map for
new fearless refactor work:

- plan: `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`
- contract index: `docs/golden-architecture.md`
- runtime contract checklist: `docs/runtime-contract-matrix.md`
- closure map: `docs/ui-closure-map.md`
- ADR overlay: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

Do not reopen the closed broad Frame Pipeline v2 lane. Treat it as evidence and start narrow
follow-ons for ViewId-first dirty ownership, scene chunks, renderer dirty uploads, and text/glyph
budget gates.

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

Execute FAC-100 / plan U2: add the responsibility source-policy checker before broader runtime
deletions. Then use identity/dirty graph metrics as the first runtime migration guardrail.
