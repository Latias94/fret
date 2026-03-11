# Authoring Surface + Ecosystem (Fearless Refactor v1) — Migration Matrix

Status: execution tracker
Last updated: 2026-03-11

This file is the execution-oriented companion to:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`

Its purpose is to keep the refactor concrete:

- what is moving,
- where it is moving to,
- what still blocks deletion,
- and when the old implementation can be removed.

This workstream is pre-release and does **not** optimize for compatibility.
Temporary aliases/adapters are allowed only as short-lived in-repo migration scaffolding.

## Status Legend

| Status | Meaning |
| --- | --- |
| Not started | no migration code landed |
| Scaffolding only | temporary alias/adapter exists, but call sites still teach/use the old path |
| In progress | new path exists and active call-site migration is underway |
| Migrated | official call sites use the new path |
| Delete-ready | migrated + docs/gates aligned; old path can be removed |
| Deleted | old path and stale docs are gone |
| Blocked | waiting on a higher-level naming or surface decision |

## Global Deletion Rule

An old surface is eligible for deletion only when all of the following are true:

1. templates are migrated,
2. official docs are migrated,
3. cookbook/default examples are migrated,
4. first-party ecosystem crates on the default path are migrated,
5. a gate exists that prevents regressions back to the old path.

## Surface Lanes

| Lane | Old surface / implementation | Target surface | Temporary migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| App prelude | broad `fret::prelude::*` re-export posture | `fret::app::prelude::*` | explicit app module during migration, then delete the root bridge | no low-level mechanism types remain in `fret::app::prelude` and the legacy bridge is deleted | Deleted | `ecosystem/fret/src/lib.rs`, `TARGET_INTERFACE_STATE.md` |
| Component prelude | mixed import posture via `fret-ui-kit` / recipe crates | `fret::component::prelude::*` | add explicit component module before moving call sites | first-party reusable crates no longer need app prelude | Scaffolding only | `TARGET_INTERFACE_STATE.md` |
| Advanced imports | accidental discovery through default imports | `fret::advanced::*` | explicit modules that forward to current low-level seams | app docs/templates no longer import low-level seams | In progress | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/examples/` |
| App builder naming | `fret::App`, `AppBuilder`, `FretApp` mixed posture | `FretApp` canonical | migrate call sites/docs to `FretApp`, then delete old aliases | docs/templates stop teaching `fret::App` | Deleted | `ecosystem/fret/src/app_entry.rs`, `TARGET_INTERFACE_STATE.md` |
| Default builder setup seam | mixed `install_app` / `install` / `init_app` naming on the default `fret` path | `FretApp::setup(...)` and `UiAppBuilder::setup_with(...)` | migrate docs/examples/tests on the `fret` facade and keep raw bootstrap names explicit | default docs/templates/examples no longer teach legacy setup names on the `fret` facade | In progress | `ecosystem/fret/src/app_entry.rs`, `ecosystem/fret/src/lib.rs`, `docs/workstreams/app-entry-builder-v1/DESIGN.md` |
| App runtime naming | `KernelApp` on the default app surface | `fret::app::App` | add an app-surface alias, then migrate docs/templates/examples | app prelude exports `App` and no longer exports `KernelApp` | In progress | `TARGET_INTERFACE_STATE.md` |
| App UI context | `ViewCx` | `AppUi` | type alias first, grouped API second, delete old teaching last | templates/examples use `AppUi` and grouped namespaces | In progress | `ecosystem/fret/src/view.rs`, `TARGET_INTERFACE_STATE.md` |
| UI return alias | `Elements` as default taught return type | `Ui` | alias `Ui = Elements` | official examples/docs use `Ui` | In progress | `TARGET_INTERFACE_STATE.md` |
| Flat state helpers | `use_local*`, `use_state*` mixed default posture | `ui.state()` | wrapper namespace over current methods | default docs/templates stop teaching flat calls | In progress | `ecosystem/fret/src/view.rs` |
| Flat action helpers | `on_action*`, `on_action_notify*`, payload variants | `ui.actions()` | wrapper namespace over current handlers | default docs/templates stop teaching flat handler list | In progress | `ecosystem/fret/src/view.rs` |
| Selector/query hooks | `use_selector`, `use_query*` on flat context | `ui.data()` | namespace wrapper + extension traits | first-party state libs use grouped data surface | In progress | `ecosystem/fret/src/view.rs`, `apps/fret-examples/src/query_demo.rs`, `docs/authoring-golden-path-v2.md` |
| Effect helpers | app-bound side effects via ad-hoc patterns | `ui.effects()` | introduce grouped entry while forwarding to current mechanisms | default path has one documented effect story | In progress | `DESIGN.md` |

