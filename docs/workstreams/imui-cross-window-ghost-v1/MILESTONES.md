# imui cross-window ghost v1 - milestones

Status: active progress record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`

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

Status: In progress

Goal:

- freeze the smallest cross-window ghost contract worth implementing.

Deliverables:

- one owner decision for generic recipe policy vs shell choreography,
- one ownership rule for which window paints the ghost,
- one fallback rule for degraded capability environments,
- one explicit transfer / no-transfer decision for preview meaning.

Exit gates:

- the repo can explain the contract without widening runtime seams prematurely,
- same-window and cross-window responsibilities are clearly separated,
- and the fallback rule is explicit.

## Phase C - Proof-first implementation

Status: Planned

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

## Phase D - Gates and closeout/defer

Status: Planned

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
