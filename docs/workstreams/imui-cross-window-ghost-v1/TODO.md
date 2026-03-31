# imui cross-window ghost v1 - TODO

Status: closed board

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`

Closeout audit:

- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Successor lane:

- `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`

Contract freeze:

- `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Lane setup and successor freeze

- [x] Create the workstream directory and initial design/TODO/milestones pack.
- [x] Freeze this lane as the direct successor to the same-window ghost closeout.
- [x] Record the primary problem statement:
      cross-window ghost ownership and shell choreography,
      not same-window ghost styling.
- [x] Record the first explicit non-goals:
      no native/external drag image,
      no process-boundary preview transport,
      no compatibility aliases.

## M1 - Freeze the contract questions before code

- [x] Decide whether the next public generic surface should remain recipe-owned or stop at
      same-window while shell layers own transfer choreography.
- [x] Decide which window owns ghost rendering when the pointer is over another Fret window.
- [x] Decide how the source window hides, keeps, or hands off its ghost during cross-window hover.
- [x] Decide whether the hovered window needs a transferred preview descriptor or whether source
      ownership can remain authoritative.
- [x] Decide the fallback rule for single-window / unreliable cross-window hover environments.
- [x] Capture the accepted decision record in
      `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`.

## M2 - Lock the first proof surface and smallest gate

- [x] Choose the smallest first-party multi-window proof surface.
      Result: `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- [x] Decide whether the first proof should use the existing main/aux window pair before any
      docking-specific surface.
      Result: yes; the generic slice lands on the existing main/aux proof first.
- [x] Define one regression gate that can prove ghost ownership across windows without relying on
      screenshots alone.
      Landed as
      `ecosystem/fret-imui/src/tests/interaction.rs::tests::interaction::cross_window_drag_preview_ghost_transfers_between_windows`.
- [x] Record the minimum evidence anchors for source window, hovered window, and no-duplicate rule.
      Anchors:
      `apps/fret-examples/src/imui_editor_proof_demo.rs`,
      `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`,
      `ecosystem/fret-imui/src/tests/interaction.rs`.

## M3 - Implementation and explicit defers

- [x] Land only the smallest mechanism delta that is proven necessary by the proof surface.
- [x] Keep shell choreography out of `fret-ui-kit::imui`.
- [x] Keep any generic recipe surface narrow and explicit if it survives the proof.
- [x] Record explicit deferred items after the first landed slice:
      docking tear-out choreography,
      aggregate previews,
      native/external preview surfaces,
      richer descriptor transport.
- [x] Capture a closeout audit once the first cross-window slice is shipped or intentionally
      deferred.