## Ecosystem Crates

| Crate / area | Current posture | Target posture | Migration notes | Old surface delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| `ecosystem/fret` | mixed app facade + broad prelude | clean app facade + app/component/advanced split | primary landing zone for naming and prelude reset | app docs/templates all teach the new names | In progress | `ecosystem/fret/src/lib.rs`, `ecosystem/fret/src/view.rs` |
| `ecosystem/fret-ui-kit` | component infra plus a broad prelude that leaks into app surface | explicit component-author surface | keep this powerful, but stop transitive leakage into app prelude | app prelude no longer bulk re-exports this prelude | Not started | `ecosystem/fret-ui-kit/src/lib.rs` |
| `ecosystem/fret-ui-shadcn` | recipe crate with some broad imports and optional app integration | recipe/taxonomy crate on top of component surface + explicit `shadcn::app` seam | keep optional app integration visible without turning shadcn root into a second app-runtime namespace; expose `fret::shadcn` as a curated facade with explicit `themes` / `raw` seams | default examples/docs use `shadcn::app::*` for setup, `shadcn::themes::*` for presets, and keep fully raw access explicit | In progress | `ecosystem/fret-ui-shadcn/src/lib.rs`, `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/examples/`, `docs/crate-usage-guide.md` |
| `ecosystem/fret-docking` | policy-heavy advanced UI layer | component + advanced split via explicit `fret::docking` facade | keep docking opt-in and visible without turning it into a parallel default runtime | app-facing docs/examples use `fret::docking::*` while advanced/component call sites stay explicit | In progress | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `docs/crate-usage-guide.md` |
| `ecosystem/fret-selector` | extension over current flat context | grouped app data extension | `fret::app::prelude::*` now re-exports `DepsBuilder` / `DepsSignature`; remaining low-level `ui` entry stays explicit for component/advanced surfaces | docs/examples stop teaching direct flat hook calls as the primary selector story | In progress | `ecosystem/fret/src/lib.rs`, `docs/authoring-golden-path-v2.md`, `apps/fretboard/src/scaffold/templates.rs` |
| `ecosystem/fret-query` | extension over current flat context | grouped app data extension | keep optional, but make the default app path obvious through `cx.data().query*`; retain raw `fret-query/ui` only for explicit low-level surfaces | docs/examples stop teaching direct flat query hooks as the primary query story | In progress | `ecosystem/fret/src/view.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`, `docs/integrating-tokio-and-reqwest.md` |
| `ecosystem/fret-router` | app-level ecosystem with its own teaching posture | explicit `fret::router` app extension module + advanced escape hatches | keep `fret-router-ui` as a thin app-owned adoption layer instead of a parallel runtime | official routing examples use `fret::router::*` instead of direct crate pairing | In progress | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `ecosystem/fret-router-ui/src/lib.rs` |
| future third-party kits | no sharp public contract yet | choose app / component / advanced tier explicitly | publish the tier rules with this workstream | public docs define the tier rules clearly | Not started | `TARGET_INTERFACE_STATE.md` |

## Docs, Templates, and Examples

