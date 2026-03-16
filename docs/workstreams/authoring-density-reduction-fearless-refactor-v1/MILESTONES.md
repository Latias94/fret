# Authoring Density Reduction (Fearless Refactor v1) — Milestones

Last updated: 2026-03-16

Related:

- Design: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/DESIGN.md`
- Target interface state: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- TODO: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO.md`
- Authoring-surface closeout: `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- Action-first post-v1 summary: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- Into-element closeout target: `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

## Current status snapshot (as of 2026-03-16)

- **M0**: Met once this directory and the main docs indices land.
- **M1**: Planned (tracked-read density reduction).
- **M2**: Planned (selector/query happy-path density reduction).
- **M3**: Planned (re-evaluate keyed/list/default child-collection pressure after read-side work).
- **M4**: Planned (hard-delete displaced wording and lock the shorter path with docs/gates).

Overall reading:

- this is the next active post-v1 authoring refactor lane,
- it is not a reopen of the app/component/advanced taxonomy,
- and it is not a stealth runtime/state-architecture redesign.

## Current execution order

1. Freeze scope and evidence rules.
2. Audit repeated tracked-read ceremony and land the smallest justified shared reduction.
3. Audit selector/query ceremony and land the smallest justified shared reduction.
4. Re-measure keyed/list/default child-collection pressure after the read-side reductions.
5. Delete displaced public-looking wording and keep docs/templates/examples/gates aligned.

## Milestone 0 — Freeze the lane

Outcome:

- Maintainers agree on what this lane owns and what it does not own.

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- roadmap/docs index updates that point to this lane explicitly

Exit criteria:

- reviewers can tell that this is the next active authoring lane without reopening
  app/component/advanced or into-element design debates.

## Milestone 1 — Shorten tracked reads

Outcome:

- high-frequency tracked reads on the default path become materially shorter.

Deliverables:

- audited before/after evidence on the canonical compare set
- at least one non-todo proof surface
- one shorter default read story that still keeps invalidation intent explicit

Exit criteria:

- default-path docs/examples/templates no longer teach the previous longer read plumbing as the
  primary story,
- the new read story is still explicit enough that maintainers can reason about invalidation.

## Milestone 2 — Shorten LocalState-first selector/query authoring

Outcome:

- derived/async state feels like an extension of the same app-facing dialect instead of a jump to a
  more internal-looking style.

Deliverables:

- selector dependency/read reduction for view-owned LocalState-first examples
- query observe/read reduction for default app-facing examples
- proof that the resulting surface still keeps read-vs-write ownership explicit

Exit criteria:

- the third-rung `todo` surface is materially shorter,
- at least one additional non-todo surface benefits from the same reduction,
- the solution does not widen the default app prelude.

## Milestone 3 — Re-evaluate keyed/list/default child-collection pressure

Outcome:

- the repo decides whether any remaining list/collection noise is still a real shared-surface
  problem after the read-side reductions.

Deliverables:

- an audit pass across the canonical compare set plus at least one non-todo surface
- either:
  - a "docs/adoption only" verdict, or
  - one narrow justified shared helper/change

Exit criteria:

- maintainers can point to evidence for why list/collection pressure does or does not justify new
  public API.

## Milestone 4 — Delete the displaced path and lock the gates

Outcome:

- the shorter default path is the only taught default path.

Deliverables:

- default docs/templates/examples updated
- stale public-looking wording removed from the taught path
- source-policy/tests/gates updated for the new baseline

Exit criteria:

- the repo does not teach two co-equal default paths,
- the remaining longer wording is either gone or clearly marked as advanced/history-only.
