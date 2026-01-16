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

## Phase 0 〞 Instrumentation and Regression Harnesses

- [ ] Add a cache-root perf breakdown to the HUD (hits/misses/replayed_ops per cache root).
- [ ] Add tracing spans for layout/paint per cache root (opt-in, low overhead).
- [ ] Add a "nested cache roots correctness" test harness (unit tests in `crates/fret-ui/src/tree/tests/`).
- [ ] Add an overlay torture-test demo scenario (popover/menu/tooltip + outside-press + focus trap).
- [ ] Add a virtual list torture-test demo scenario (10k+ rows, selection + hover + inline text input).

## Phase 1 〞 Cache Roots (ViewCache) Closed Loop (Paint Stream)

- [ ] Record nearest cache root ownership per node during declarative mount (`GlobalElementId -> NodeId` bridge).
- [ ] Uplift model/global observations to the nearest cache root during layout/paint recording.
- [ ] Propagate invalidation across cache roots (nested boundaries must invalidate ancestors for replay safety).
- [ ] Restrict paint replay to cache roots only when view-cache mode is active (enforce contract).
- [ ] Ensure inspection/probe modes disable caching (cache roots and paint cache policy respect `inspection_active`).
- [ ] Document authoring guidance for where to place cache roots (panel granularity; avoid micro-boundaries).

## Phase 2 〞 Multi-Stream Frame Recording (Prepaint/Interaction)

- [ ] Define the next minimal stream(s) for replay (hitboxes/dispatch tree/tooltips/cursor styles).
- [ ] Introduce a range-replay recording abstraction per stream (compatible with ADR 0055).
- [ ] Thread cache root semantics through all streams (a cache hit must replay all stream ranges).
- [ ] Add acceptance tests for interaction correctness under caching (hit-test, outside-press, focus path).

## Phase 3 〞 Authoring Ergonomics (Ecosystem)

- [ ] Add a fluent authoring surface in `ecosystem/fret-ui-kit` mirroring gpui-component density (`StyledExt`-like).
- [ ] Add shadcn-aligned recipes that default to cache roots for expensive panels (opt-in, guided).
- [ ] Add an author-facing “cached subtree” helper API in ecosystem (do not bloat `fret-ui`).

## Phase 4 〞 Text System and Editor-Grade Inputs (Parallel Track)

- [ ] Implement font stack bootstrap + stable IDs (ADR 0162).
- [ ] Establish a stable text layout revision key that participates in caching keys/stream reuse.
- [ ] Validate IME/caret/selection behavior under caching and nested overlays.

## Open Questions (Keep Short)

- Should cache roots be strictly opt-in only, or do we want an auto policy in demos (never in core)?
- What is the first non-paint stream to land for maximum feel improvement (hitboxes vs cursor/styles vs dispatch tree)?

