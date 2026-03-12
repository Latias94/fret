# Into-Element Surface — Migration Matrix

Status: execution tracker
Last updated: 2026-03-12

This matrix tracks how the current conversion surface should move toward the target state.

It focuses on:

- public trait vocabulary,
- helper return types,
- first-party teaching surfaces,
- and which old names become delete-ready.

## Status Legend

| Status | Meaning |
| --- | --- |
| Not started | no migration code landed |
| Scaffolding only | a new path exists, but public teaching/call sites still use the old path |
| In progress | first-party migration is underway |
| Migrated | official first-party call sites use the new path |
| Delete-ready | migrated and guarded; old path can be removed |
| Deleted | old public path is gone |

## Global Deletion Rule

An old conversion name is eligible for deletion only when all of the following are true:

1. app docs/templates no longer teach it,
2. component docs/examples no longer teach it,
3. first-party reusable crates no longer depend on it as public API,
4. a gate exists to prevent it from reappearing on curated surfaces.

## Current Name Classification (2026-03-12)

| Name | Intended posture | Current reality | Status |
| --- | --- | --- | --- |
| `Ui` | keep publicly on the app surface | app-facing alias over `Elements` | Kept publicly |
| `UiChild` | keep publicly on the app surface | app-owned marker over `UiChildIntoElement<App>` | Kept publicly |
| `IntoUiElement<H>` | keep publicly on the component surface | curated conversion name on `fret-ui-kit` / `fret::component::prelude::*` | Kept publicly |
| `AnyElement` | keep publicly as an explicit raw type | still legal and intentional on advanced/raw seams | Moved to advanced/raw only |
| `Elements` | keep publicly as an explicit raw type; teach `Ui` instead on the app surface | still present as the raw container type behind `Ui` | Moved to advanced/raw only |
| `UiIntoElement` | stop teaching publicly; keep only as temporary implementation scaffolding | still public at crate root but no longer curated on `fret::component::prelude::*` | Kept internally only (target) / public scaffolding (current) |
| `UiHostBoundIntoElement<H>` | stop teaching publicly; compatibility bridge only | deleted from code; no curated or root-level export remains | Deleted |
| `UiChildIntoElement<H>` | stop teaching publicly; app-internal/component-internal mechanism only | still public at `fret-ui-kit` root, but now only as a thin child-pipeline bridge over `IntoUiElement<H>` | Kept internally only (target) / public scaffolding (current) |
| `UiBuilderHostBoundIntoElementExt<H>` | hidden bridge only, then delete | deleted from code; method syntax now lands through `IntoUiElement<H>` directly | Deleted |
| legacy split public conversion vocabulary | delete from curated product surfaces | already absent from curated `fret` component exports; root-level cleanup remains | Deleted entirely from curated surfaces / not yet deleted from roots |

## Surface Lanes

