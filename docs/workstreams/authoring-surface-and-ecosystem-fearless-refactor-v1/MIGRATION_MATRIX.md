# Authoring Surface + Ecosystem (Fearless Refactor v1) — Migration Matrix

Status: execution tracker
Last updated: 2026-03-12

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
| Component prelude | mixed import posture via `fret-ui-kit` / recipe crates | `fret::component::prelude::*` | replace blanket `fret-ui-kit::prelude::*` forwarding with curated component-author exports and migrate first-party helper surfaces | first-party reusable crates no longer need app prelude and component docs/examples stay on the curated surface | In progress | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/scaffold.rs`, `TARGET_INTERFACE_STATE.md` |
| Advanced imports | accidental discovery through default imports | `fret::advanced::*` | explicit modules that forward to current low-level seams; keep `advanced::prelude::*` curated instead of blanket-forwarding `fret_ui_kit::prelude::*` | app docs/templates no longer import low-level seams | In progress | `ecosystem/fret/src/lib.rs`, `ecosystem/fret/tests/advanced_prelude_surface.rs`, `apps/fret-cookbook/examples/` |
| App builder naming | `fret::App`, `AppBuilder`, `FretApp` mixed posture | `FretApp` canonical | migrate call sites/docs to `FretApp`, then delete old aliases | docs/templates stop teaching `fret::App` | Deleted | `ecosystem/fret/src/app_entry.rs`, `TARGET_INTERFACE_STATE.md` |
| Default builder setup seam | mixed `install_app` / `install` / `init_app` naming on the default `fret` path | `FretApp::setup(...)` and `UiAppBuilder::setup_with(...)` | migrate docs/examples/tests on the `fret` facade and keep raw bootstrap names explicit | default docs/templates/examples no longer teach legacy setup names on the `fret` facade | In progress | `ecosystem/fret/src/app_entry.rs`, `ecosystem/fret/src/lib.rs`, `docs/workstreams/app-entry-builder-v1/DESIGN.md` |
| App runtime naming | `KernelApp` on the default app surface | `fret::app::App` | add an app-surface alias, then migrate docs/templates/examples | app prelude exports `App`, first-party app-path examples stop spelling `KernelApp`, and the retained raw alias lives only under `fret::advanced::*` | Deleted | `ecosystem/fret/src/lib.rs`, `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`, `TARGET_INTERFACE_STATE.md` |
| App UI context | `ViewCx` | `AppUi` | type alias first, grouped API second, then move the runtime-owned context type itself to `AppUi` and delete `ViewCx` once official surfaces are migrated | templates/examples use `AppUi`, grouped namespaces are live, and the runtime-owned context no longer defines `ViewCx` at all | Deleted | `ecosystem/fret/src/view.rs`, `apps/fret-ui-gallery/src/ui/pages/command.rs`, `TARGET_INTERFACE_STATE.md` |
| Extracted app helper context | raw `ElementContext<'_, App>` / `ElementContext<'_, KernelApp>` in default app-facing extracted helpers | `UiCx` | add the alias first, then migrate helper-style snippets/examples that do not need a generic host type in their teaching surface | first-party app-facing snippets/docs stop spelling raw `ElementContext` when `UiCx` is enough; generic `H: UiHost` reusable snippets and gallery page-host composition stay explicit | Migrated | `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs`, `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`, `TARGET_INTERFACE_STATE.md` |
| UI return alias | `Elements` as default taught return type | `Ui` | alias `Ui = Elements` | official examples/docs use `Ui` and retained runtime helpers no longer surface bare `Elements` on the app-facing path | Migrated | `ecosystem/fret/src/view.rs`, `TARGET_INTERFACE_STATE.md` |
| Flat state helpers | public `use_local*` and `use_state*` mixed default posture on `AppUi` | `ui.state()` for the default path, raw `use_state*` only through an explicit advanced trait seam | delete public `use_local*`, then move raw `use_state*` off `AppUi` once first-party app surfaces are migrated | default docs/templates stop teaching flat calls and `AppUi` no longer exposes public `use_local*` / `use_state*` helpers | Deleted | `ecosystem/fret/src/view.rs`, `TARGET_INTERFACE_STATE.md` |
| Flat action helpers | specialized `AppUi` mutation helpers (`on_action_notify_local_*`, `on_action_notify_models/locals/transient`, payload-local variants) mixed into the same surface as raw registration seams | `ui.actions()` for the default path, raw `on_action*` / `on_payload_action*` registration only as explicit advanced seams | route default mutations through `ui.actions()` and delete the redundant specialized flat wrappers once first-party app surfaces are migrated | default docs/templates stop teaching flat handler list and `AppUi` no longer exposes the redundant mutation wrappers | Deleted | `ecosystem/fret/src/view.rs`, `TARGET_INTERFACE_STATE.md` |
| Selector/query hooks | `use_selector`, `use_query*` on flat context | `ui.data()` | namespace wrapper + extension traits, then delete flat `AppUi` data helpers once first-party app surfaces are migrated | first-party state libs use grouped data surface, including extracted `UiCx` helpers on app-facing examples | Deleted | `ecosystem/fret/src/view.rs`, `apps/fret-examples/src/async_playground_demo.rs`, `apps/fret-examples/src/markdown_demo.rs`, `ecosystem/fret/tests/uicx_data_surface.rs`, `docs/authoring-golden-path-v2.md` |
| Effect helpers | app-bound side effects via ad-hoc patterns | `ui.effects()` | introduce grouped entry while forwarding to current mechanisms, then delete the old flat render-time transient helper | default path has one documented effect story | Deleted | `ecosystem/fret/src/view.rs`, `apps/fret-examples/src/query_demo.rs`, `apps/fret-examples/src/markdown_demo.rs` |

## Ecosystem Crates

| Crate / area | Current posture | Target posture | Migration notes | Old surface delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| `ecosystem/fret` | mixed app facade + broad prelude | clean app facade + app/component/advanced split | primary landing zone for naming and prelude reset; `advanced::prelude::*` must stay a curated advanced lane rather than a hidden `fret_ui_kit::prelude::*` tunnel, low-level action registry aliases stay under `fret::actions::*`, workspace-shell helpers stay under `fret::workspace_shell::*`, and ecosystem app setup on the default lane flows through explicit `.setup(...::app::install)` seams instead of root aliases or pack-specific builder helpers | app docs/templates all teach the new names and compatibility-only root aliases are removed | In progress | `ecosystem/fret/src/lib.rs`, `ecosystem/fret/src/view.rs`, `ecosystem/fret/tests/advanced_prelude_surface.rs` |
| `ecosystem/fret-ui-kit` | component infra plus a broad prelude that leaks into app surface | explicit component-author surface | keep this powerful, but stop transitive leakage into app prelude | app prelude no longer bulk re-exports this prelude | Not started | `ecosystem/fret-ui-kit/src/lib.rs` |
| `ecosystem/fret-ui-shadcn` | recipe crate with some broad imports and optional app integration | recipe/taxonomy crate on top of component surface + explicit `shadcn::app` seam, with advanced service hooks kept off the curated facade | keep optional app integration visible without turning shadcn root into a second app-runtime namespace; expose `fret::shadcn` as a curated facade with explicit `themes` / `raw` seams, teach direct-crate advanced hooks through `fret_ui_shadcn::advanced::*` instead of the raw crate root, and keep first-party raw usage limited to documented escape-hatch lanes | default examples/docs use `shadcn::app::*` for setup, `shadcn::themes::*` for presets, keep advanced hooks explicit, and keep fully raw access explicit through `shadcn::raw::*` | Migrated | `ecosystem/fret-ui-shadcn/src/lib.rs`, `ecosystem/fret-ui-shadcn/src/advanced.rs`, `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs`, `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`, `docs/crate-usage-guide.md`, `docs/shadcn-declarative-progress.md` |
| `ecosystem/fret-docking` | policy-heavy advanced UI layer | component + advanced split via explicit `fret::docking` facade | keep docking opt-in and visible without turning it into a parallel default runtime; first-party app-facing demos/docs should use `fret::docking::*`, while advanced harnesses and interop examples keep direct `fret_docking::*` imports explicit | app-facing docs/examples use `fret::docking::*` while advanced/component call sites stay explicit | Migrated | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `apps/fret-examples/src/docking_demo.rs`, `apps/fret-examples/src/container_queries_docking_demo.rs`, `apps/fret-examples/src/lib.rs`, `docs/crate-usage-guide.md` |
| `ecosystem/fret-selector` | extension over current flat context | grouped app data extension | `fret::app::prelude::*` now re-exports `DepsBuilder` / `DepsSignature`; remaining low-level `ui` entry stays explicit for component/advanced surfaces | docs/examples stop teaching direct flat hook calls as the primary selector story, and raw selector hooks remain explicit outside the app path | Migrated | `ecosystem/fret/src/lib.rs`, `docs/crate-usage-guide.md`, `docs/authoring-golden-path-v2.md`, `apps/fretboard/src/scaffold/templates.rs`, `ecosystem/fret/tests/crate_usage_grouped_selector_surface.rs`, `ecosystem/fret/tests/uicx_data_surface.rs` |
| `ecosystem/fret-query` | extension over current flat context | grouped app data extension | keep optional, but make the default app path obvious through `cx.data().query*`; retain raw `fret-query/ui` only for explicit low-level surfaces | docs/examples stop teaching direct flat query hooks as the primary query story, and helper-heavy samples keep grouped `cx.data().query*` entry points | Migrated | `ecosystem/fret/src/view.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`, `apps/fret-examples/src/async_playground_demo.rs`, `ecosystem/fret/tests/uicx_data_surface.rs`, `ecosystem/fret/tests/crate_usage_grouped_query_surface.rs`, `docs/integrating-tokio-and-reqwest.md` |
| `ecosystem/fret-router` | app-level ecosystem with its own teaching posture | explicit `fret::router` app extension module + advanced escape hatches | keep `fret-router-ui` as a thin app-owned adoption layer instead of a parallel runtime; first-party app-facing docs/examples should use `fret::router::*`, leaving direct crate imports to lower-level platform or advanced integration seams | official routing examples use `fret::router::*` instead of direct crate pairing, and `fret-router-ui` keeps only thin adoption helpers | Migrated | `ecosystem/fret/src/lib.rs`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `apps/fret-ui-gallery/tests/router_facade_surface.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `docs/crate-usage-guide.md` |
| future third-party kits | no sharp public contract yet | choose app / component / advanced tier explicitly | publish the tier rules with this workstream | public docs define the tier rules clearly | Not started | `TARGET_INTERFACE_STATE.md` |

