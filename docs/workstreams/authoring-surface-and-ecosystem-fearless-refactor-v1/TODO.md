# Authoring Surface + Ecosystem (Fearless Refactor v1) — TODO

This TODO list tracks the work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete the old surface rather than
carrying compatibility-only baggage.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

## M0 — Freeze the target product surface

- [ ] Finalize `TARGET_INTERFACE_STATE.md` as the single source of truth for the desired public surface.
- [x] Finalize `MIGRATION_MATRIX.md` as the single execution tracker for old surface removal.
- [ ] Decide and lock the canonical names:
  - [x] `FretApp`
  - [x] `App`
  - [x] `KernelApp`
  - [x] `WindowId`
  - [x] `AppUi`
  - [x] `Ui`
- [ ] Define the three public surface tiers:
  - [x] app surface
  - [x] component surface
  - [x] advanced surface
- [ ] List every public-looking symbol that should be:
  - [ ] kept on the app surface
  - [ ] moved to the component surface
  - [ ] moved to the advanced surface
  - [ ] deleted entirely
- [ ] Mark the initial status for every migration row:
  - [ ] surface lanes
  - [ ] ecosystem crates
  - [ ] docs/templates/examples
  - [ ] hard-delete rows

## M1 — Split the public preludes and imports

- [x] Make `fret::app::prelude::*` the only canonical app import and delete the broad `fret::prelude::*` bridge.
- [x] Add `fret::component::prelude::*`.
- [x] Add explicit advanced import modules under `fret::advanced::*`.
- [x] Remove broad transitive re-export of `fret_ui_kit::declarative::prelude::*` from the app surface.
- [x] Remove broad transitive re-export of `fret_ui_kit::prelude::*` from the advanced prelude convenience lane.
- [ ] Remove low-level mechanism types from the default app prelude:
  - [x] `AppWindowId`
  - [x] `Event`
  - [x] `ActionId`
  - [x] `TypedAction`
  - [x] `UiBuilder`
  - [x] `UiPatchTarget`
  - [x] `Length`
  - [x] `SemanticsProps`
  - [x] `HoverRegionProps`
  - [x] `ContainerQueryHysteresis`
  - [x] `ViewportQueryHysteresis`
  - [x] `ImageMetadata`
  - [x] `ImageMetadataStore`
  - [x] `ImageSamplingExt`
  - [x] `MarginEdge`
  - [x] `OverrideSlot`
  - [x] `WidgetState`
  - [x] `WidgetStateProperty`
  - [x] `WidgetStates`
  - [x] `merge_override_slot`
  - [x] `merge_slot`
  - [x] `resolve_override_slot`
  - [x] `resolve_override_slot_opt`
  - [x] `resolve_override_slot_opt_with`
  - [x] `resolve_override_slot_with`
  - [x] `resolve_slot`
  - [x] `ColorFallback`
  - [x] `SignedMetricRef`
  - [x] `Corners4`
  - [x] `Edges4`
  - [x] `ViewportOrientation`
  - [x] `ElementContext`
  - [x] `UiTree`
  - [x] `UiServices`
  - [x] `UiHost`
  - [x] `AnyElement`
  - [ ] other runner/maintainer-only types
- [ ] Update crate-level docs to teach the new split.

## M2 — Reset the app authoring API

- [ ] Introduce grouped app-facing context namespaces:
  - [x] `state()`
  - [x] `actions()`
  - [x] `data()`
  - [x] `effects()`
- [ ] Add the new default operations:
  - [x] local state creation/init
  - [x] local state watch/read
  - [x] default local transactions
  - [x] payload-local handlers
  - [x] transient action helpers
  - [x] selector/query integration points
- [ ] Rename or replace flat helpers that are no longer part of the blessed path.
- [ ] Remove redundant first-contact aliases from the app surface.

## M3 — Migrate first-party ecosystems to the new surface