| Lane | Current surface | Target surface | Migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| App render return | `Ui = Elements` alias already exists, but raw `Elements` still appears in some checks and historical docs | keep `Ui` as the app-facing render alias | continue treating `Ui` as canonical and delete stale `Elements` teaching where it survives | default app docs/examples only teach `Ui` | Migrated | `ecosystem/fret/src/lib.rs`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md` |
| App helper child return | `UiChild` already exists as an app-owned marker over `UiChildIntoElement<App>` | keep `UiChild` as the only app-facing child concept | migrate app-facing helper docs/examples to `impl UiChild` and stop teaching the underlying trait | default app docs/examples never spell `UiChildIntoElement<App>` | Migrated | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs` |
| Component conversion contract | public split across `UiIntoElement`, `UiHostBoundIntoElement<H>`, and `UiChildIntoElement<H>` | one public conversion trait generic over `H: UiHost` | introduce unified trait, temporarily adapt old impls, then delete the old public names | curated component prelude exports only one public conversion trait | In progress | `ecosystem/fret/src/lib.rs`, `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs`, `docs/component-authoring-contracts.md` |
| Host-bound builder landing | host-bound builders previously needed `UiBuilderHostBoundIntoElementExt<H>` to recover `.into_element(cx)` syntax | method syntax provided through the unified conversion trait | move host-bound builder landing behind the new public trait and keep any extra bridging internal | app/component preludes stop importing the old extension trait | Migrated | `ecosystem/fret/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui_builder.rs` |
| Child pipelines | `ui::children!` and container helpers still name `UiChildIntoElement<H>`, but that trait is now only a thin bridge over `IntoUiElement<H>` | heterogeneous child collection consumes the unified contract semantics without parallel component-specific impls | keep the child bridge thin while deleting duplicate component impls and migrating teaching surfaces away from the old name | no curated child helper depends on `UiChildIntoElement<H>` publicly | In progress | `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-kit/src/ui.rs`, `ecosystem/fret-ui-kit/src/imui.rs`, `ecosystem/fret-ui-shadcn/src/` |
| Component helper signatures | first-party reusable snippets often return `AnyElement` even when raw landing is not conceptually required | generic helpers prefer `impl IntoUiElement<H>` | migrate first-party reusable helpers opportunistically during snippet/component audits | first-party reusable docs/snippets reserve `AnyElement` for justified raw seams | Not started | `apps/fret-ui-gallery/src/ui/snippets/`, `ecosystem/fret-ui-shadcn/src/` |
| App helper signatures | most official app surfaces already prefer `Ui` and `UiCx`, but some helpers still land raw children | app-facing helpers prefer `impl UiChild` | continue app-surface cleanup in cookbook/examples/gallery teaching surfaces | app teaching surfaces no longer need raw child return types by default | In progress | `apps/fret-cookbook/examples/`, `apps/fret-examples/src/`, `apps/fretboard/src/scaffold/templates.rs`, `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` |
| Raw explicit IR | `AnyElement` and `Elements` are used widely in low-level helpers, tests, overlays, diagnostics, and manual assembly | retain raw types explicitly on advanced/internal surfaces | document raw use as intentional rather than accidental | raw surfaces are clearly documented as advanced/internal rather than default teaching | In progress | `ecosystem/fret/src/lib.rs`, `crates/fret-ui/src/`, `apps/fret-ui-gallery/src/driver/` |

## Hard Delete Matrix

| Old name / posture | Replacement | Delete when | Status |
| --- | --- | --- | --- |
| `UiIntoElement` as curated public conversion vocabulary | unified trait (`IntoUiElement<H>` or final equivalent) | component prelude only exports the unified trait and first-party reusable code is migrated | In progress |
| `UiHostBoundIntoElement<H>` as curated public conversion vocabulary | unified trait (`IntoUiElement<H>` or final equivalent) | host-bound builders land through the unified trait and no curated docs teach the split | Deleted |
| `UiChildIntoElement<H>` as curated public conversion vocabulary | unified trait for component code, `UiChild` for app-facing helpers | child pipelines and curated docs no longer require the old trait name | Not started |
| `UiBuilderHostBoundIntoElementExt<H>` as curated public bridge trait | unified trait-backed method syntax | app/component preludes stop importing the old bridge and first-party code compiles through the unified trait | Deleted |
| `AnyElement` as the default first-contact helper return type in app docs/examples | `impl UiChild` or `Ui` as appropriate | app-facing docs/examples are migrated and source gates are in place | In progress |

## Recommended Execution Order

| Order | Track | Why |
| --- | --- | --- |
| 1 | unify the public conversion contract | everything else depends on one target concept |
| 2 | migrate builder/macro landing paths | reduces churn for downstream call sites |
| 3 | migrate curated component surfaces | proves the new trait is sufficient for real reusable code |
| 4 | migrate app-facing helper teaching | sharpens the default product surface |
| 5 | delete old conversion names and add gates | locks the cleanup so drift cannot return |

## Completion Rule

This workstream is complete when:

- app-facing docs teach `Ui` / `UiChild`,
- component-facing docs teach one public conversion trait,
- raw `AnyElement` use is clearly advanced/internal rather than default teaching,
- the old split conversion traits are deleted from curated public surfaces.
