# Authoring Surface + Ecosystem (Fearless Refactor v1) — TODO

This TODO list tracks the remaining closeout work described in `DESIGN.md`.

Because this is a pre-release reset, "done" means we actually delete the old surface rather than
carrying compatibility-only baggage.

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Execution note on 2026-03-12:

- treat this file as a closeout tracker,
- do not reopen broad surface redesign here,
- route remaining conversion-surface work to
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`.

Status note:

- treat `MIGRATION_MATRIX.md` as the source of truth for lane/row status and delete-readiness,
- treat unchecked early bookkeeping items in this file as historical planning residue unless they
  still map to an active closeout task below,
- when this file and the matrix disagree, prefer the matrix plus the current source gates/tests.

Closeout note on 2026-03-15:

- this workstream should now be treated as a **targeted closeout lane**, not as a broad redesign
  backlog and not as a "maintenance only" archive,
- the app/component/advanced split itself does not need another broad redesign pass here,
- but three high-priority closeout tasks still belong here because they materially affect the
  public product surface:
  - narrowing `fret::app::prelude::*` so it is materially smaller than
    `fret::component::prelude::*` in both exports and autocomplete pressure,
  - reducing shadcn first-contact discovery to the curated facade lane rather than relying on
    source-policy tests to keep crate-root/facade/raw paths mentally sorted,
  - keeping `TARGET_INTERFACE_STATE.md` and its status matrix honest while
    `into-element-surface-fearless-refactor-v1` is still actively deleting surface families,
- remaining work is therefore a mix of docs cleanup, delete-ready follow-through, and explicit
  surface narrowing that is already implied by the target state but not yet fully reflected in the
  shipped exports.
- the next real product-surface pressure is no longer "how do we split app/component/advanced?",
  but rather:
  - finishing delete-ready cleanup on old root aliases and stale docs,
  - narrowing the default app prelude until it stops overlapping with the component prelude on
    styling/layout/semantics helper families,
  - keeping the conversion surface accurate in
    `docs/workstreams/into-element-surface-fearless-refactor-v1/`,
  - simplifying the shadcn discovery lane so `facade as shadcn` is the only first-contact story,
  - handling any future action-surface ergonomics in
    `docs/workstreams/action-first-authoring-fearless-refactor-v1/`.

Priority correction on 2026-03-15:

1. narrow `fret::app::prelude::*`
2. simplify shadcn first-contact discovery (`facade` first, `raw` explicit, crate root de-emphasized)
3. finish the conversion-surface reset under
   `docs/workstreams/into-element-surface-fearless-refactor-v1/`
4. only then add more small-app authoring sugar on top of the stabilized lane

## M0 — Freeze the target product surface

- [x] Finalize `TARGET_INTERFACE_STATE.md` as the single source of truth for the desired public surface.
- [x] Finalize `MIGRATION_MATRIX.md` as the single execution tracker for old surface removal.
- [x] Decide and lock the canonical names:
  - [x] `FretApp`
  - [x] `App`
  - [x] `KernelApp`
  - [x] `WindowId`
  - [x] `AppUi`
  - [x] `Ui`
- [x] Define the three public surface tiers:
  - [x] app surface
  - [x] component surface
  - [x] advanced surface
- [x] List every public-looking symbol that should be:
  - [x] kept on the app surface
  - [x] moved to the component surface
  - [x] moved to the advanced surface
  - [x] deleted entirely
- [x] Mark the initial status for every migration row:
  - [x] surface lanes
  - [x] ecosystem crates
  - [x] docs/templates/examples
  - [x] hard-delete rows

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
- [ ] Remove component-author overlap from `fret::app::prelude::*`.
  - Goal: an ordinary app author should not discover the same style/layout/semantics helper
    families from both `fret::app::prelude::*` and `fret::component::prelude::*`.
  - [x] First batch on 2026-03-15: move overlap-heavy extension traits (`TrackedStateExt`,
    `StyledExt`, `UiExt`, `AnyElementSemanticsExt`, `ElementContextThemeExt`,
    `UiElementA11yExt`, `UiElementKeyContextExt`, `UiElementTestIdExt`) to anonymous app-prelude
    imports so their methods remain usable without turning the trait names into default app-lane
    vocabulary.
  - [x] First batch on 2026-03-15: remove raw `on_activate`, `on_activate_notify`,
    `on_activate_request_redraw`, and `on_activate_request_redraw_notify` free-function exports
    from `fret::app::prelude::*`; the default app lane now teaches widget-local
    `.on_activate(cx.actions().dispatch::<A>())` / `.listener(...)` instead.
  - Minimum audit set:
    - broad styling/layout patch traits and types that primarily serve reusable component authors,
    - semantics/test-id/key-context helper families that are still duplicated across app and
      component preludes without an app-specific justification,
    - raw `on_activate*` helper exports that now compete with the grouped app-facing
      `cx.actions().dispatch/listener` story.
    - remaining high-frequency style/token/theme nouns (`Space`, `Radius`, `LayoutRefinement`,
      `Theme`, `ThemeSnapshot`, `IconId`) that may still justify app-lane presence but need an
      explicit target decision rather than accidental overlap.
  - Exit condition: the app prelude teaches the app nouns plus a small set of app-justified helper
    traits, while reusable component plumbing remains discoverable through the component lane.
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
  - [ ] Reduce first-contact shadcn discovery to one taught lane.
    - Goal: `use fret_ui_shadcn::{facade as shadcn, prelude::*};` is the only default first-contact
      story, while crate-root exports are treated as compatibility/implementation residue and
      `shadcn::raw::*` stays explicit.
    - This is not just a docs issue: if first-party tests must keep forbidding alternative import
      paths, the public surface still needs more self-constraint.
    - Exit condition: docs and status docs stop talking about crate root / facade as peer teaching
      lanes, and the remaining root-level exposure is explicitly classified as retained raw or
      compatibility surface.
    - 2026-03-15 follow-up: first-party UI Gallery snippet/page surfaces no longer use
      `fret_ui_shadcn::icon::*`, `fret_ui_shadcn::empty::*`, `fret_ui_shadcn::select::*`,
      `fret_ui_shadcn::tabs::*`, or similar flat root/module lanes; gallery authoring now flows
      through `shadcn::*`, `shadcn::raw::*`, or prelude glue only.
    - 2026-03-15 follow-up: after continuing through `fret-ui-ai`, `fret-bootstrap`, and
      `ecosystem/fret`, non-test first-party workspace code no longer contains
      `fret_ui_shadcn::*` flat root/component calls outside explicit `facade::*` / `raw::*` /
      `advanced::*` seams.
    - Remaining bounded cleanup after the gallery pass: non-gallery first-party consumers
      is now reduced to selected internal tests/docs strings plus any future crates that reintroduce
      flat root drift.
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
- [x] Keep the follow-on `into-element-surface-fearless-refactor-v1` tracker linked from repo
  indexes and active workstream docs so conversion-surface cleanup has an explicit owner after the
  app/component/advanced split lands.
- [x] Record that future default-authoring ergonomics work belongs to
  `action-first-authoring-fearless-refactor-v1` rather than reopening this split workstream.
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
- [ ] Keep first-party docs/examples/UI Gallery copy aligned with the next-phase target:
  app-facing lanes teach `Ui` / `UiChild`, while reusable generic helpers move to the unified
  component conversion trait once that workstream lands it.
  - 2026-03-15: UI Gallery code examples and helper snippets were normalized away from
    `fret_ui_shadcn::*` flat root/module paths; remaining stale references are bounded to a small
    set of narrative copy strings and non-gallery first-party crates.

## M5 — Delete the old surface

- [x] Remove `run_view::<V>()` / `run_view_with_hooks::<V>(...)` from the default app surface once
  docs/templates/examples and gates all prefer `view::<V>()?.run()`.
- [ ] Remove old default-path names that are no longer canonical.
- [ ] Remove root-level low-level aliases that are no longer part of the default facade vocabulary.
  - [x] 2026-03-12: removed `fret::ActionMeta` / `fret::ActionRegistry`; low-level registry
    access remains explicit under `fret::actions::*`.
  - [x] 2026-03-12: removed `fret::IconRegistry`; raw icon registry access now stays explicit via
    `fret-icons` / `fret-bootstrap` while app-facing icon packs install through `.setup(...::app::install)`.
  - [x] 2026-03-12: removed root `workspace_shell_model*` shortcuts; editor-style workspace shell
    assembly now stays explicit under `fret::workspace_shell::*`.
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
  - [x] 2026-03-12: removed the `fret/icons-lucide` compatibility feature alias; the canonical
    feature name for the default Lucide pack is now just `icons`.
  - [x] 2026-03-12: removed `FretApp::register_icon_pack(...)`,
    `UiAppBuilder::register_icon_pack(...)`, and `UiAppBuilder::with_lucide_icons()` from the
    default `fret` facade; explicit pack setup now flows through `setup(...::app::install)`.
  - [x] 2026-03-12: removed the root `fret::router::install_app(...)` exception; router setup on
    the default app lane now follows the same `fret::router::app::install(...)` pattern as the
    other ecosystem app installers.
  - [x] 2026-03-12: removed the naked root `fret::run_native_with_compat_driver(...)` entry;
    retained low-level interop now stays on the explicit
    `fret::advanced::interop::run_native_with_compat_driver(...)` path.
  - [x] 2026-03-12: removed the naked root `fret::run_native_with_fn_driver*` helpers; advanced
    runner escape hatches now stay on the explicit `fret::advanced::*` path.
  - [x] 2026-03-12: removed root `fret::kernel::*` / `fret::interop::*` module exports; low-level
    runtime/render/viewport seams now stay on the explicit `fret::advanced::{kernel, interop}`
    lane.
- [ ] Remove dead docs and stale guidance after the migration is complete.

## M6 — Add gates so the surface stays clean

- [x] Add a gate that checks the app prelude stays app-only.
- [x] Add a gate that checks low-level mechanism types do not leak into the app prelude.
- [x] Add a gate that templates only use blessed app-surface APIs.
- [x] Add source gates that keep default docs/examples/templates on `view::<V>()?.run()`.
- [x] Add a gate that README/docs/first-hour agree on the default action model.
- [x] Add source gates that keep default selector/query teaching on grouped `cx.data()` helpers.
- [x] Add a gate that keeps `.setup(...)` on named installers/tuples/bundles and reserves inline
  closures for `setup_with(...)`.
  - Landed on 2026-03-12 in `ecosystem/fret` authoring-surface policy tests and the first-party
    source-policy tests for `apps/fret-examples` and `apps/fret-cookbook`.
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
- [x] Add a gate that first-party ecosystem crates use documented extension seams.
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
  - 2026-03-12: `python3 tools/check_layering.py` passed after the split-ecosystem shortcut audit
    guards landed.
  - 2026-03-12: keep the `fret` root facade free of low-level action registry aliases; the source
    gate now requires those names to stay under `fret::actions::*` instead.
  - 2026-03-12: keep icon registry / icon-pack builder helpers off the default `fret` facade; the
    source gates now require app-facing icon setup to stay on `.setup(...::app::install)`.
  - 2026-03-12: keep router app wiring on `fret::router::app::install(...)`; the source/docs
    gates now forbid `fret::router::install_app(...)` from returning on the default lane.
  - 2026-03-12: keep the `fret` feature surface on canonical names only; the source gate now
    forbids the old `icons-lucide = ["icons"]` alias from returning.
  - 2026-03-12: keep workspace-shell helpers module-scoped; the source gate now forbids root
    `workspace_shell_model*` shortcuts from returning.

## Exit Criteria

- [ ] A new user can write a small app without seeing low-level mechanism types.
- [ ] A component author can identify the reusable surface quickly and confidently.
- [ ] First-party ecosystems share one authoring vocabulary.
- [ ] The old broad surface is actually removed.
- [ ] The new surface is guarded by tests/scripts, not just prose.
