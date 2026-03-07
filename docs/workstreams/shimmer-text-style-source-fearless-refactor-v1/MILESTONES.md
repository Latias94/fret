# Shimmer Text Style Source (Fearless Refactor v1) — Milestones

Status: In progress.

Primary contract references:

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`
- `docs/adr/0315-shimmer-resolved-text-style-source-v1.md`

## Milestone 0 — Gap closure audit

Goal:

- prove the remaining gap is mechanism-level,
- classify `Shimmer` call sites,
- avoid overdesign.

Deliverables:

- small workstream doc set,
- call-site inventory,
- explicit list of surfaces that should stay explicit vs consume subtree-resolved style.

Exit criteria:

- the repo has one written explanation for the remaining post-plan `Shimmer` call-site audit.

## Milestone 1 — Resolved passive text source contract

Goal:

- define the minimum mechanism needed for visual-text recipes to consume inherited text style.

Deliverables:

- ADR 0315 proposed,
- chosen contract outcome documented,
- evidence anchors pointing to the current gap (`plan.rs`, `shimmer.rs`).

Exit criteria:

- it is clear what contract belongs in `crates/fret-ui` and what stays in `fret-ui-ai`.

## Milestone 2 — `Shimmer` compatibility bridge

Goal:

- keep explicit `.text_style(...)` working while enabling subtree-resolved mode.

Deliverables:

- `Shimmer` supports an inherited / resolved style source,
- explicit compatibility path retained for demos / visual overrides,
- focused regression tests for base vs overlay style parity.

Exit criteria:

- `Shimmer` can be used without manual card-title/card-description `TextStyle` assembly when the
  subtree already encodes that intent.

## Milestone 3 — Streaming semantic migration

Goal:

- remove the semantic split between streaming and non-streaming AI plan text.

Deliverables:

- `PlanTitle` streaming path aligned with `CardTitle`,
- `PlanDescription` streaming path aligned with `CardDescription`,
- TODO / ADR alignment docs updated.

Exit criteria:

- streaming and non-streaming plan text teach the same typography contract.

## Milestone 4 — Guardrails and cleanup

Goal:

- lock in the new path and prevent new manual style assembly drift.

Deliverables:

- tests / diag gates covering the `Shimmer` bridge,
- audit of remaining explicit `Shimmer::text_style(...)` uses,
- cleanup notes for any temporary compatibility paths.

Exit criteria:

- the repo can distinguish intentional explicit visual text from accidental semantic duplication.
