# Target Interface State

Status: target state for the pre-release surface reset
Last updated: 2026-03-16

This document is the single place that records the **intended public interface state** for the
authoring-surface reset.

It is intentionally concrete:

- what names should exist,
- who should import them,
- which layer owns them,
- which names should disappear.

Closeout note on 2026-03-16:

- this file now describes a mostly-settled surface rather than an open redesign backlog,
- the app/component/advanced split is considered landed **at the lane-definition level**,
- but the closeout is still incomplete while:
  - shadcn first-contact discovery still needs final doc tightening even though the curated
    `prelude`, crate-internal recipe/helper glue, and public component families are now off the
    hidden flat root and the default component lane is `facade as shadcn`,
  - the dedicated conversion-surface tracker is still actively deleting vocabulary families,
- remaining work is therefore narrow follow-through rather than broad redesign:
  delete-ready cleanup, ceremony reduction, bridge shrinkage, stale-doc correction, and coordination
  with follow-on workstreams such as
  `into-element-surface-fearless-refactor-v1` and `action-first-authoring-fearless-refactor-v1`.

## Current Release-Blocking Closeout Items (2026-03-16)

The lane split itself is no longer the blocker. The remaining pre-release blockers are the
authoring surfaces that still feel too wide or too indirect even though the underlying ownership
model is already correct.

Fresh-audit reading rule on 2026-03-16:

- if a new audit says "the default path still feels too wide", treat that as a closeout signal on
  discovery, root budgeting, or ceremony,
- do **not** treat it as evidence that the app/component/advanced tier model is still undecided,
- post-closeout follow-ons such as ecosystem integration traits, macros, or other sugar should
  read from the stabilized lane story below rather than running ahead of it.

1. `fret-ui-shadcn` discovery-lane closure
   - Default teaching lane: `use fret_ui_shadcn::{facade as shadcn, prelude::*};`
   - Explicit escape hatch: `shadcn::raw::*` / `fret_ui_shadcn::raw::*`
   - Explicit setup lanes: `shadcn::app::*` and `shadcn::themes::*` for app installation/theme
     work only, not as peer component-family discovery lanes
   - Non-goal: do not normalize the hidden flat crate root, `advanced::*`, and `raw::*` as equal
     first-contact choices for ordinary component authoring
   - Status on 2026-03-16: component-family root modules are crate-private and first-party
     source-policy/gallery gates now pass on the curated facade + explicit raw posture
2. `fret` root lane budget freeze
   - `fret::app`, `fret::component`, and `fret::advanced` are the product tiers
   - root-level secondary lanes such as `assets`, `env`, `router`, and `docking` remain allowed,
     but only as explicit opt-in lanes, not as vocabulary that should compete with
     `fret::app::prelude::*`
   - Non-goal: do not collapse the root into another "default import" lane or re-widen the app
     surface just because optional ecosystems exist
   - New root lanes should require an explicit justification for why the surface cannot live under
     an existing lane or remain crate-local
   - Status on 2026-03-16: root public-module/direct-reexport budgets are now source-gated, and
     raw view-runtime/devloop seams moved to `fret::advanced::{view, dev}`
3. Happy-path ceremony reduction
   - The next density problem is no longer "mechanism vs policy"; it is reducing the amount of
     surface an ordinary app author has to hold in their head in the first hour
   - Priority targets: tracked-value reads, common local/payload write paths, and list/keyed-row
     defaults
   - Status on 2026-03-16:
     - first batch already landed on the canonical trio (`simple_todo`,
       `simple_todo_v2_target`, `todo_demo`), the generated todo/simple-todo templates, and the
       default-path docs,
     - the taught tracked-read wording is now `state.layout(cx).value_*` /
       `state.paint(cx).value_*`,
      - the taught keyed-row payload write wording is now
        `cx.actions().payload_local_update_if::<A, _>(...)`,
      - the narrow single-child late-landing wording is now `ui::single(cx, child)` for root or
        wrapper closures that only forward one typed subtree,
      - the next batch should stay focused on keyed/list/default child-collection ergonomics and
        should prefer already-shipped helpers before any new public sugar is minted
   - Ownership split: this workstream sets the target product surface, while
     `action-first-authoring-fearless-refactor-v1` and
     `into-element-surface-fearless-refactor-v1` land the concrete API reductions
