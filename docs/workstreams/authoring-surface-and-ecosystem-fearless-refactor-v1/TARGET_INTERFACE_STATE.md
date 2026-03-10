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
| kernel runtime app | `KernelApp` | Advanced / component as needed | no longer taught as bare `App` in app prelude |
| app-facing view context | `AppUi` | App | replaces `ViewCx` as the taught surface |
| rendered UI return alias | `Ui` | App | canonical alias over `Elements` |
| reusable component context | `ComponentCx` or explicit component-surface alias | Component | wraps the mechanism context for reusable component work |

## `fret::app::prelude` (App Surface)

Target exports:

- `FretApp`
- `View`
- `AppUi`
- `LocalState`
- `Ui`
- `ui`
- `shadcn` (feature-gated)
- `ThemeSnapshot`
- typed action/payload action macros and IDs needed by ordinary apps

Target non-exports:

- `ElementContext`
- `UiTree`
- `UiServices`
- `ModelStore`
- runner/driver traits
- viewport/interop traits
- broad component-author internals

Target rule:

- if a symbol is primarily useful for component authors or runner authors, it does not belong in
  `fret::app::prelude`.

Legacy bridge to delete:

- `fret::prelude::*` may exist only as a short-lived in-repo migration bridge and is not the target
  public import surface.

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
- `local_init(|| ...)`
- `watch(&state)`
- `get(&state)` / `get_or(...)` through transactions or explicit watch handles

### `ui.actions()`

Target operations:

- `locals::<A>(|tx| ...)`
- `payload::<A>().local(&state, |value, payload| ...)`
- `transient::<A>(...)`
- explicit advanced escape hatch for shared model graphs

### `ui.data()`

Target operations:

- `selector(...)`
- `query(...)`
- future router/state-library integration hooks

### `ui.effects()`

Target operations:

- app-bound side-effect helpers intended for the default app surface

Target rule:

- app authors should discover behavior by going to the relevant namespace first, not by scanning a
  flat list of dozens of methods.

## Ecosystem Integration Targets

| Crate category | Target integration model | Notes |
| --- | --- | --- |
| design-system kit (`fret-ui-shadcn`, future kits) | component surface + optional app integration module | recipe crates must not define a competing app runtime |
| docking | component surface + explicit advanced seams | keeps docking policy powerful without leaking runner ideas into the app default path |
| selector/query/router | grouped app-surface extension traits | first-party ecosystems must use the same seams expected of third parties |
| third-party reusable kits | component surface | default choice for portable UI packages |
| third-party workflow/app addons | app surface | acceptable when the crate is intentionally app-level |
| third-party interop crates | advanced surface | explicit power-user posture |

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
| grouped `AppUi` namespaces landed | not started |
| first-party ecosystems migrated | in progress |
| templates/docs aligned | in progress |
| old surface deleted | not started |
| guards/gates added | not started |

## Definition of Complete

This target state is complete when:

1. the names in this document match the real public exports,
2. official docs and templates teach the same names,
3. first-party ecosystem crates use the same extension seams described here,
4. the removed names are truly gone rather than merely discouraged.
