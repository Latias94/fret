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

Closeout note on 2026-03-12:

- the ecosystem trait budget is now largely settled rather than open-ended,
- `fret-ui-shadcn` helper glue (`ui_ext/*`) is already on `IntoUiElement<H>`-based surfaces,
- `fret-ui-shadcn` `ui_builder_ext/*::into_element(...)` methods are now treated as intentional
  explicit landing seams rather than unfinished helper-return migrations,
- remaining work is mainly continued first-party helper cleanup and broader conversion-surface
  closure under the dedicated into-element workstream.

Closeout note on 2026-03-15:

- the app/component/advanced split is now effectively the settled first-party posture,
- the highest-value first-party docs/examples/gates are already aligned to that posture,
- selector/query helper nouns now live on explicit `fret::selector::*` / `fret::query::*` lanes
  instead of widening `fret::app::prelude::*`,
- raw `on_activate*` helper glue is now also explicit on `fret::activate::{...}` instead of
  widening `fret::component::prelude::*`,
- but the closeout is **not** complete while:
  - shadcn first-contact discovery still depends on source-policy tests to keep crate-root/facade
    lanes from competing in first-party teaching surfaces,
  - status docs imply the surface is more closed than the active conversion-surface tracker
    actually allows,
- any still-unchecked early TODO bullets in this folder should now be read as either:
  - historical bookkeeping residue, or
  - one of the targeted surface-closeout tasks above.

Closeout note on 2026-03-16:

- the default `fret` app surface is now effectively in the "docs/export hygiene" phase rather
  than the "public product surface redesign" phase,
- `fret::app::prelude::*` is down to app nouns plus intentional hidden ergonomic helper imports;
  the remaining export audit is primarily on `fret::component::prelude::*` and
  `fret::advanced::prelude::*`, not on the default app lane,
- `fret-ui-kit` no longer leaks into the default `fret` app lane through transitive prelude
  forwarding; any future slimming of `fret_ui_kit::prelude::*` is a crate-local follow-up rather
  than a blocker for the `fret` facade closeout,
- the latest shadcn overlay-chrome parity sweep is green again
  (`cargo test -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome`:
  `20 passed; 0 failed`), so shadcn closeout is back to discovery/docs/gate maintenance rather
  than active recipe drift.
- the app-entry builder workstream docs now match the shipped builder surface again:
  `FretApp` keeps `view::<V>()?` / `view_with_hooks::<V>(...)?` plus `.run()`, while the removed
  `run_view*` convenience is now documented only as deletion history rather than live posture.
- the async integration guides (`Tokio/Reqwest`, `SQLite/SQLx`) are now fully on the
  "default `cx.data().query_*` path first, raw `ElementContext` note second" posture, so that
  authoring-surface matrix row is no longer an active blocker.
- `fret::advanced::prelude::*` now also stops silently forwarding the whole component lane:
  advanced/manual-assembly examples that still need component authoring helpers add
  `use fret::component::prelude::*;` explicitly, and the corresponding migration-matrix row is now
  in the closeout/maintenance state rather than active redesign.
- `fret::component::prelude::*` also no longer forwards environment/responsive helper families;
  reusable component code now reaches breakpoint/media/pointer/safe-area helpers through the
  explicit `fret::env::{...}` lane instead of rediscovering those helpers through wildcard
  component imports.
- the `Component prelude` migration-matrix row is now also considered migrated:
  the `fret` facade no longer widens that lane with env/activation/overlay-heavy vocabulary, and
  the component-author docs now lock `fret::env::{...}`, `fret::activate::{...}`, and
  `fret::overlay::*` as explicit secondary lanes instead of wildcard-prelude discoveries.
- that advanced-import split is now also validated end-to-end by the first-party source gates:
  `cargo check -p fret-examples --all-targets` and
  `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app` both pass with
  explicit dual imports on the advanced examples/gallery surfaces that still need ordinary
  component authoring vocabulary.
- the remaining release-blocking work is therefore not a new lane redesign; it is the final
  product-surface closeout:
  - reduce `fret-ui-shadcn` discovery pressure to one taught lane,
  - keep the `fret` root from reading like a second prelude,
  - hand the remaining density/ceremony reductions to the action-first and conversion follow-ons,
  - and keep `AppActivateExt` on a shrinking bridge-only trajectory.
- fresh audits that still call out app/component overlap should therefore be treated as a request
  to finish lane budgeting and discovery curation, not as a reason to reopen the
  app/component/advanced taxonomy itself.
- ecosystem integration-trait budgeting remains a real follow-on, but it should read from the
  final lane story after this closeout rather than compete with the closeout itself.

If a proposed change is mainly about "too many `into_element` concepts" or "helper/component code
still falls back to raw conversion vocabulary", it belongs in the follow-on workstream rather than
reopening this one.

## Current next-step order (2026-03-16)

1. Close `fret-ui-shadcn` discovery-lane duplication.
   Exit criteria: first-party docs/gallery teach only `facade as shadcn` for component families,
   `raw::*` is the only explicit escape hatch, and the hidden flat root is treated purely as
   compatibility residue.
2. Freeze the `fret` root lane budget.
   Exit criteria: the root still offers explicit opt-in lanes (`assets`, `env`, `router`,
   `docking`, etc.), but default app authors can stay on `fret::app::prelude::*` without new root
   vocabulary growth or new lane drift.
3. Resume happy-path ceremony reduction only after the two lane-curation items above are stable.
   Exit criteria: the default todo/first-hour path gets materially shorter on tracked reads,
   local/payload writes, and keyed/list composition without reopening mechanism/policy confusion.
4. Treat `AppActivateExt` as shrinking bridge residue, not as a growth surface.
   Exit criteria: no new first-party bridge impls are added for widgets that can expose native
   action slots; the remaining bridge list is intentional activation-only residue and is tracked as
   such in docs/gates.
5. Resume ecosystem integration-trait budgeting only after items 1-4 are stable.
   Exit criteria: install/router/query/docking/catalog trait seams are reviewed against the final
   `fret` / `fret-ui-shadcn` lane story instead of an interim pre-closeout discovery posture.

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
- the app prelude stops exporting helper families whose main audience is reusable component
  authors.

Exit criteria:

- A new app author cannot accidentally discover most advanced/runtime mechanisms through the app
  surface.
- A new app author also should not have to sort through most component-author styling/layout/raw
  seams before reaching the default app nouns.

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
- one taught first-contact lane for `fret-ui-shadcn`
  (`use fret_ui_shadcn::{facade as shadcn, prelude::*};`) with raw/advanced seams explicit and the
  flat crate root removed from component-family discovery.

Exit criteria:

- The same extension seams are used by first-party and expected of third-party libraries.
- First-party shadcn teaching surfaces no longer need to compensate for multiple peer discovery
  lanes because component-family flat-root exports are gone.
  lanes beyond the explicitly documented raw/advanced escape hatches.

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
- 2026-03-15 follow-up: UI Gallery snippet/page surfaces no longer route through
  `fret_ui_shadcn::*` flat root/module paths for shadcn authoring,
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
