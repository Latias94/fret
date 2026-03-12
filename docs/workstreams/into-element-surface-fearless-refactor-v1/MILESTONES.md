# Into-Element Surface (Fearless Refactor v1) — Milestones

This file defines milestones for the workstream in `DESIGN.md`.

## Current execution stance (2026-03-12)

This workstream is the **current main authoring lane**.

Reason:

- the app-facing surface is already mostly converged,
- the ecosystem trait budget is already mostly decided,
- the clearest remaining "write UI" gap is still the fragmented conversion vocabulary.

Execution rule:

- prioritize M0/M1 here before reopening broader helper design elsewhere,
- use the canonical compare set
  (`simple_todo_v2_target`, `todo_demo`, scaffold simple-todo template)
  as the first downstream proof that the new conversion surface improves authoring feel,
- let ecosystem trait/docs cleanup follow this work rather than compete with it.

Current readout on 2026-03-12:

| Milestone | State | Notes |
| --- | --- | --- |
| M0 | Done | target vocabulary is locked and the classification table is now recorded in `MIGRATION_MATRIX.md` |
| M1 | Done | `IntoUiElement<H>` is the curated component conversion name; docs/preludes/tests reflect it |
| M2 | Done | `UiBuilder<T>` and host-bound child builders now land through `IntoUiElement<H>`; `UiBuilderHostBoundIntoElementExt` is deleted; child collection now also consumes `IntoUiElement<H>` directly |
| M3 | In progress | curated `fret` / `fret-ui-kit` surfaces and the canonical todo/scaffold compare set are aligned; `fret::UiChild` now lands directly through `IntoUiElement<App>`; `fret-ui-shadcn` ui_ext glue, `ui_builder_ext` helper closures, overlay/single-child builders, and `fret-router-ui` outlet helpers now land through `IntoUiElement<H>`; selected advanced examples (`assets_demo`, `async_playground_demo`, `postprocess_theme_demo`, `drop_shadow_demo`, `markdown_demo`) now also prefer `impl IntoUiElement<...>` for non-raw helpers; selected UI Gallery AI doc pages now keep page-local helpers on `impl UiChild + use<>`; broader shadcn/gallery/helper cleanup still remains |
| M4 | In progress | prelude gates are in place, curated component-authoring docs now teach only `IntoUiElement<H>`, stale-name source/doc guards now cover curated docs, and `UiChildIntoElement` is now deleted from code; `UiIntoElement` still survives as internal doc-hidden scaffolding, so the delete phase is not complete yet |

## Milestone 0 — Lock the target conversion vocabulary

Outcome:

- Maintainers can answer which conversion names belong to app, component, and advanced surfaces.

Deliverables:

- `TARGET_INTERFACE_STATE.md` finalized.
- `MIGRATION_MATRIX.md` finalized.
- one decided public name for the unified component conversion trait.

Exit criteria:

- we no longer debate whether `UiIntoElement`, `UiChildIntoElement`, and
  `UiBuilderHostBoundIntoElementExt` are all part of the intended public product surface.
- classification of current names is written down rather than implied from code comments.

## Milestone 1 — Land one public conversion contract

Outcome:

- the component surface has one obvious conversion concept.

Deliverables:

- unified public conversion trait added,
- temporary internal adapters if needed,
- `.into_element(cx)` works for both host-agnostic and host-bound builder values.

Exit criteria:

- the curated component surface can teach one trait without caveats about bridge traits.
- the landing is verified in `fret-ui-kit`, `fret`, `fret-examples`, and `fretboard`.

## Milestone 2 — Migrate builders and curated first-party surfaces

Outcome:

- the new conversion contract is proven by real first-party usage.

Deliverables:

- `UiBuilder` and child pipelines migrate to the unified contract,
- `ecosystem/fret`, `fret-ui-kit`, and selected first-party component/helper surfaces migrate,
- the canonical authoring compare set migrates together:
  `apps/fret-cookbook/examples/simple_todo_v2_target.rs`,
  `apps/fret-examples/src/todo_demo.rs`, and
  `apps/fretboard/src/scaffold/templates.rs`,
- app-facing helpers continue moving toward `UiChild`.

Exit criteria:

- first-party curated examples do not need the old public conversion names to compile or teach.
- the canonical compare set shows one consistent explicit landing story instead of three
  different ad-hoc `.into_element(cx)` patterns.
- `UiBuilderHostBoundIntoElementExt` is no longer required to recover method syntax for host-bound
  builders.

## Milestone 3 — Delete the split public conversion surface

Outcome:

- public conversion vocabulary becomes materially smaller.

Deliverables:

- old curated conversion traits removed,
- stale docs/examples rewritten,
- remaining raw `AnyElement` use is intentional and scoped.

Exit criteria:

- reviewing the public surface no longer requires mentally translating several "into element"
  concepts into one operation.
- root-level scaffolding traits that survive the milestone are explicitly justified as temporary
  compatibility shims rather than silent product surface.

## Milestone 4 — Lock the surface with gates

Outcome:

- conversion-surface regressions fail fast.

Deliverables:

- prelude export gates,
- source/doc teaching gates,
- stale-name regression gates.

Exit criteria:

- new curated surfaces cannot drift back toward the old multi-trait conversion vocabulary without
  an explicit review failure.
