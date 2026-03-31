# imui shell transparent payload z-order v1 - milestones

Status: closed progress record

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-shell-transparent-payload-zorder-v1/DESIGN.md`

TODO board: `docs/workstreams/imui-shell-transparent-payload-zorder-v1/TODO.md`

Contract freeze:

- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Closeout:

- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`

## Phase A - Successor lane setup

Status: Completed

Goal:

- open the next unresolved shell-preview lane explicitly,
- keep the repo from pretending transparent moving-window overlap behavior is already closed,
- and freeze the first proof target before implementation drifts.

Deliverables:

- one new workstream directory with design/TODO/milestones,
- one explicit predecessor link from the shell ghost choreography closeout,
- one clear problem framing centered on transparent payload z-order behavior.

## Phase B - Contract freeze

Status: Completed

Goal:

- freeze the smallest overlap/z-order contract worth implementing next.

Deliverables:

- one owner split for runner truth vs docking preview expectations,
- one primary proof-surface and script pair,
- one explicit diagnostics package expectation,
- one accepted decision record:
  `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

## Phase C - Launched proof

Status: Completed

Goal:

- validate the current transparent payload overlap behavior through launched arbitration scripts
  before changing contracts further.

Deliverables:

- one bounded artifact read for the base overlap script,
- one bounded artifact read for the large-layout overlap script,
- one explicit verdict on whether current diagnostics are enough.

Progress notes:

- the base transparent-payload overlap script now passes with launched proof in
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664`,
- bounded dock-routing evidence confirms:
  `moving_window` is present,
  `window_under_moving_window` points at the overlapped main window,
  `transparent_payload_applied = true`,
  `transparent_payload_hit_test_passthrough_applied = true`,
  and `payload_ghost_visible = false`,
- the large preset failure was reduced to a diagnostics/runtime `pointer_move` stall at
  `step_index = 19`,
- the closeout proof now includes a passing single-move probe in
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774873805515-28452`,
- the full large transparent-payload z-order gate now passes in
  `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774874752791-64356`,
- and the last non-runtime failure before closeout was stale script signature text
  (`demo.viewport.extra.0` vs `demo.viewport.extra#0`), not incorrect docking state.

## Phase D - Narrow delta or successor split

Status: Completed

Goal:

- land the smallest proven contract delta, close the lane if the overlap contract is now proven,
  and avoid widening the scope after the proof has already converged.

Deliverables:

- one narrow diagnostics/runtime implementation slice,
- updated gates and evidence,
- and a closeout audit:
  `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`