## Docs, Templates, and Examples

| Surface | Current posture | Target posture | Delete trigger for old guidance | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- |
| `README.md` | mostly converged, but still surface drift risk | only teach app-facing canonical names and grouped default model | docs consistency gate covers README | Migrated | `README.md` |
| `docs/README.md` | canonical index, but still references old names/mental models in places | points to new product surface and migration tracker | docs consistency gate covers docs index | Migrated | `docs/README.md`, `ecosystem/fret/src/lib.rs`, `tools/gate_fret_builder_only_surface.py` |
| `docs/first-hour.md` | strong default guidance, still tied to current naming | teach `FretApp`, `AppUi`, grouped namespaces | templates and cookbook match exactly | Migrated | `docs/first-hour.md`, `ecosystem/fret/src/lib.rs`, `tools/gate_fret_builder_only_surface.py` |
| `docs/authoring-golden-path-v2.md` | mixed grouped and flat helper language | grouped `state/actions/data/effects` authoring guidance | docs gate covers grouped selector/query helpers | Migrated | `docs/authoring-golden-path-v2.md`, `ecosystem/fret/src/lib.rs` |
| `ecosystem/fret/README.md` | feature list missed explicit router extension seam | documents `router` as an opt-in app-level module | crate README gate covers router feature surface | Migrated | `ecosystem/fret/README.md`, `ecosystem/fret/src/lib.rs` |
| scaffold templates | current `hello/simple-todo/todo` teach old surface names | teach only new app surface | template gate added and green | Migrated | `apps/fretboard/src/scaffold/templates.rs` |
| async integration docs | low-level `ElementContext` query helpers as the only visible story | default app surface first, low-level note second | docs gate covers grouped async query helpers | In progress | `docs/integrating-tokio-and-reqwest.md`, `docs/integrating-sqlite-and-sqlx.md`, `ecosystem/fret/src/lib.rs` |
| cookbook official examples | current view runtime posture | new app surface posture | no official example teaches the old surface | Migrated | `apps/fret-cookbook/examples/`, `apps/fret-cookbook/src/lib.rs` |
| comparison/advanced examples | mixed by design | explicit advanced/component imports where appropriate | examples taxonomy is clear and documented | Migrated | `apps/fret-cookbook/examples/`, `apps/fret-cookbook/src/lib.rs`, `apps/fret-examples/src/lib.rs`, `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs` |