4. `AppActivateExt` bridge retirement path
   - `AppActivateExt` is now intentionally off `fret::app::prelude::*`, but it still exists as a
     facade-level bridge table
   - As of 2026-03-16, that first-party default widget bridge table is intentionally empty:
     ordinary first-party buttons/wrappers stay on native `.action(...)` /
     `.action_payload(...)` or widget-owned `.on_activate(...)` hooks instead of reopening the
     bridge
   - The bridge remains acceptable only for truly activation-only legacy/default-path surfaces
   - New first-party widgets should not add fresh `AppActivateSurface` impls when they can expose
     native `.action(...)` / `.action_payload(...)` slots directly
   - extracted `UiCx` helper functions should stay on grouped `cx.actions()` / `cx.data()` via
     `UiCxActionsExt` / `UiCxDataExt` before anyone proposes new bridge entries just to make split
     helper functions authorable again
   - The target end-state is "shrinking bridge residue", not "permanent growing integration list"

Post-closeout follow-on:

- ecosystem integration-trait budgeting is not a release blocker ahead of items 3-4
- resume it only after the public lane story above is stable enough that router/query/docking/
  catalog integration seams can be audited against final, not transitional, imports
- sequencing rule:
  - do not start new public trait budgeting while the canonical trio/templates/docs still need
    default-path wording churn,
  - every new ecosystem contract must declare its target tier up front:
    app-level install/setup/integration, component-level composition/adaptor, or explicit
    advanced/raw hook,
  - non-goal: do not solve ecosystem extensibility by widening `fret::app::prelude::*`,
    `fret::component::prelude::*`, or the `fret` crate root again

Revalidation note on 2026-03-16:

- the current target-state reading above is backed by:
  - `cargo nextest run -p fret --lib authoring_surface_policy_tests:: --no-fail-fast`
  - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
  - `cargo check -p fret-examples --all-targets`
- if one of those commands starts failing, treat that as target-state drift and update the lane
  docs or shipped exports before broadening the surface again.

## Public Surface Tiers

| Tier | Intended audience | Canonical import | Allowed concepts |
| --- | --- | --- | --- |
| App | ordinary app authors | `use fret::app::prelude::*;` | app builder, app-facing state/actions/data/effects, default UI composition, default components |
| Component | reusable component authors | `use fret::component::prelude::*;` | component composition contracts, styling/layout patch APIs, semantics/layout helpers |
| Advanced | power users / runners / interop | explicit `fret::advanced::*` imports | low-level runtime/driver/viewport/manual-assembly seams |

## Canonical Names

| Concept | Target name | Audience | Notes |
| --- | --- | --- | --- |
| app builder | `FretApp` | App | canonical user-facing entry point |
| app runtime handle | `App` | App | default app-surface alias under `fret::app`; hides `KernelApp` on the common path |
| kernel runtime app | `KernelApp` | Advanced / component as needed | explicit advanced/component name under `fret::advanced`; not taught on the default app path |
| app window identity | `WindowId` | App | hides `AppWindowId` noise in default app code |
| app-facing view context | `AppUi` | App | replaces `ViewCx` as the taught surface |
| rendered UI return alias | `Ui` | App | canonical alias over `Elements` |
| extracted app helper context | `UiCx` | App | hides `ElementContext<'_, KernelApp>` noise in default app-facing extracted helpers; reusable `H: UiHost` snippets stay generic |
| extracted app helper child return | `UiChild` | App | hides `UiChildIntoElement<KernelApp>` noise in default app code |
| reusable component context | `ComponentCx` or explicit component-surface alias | Component | wraps the mechanism context for reusable component work |

## `fret::app::prelude` (App Surface)

Canonical startup chain:

- `FretApp::new("my-app").window("My App", (...)).view::<MyView>()?.run()`
- default docs, templates, cookbook examples, and in-tree app examples must prefer the explicit
  builder-then-run chain over `run_view::<V>()`
- `run_view::<V>()` / `run_view_with_hooks::<V>(...)` are deleted from the pre-release target
  surface and must not reappear on the default app path

Target exports:

- `FretApp`
- `View`
- `App`
- `WindowId`
- `AppUi`
- `UiCx`
- `Ui`
- `UiChild`
- `ui`
- `Px`
- `shadcn` (feature-gated)
- typed action/payload action macros

Explicit secondary app lanes:

- `fret::actions::*` for explicit command identity/metadata, payload-action contracts, and
  action-registry helper nouns; add `ElementCommandGatingExt as _` when code needs explicit
  `cx.action_is_enabled(...)` reads
- `fret::app::{AppActivateSurface, AppActivateExt}` for helper signatures that intentionally name
  the narrow activation-widget contract, plus explicit `use fret::app::AppActivateExt as _;`
  imports at activation-only call sites
