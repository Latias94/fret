# imui shell ghost choreography v1 - milestones

Status: closed progress record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-shell-ghost-choreography-v1/TODO.md`

Contract freeze:

- `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Phase A - Successor lane setup

Status: Completed

Goal:

- open the post-generic cross-window lane explicitly,
- keep the repo from pretending shell-aware choreography is already solved,
- and freeze the next owner question before code changes begin.

Deliverables:

- one new workstream directory with design/TODO/milestones,
- one explicit successor link from the generic cross-window ghost closeout,
- one clear problem framing centered on shell-aware choreography.

Exit gates:

- readers can tell that the generic cross-window baseline is closed,
- and the next unresolved shell-specific contract is named without ambiguity.

## Phase B - Contract freeze

Status: Completed

Goal:

- freeze the smallest shell-aware ghost contract worth implementing.

Deliverables:

- one owner decision for shell-aware choreography,
- one proof-surface decision for docking/workspace/viewport shells,
- one explicit rule for transient no-hover gaps or shell handoff behavior,
- one minimum diagnostics/gate package for shell regressions.
- one accepted decision record:
  `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Exit gates:

- the repo can explain the shell-specific contract without widening generic surfaces prematurely,
- generic and shell-aware responsibilities are clearly separated,
- and the proof package is explicit.

Closeout note:

- Phase B is now frozen by
  `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`.

## Phase C - Proof-first implementation

Status: Completed

Goal:

- land the first shell-aware ghost behavior where first-party shell surfaces can actually prove it.

Deliverables:

- one smallest shell-aware proof surface,
- one landed shell choreography decision,
- one minimal implementation slice aligned to that decision.

Exit gates:

- the proof surface shows coherent ghost ownership through a shell transition,
- duplicate or missing ghosts are not possible under the chosen contract,
- and generic recipe behavior is not silently hardened into shell-specific policy.

Progress notes:

- the first landed slice is docking-local rather than generic:
  `ecosystem/fret-docking/src/dock/{space,paint}.rs`
- current behavior paints a tab-shaped payload ghost only for the runner-selected
  `current_window`, and suppresses that ghost once a real `moving_window` exists,
- diagnostics now publish that shell-local visibility explicitly through
  `DockDragDiagnostics.payload_ghost_visible` and the
  `dock_drag_payload_ghost_visible_is` predicate,
- scene-level regression coverage now lives in
  `ecosystem/fret-docking/src/dock/tests/drop_hints.rs`,
- and the docking arbitration demo / diag path now gates both sides of the contract:
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-reorder-two-tabs.json`
  proves `visible = true` for in-window drag,
  while
  `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
  proves `visible = false` once `moving_window` takes ownership.

Closeout note:

- Phase C is now considered complete by the launched diag evidence recorded in
  `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`.

## Phase D - Gates and closeout/defer

Status: Completed

Goal:

- leave a durable proof/gate package and document what remains after the first shell-aware slice.

Deliverables:

- one real regression artifact for shell-aware choreography,
- explicit defer notes for native/external preview and wider transport questions,
- and a closeout audit once the first slice is shipped or intentionally deferred.

Exit gates:

- the new shell-specific contract is reviewable through docs + proof + gate,
- remaining gaps are identified as future lanes instead of hidden backlog,
- and the owner split remains intact.

Closeout note:

- Phase D is now complete via
  `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`.