- [x] Migrate `fret-ui-shadcn` to the component surface + explicit optional app integration seams.
  - [x] Move app integration helpers under `shadcn::app::*` instead of the recipe root.
  - [x] Move environment / `UiServices` hooks off the default app lane and keep them explicit via
    `fret_ui_shadcn::advanced::*` (or `fret::shadcn::raw::advanced::*` from the `fret` facade).
  - [x] Move first-party advanced cookbook examples to `shadcn::app::install`.
  - [x] Replace the broad `fret::shadcn` whole-crate re-export with a curated facade
    (`shadcn::{..., app, themes, raw}`).
  - [x] Migrate first-party direct-crate examples to `fret_ui_shadcn::{facade as shadcn, prelude::*}`
    and require raw-only helpers to flow through `shadcn::raw::*`.
  - [x] Classify the first-party raw escape hatches and gate them to the documented set
    (`typography`, `extras`, breadcrumb primitives, low-level icon helpers, advanced/raw prelude
    seams where explicitly justified).
  - [x] Add a source gate that forbids first-party curated examples from drifting back to
    `use fret_ui_shadcn as shadcn;`, `shadcn::shadcn_themes::*`, or root
    `shadcn::typography::*`.
  - [x] Audit remaining first-party docs/examples for root-level shadcn app-install teaching.
- [x] Migrate `fret-docking` to the component/advanced split without redefining the app authoring model.
  - [x] Add an explicit `fret::docking` facade module behind a `fret/docking` feature.
  - [x] Move the cookbook docking example to the `fret::docking::*` seam.
  - [x] Move app-facing `fret-examples` docking demos (`docking_demo`, `container_queries_docking_demo`) to the `fret::docking::*` seam.
  - [x] Audit remaining advanced/component call sites and keep direct `fret-docking` imports explicit.
- [x] Migrate `fret-selector` to the grouped app data surface.
  - [x] Re-export `DepsBuilder` / `DepsSignature` from `fret::app::prelude::*`.
  - [x] Move default docs/templates/examples to `cx.data().selector(...)`.
  - [x] Audit remaining advanced/component call sites and keep them explicit.
- [x] Migrate `fret-query` to the grouped app data surface.
  - [x] Move default docs/examples to `cx.data().query(...)` / `cx.data().query_async(...)`.
  - [x] Add the grouped `data()` namespace to extracted `UiCx` helpers so helper-heavy examples no
    longer fall back to raw `use_query*`.
  - [x] Add source/doc gates that forbid default teaching text from drifting back to flat query hooks.
  - [x] Audit remaining advanced/component call sites and keep them explicit.
- [x] Migrate `fret-router` to the new explicit app/advanced extension seams.
  - [x] Add an explicit `fret::router` facade module behind a `fret/router` feature.
  - [x] Move the cookbook router example to the `fret::router::*` extension seam.
  - [x] Keep `fret-router-ui` thin and app-owned instead of turning it into a competing default runtime.
  - [x] Audit remaining direct imports of `fret-router` / `fret-router-ui` in first-party app-facing examples and docs.
- [x] Audit first-party ecosystem crates for private or accidental shortcuts that bypass the new public contracts.
  - [x] Explicit app/advanced split crates (`fret-ui-assets`, `fret-icons-lucide`,
    `fret-icons-radix`, `fret-node`, `fret-router-ui`) now gate against root-level shortcut
    re-exports or install helpers that would bypass their documented seams.

## M4 — Migrate docs, templates, and examples

- [x] Update `README.md`.
- [x] Update `docs/README.md`.
- [x] Update `docs/first-hour.md`.
- [x] Update the golden-path todo docs.
- [x] Update scaffold templates in `apps/fretboard`.
- [x] Update official cookbook examples to use the new app surface.
- [x] Move advanced examples to explicit advanced imports when needed.
- [x] Migrate first-party extracted helper teaching snippets to `UiCx` unless they intentionally
  stay generic over `H: UiHost` or define an explicit advanced entry seam.
- [x] Normalize the first-party UI Gallery routed page surface to `UiCx` and add source gates for
  the default app-facing teaching surface.
