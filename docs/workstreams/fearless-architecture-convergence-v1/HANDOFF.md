# Fearless Architecture Convergence v1 - Handoff

Updated: 2026-07-02

## Current State

This coordinator lane is closed. Read it as a first-open owner map and closeout record, not as an
active implementation folder.

The 2026 UI framework convergence plan executed through U1-U9 and closed with explicit retained and
deferred follow-ons:

- plan: `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`
- contract index: `docs/golden-architecture.md`
- runtime contract checklist: `docs/runtime-contract-matrix.md`
- closure map: `docs/ui-closure-map.md`
- ADR overlay: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- closeout: `docs/workstreams/fearless-architecture-convergence-v1/CLOSEOUT_AUDIT_2026-07-02.md`

Do not reopen the closed broad Frame Pipeline v2 lane. Treat it as evidence and start narrow
follow-ons for any retained/deferred item named in the closeout audit.

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

Initial retained-surface gates:

- `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
- `cargo check -p fret-ui`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

The `cargo check -p fret-ui` gate passed with existing warnings in
`crates/fret-ui/src/tree/layout/clean_geometry.rs` and `current_effective_opacity`; these warnings
were not introduced by the retained public-surface slice.

## Closed Plan Evidence

The final closeout records evidence for:

- U1 contract freeze,
- U2 source-policy gate,
- U3 `workbench-lite` public app scaffold,
- U4 identity/dirty graph diagnostics,
- U5 `ViewId` / boundary frame product ownership,
- U6 policy vocabulary demotion,
- U7 scene chunks and guarded quad partial uploads,
- U8 text/glyph/wasm budgets, and
- U9 modular consumption profiles and `AppUi` facade split.

## Next Step

Start a narrow follow-on only when fresh work has a concrete owner and gate. The highest-value
follow-ons named by the closeout are:

- stable-handle deletion after U4 observability,
- entity-first `ViewId` ownership after the v1 boundary-node bridge,
- per-boundary ownership for currently window/layer-forest products only when cross-layer behavior
  remains proven,
- retained mechanism vocabulary audits for `Roving*` and explicit resizable module paths,
- full second-hour starter expansion beyond `workbench-lite`,
- `workbench-lite` settings dialog / real async-mutation submit diagnostics,
- advanced/manual source-policy allowlist cleanup as public wrappers land,
- renderer output migration beyond the flat `Scene` compatibility bridge,
- non-quad resident partial uploads,
- full-blob text helper deletion after chunk-local text closure,
- full aggregate pre-release runs when release scope needs them; the duplicate ADR ID `0324`
  blocker was resolved by renumbering the later a11y state-description ADR to `0332`.
