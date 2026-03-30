# imui cross-window ghost v1 - TODO

Status: active board

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`

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

- [ ] Decide whether the next public generic surface should remain recipe-owned or stop at
      same-window while shell layers own transfer choreography.
- [ ] Decide which window owns ghost rendering when the pointer is over another Fret window.
- [ ] Decide how the source window hides, keeps, or hands off its ghost during cross-window hover.
- [ ] Decide whether the hovered window needs a transferred preview descriptor or whether source
      ownership can remain authoritative.
- [ ] Decide the fallback rule for single-window / unreliable cross-window hover environments.

## M2 - Lock the first proof surface and smallest gate

- [ ] Choose the smallest first-party multi-window proof surface.
      Current preference: `apps/fret-examples/src/imui_editor_proof_demo.rs`.
- [ ] Decide whether the first proof should use the existing main/aux window pair before any
      docking-specific surface.
- [ ] Define one regression gate that can prove ghost ownership across windows without relying on
      screenshots alone.
- [ ] Record the minimum evidence anchors for source window, hovered window, and no-duplicate rule.

## M3 - Implementation and explicit defers

- [ ] Land only the smallest mechanism delta that is proven necessary by the proof surface.
- [ ] Keep shell choreography out of `fret-ui-kit::imui`.
- [ ] Keep any generic recipe surface narrow and explicit if it survives the proof.
- [ ] Record explicit deferred items after the first landed slice:
      docking tear-out choreography,
      aggregate previews,
      native/external preview surfaces,
      richer descriptor transport.
- [ ] Capture a closeout audit once the first cross-window slice is shipped or intentionally
      deferred.
