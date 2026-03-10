# Authoring Surface + Ecosystem (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Milestone 0 — Lock the target product surface

Outcome:

- We have one explicit description of the desired public authoring surface.
- Naming questions are resolved before code churn starts.

Deliverables:

- `TARGET_INTERFACE_STATE.md` finalized.
- `MIGRATION_MATRIX.md` finalized.
- Canonical names chosen for app builder, app runtime, app-facing UI context, and UI return alias.

Exit criteria:

- Maintainers can answer "what is the intended public API?" without consulting multiple historical docs.
- Maintainers can answer "what still blocks deletion?" without reconstructing it from commits or TODO bullets.

## Milestone 1 — Make imports encode the product tiers

Outcome:

- "default" vs "component" vs "advanced" becomes a real import boundary, not only a docs convention.

Deliverables:

- `fret::prelude::*` becomes app-only.
- `fret::component::prelude::*` exists and is documented.
- `fret::advanced::*` becomes the explicit place for low-level seams.

Exit criteria:

- A new app author cannot accidentally discover most advanced/runtime mechanisms through the default prelude.

## Milestone 2 — Shrink the default app mental model

Outcome:

- The default app-facing context becomes small and intention-revealing.

Deliverables:

- grouped app-facing context APIs (`state`, `actions`, `data`, `effects`),
- canonical default action/state patterns,
- removal or demotion of redundant helper variants.

Exit criteria:

- Official examples and templates can teach one small model without caveats about "other equally valid ways".

## Milestone 3 — Align the ecosystem on one extension model

Outcome:

- First-party ecosystem crates stop teaching parallel micro-frameworks.
- Third-party authors can identify the right contract tier quickly.

Deliverables:

- migration plan and implementation for:
  - `fret-ui-shadcn`
  - `fret-docking`
  - `fret-selector`
  - `fret-query`
  - `fret-router`
- clear app/component/advanced ownership per ecosystem crate.

Exit criteria:

- The same extension seams are used by first-party and expected of third-party libraries.

## Milestone 4 — Delete the old surface and clean the docs

Outcome:

- The public story becomes materially simpler because redundant names and helpers are gone.

Deliverables:

- old aliases/helpers removed,
- templates/docs/examples rewritten,
- stale docs deleted or archived.

Exit criteria:

- Reviewing the public surface no longer requires mentally subtracting "legacy-but-still-present" names.

## Milestone 5 — Keep the surface clean with gates

Outcome:

- Surface quality becomes enforceable.

Deliverables:

- prelude leakage gates,
- docs consistency gates,
- template authoring-surface gates,
- ecosystem extension-seam checks.

Exit criteria:

- Surface regressions fail fast in review/CI instead of returning through documentation drift months later.