| Surface | Current posture | Target posture | Delete trigger for old guidance | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- |
| `README.md` | mostly converged, but still surface drift risk | only teach app-facing canonical names and grouped default model | docs consistency gate covers README | Migrated | `README.md` |
| `docs/README.md` | canonical index, but still references old names/mental models in places | points to new product surface and migration tracker | docs consistency gate covers docs index | Migrated | `docs/README.md`, `ecosystem/fret/src/lib.rs`, `tools/gate_fret_builder_only_surface.py` |
| `docs/first-hour.md` | strong default guidance, still tied to current naming | teach `FretApp`, `AppUi`, grouped namespaces | templates and cookbook match exactly | Migrated | `docs/first-hour.md`, `ecosystem/fret/src/lib.rs`, `tools/gate_fret_builder_only_surface.py` |
| `docs/authoring-golden-path-v2.md` | mixed grouped and flat helper language | grouped `state/actions/data/effects` authoring guidance | docs gate covers grouped selector/query helpers | Migrated | `docs/authoring-golden-path-v2.md`, `ecosystem/fret/src/lib.rs` |
| `ecosystem/fret/README.md` | feature list missed explicit router extension seam | documents `router` as an opt-in app-level module | crate README gate covers router feature surface | In progress | `ecosystem/fret/README.md`, `ecosystem/fret/src/lib.rs` |
| scaffold templates | current `hello/simple-todo/todo` teach old surface names | teach only new app surface | template gate added and green | Migrated | `apps/fretboard/src/scaffold/templates.rs` |
| async integration docs | low-level `ElementContext` query helpers as the only visible story | default app surface first, low-level note second | docs gate covers grouped async query helpers | In progress | `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md`, `ecosystem/fret/src/lib.rs` |
| cookbook official examples | current view runtime posture | new app surface posture | no official example teaches the old surface | Migrated | `apps/fret-cookbook/examples/`, `apps/fret-cookbook/src/lib.rs` |
| comparison/advanced examples | mixed by design | explicit advanced/component imports where appropriate | examples taxonomy is clear and documented | Migrated | `apps/fret-cookbook/examples/`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs`, `apps/fret-ui-gallery/src/lib.rs` |

## Hard Delete Matrix

These are the concrete "remove this" lanes.

| Old symbol / posture | Replacement | Delete when | Status |
| --- | --- | --- | --- |
| `fret::App` as default taught builder name | `FretApp` | app docs/templates/examples all migrated | Deleted |
| `KernelApp` on the default app surface | `App` | app prelude no longer exports `KernelApp` and official app-path call sites are migrated | In progress |
| `ViewCx` as the taught app-facing context name | `AppUi` | grouped context APIs are live and official surfaces migrated | In progress |
| `Elements` as the taught app-facing return alias | `Ui` | official examples/docs migrated | In progress |
| `run_view::<V>()` / `run_view_with_hooks::<V>(...)` convenience entry | `view::<V>()?.run()` / `view_with_hooks::<V>(...)?.run()` | default docs/templates/examples are migrated and a gate forbids the convenience APIs from returning | Deleted |
| flat default-path calls like `use_local*` in official docs/templates | `ui.state()` | templates + docs migrated | In progress |
| flat default-path calls like `on_action_notify_*` in official docs/templates | `ui.actions()` | templates + docs migrated | In progress |
| broad `fret::prelude::*` bridge and app-prelude mechanism leakage | `fret::app` / `fret::component` / `fret::advanced` split | explicit surface gate exists, passes, and the bridge is deleted | Deleted |
| stale docs that teach superseded names | rewritten docs | new docs merged and linked | In progress |

## Recommended Execution Order

| Order | Track | Why |
| --- | --- | --- |
| 1 | canonical naming reset | every other migration table entry depends on the names stabilizing first |
| 2 | prelude split | encodes the product tiers in imports early |
| 3 | grouped `AppUi` namespaces | lets templates/docs migrate without waiting for deep runtime rewrites |
| 4 | docs/templates/cookbook migration | proves the new surface is actually teachable |
| 5 | first-party ecosystem migrations | aligns `shadcn`, `docking`, `query`, `selector`, `router` on one public story |
| 6 | hard-delete old surface | remove aliases/helpers once no official path relies on them |
| 7 | add and tighten guardrails | lock the cleaned surface against drift |

## Completion Rule

This migration matrix is complete when every row that describes an old public-facing surface is
either:

- `Deleted`, or
- intentionally retained as an explicit advanced seam outside the default app surface.