## Hard Delete Matrix

These are the concrete "remove this" lanes.

| Old symbol / posture | Replacement | Delete when | Status |
| --- | --- | --- | --- |
| `fret::App` as default taught builder name | `FretApp` | app docs/templates/examples all migrated | Deleted |
| `KernelApp` on the default app surface | `App` | app prelude no longer exports `KernelApp`, official app-path call sites are migrated, and the retained raw alias lives only under `fret::advanced::*` | Deleted |
| `ViewCx` as the taught app-facing context name | `AppUi` | grouped context APIs are live, official surfaces migrated, and the retained alias is deleted | Deleted |
| `Elements` as the taught app-facing return alias | `Ui` | official examples/docs migrated and retained runtime helpers stop returning bare `Elements` on the app path | Migrated |
| `run_view::<V>()` / `run_view_with_hooks::<V>(...)` convenience entry | `view::<V>()?.run()` / `view_with_hooks::<V>(...)?.run()` | default docs/templates/examples are migrated and a gate forbids the convenience APIs from returning | Deleted |
| flat `AppUi::use_selector*` / `AppUi::use_query*` helper family | `cx.data().selector/query*` | first-party app-facing code is migrated and `AppUi` no longer exposes the flat methods | Deleted |
| `AppUi::take_transient_on_action_root(...)` | `cx.effects().take_transient(...)` | first-party app-facing code is migrated and `AppUi` no longer exposes the flat helper | Deleted |
| `AppUi::use_state*` / `AppUi::use_local*` | `cx.state()` for the default path, `AppUiRawStateExt::use_state*` as the explicit advanced raw-model seam | `AppUi` no longer exposes the flat methods directly | Deleted |
| flat default-path calls like `use_local*` in official docs/templates | `ui.state()` | templates + docs migrated and public `AppUi::use_local*` helpers are deleted | Deleted |
| flat default-path calls like `on_action_notify_*` in official docs/templates | `ui.actions()` | templates + docs migrated and specialized `AppUi` mutation wrappers are deleted | Deleted |
| broad `fret::prelude::*` bridge and app-prelude mechanism leakage | `fret::app` / `fret::component` / `fret::advanced` split | explicit surface gate exists, passes, and the bridge is deleted | Deleted |
| root-level `fret::ActionMeta` / `fret::ActionRegistry` compatibility aliases | `fret::actions::{ActionMeta, ActionRegistry, ActionRegistryExt, ...}` | root facade no longer teaches registry internals and source gates keep the aliases scoped to `fret::actions::*` | Deleted |
| root-level `fret::IconRegistry` alias | explicit `fret-icons::IconRegistry` dependency for raw registry work, or `.setup(...::app::install)` on the default app lane | root facade no longer leaks icon registry internals and source gates keep app-facing icon setup on explicit installer seams | Deleted |
| `FretApp::register_icon_pack(...)` / `UiAppBuilder::register_icon_pack(...)` / `UiAppBuilder::with_lucide_icons()` | `FretApp::setup(...::app::install)` / `UiAppBuilder::setup(...::app::install)` on `fret`, raw icon helpers on `fret-bootstrap::BootstrapBuilder` | default `fret` docs/examples stop teaching builder-level icon-pack helpers and source gates keep raw icon registration on explicit bootstrap/manual-assembly surfaces | Deleted |
| root-level `fret::router::install_app(...)` exception | `fret::router::app::install(...)` | router setup follows the same explicit ecosystem installer shape as shadcn/icons/assets/node, and source/doc gates keep the root exception deleted | Deleted |
| root-level `fret::run_native_with_compat_driver(...)` | `fret::advanced::interop::run_native_with_compat_driver(...)` | retained low-level interop stays available, but no longer occupies the main default-facing facade; docs/gates keep the quarantined path explicit | Deleted |
| root-level `fret::run_native_with_fn_driver(...)` / `run_native_with_fn_driver_with_hooks(...)` / `run_native_with_configured_fn_driver(...)` | `fret::advanced::run_native_with_fn_driver(...)` / `fret::advanced::run_native_with_fn_driver_with_hooks(...)` / `fret::advanced::run_native_with_configured_fn_driver(...)` | advanced runner escape hatches stay available, but no longer occupy the main default-facing facade; docs/gates keep the advanced path explicit | Deleted |
| `fret/icons-lucide` compatibility feature alias | `fret/icons` | feature docs/templates/examples only teach the canonical `icons` feature and source gates keep the alias deleted | Deleted |
| root-level `fret::workspace_shell_model*` shortcuts | `fret::workspace_shell::{workspace_shell_model, workspace_shell_model_default_menu}` | root facade no longer forwards editor/workspace-shell helpers and source gates keep them module-scoped | Deleted |
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
