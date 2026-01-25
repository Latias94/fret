# Overlay & Input Arbitration v2 (Pointer Occlusion) — TODO Tracker

Status: Active (workstream tracker; keep updated during refactors)

This document tracks executable TODOs for the overlay/input arbitration v2 workstream.

- Narrative progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- Cross-cutting contract gate (phases / prevent_default / availability): `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- Related tracker: `docs/workstreams/input-dispatch-v2-todo.md`

## Tracking Format

Each TODO is labeled:

- ID: `OIA2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Building Blocks)

Keep this list short and evidence-backed:

- Pointer occlusion mechanism exists (`PointerOcclusion` stored on `UiLayer`).
  - Evidence: `crates/fret-ui/src/tree/layers.rs`
- Hit-tested pointer routing respects occlusion while observer policy can still request pointer-move observers.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`, `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Capture revocation dispatches deterministic `PointerCancel` reasons.
  - Evidence: `crates/fret-core/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`

## P0 — Conformance Suite Expansion (Hard-to-change Boundaries)

- [ ] OIA2-test-001 Expand docking drag × overlay hygiene edges (ADR 0072).
  - Target: cover “start drag while submenu open / while capture active / cross-window hover window”.
  - Evidence anchors (existing): `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`, `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- [ ] OIA2-test-002 Expand viewport tool capture vs hover overlays/tooltips (ADR 0049 follow-up).
  - Target: ensure capture sessions suppress incidental hover overlays deterministically across windows.
  - Evidence anchors (existing): `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- [ ] OIA2-test-003 Add “present vs interactive” regression coverage for menu-like overlays under close transitions.
  - Target: observer requests are disabled while `interactive=false` and click-through invariants hold.
  - Evidence anchors (existing): `ecosystem/fret-ui-kit/src/window_overlays/state.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`

## P0 — Diagnostics & Debuggability

- [ ] OIA2-diag-010 Add a stable “hit-test scope roots” diagnostic (modal barrier + pointer occlusion + capture owner).
  - Target: enable scripted comparisons without relying on `Debug` formatting.
  - Evidence anchors (existing): `crates/fret-ui/src/tree/mod.rs` (`debug_hit_test`), `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## P0 — Policy Normalization (Reduce Future Refactor Risk)

- [ ] OIA2-pol-020 Continue consolidating menu-like overlay invariants into shared helpers.
  - Target: normalize “visibility, hit-testability, observer flags, and occlusion” beyond dismissible popovers.
  - Evidence anchors (existing): `ecosystem/fret-ui-kit/src/window_overlays/render.rs`

## Notes

- Keep mechanism contracts in `crates/*` and policy in `ecosystem/*`.
- Prefer adding tests before large refactors; conformance tests are the “guard rails” for fearless changes.

