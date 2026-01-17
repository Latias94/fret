# GPUI Parity Refactor 〞 TODO Tracker (Experience + Performance)

Status: Active (tracking doc; keep updated during refactors)

This document tracks executable TODOs for the GPUI parity refactor workstream.

Primary design anchors:

- Cache roots + cached subtree semantics: `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`
- Element identity + frame-staged element state: `docs/adr/1151-element-identity-debug-paths-and-frame-staged-element-state.md`
- Frame recording + subtree replay caching (baseline): `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`

## Guiding Principles

- Prefer locking contracts early (ADRs) over shipping more surface area.
- Optimize for long-term composability: identity -> observation -> invalidation -> caching -> introspection must be a closed loop.
- Keep `crates/fret-ui` mechanism-only; policy/recipes live in ecosystem crates.

## Current Snapshot (What Already Landed)

These are implemented today (not just ADR intent). Keep this list short and evidence-backed.

- ViewCache (v1) mechanics and correctness scaffolding exist:
  - Declarative skip-on-hit semantics via `ElementContext::view_cache(...)` and mount reuse.
  - Paint replay gating to cache roots when view-cache mode is active.
  - Observation uplift to the nearest cache root + nested invalidation propagation (ancestor safety).
  - Evidence: `crates/fret-ui/src/tree/tests/view_cache.rs`, `crates/fret-ui/src/declarative/tests/view_cache.rs`,
    `crates/fret-ui/src/tree/paint.rs`, `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/elements/cx.rs`.
- Inspection/diagnostics path exists and disables caching in inspection mode:
  - `UiTree::set_inspection_active(...)` is wired from `fret-bootstrap` diagnostics.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`,
    `crates/fret-ui/src/tree/mod.rs`.
- Ecosystem authoring density is in-progress (fluent surfaces exist, still expanding):
  - `UiBuilder` / `UiExt` patch-based authoring, plus `StyledExt` style refinements.
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`, `ecosystem/fret-ui-kit/src/styled.rs`,
    `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`.
- Virtualization baseline is aligned and has stress coverage:
  - Core contract + tests + demo exist for `VirtualList`.
  - Evidence: `crates/fret-ui/src/virtual_list.rs`, `crates/fret-ui/src/declarative/tests/virtual_list.rs`,
    `apps/fret-examples/src/virtual_list_stress_demo.rs`.

## Phase 0 〞 Instrumentation and Regression Harnesses

- [x] Add a cache-root perf breakdown to the HUD (hits/misses/replayed_ops per cache root).
  - Evidence: `crates/fret-ui/src/tree/mod.rs`, `apps/fret-ui-gallery/src/driver.rs`.
- [x] Add tracing spans for layout/paint per cache root (opt-in, low overhead).
  - Evidence: `crates/fret-ui/src/tree/layout.rs`, `crates/fret-ui/src/tree/paint.rs`.
- [x] Add a "nested cache roots correctness" test harness (unit tests in `crates/fret-ui/src/tree/tests/`).
  - Evidence: `crates/fret-ui/src/tree/tests/view_cache.rs`.
- [x] Add an overlay torture-test demo scenario (popover/menu/tooltip + outside-press + focus trap).
  - Evidence: `tools/diag-scripts/ui-gallery-overlay-torture.json`, `apps/fretboard/src/diag.rs`,
    `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fret-ui-gallery/src/ui.rs`.
- [x] Add a virtual list torture-test demo scenario (10k+ rows, selection + hover + inline text input).
  - Evidence: `apps/fret-ui-gallery/src/ui.rs`, `apps/fret-ui-gallery/src/spec.rs`,
    `tools/diag-scripts/ui-gallery-virtual-list-torture.json`.

## Phase 1 〞 Cache Roots (ViewCache) Closed Loop (Paint Stream)

- [ ] Record nearest cache root ownership per node during declarative mount (`GlobalElementId -> NodeId` bridge).
  - Partial: the runtime can derive the nearest root via parent walking; a mount-time ownership map is still useful
    for lower overhead and for future multi-stream recording.
  - Evidence (current approach): `crates/fret-ui/src/tree/mod.rs`.
- [x] Uplift model/global observations to the nearest cache root during layout/paint recording.
  - Evidence: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/tree/tests/view_cache.rs`.
- [x] Propagate invalidation across cache roots (nested boundaries must invalidate ancestors for replay safety).
  - Evidence: `crates/fret-ui/src/tree/tests/view_cache.rs`.
- [x] Restrict paint replay to cache roots only when view-cache mode is active (enforce contract).
  - Evidence: `crates/fret-ui/src/tree/paint.rs`, `crates/fret-ui/src/tree/tests/view_cache.rs`.
- [x] Ensure inspection/probe modes disable caching (cache roots and paint cache policy respect `inspection_active`).
  - Evidence: `crates/fret-ui/src/tree/mod.rs`, `ecosystem/fret-bootstrap/src/ui_app_driver.rs`.
- [ ] Document authoring guidance for where to place cache roots (panel granularity; avoid micro-boundaries).
  - Partial: examples exist in UI gallery and docking/workspace code, but we need a concise “rules of thumb” doc.
  - Evidence (examples): `apps/fret-ui-gallery/src/docs.rs`, `ecosystem/fret-workspace/src/panes.rs`,
    `ecosystem/fret-docking/src/dock/panel_registry.rs`.

## Phase 2 〞 Multi-Stream Frame Recording (Prepaint/Interaction)

- [ ] Define the next minimal stream(s) for replay (hitboxes/dispatch tree/tooltips/cursor styles).
- [ ] Introduce a range-replay recording abstraction per stream (compatible with ADR 0055).
- [ ] Thread cache root semantics through all streams (a cache hit must replay all stream ranges).
- [ ] Add acceptance tests for interaction correctness under caching (hit-test, outside-press, focus path).

## Phase 3 〞 Authoring Ergonomics (Ecosystem)

- [ ] Add a fluent authoring surface in `ecosystem/fret-ui-kit` mirroring gpui-component density (`StyledExt`-like).
  - Partial: `UiBuilder` and `StyledExt` exist; expand the vocabulary, consistency, and coverage across primitives.
  - Evidence: `ecosystem/fret-ui-kit/src/ui_builder.rs`, `ecosystem/fret-ui-kit/src/styled.rs`.
- [ ] Add shadcn-aligned recipes that default to cache roots for expensive panels (opt-in, guided).
  - Partial: there are already cache-root usages in higher-level crates, but no standardized “default cache root”
    guidance per recipe/panel yet.
  - Evidence: `ecosystem/fret-workspace/src/panes.rs`, `ecosystem/fret-docking/src/dock/panel_registry.rs`,
    `apps/fret-ui-gallery/src/ui.rs`.
- [ ] Add an author-facing “cached subtree” helper API in ecosystem (do not bloat `fret-ui`).
  - Goal: a GPUI-like helper that makes cache placement and cache-key inputs explicit (without leaking runtime policy).

## Phase 4 〞 Text System and Editor-Grade Inputs (Parallel Track)

- [ ] Implement font stack bootstrap + stable IDs (ADR 0162).
- [ ] Establish a stable text layout revision key that participates in caching keys/stream reuse.
- [ ] Validate IME/caret/selection behavior under caching and nested overlays.

## Open Questions (Keep Short)

- Should cache roots be strictly opt-in only, or do we want an auto policy in demos (never in core)?
- What is the first non-paint stream to land for maximum feel improvement (hitboxes vs cursor/styles vs dispatch tree)?
