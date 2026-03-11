# Target Interface State

Status: target state for the pre-release surface reset

This document is the single place that records the **intended public interface state** for the
authoring-surface reset.

It is intentionally concrete:

- what names should exist,
- who should import them,
- which layer owns them,
- which names should disappear.

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
- `LocalState`
- `Ui`
- `UiChild`
- `ui`
- `shadcn` (feature-gated)
- `DepsBuilder` / `DepsSignature` (feature-gated with `state-selector`)
- `ThemeSnapshot`
- typed action/payload action macros plus `CommandId`

Target non-exports:

- `Event`
- `KernelApp`
- `AppWindowId`
- `ElementContext`
- `UiTree`
- `UiServices`
- `UiHost`
- `AnyElement`
- `ModelStore`
- `ActionId`
- `TypedAction`
- `UiBuilder`
- `UiPatchTarget`
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
- runner/driver traits
- viewport/interop traits
- broad component-author internals

Target rule:

- if a symbol is primarily useful for component authors or runner authors, it does not belong in
  `fret::app::prelude`.
- `fret::app::prelude` must use curated exports; it does not blanket re-export
  `fret_ui_kit::declarative::prelude::*`.
- `AppUi` is taught through grouped helper families:
  `cx.state().local*`, `cx.actions().models/locals/transient`,
  `cx.actions().payload::<A>().models/locals/local_update_if`, `cx.data().selector/query*`, and
  `cx.effects().take_transient(...)`.

App-level ecosystem integration seam:

- `FretApp::setup(fn(&mut App))` is the canonical builder hook for app-level integrations such as
  command registration, theme/bootstrap setup, optional shadcn installs, router/query defaults,
  and other non-reusable product wiring.
- `UiAppBuilder::setup_with(|app| ...)` is the inline closure variant for one-off app setup that
  still belongs on the default app path.
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
- `UiIntoElement`
- layout/style refinement types
- semantics/layout utilities needed by reusable components
- explicit overlay/focus composition helpers intended for reusable libraries

Target non-exports:

- app builder
- app-runtime-only globals/installation seams
- runner/manual assembly types

Target rule:

- a reusable component crate should be able to stay entirely on this surface unless it is
  intentionally shipping app-specific integration helpers.

## `fret::advanced`

Target exports:

- `ui_app(...)` / `ui_app_with_hooks(...)` for explicit golden-path manual assembly
- advanced builder extension traits (for `UiServices`, GPU-ready hooks, custom effects)
- driver hooks
- viewport embedding / interop seams
- manual assembly surfaces
- low-level runtime/service/context types
- explicit integration points for editor-grade products

Target rule:

- advanced users should still have full power, but must opt into it explicitly.

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
- selector dependency building through `DepsBuilder` from `fret::app::prelude::*`
- `query(...)`
- `query_async(...)`
- `query_async_local(...)`
- future router/state-library integration hooks

Target rule:

- flat `AppUi::use_selector*` / `AppUi::use_query*` helpers are removed from the default app
  surface; low-level `ElementContext` query/selector helpers remain explicit for component or
  advanced call sites.

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
| design-system kit (`fret-ui-shadcn`, future kits) | component surface + optional explicit app integration module (`shadcn::app`) + explicit theme/raw seams (`shadcn::themes`, `shadcn::raw`) | recipe crates must not define a competing app runtime or leak their full crate root onto the default app path |
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
- theme presets stay on `shadcn::themes::*`
- full crate-root escape hatches must be explicit via `shadcn::raw::*`

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
| component prelude is explicit | in progress |
| advanced surface is explicit | in progress |
| canonical naming reset landed | in progress |
| grouped `AppUi` namespaces landed | in progress |
| first-party ecosystems migrated | in progress |
| templates/docs aligned | in progress |
| old surface deleted | in progress |
| guards/gates added | in progress |

## Definition of Complete

This target state is complete when:

1. the names in this document match the real public exports,
2. official docs and templates teach the same names,
3. first-party ecosystem crates use the same extension seams described here,
4. the removed names are truly gone rather than merely discouraged.