- `fret::app::LocalState` for helper signatures or validators that intentionally name local
  state-handle types
- `fret::activate::{on_activate, on_activate_notify, on_activate_request_redraw,
  on_activate_request_redraw_notify}` for intentional raw activation-helper glue outside the
  default app lane
- `fret::workspace_menu::*` for opt-in in-window menubar/menu-model integration helpers
- `fret::semantics::SemanticsRole` for explicit semantic-role nouns
- `fret::style::ThemeSnapshot` for extracted helper signatures that intentionally take snapshot
  value types
- `fret::selector::{DepsBuilder, DepsSignature}` for selector dependency signatures
- `fret::query::{QueryError, QueryKey, QueryPolicy, QueryState, ...}` for explicit query nouns
- `fret::children::UiElementSinkExt as _` for explicit sink-style `*_build(|cx, out| ...)`
  collection when a view intentionally opts into manual child pipelines

Target non-exports:

- `Event`
- `KernelApp`
- `AppWindowId`
- `AppActivateExt` as a named app-prelude export
- `TrackedStateExt` as a named app-prelude export
- `ElementContext`
- `UiTree`
- `UiServices`
- `UiHost`
- `AnyElement`
- `AnyElementSemanticsExt` as a named app-prelude export
- `ElementContextThemeExt` as a named app-prelude export
- `ModelStore`
- `ActionId`
- `SemanticsRole`
- `TypedAction`
- `actions` as a default app-prelude module export
- `workspace_menu` as a default app-prelude module export
- `ElementCommandGatingExt`
- `DepsBuilder`
- `DepsSignature`
- `QueryKey`
- `QueryPolicy`
- `QueryState`
- `UiBuilder`
- `UiPatchTarget`
- `StyledExt` as a named app-prelude export
- `UiExt` as a named app-prelude export
- `UiElementA11yExt` as a named app-prelude export
- `UiElementKeyContextExt` as a named app-prelude export
- `UiElementTestIdExt` as a named app-prelude export
- `UiElementSinkExt`
- `Length`
- `SemanticsProps`
- `HoverRegionProps`
- `ContainerQueryHysteresis`
- `ViewportQueryHysteresis`
- `ImageMetadata`
- `ImageMetadataStore`
- `ImageSamplingExt`
- `MarginEdge`
- `OverrideSlot`
- `WidgetState`
- `WidgetStateProperty`
- `WidgetStates`
- `merge_override_slot`
- `merge_slot`
- `resolve_override_slot`
- `resolve_override_slot_opt`
- `resolve_override_slot_opt_with`
- `resolve_override_slot_with`
- `resolve_slot`
- `ColorFallback`
- `SignedMetricRef`
- `Corners4`
- `Edges4`
- `ViewportOrientation`
- raw `on_activate`
- raw `on_activate_notify`
- raw `on_activate_request_redraw`
- raw `on_activate_request_redraw_notify`
- runner/driver traits
- viewport/interop traits
- broad component-author internals

Intentional root-level exception on 2026-03-15:

- `ActionId` / `TypedAction` remain intentionally absent from `fret::app::prelude::*`,
  but they still exist on the `fret` crate root and under `fret::actions::*` for the typed-action
  macro lane.
- Treat that as action-surface support rather than default app-prelude vocabulary.
- Do not re-export those names from `fret::app::prelude::*` unless the action-first workstream
  explicitly changes the default product story.

Target rule:

- if a symbol is primarily useful for component authors or runner authors, it does not belong in
  `fret::app::prelude`.
- `fret::app::prelude` must use curated exports; it does not blanket re-export
  `fret_ui_kit::declarative::prelude::*`.
- app code that needs explicit icon/style nouns should import them from `fret::icons::{icon,
  IconId}` and `fret::style::{Theme, ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius,
  ShadowPreset, Size, Space}` rather than expecting them from the default prelude.
- app code that needs explicit semantic-role nouns should import them from
  `fret::semantics::SemanticsRole` rather than expecting them from the default prelude.
- `Px` remains an intentional shared primitive on both app and component lanes because raw pixel
  units are part of the basic Fret authoring vocabulary rather than a component-only mechanism
  noun.
- app code that needs adaptive declarative helpers should import them from `fret::env::{...}`
  rather than expecting breakpoint/media/preference helpers from the default prelude.
- overlap-heavy extension traits may still arrive through anonymous `as _` imports so method calls
  remain ergonomic, but those trait names are not part of the default app-lane vocabulary and
  should not be taught as first-contact imports.
