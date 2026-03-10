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

- [ ] Migrate `fret-ui-shadcn` to the component surface + explicit optional app integration seams.
- [ ] Migrate `fret-docking` to the component/advanced split without redefining the app authoring model.
- [ ] Migrate `fret-selector` to the grouped app data surface.
- [ ] Migrate `fret-query` to the grouped app data surface.
- [ ] Migrate `fret-router` to the new explicit app/advanced extension seams.
- [ ] Audit first-party ecosystem crates for private or accidental shortcuts that bypass the new public contracts.

## M4 — Migrate docs, templates, and examples

- [x] Update `README.md`.
- [ ] Update `docs/README.md`.
- [ ] Update `docs/first-hour.md`.
- [x] Update the golden-path todo docs.
- [x] Update scaffold templates in `apps/fretboard`.
- [x] Update official cookbook examples to use the new app surface.
- [x] Move advanced examples to explicit advanced imports when needed.
- [ ] Remove or rewrite examples that still teach superseded patterns.

## M5 — Delete the old surface

- [x] Remove `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from the default app surface once
  docs/templates/examples and gates all prefer `view::<V>()?.run()`.
- [ ] Remove old default-path names that are no longer canonical.
- [ ] Remove duplicate or ambiguous exports from the app prelude.
- [ ] Remove redundant action helpers from the default app surface.
- [ ] Remove compatibility-only aliases that survive only for internal inertia.
- [ ] Remove dead docs and stale guidance after the migration is complete.

## M6 — Add gates so the surface stays clean

- [x] Add a gate that checks the app prelude stays app-only.
- [x] Add a gate that checks low-level mechanism types do not leak into the app prelude.
- [x] Add a gate that templates only use blessed app-surface APIs.
- [x] Add source gates that keep default docs/examples/templates on `view::<V>()?.run()`.
- [ ] Add a gate that README/docs/first-hour agree on the default action model.
- [ ] Add a gate that first-party ecosystem crates use documented extension seams.
- [ ] Keep layering checks green.

## Exit Criteria

- [ ] A new user can write a small app without seeing low-level mechanism types.
- [ ] A component author can identify the reusable surface quickly and confidently.
- [ ] First-party ecosystems share one authoring vocabulary.
- [ ] The old broad surface is actually removed.
- [ ] The new surface is guarded by tests/scripts, not just prose.
