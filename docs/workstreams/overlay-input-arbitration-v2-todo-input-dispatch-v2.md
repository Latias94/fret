# Overlay & Input Arbitration v2 (Pointer Occlusion) — TODO Tracker (input-dispatch-v2 worktree)

Status: Active (branch/worktree-local tracker to avoid doc ownership conflicts)

This tracker exists because `docs/workstreams/overlay-input-arbitration-v2-todo.md` is maintained on
`main` by another parallel workstream. Keep this file as the single source of truth for work done
in the `input-dispatch-v2` worktrees/branches; when backporting to `main`, reconcile into the main
tracker.

- Narrative progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- Cross-cutting contracts (phases / prevent_default / availability): `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`

## Tracking Format

Each TODO is labeled:

- ID: `OIA2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Building Blocks)

- Pointer occlusion mechanism exists (`PointerOcclusion` stored on `UiLayer`).
  - Evidence: `crates/fret-ui/src/tree/layers.rs`
- Hit-tested pointer routing respects occlusion while observer policy can still request pointer-move observers.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`
- Capture revocation dispatches deterministic `PointerCancel` reasons.
  - Evidence: `crates/fret-core/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`

## P0 — Conformance Suite Expansion (Hard-to-change Boundaries)

- [x] OIA2-test-001 Dock drag closes/hides overlays (ADR 0072 baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- [x] OIA2-test-002 Expand docking drag × overlay hygiene edges (ADR 0072).
  - Target: cover “start drag while submenu open / while capture active / cross-window hover window”.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (cross-window drag coverage)
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (submenu + dock drag conformance)
- [x] OIA2-test-003 Viewport tool capture suppresses hover overlays/tooltips (ADR 0049 follow-up baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- [x] OIA2-test-004 “Present vs interactive” close-transition invariants are locked by tests.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`

## P0 — Diagnostics & Debuggability

- [x] OIA2-diag-010 Hit-test scope roots are exposed in diagnostics (baseline).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- [x] OIA2-diag-011 Stable labels for scope roots (avoid relying on `Debug` formatting).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.hit_test.scope_roots[].label`)
- [x] OIA2-diag-012 Stable layer identifiers and occlusion labels are exposed in diagnostics.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.layers_in_paint_order[].layer_id`, `pointer_occlusion`)
- [x] OIA2-diag-013 Snapshot input arbitration state (modal / occlusion / capture) for scripted comparisons.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.input_arbitration`)

## P0 — Policy Normalization (Reduce Future Refactor Risk)

- [~] OIA2-pol-020 Consolidate menu-like overlay invariants into shared helpers.
  - Target: minimize per-overlay special casing (visibility, hit-testability, observer flags, occlusion lifecycle).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`non_modal_overlay_effective_interactive`)

## Notes

- `PointerOcclusion` is a routing/scope mechanism; it should remain orthogonal to dispatch phases and `prevent_default`.
- Keep mechanism contracts in `crates/*` and policy in `ecosystem/*`.
- Prefer adding/expanding conformance tests before larger refactors.
