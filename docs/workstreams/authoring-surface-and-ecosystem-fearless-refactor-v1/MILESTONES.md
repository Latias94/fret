# Authoring Surface + Ecosystem (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Current execution stance (2026-03-12)

This workstream should now be read as a **closeout lane**, not the main place for new authoring
surface invention.

Meaning:

- keep deleting stale names and ambiguous exports,
- keep first-party docs/examples aligned with the already-decided app/component/advanced split,
- keep gates/layering checks green,
- but hand off the remaining conversion-vocabulary redesign to
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`.

If a proposed change is mainly about "too many `into_element` concepts" or "helper/component code
still falls back to raw conversion vocabulary", it belongs in the follow-on workstream rather than
reopening this one.

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

- `fret::app::prelude::*` becomes the canonical app import.
- `fret::component::prelude::*` exists and is documented.
- `fret::advanced::*` becomes the explicit place for low-level seams.

Exit criteria:

- A new app author cannot accidentally discover most advanced/runtime mechanisms through the app surface.

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
- first-party UI Gallery teaching surfaces normalized to `UiCx` so the default story no longer
  leaks `ElementContext<'_, App>`,
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
- UI Gallery source gates for first-party teaching surfaces,
- ecosystem extension-seam checks.

Exit criteria:

- Surface regressions fail fast in review/CI instead of returning through documentation drift months later.

## Post-v1 handoff — Conversion-surface closure

Outcome:

- This workstream has a named follow-on owner for the remaining conversion-vocabulary cleanup.

Deliverables:

- `docs/workstreams/into-element-surface-fearless-refactor-v1/*` stays linked from repo indexes and active workstreams.
- first-party app-facing teaching continues to prefer `Ui` / `UiChild`,
- first-party reusable component helpers stop expanding legacy conversion vocabulary while the unified conversion trait lands,
- shadcn and UI Gallery exemplars follow the active authoring target instead of re-teaching raw or legacy conversion terms by default.

Exit criteria:

- maintainers can answer "what is the next refactor after the app/component/advanced split?" without reconstructing it from chat history or commits.
