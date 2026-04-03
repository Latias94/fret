# Execution mode selection

Use this note before deciding how much process a task needs.

The goal is not to force every task into a workstream. The goal is to make the process weight match
the risk, continuation cost, and verification needs of the change.

## The three modes

### `Fast`

Choose `Fast` when all of the following are true:

- the task is trivial or obviously local,
- it stays inside one small surface,
- it does not change a hard contract or repo-level invariant,
- and it does not need a new repro/gate artifact to be reviewable.

Typical examples:

- wording/docs cleanup,
- typo or naming fixes,
- a narrow local bug where the existing tests/scripts already prove the behavior,
- a small mechanical cleanup with no continuation value.

What to leave behind:

- the code/doc change,
- the smallest verification already available,
- and concise evidence in the final response or commit.

Do not open or expand a workstream just to legitimize a tiny task.

### `Quick slice`

Choose `Quick slice` by default for non-trivial work when one bounded slice can still land cleanly.

Signals:

- one user-facing invariant or one technical concern,
- one smallest repro target,
- one gate,
- one evidence set,
- no need for multi-session lane bookkeeping.

Typical examples:

- focused behavior bug fix,
- parity adjustment,
- one reviewable refactor step,
- one targeted framework improvement with a clear before/after gate.

What to leave behind:

- Repro,
- Gate,
- Evidence.

If the slice is hard to explain without session history, it is probably not a `Quick slice` anymore.

### `Workstream`

Choose `Workstream` when the task needs explicit state across slices or maintainers.

Signals:

- multiple landable slices are expected,
- the task changes a hard-to-reverse contract,
- ADR or alignment documents must move with the code,
- broad refactor risk makes continuation decisions non-obvious,
- or future agents need explicit closeout versus follow-on guidance.

Typical examples:

- fearless refactor lanes,
- large authoring-surface changes,
- diagnostics/perf infrastructure evolution,
- framework contract migrations that will take several passes.

What to leave behind:

- explicit lane state,
- authoritative docs,
- repro/gate/evidence per slice,
- closeout or follow-on guidance when the lane changes status.

Use `fret-workstream-lifecycle` for this mode.

## Escalation rules

- Start at the lightest mode that still leaves trustworthy evidence.
- Escalate from `Fast` to `Quick slice` when a new regression artifact is needed.
- Escalate from `Quick slice` to `Workstream` when the task needs state, staged decisions, or cross-session continuity.
- Do not de-escalate a true workstream into ad hoc chat notes.

## Fret-specific bias

For Fret, process weight should track contract risk:

- UI recipe/policy fixes often fit `Quick slice`.
- `crates/fret-ui` or cross-crate contract work often wants `Workstream`.
- diag/perf additions should usually at least be `Quick slice`, because they need durable artifacts.
- ADR-affecting changes should generally be treated as `Workstream`, even if the code diff is not large.