- [x] Finish migrating the remaining first-party UI Gallery internal preview surface to `UiCx`
  before deleting the old `ElementContext<'_, App>` teaching seam.
  - Current bounded remainder on 2026-03-11 after the editor/torture batch: `0 / 92`
    preview-surface files in
    `apps/fret-ui-gallery/src/ui/previews/**`.
  - The remaining cleanup work is deletion/compaction of legacy helpers, not interface migration.
  - 2026-03-11 follow-up cleanup removed the first dead legacy helpers from the gallery
    atoms/components buckets and deleted orphan `gallery/data/table*.rs` preview bridge files.
  - 2026-03-11 follow-up cleanup also started feature-boundary alignment for UI Gallery dev-only
    teaching surfaces (`harness.rs`, `content.rs`, routed dev pages), restoring a green
    `cargo check -p fret-ui-gallery --lib --features gallery-full`.
- [ ] Remove or rewrite examples that still teach superseded patterns.

## M5 — Delete the old surface

- [x] Remove `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from the default app surface once
  docs/templates/examples and gates all prefer `view::<V>()?.run()`.
- [ ] Remove old default-path names that are no longer canonical.
- [x] Remove flat `AppUi` data/effects helpers that duplicate the grouped `cx.data()` /
  `cx.effects()` surface.
- [x] Remove public flat `AppUi::use_local*` helpers that duplicate the grouped `cx.state()`
  surface while keeping raw `use_state*` as an explicit advanced seam for now.
- [x] Move raw `use_state*` off the default `AppUi` inherent surface and keep it only as an
  explicit advanced trait seam.
- [x] Remove flat `AppUi` action mutation helpers that duplicate the grouped `cx.actions()` surface
  while keeping raw handler registration as an explicit advanced seam.
- [ ] Remove duplicate or ambiguous exports from the app prelude.
- [ ] Remove compatibility-only aliases that survive only for internal inertia.
- [ ] Remove dead docs and stale guidance after the migration is complete.

## M6 — Add gates so the surface stays clean

- [x] Add a gate that checks the app prelude stays app-only.
- [x] Add a gate that checks low-level mechanism types do not leak into the app prelude.
- [x] Add a gate that templates only use blessed app-surface APIs.
- [x] Add source gates that keep default docs/examples/templates on `view::<V>()?.run()`.
- [x] Add a gate that README/docs/first-hour agree on the default action model.
- [x] Add source gates that keep default selector/query teaching on grouped `cx.data()` helpers.
- [x] Add a source gate that keeps default extracted helper teaching on `UiCx` instead of raw
  `ElementContext`.
- [x] Add focused UI Gallery source gates for the first migrated teaching surfaces:
  routed pages, gallery shell helpers, the retired Material 3 surface, Magic previews, and
  component preview modules.
- [x] Extend the internal preview gates to cover the first harness-shell batch.
- [x] Extend the internal preview gates to cover gallery atoms/forms/data/overlays.
- [x] Extend the internal preview gates to cover the remaining harness/editor/torture preview buckets.
  - On 2026-03-11 these UI Gallery gates were moved out of `apps/fret-ui-gallery/src/lib.rs` into
    dedicated integration tests under `apps/fret-ui-gallery/tests/ui_authoring_surface_*.rs` to
    reduce merge conflicts on the crate entry file.
- [ ] Add a gate that first-party ecosystem crates use documented extension seams.
  - [x] Shadcn docs/examples now gate the curated `shadcn::app::*` seam, explicit advanced hooks,
    and the documented raw escape-hatch lanes.
  - [x] Router cookbook/docs now gate the `fret::router::*` seam.
  - [x] `fret-router-ui` now gates its thin adoption-layer posture and forbids growing a second
    app runtime surface.
  - [x] Docking cookbook/docs now gate the `fret::docking::*` seam.
  - [x] Selector/query docs, templates, and helper-heavy examples now gate grouped
    `cx.data().selector/query*` teaching while keeping raw hook entry explicit to advanced or
    component surfaces.
  - [x] Optional split ecosystem crates (`fret-ui-assets`, icon packs, `fret-node`) now gate
    against root-level app/advanced shortcut re-exports that would bypass their explicit seams.
- [ ] Keep layering checks green.

## Exit Criteria

- [ ] A new user can write a small app without seeing low-level mechanism types.
- [ ] A component author can identify the reusable surface quickly and confidently.
- [ ] First-party ecosystems share one authoring vocabulary.
- [ ] The old broad surface is actually removed.
- [ ] The new surface is guarded by tests/scripts, not just prose.
