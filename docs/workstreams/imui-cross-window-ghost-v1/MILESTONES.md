# imui cross-window ghost v1 - milestones

Status: closed closeout record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`

Closeout audit:

- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Successor lane:

- `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`

Predecessor closeout:

- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Phase A - Successor lane setup

Status: Completed

Goal:

- open the post-same-window ghost lane explicitly,
- keep the repo from pretending cross-window choreography is already solved,
- and freeze the immediate owner question before code changes begin.

Deliverables:

- one new workstream directory with design/TODO/milestones,
- one explicit successor link from the same-window ghost closeout,
- one clear problem framing centered on cross-window ownership and shell choreography.

Exit gates:

- readers can tell that same-window ghost is closed,
- and the next unresolved contract is named without ambiguity.

## Phase B - Contract freeze

Status: Completed

Goal:

- freeze the smallest cross-window ghost contract worth implementing.

Deliverables:

- one owner decision for generic recipe policy vs shell choreography,
- one ownership rule for which window paints the ghost,
- one fallback rule for degraded capability environments,
- one explicit transfer / no-transfer decision for preview meaning.
- one accepted decision record:
  `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Exit gates:

- the repo can explain the contract without widening runtime seams prematurely,
- same-window and cross-window responsibilities are clearly separated,
- and the fallback rule is explicit.

Closeout note:

- Phase B is now frozen by
  `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`.

## Phase C - Proof-first implementation

Status: Completed

Goal:

- land the first cross-window ghost behavior where first-party multi-window surfaces can prove it.

Deliverables:

- one smallest multi-window proof surface,
- one landed choreography decision,
- one minimal implementation slice aligned to that decision.

Exit gates:

- the proof surface shows ownership moving coherently across windows,
- duplicate ghosts are not possible under the chosen contract,
- and shell-specific behavior is not silently hardened into the generic layer.

Completion notes:

- the first proof landed on `apps/fret-examples/src/imui_editor_proof_demo.rs` using the existing
  main/aux window pair,
- the generic transfer helper remained recipe-owned as
  `publish_cross_window_drag_preview_ghost(...)`,
  `publish_cross_window_drag_preview_ghost_with_options(...)`, and
  `render_cross_window_drag_preview_ghosts(...)`,
- and the mechanism delta stayed narrow: `DragSourceResponse` gained only the identity accessors
  needed to bind recipe-owned descriptors back to a live drag session.

## Phase D - Gates and closeout/defer

Status: Completed

Goal:

- leave a durable proof/gate package and document what remains after the first cross-window slice.

Deliverables:

- one real regression gate for cross-window ownership,
- explicit defer notes for wider shell and native/external preview choreography,
- and a closeout audit once the first slice is shipped or intentionally deferred.

Exit gates:

- the new contract is reviewable through docs + proof + gate,
- remaining gaps are identified as future lanes instead of hidden backlog,
- and the owner split remains intact.

Completion notes:

- the focused compile-surface smoke coverage now lives in
  `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs` and
  `ecosystem/fret-ui-kit/tests/imui_drag_preview_smoke.rs`,
- the real interaction gate now lives in
  `ecosystem/fret-imui/src/tests/interaction.rs::tests::interaction::cross_window_drag_preview_ghost_transfers_between_windows`,
- stale descriptor pruning now intentionally waits one frame so all participating windows can
  receive the `open=false` sync before cleanup,
- and shell-aware docking/tear-out choreography is intentionally moved to the successor lane.
