# Local-State Architecture (Fearless Refactor v1) — Milestones

Last updated: 2026-03-16

Related:

- Design: `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/local-state-architecture-fearless-refactor-v1/TODO.md`
- Invariant matrix: `docs/workstreams/local-state-architecture-fearless-refactor-v1/INVARIANT_MATRIX.md`
- Surface classification: `docs/workstreams/local-state-architecture-fearless-refactor-v1/SURFACE_CLASSIFICATION_2026-03-16.md`
- Action-first closeout: `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Action-first endgame summary: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- Authoring-density closeout: `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- ADR 0308: `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

---

## Current status snapshot (as of 2026-03-16)

This lane exists because the broad authoring-reset work is already closed.

- **M0**: Met (the workstream is opened, indexed, and scoped as a separate architecture lane).
- **M1**: Met (invariant matrix + surface classification now freeze what is genuinely
  architectural versus what is already closed or intentionally explicit).
- **M2**: Planned (compare architecture options without coding first).
- **M3**: Planned only if justified (smallest proof surface + gates).
- **M4**: Planned (close the lane or spin out a narrower implementation lane).

Execution rule:

- do not treat this as another helper-growth lane,
- and do not start code refactors before M1/M2 have actually frozen the contract question.

---

## Milestone 0 — Open the lane

Exit target:

- the repo has one explicit place to discuss the long-term `LocalState<T>` contract,
- the lane is linked from the main docs indices,
- and adjacent closed workstreams no longer have to carry this question as an inline footnote.

Initial result:

- `DESIGN.md`, `TODO.md`, and `MILESTONES.md` now exist,
- `docs/README.md`, `docs/roadmap.md`, and `docs/workstreams/README.md` point to the lane,
- and the workstream is framed as contract-first rather than code-first.

## Milestone 1 — Freeze invariants and boundaries

Exit target:

- one invariant matrix exists,
- one ownership classification exists,
- and the repo can distinguish architecture pressure from docs/adoption drift.

Current result (2026-03-16):

- `INVARIANT_MATRIX.md` now records the non-negotiable constraints,
- `SURFACE_CLASSIFICATION_2026-03-16.md` now classifies current pressure into architecture vs
  already-closed default-path work vs intentional hybrid/runtime-owned seams,
- and M2 can therefore focus on option comparison rather than on rediscovering the same evidence.

Key questions:

- which current `LocalState<T>` properties are non-negotiable?
- which explicit `Model<T>` seams are intentional?
- which pain points are really storage/ownership questions rather than authoring-density residue?

## Milestone 2 — Choose the direction

Exit target:

- the repo explicitly chooses whether to:
  - keep the current model-backed contract,
  - harden the facade only,
  - or open a narrower alternative-storage prototype.

Decision rule:

- no option passes unless it preserves explicit invalidation, diagnostics, selector/query layering,
  and shared-model interop.

## Milestone 3 — Prove the chosen direction

Exit target:

- only if M2 chooses a code path,
- one smallest proof batch exists with tests/gates and clear non-goals.

Scope rule:

- require one default-path proof plus one hybrid proof,
- and keep advanced/runtime-owned boundaries explicit instead of pretending everything should look
  like a toy app.

## Milestone 4 — Close cleanly

Exit target:

- either the repo records that the current contract stands,
- or the chosen new direction is documented/gated and any follow-on work is spun into a narrower
  implementation lane.

Definition of done:

- no ambiguous “maybe later” wording remains inside already-closed authoring workstreams,
- and this lane itself no longer mixes decision-making with unrelated sugar growth.
