# Authoring Surface + Ecosystem (Fearless Refactor v1) — Migration Matrix

Status: execution tracker
Last updated: 2026-03-10

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
| App prelude | broad `fret::prelude::*` re-export posture | `fret::app::prelude::*` | explicit app module plus short-lived legacy bridge | no low-level mechanism types remain in `fret::app::prelude` and the legacy bridge is deleted | In progress | `ecosystem/fret/src/lib.rs`, `TARGET_INTERFACE_STATE.md` |
| Component prelude | mixed import posture via `fret-ui-kit` / recipe crates | `fret::component::prelude::*` | add explicit component module before moving call sites | first-party reusable crates no longer need app prelude | Scaffolding only | `TARGET_INTERFACE_STATE.md` |
| Advanced imports | accidental discovery through default imports | `fret::advanced::*` | explicit modules that forward to current low-level seams | app docs/templates no longer import low-level seams | Scaffolding only | `DESIGN.md` |
| App builder naming | `fret::App`, `AppBuilder`, `FretApp` mixed posture | `FretApp` canonical | short-lived alias from old builder name to new canonical name | docs/templates stop teaching `fret::App` | In progress | `ecosystem/fret/src/app_entry.rs`, `TARGET_INTERFACE_STATE.md` |
| Kernel runtime naming | bare `App` in app-facing imports | `KernelApp` | type alias only for migration, then stop exporting bare `App` on app path | app prelude no longer exposes kernel runtime as bare `App` | In progress | `TARGET_INTERFACE_STATE.md` |
| App UI context | `ViewCx` | `AppUi` | type alias first, grouped API second, delete old teaching last | templates/examples use `AppUi` and grouped namespaces | In progress | `ecosystem/fret/src/view.rs`, `TARGET_INTERFACE_STATE.md` |
| UI return alias | `Elements` as default taught return type | `Ui` | alias `Ui = Elements` | official examples/docs use `Ui` | In progress | `TARGET_INTERFACE_STATE.md` |
| Flat state helpers | `use_local*`, `use_state*` mixed default posture | `ui.state()` | wrapper namespace over current methods | default docs/templates stop teaching flat calls | Not started | `ecosystem/fret/src/view.rs` |
| Flat action helpers | `on_action*`, `on_action_notify*`, payload variants | `ui.actions()` | wrapper namespace over current handlers | default docs/templates stop teaching flat handler list | Not started | `ecosystem/fret/src/view.rs` |
| Selector/query hooks | `use_selector`, `use_query*` on flat context | `ui.data()` | namespace wrapper + extension traits | first-party state libs use grouped data surface | Not started | `ecosystem/fret/src/view.rs` |
| Effect helpers | app-bound side effects via ad-hoc patterns | `ui.effects()` | introduce grouped entry while forwarding to current mechanisms | default path has one documented effect story | Not started | `DESIGN.md` |

## Ecosystem Crates

| Crate / area | Current posture | Target posture | Migration notes | Old surface delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| `ecosystem/fret` | mixed app facade + broad prelude | clean app facade + app/component/advanced split | primary landing zone for naming and prelude reset | app docs/templates all teach the new names | In progress | `ecosystem/fret/src/lib.rs`, `ecosystem/fret/src/view.rs` |
| `ecosystem/fret-ui-kit` | component infra plus a broad prelude that leaks into app surface | explicit component-author surface | keep this powerful, but stop transitive leakage into app prelude | app prelude no longer bulk re-exports this prelude | Not started | `ecosystem/fret-ui-kit/src/lib.rs` |
| `ecosystem/fret-ui-shadcn` | recipe crate with some broad imports and optional app integration | recipe/taxonomy crate on top of component surface | preserve optional app integration, but keep it visibly separate | default examples do not rely on shadcn leaking mechanism types | Not started | `ecosystem/fret-ui-shadcn/src/lib.rs` |
| `ecosystem/fret-docking` | policy-heavy advanced UI layer | component + advanced split | docking should not redefine the default app mental model | app-facing docs use docking as advanced/component layer only | Not started | `docs/repo-structure.md`, docking sources |
| `ecosystem/fret-selector` | extension over current flat context | grouped app data extension | move to explicit `ui.data()` extension traits | docs/examples stop teaching direct flat hook calls as the primary selector story | Not started | selector crate, `DESIGN.md` |
| `ecosystem/fret-query` | extension over current flat context | grouped app data extension | keep optional, but make integration path obvious | docs/examples stop teaching direct flat query hooks as the primary query story | Not started | query crate, `DESIGN.md` |
| `ecosystem/fret-router` | app-level ecosystem with its own teaching posture | grouped app/advanced extension surface | app-owned routing surface should build on the same grouped app context | official routing examples use the new grouped posture | Not started | router crate, `DESIGN.md` |
| future third-party kits | no sharp public contract yet | choose app / component / advanced tier explicitly | publish the tier rules with this workstream | public docs define the tier rules clearly | Not started | `TARGET_INTERFACE_STATE.md` |

## Docs, Templates, and Examples

| Surface | Current posture | Target posture | Delete trigger for old guidance | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- |
| `README.md` | mostly converged, but still surface drift risk | only teach app-facing canonical names and grouped default model | docs consistency gate covers README | Migrated | `README.md` |
| `docs/README.md` | canonical index, but still references old names/mental models in places | points to new product surface and migration tracker | docs consistency gate covers docs index | Not started | `docs/README.md` |
| `docs/first-hour.md` | strong default guidance, still tied to current naming | teach `FretApp`, `AppUi`, grouped namespaces | templates and cookbook match exactly | Not started | `docs/first-hour.md` |
| scaffold templates | current `hello/simple-todo/todo` teach old surface names | teach only new app surface | template gate added and green | Migrated | `apps/fretboard/src/scaffold/templates.rs` |
| cookbook official examples | current view runtime posture | new app surface posture | no official example teaches the old surface | In progress | `apps/fret-cookbook/examples/` |
| comparison/advanced examples | mixed by design | explicit advanced/component imports where appropriate | examples taxonomy is clear and documented | Not started | `docs/examples/README.md` |

## Hard Delete Matrix

These are the concrete "remove this" lanes.

| Old symbol / posture | Replacement | Delete when | Status |
| --- | --- | --- | --- |
| `fret::App` as default taught builder name | `FretApp` | app docs/templates/examples all migrated | In progress |
| bare `App` in app-facing prelude for kernel runtime | `KernelApp` | app prelude no longer exports it bare and call sites are migrated | In progress |
| `ViewCx` as the taught app-facing context name | `AppUi` | grouped context APIs are live and official surfaces migrated | In progress |
| `Elements` as the taught app-facing return alias | `Ui` | official examples/docs migrated | In progress |
| flat default-path calls like `use_local*` in official docs/templates | `ui.state()` | templates + docs migrated | Not started |
| flat default-path calls like `on_action_notify_*` in official docs/templates | `ui.actions()` | templates + docs migrated | Not started |
| broad `fret::prelude::*` bridge and app-prelude mechanism leakage | `fret::app` / `fret::component` / `fret::advanced` split | explicit surface gate exists, passes, and the bridge is deleted | In progress |
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
