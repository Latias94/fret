# imui shell ghost choreography v1 - TODO

Status: closed board (historical closeout evidence)

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-shell-ghost-choreography-v1/MILESTONES.md`

Contract freeze:

- `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Lane setup and successor freeze

- [x] Create the workstream directory and initial design/TODO/milestones pack.
- [x] Freeze this lane as the direct successor to the generic cross-window ghost closeout.
- [x] Record the primary problem statement:
      shell-aware docking/workspace/tear-out ghost choreography,
      not generic cross-window descriptor transfer.
- [x] Record the first explicit non-goals:
      no compatibility aliases,
      no native/external preview surface,
      no generic recipe-policy widening without proof.

## M1 - Freeze the shell-specific contract questions before code

- [x] Decide which shell-aware layer should own the first landed choreography slice.
- [x] Decide whether the first proof must start in docking, workspace shells, or another viewport
      host surface.
- [x] Decide how transient no-hover gaps should behave during shell transitions.
- [x] Decide whether shell layers need a dedicated host/controller or only a wrapper around the
      generic recipe transfer helper.
- [x] Decide the minimum diagnostics/gate package needed for shell choreography regressions.
      Captured in
      `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`.

## M2 - Lock the first proof surface and smallest gate

- [x] Choose the smallest first-party shell-aware proof surface.
      Result: `apps/fret-examples/src/docking_arbitration_demo.rs`.
- [x] Record why the generic main/aux proof is insufficient for this next lane.
      Result: it cannot exercise `moving_window` / `window_under_moving_window` choreography.
- [x] Define one regression gate or scripted repro that can prove handoff/no-duplicate behavior
      across a shell transition.
      Result:
      `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`.
- [x] Record the minimum evidence anchors for source window, shell transition, and final owner
      window behavior.
      Anchors:
      `apps/fret-examples/src/docking_arbitration_demo.rs`,
      `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`,
      `ecosystem/fret-docking/src/dock/space.rs`,
      `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`.

## M3 - Implementation and explicit defers

- [x] Land only the smallest shell-aware delta proven necessary by the proof surface.
      Landed first slice:
      docking now paints a tab-shaped payload ghost only when `current_window == self.window`
      and `moving_window.is_none()`.
- [x] Keep generic transfer policy out of shell-specific code and keep shell policy out of
      `fret-ui-kit::imui`.
- [x] Keep any shell-facing public API narrow and explicit if it survives the proof.
      Result: no new public API was introduced for the first slice.
- [x] Thread shell-local payload ghost visibility into diagnostics so scripted gates can assert
      visible-in-current-window vs suppressed-under-`moving_window` behavior.
      Result:
      `crates/fret-runtime/src/interaction_diagnostics.rs`,
      `ecosystem/fret-bootstrap/src/ui_diagnostics/{docking_diagnostics,predicates}.rs`,
      `crates/fret-diag-protocol/src/lib.rs`.
- [x] Record explicit deferred items after the first landed shell-aware slice:
      aggregate previews,
      native/external preview surfaces,
      wider descriptor transport,
      non-docking shell families that remain unproven.
- [x] Add a first-party shell proof/gate read that exercises the landed slice through the docking
      arbitration demo / diag path instead of scene-only crate tests.
      Result:
      `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-reorder-two-tabs.json`
      asserts `dock_drag_payload_ghost_visible_is = true`,
      and
      `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
      asserts `dock_drag_payload_ghost_visible_is = false` once `moving_window` owns the drag.
- [x] Capture a closeout audit once the first shell-aware slice is shipped or intentionally
      deferred.
      Result:
      `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`.