- `AppActivateExt` follows the explicit-bridge rule: it is not part of
  `fret::app::prelude::*`. App code adds `use fret::app::AppActivateExt as _;` only when an
  activation-only widget still needs `.action(act::Save)`, `.action_payload(act::Remove, payload)`,
  `.listen(...)`, or the explicit `.dispatch::<A>()` / `.dispatch_payload::<A>(...)` aliases.
  Helper code that intentionally names the contract imports it from `fret::app`.
- first-party ecosystem widgets with stable activation semantics should keep graduating to native
  `.action(...)` / `.action_payload(...)` slots so `AppActivateExt` increasingly reads as a bridge
  for activation-only legacy surfaces rather than the default way to wire ordinary buttons/wrappers.
- the first-party default widget bridge table is intentionally empty; if a future first-party
  widget proposes a new `AppActivateSurface` impl, that should be treated as a regression to
  justify rather than normal growth of the app surface.
- the remaining anonymous semantics/a11y/test-id helpers are intentionally retained on the app
  lane: `.role(...)`, `.a11y_role(...)`, and `.test_id(...)` are treated as app-justified
  diagnostics/accessibility affordances even though the underlying trait names stay hidden.
- `AppUi` is taught through grouped helper families:
  `cx.state().local*`, `cx.actions().models/locals/transient`,
  `cx.actions().payload::<A>().models/locals/local_update_if`, `cx.data().selector/query*`, and
  `cx.effects().take_transient(...)`.

App-level ecosystem integration seam:

- `FretApp::setup(fn(&mut App))` is the canonical builder hook for app-level integrations such as
  command registration, theme/bootstrap setup, optional shadcn installs, router/query defaults,
  and other non-reusable product wiring.
- `FretApp::setup(...)` should be taught with named installer functions, small tuples, or named
  `InstallIntoApp` bundles rather than inline closures.
- `UiAppBuilder::setup_with(|app| ...)` is the explicit one-off inline closure seam for setup that
  needs captured runtime values or intentionally local call-site wiring.
- lower-level raw bootstrap builders may keep mechanism-oriented names such as
  `fret_bootstrap::ui_app(...).init_app(...)`; that naming does not define the default authoring
  vocabulary.
- callbacks that require `UiServices`, custom effect services, runner hooks, or manual assembly
  stay outside the default app path and should remain explicit advanced seams.

Legacy bridge status:

- `fret::prelude::*` is deleted; the target public import surfaces are `fret::app`,
  `fret::component`, and `fret::advanced`.

## `fret::component::prelude` (Component Surface)

Target exports:

- component-facing context alias/wrapper
- `UiBuilder`
- `UiPatchTarget`
- `IntoUiElement`
- `UiExt`
- layout/style refinement types
- semantics/layout utilities needed by reusable components
- explicit overlay/focus composition helpers intended for reusable libraries
- explicit raw escape hatches such as `AnyElement`, `UiHost`, and `Invalidation`
- only the high-frequency overlay builder nouns stay directly on this lane
  (`OverlayController`, `OverlayRequest`, `OverlayPresence`); overlay stack snapshots, anchoring
  helpers, and lower-level overlay introspection vocabulary live on explicit `fret::overlay::*`
  imports instead of widening first-contact autocomplete

Target non-exports:

- app builder
- app-runtime-only globals/installation seams
- `CommandId` as a default component-prelude noun
- raw `on_activate*` helper families
- runner/manual assembly types
- legacy split conversion names such as `UiIntoElement`, `UiHostBoundIntoElement`,
  and `UiChildIntoElement`
- overlay introspection/stack snapshot nouns such as `OverlayArbitrationSnapshot`,
  `OverlayStackEntryKind`, `WindowOverlayStackEntry`, and `WindowOverlayStackSnapshot`

Target rule:

- a reusable component crate should be able to stay entirely on this surface unless it is
  intentionally shipping app-specific integration helpers.
- reusable component code that needs explicit command identity values should import
  `fret::actions::CommandId` (or `fret-runtime`) explicitly instead of relying on the component
  prelude.
- reusable component code that needs environment/responsive helpers should import them explicitly
  from `fret::env::{...}` instead of relying on the component prelude for breakpoint/media/pointer
  helper families.
- overlap-heavy helper traits should stay anonymous `as _` imports on this lane as well, so
  reusable component method ergonomics do not force extension-trait names into first-contact
  autocomplete.
