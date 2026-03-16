# Dataflow Authoring Surface (Fearless Refactor v1) — Milestones

Status: Active planning lane
Last updated: 2026-03-16

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`

## Current execution stance (2026-03-16)

This is the next focused authoring lane after the broader authoring closeout chain.

Read it as:

- narrow and evidence-driven,
- app-facing by default,
- explicitly compatible with editor-grade and reusable-ecosystem consumers,
- and intentionally separate from router/history scope.

## Milestone 0 — Freeze the lane

Outcome:

- maintainers agree on scope, ownership, proof surfaces, and the router boundary.

Deliverables:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`
- `MIGRATION_MATRIX.md`
- docs index / roadmap updates

Exit criteria:

- reviewers can answer what belongs here versus:
  - closed broad authoring lanes,
  - router workstreams,
  - and domain-engine work in `fret-selector` / `fret-query`.

## Milestone 1 — Collapse the action surface

Outcome:

- the default app lane has one compact action language for common local-state writes.

Deliverables:

- current action helper inventory
- chosen target surface for:
  - one-slot local writes,
  - multi-slot LocalState transactions,
  - keyed payload row writes,
  - transients,
  - explicit shared-model fallback
- migration plan for docs/templates/examples/source-gates

Exit criteria:

- the default app path no longer teaches several near-equivalent mutation helper families for the
  same common use case,
- shared-model coordination is still clearly explicit.

## Milestone 2 — Collapse the LocalState-first selector surface

Outcome:

- the default app lane has one compact LocalState-first derived-state story.

Deliverables:

- LocalState-first selector dependency/read inventory
- chosen target surface for default app authoring
- explicit advanced fallback story for shared-model dependency signatures

Exit criteria:

- default app docs/templates/examples no longer require raw dependency-builder knowledge as the
  first-contact selector story,
- reusable ecosystem crates can still use `fret-selector` directly without app-facade coupling.

## Milestone 3 — Collapse the query read-side surface

Outcome:

- query remains explicit but the default read side is materially lower-noise.

Deliverables:

- query read-pattern inventory
- chosen default read-side posture
- explicit advanced handle/state-machine fallback posture

Exit criteria:

- key/policy/fetch remain visible,
- common loading/error/success reads on the default app path are shorter,
- reusable ecosystem crates can still consume `fret-query` directly.

## Milestone 4 — Ecosystem adaptation and closeout

Outcome:

- the chosen dataflow dialect works across generic apps, editor-grade apps, and reusable ecosystem
  adapters.

Deliverables:

- ecosystem adaptation audit
- docs/templates/examples/source-policy updates
- compatibility audit against router/query/selector workstreams

Exit criteria:

- the same default teaching story appears in first-contact docs, scaffold templates, cookbook
  surfaces, and first-party demos,
- reusable ecosystem crates are not forced onto the wrong dependency tier,
- router remains compatible but separate.