- reusable component code that needs raw activation helper glue should import it explicitly from
  `fret::activate::{on_activate, on_activate_notify, on_activate_request_redraw,
  on_activate_request_redraw_notify}` instead of relying on the component prelude.
- `IntoUiElement<H>` is the single public conversion vocabulary on this lane; the older split
  conversion taxonomy is considered deleted from the intended public product surface.

## `fret::advanced`

Target exports:

- `ui_app(...)` / `ui_app_with_hooks(...)` for explicit golden-path manual assembly
- `fret::advanced::kernel::*` for low-level runtime/render contracts
- `fret::advanced::interop::*` for viewport embedding and foreign-surface interop
- advanced builder extension traits (for `UiServices`, GPU-ready hooks, custom effects)
- driver hooks
- viewport embedding / interop seams
- manual assembly surfaces
- low-level runtime/service/context types
- explicit integration points for editor-grade products

Target rule:

- advanced users should still have full power, but must opt into it explicitly.
- `fret::advanced::prelude::*` is a curated advanced/manual-assembly convenience lane, not a
  hidden umbrella over the component surface.
- if advanced code also needs ordinary component authoring vocabulary (`ui::*`, `.ui()`,
  `.into_element(...)`, watch/helper extension traits, overlay authoring helpers), it should add
  an explicit `use fret::component::prelude::*;` import rather than relying on advanced-prelude
  forwarding.
- first-party evidence for that rule now lives on both advanced examples and gallery/manual-app
  snippets (for example `async_playground_demo`, `imui_editor_proof_demo`, and
  `action_first_view`), so future drift should be treated as a regression rather than unresolved
  surface design.

## Target `AppUi` Structure

The default app context should group its public API by intent.

### `ui.state()`

Target operations:

- `local::<T>()`
- `local_keyed(key)`
- `local_init(|| ...)`
- `watch(&state)`
- `get(&state)` / `get_or(...)` through transactions or explicit watch handles

Target rule:

- the default app surface does not expose public flat `use_local*` helpers; those flows live
  under `cx.state()`.
- raw `use_state(...)` / `use_state_keyed(...)` remain explicit advanced seams through
  `AppUiRawStateExt`, not as direct `AppUi` methods.

### `ui.actions()`

Target operations:

- `local_update::<A>(&state, ...)`
- `local_set::<A, T>(&state, value)`
- `toggle_local_bool::<A>(&state)`
- `models::<A>(|models| ...)`
- `locals::<A>(|tx| ...)`
- `payload::<A>().models(|models, payload| ...)`
- `payload::<A>().locals(|tx, payload| ...)`
- `payload::<A>().local_update_if(&state, |value, payload| ...)`
- `transient::<A>(...)`
- `availability::<A>(...)`

Target rule:

- the default app surface does not expose flat specialized mutation helpers such as
  `on_action_notify_local_*`, `on_action_notify_models/locals/transient`, or
  `on_payload_action_notify_local_update_if` / `on_payload_action_notify_locals`; those stories
  live under `cx.actions()`.
- raw `on_action(...)`, `on_action_notify(...)`, `on_payload_action(...)`,
  `on_payload_action_notify(...)`, and `on_action_availability(...)` remain explicit advanced
  seams for manual handler registration.

### `ui.data()`

Target operations:

- `selector(...)`
- selector dependency building through `fret::selector::DepsBuilder`
- `query(...)`
- `query_async(...)`
- `query_async_local(...)`
- explicit query nouns through `fret::query::{QueryKey, QueryPolicy, QueryState, ...}`
- future router/state-library integration hooks

Target rule:

- flat `AppUi::use_selector*` / `AppUi::use_query*` helpers are removed from the default app
  surface; low-level `ElementContext` query/selector helpers remain explicit for component or
  advanced call sites.
- extracted `UiCx` helpers on the default/advanced app-facing surface use the same grouped
  `actions()` / `data()` namespaces through `UiCxActionsExt` / `UiCxDataExt`, so first-party helper
  functions do not fall back to raw `on_action*` / `use_query*` / `use_selector*` calls just
  because they were split out of `render()`.

### `ui.effects()`

Target operations:

- app-bound side-effect helpers intended for the default app surface

Target rule:

- the default app surface does not expose a flat render-time transient helper like
  `take_transient_on_action_root(...)`; use `cx.effects().take_transient(...)` instead.
- app authors should discover behavior by going to the relevant namespace first, not by scanning a
  flat list of dozens of methods.

## Helper Context Boundary

- Use `UiCx` for extracted helper functions that still belong to the default app-facing teaching
  surface.
- Those extracted helpers should keep the same grouped selector/query posture as `AppUi`
  (`cx.data().selector/query*`) rather than dropping to raw `ElementContext` hook names.
- First-party teaching snippets in `apps/fret-cookbook`, `apps/fret-examples`, and curated gallery
  surfaces should be source-gated so raw `ElementContext<'_, KernelApp>` only remains on
  intentional advanced/manual-assembly seams.
- Keep reusable/generic snippets on `ComponentCx<'_, H>` or explicit `ElementContext<'_, H>` when
  the helper must stay portable across `H: UiHost`.
- Gallery page hosts, drivers, and other app-shell composition code are not the same thing as the
  default extracted-helper teaching surface; they can stay explicit until a narrower host/page
  abstraction is intentionally introduced.

## Ecosystem Integration Targets

| Crate category | Target integration model | Notes |
| --- | --- | --- |
| design-system kit (`fret-ui-shadcn`, future kits) | component surface + optional explicit app integration module (`shadcn::app`) + explicit advanced/theme/raw seams (`fret_ui_shadcn::advanced`, `shadcn::themes`, `shadcn::raw`) | recipe crates must not define a competing app runtime or leak their full crate root onto the default app path |
| docking | explicit `fret::docking` extension module over dock core contracts + `fret-docking` UI/runtime adoption | keeps docking policy powerful without leaking runner ideas into the app default path |
| selector/query | grouped app-surface extension traits | first-party ecosystems must use the same seams expected of third parties |
| router | explicit app-level extension module (`fret::router`) over router core + thin UI adoption | keeps routing opt-in and visible without leaking it into `fret::app::prelude::*` |
| third-party reusable kits | component surface | default choice for portable UI packages |
| third-party workflow/app addons | app surface | acceptable when the crate is intentionally app-level |
| third-party interop crates | advanced surface | explicit power-user posture |

Direct crate usage rule for first-party recipe crates:

- official examples/docs should prefer `use fret_ui_shadcn::{facade as shadcn, prelude::*};`
- common component names stay on `shadcn::*`
- app-level setup stays on `shadcn::app::*`
- environment / `UiServices` hooks stay on `fret_ui_shadcn::advanced::*` (or
  `fret::shadcn::raw::advanced::*` when intentionally exiting the curated `fret` facade)
- theme presets stay on `shadcn::themes::*`
- full crate-root escape hatches must be explicit via `shadcn::raw::*`
- the full `fret_ui_shadcn` crate root is no longer a component-family or direction-utility
  discovery lane; remaining flat-root exposure is limited to doc-hidden compatibility/glue residue
  rather than peer first-contact API
- first-party teaching surfaces may currently use only the documented raw lanes:
  `shadcn::raw::typography::*`, `shadcn::raw::extras::*`,
  `shadcn::raw::breadcrumb::primitives`, `shadcn::raw::experimental::*` for the
  `DataGridElement` prototype, low-level `shadcn::raw::icon::*`, module-local advanced
  customization enums/styles such as `shadcn::raw::{button, calendar, context_menu,
  dropdown_menu, kbd, menubar, select, switch, tabs, toggle_group}::*`, and
  explicitly justified retained/interop seams such as `fret::shadcn::raw::prelude::*`
- 2026-03-15 implementation note: first-party UI Gallery snippet/page surfaces are now aligned to
  this rule; remaining direct-crate cleanup is bounded to non-gallery first-party consumers and
  selected internal tests/docs strings.

## Symbols to Remove

These names/surfaces should disappear entirely if the replacement exists:

- broad transitive app-prelude re-exports of component/maintainer internals,
- ambiguous app-builder naming that collides with the kernel runtime app,
- redundant "same intent, slightly different name" action helpers on the default path,
- public-looking aliases that only exist because of past migrations,
- examples/docs that teach superseded authoring models.

## Status Matrix

| Area | Target status |
| --- | --- |
| app prelude is app-only | in progress |
| component prelude is explicit | done |
| advanced surface is explicit | done |
| canonical naming reset landed | in progress |
| grouped `AppUi` namespaces landed | done |
| first-party ecosystems migrated | in progress |
| templates/docs aligned | in progress |
| old surface deleted | in progress |
| guards/gates added | done |

## Definition of Complete

This target state is complete when:

1. the names in this document match the real public exports,
2. official docs and templates teach the same names,
3. first-party ecosystem crates use the same extension seams described here,
4. the removed names are truly gone rather than merely discouraged.
