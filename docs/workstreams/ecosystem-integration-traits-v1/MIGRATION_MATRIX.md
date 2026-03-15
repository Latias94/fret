# Ecosystem Integration Traits v1 — Migration Matrix

Status: Maintenance closeout tracker
Last updated: 2026-03-15

This matrix keeps the ecosystem integration cleanup concrete:

- what the current posture is,
- what the target seam should be,
- what still blocks deletion,
- and which old guidance must disappear.

This workstream is pre-release and does not optimize for compatibility.

Closeout note on 2026-03-15:

- the accepted ecosystem seams are landed and now have an explicit evidence set,
- `QueryAdapter` is no longer a pending design question for v1; it is intentionally deferred,
- the remaining open rows on this matrix are primarily docs/export cleanup and delete-ready
  follow-through.
- reusable helper signature audits are now closed for the first-party workspace shell and router
  helpers, with typed inputs preserved until explicit landing seams.

## Status Legend

| Status | Meaning |
| --- | --- |
| Not started | no target migration code or docs exist yet |
| Scaffolding only | target concept documented or partially wired, but official call sites still rely on the old posture |
| In progress | target path exists and first-party migration is underway |
| Migrated | official call sites use the target path |
| Delete-ready | old posture is no longer taught and can be removed |
| Deleted | old posture and docs are gone |
| Deferred | intentionally postponed with written rationale |

## Global Delete Rule

An old ecosystem integration posture is delete-ready only when:

1. official docs are migrated,
2. templates/examples are migrated where applicable,
3. first-party ecosystems use the target seam,
4. the old posture is no longer described as a recommended path.

## Cross-Ecosystem Lanes

| Lane | Current posture | Target posture | Migration tactic | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| App installer composition | mixed `install_app`, `install`, and ad-hoc bundle patterns | canonical installer functions plus `InstallIntoApp` for composition | add a thin trait adapter over existing installer functions; allow small tuple composition but keep reusable packs named | official examples use one consistent app wiring story | Migrated | `ecosystem/fret/src/integration.rs`, `ecosystem/fret/src/app_entry.rs`, `ecosystem/fret/src/lib.rs`, `ecosystem/fret/README.md`, `apps/fret-cookbook/examples/docking_basics.rs`, `docs/crate-usage-guide.md`, `DESIGN.md` |
| Universal plugin pressure | temptation to widen `fret-app::Plugin` into the ecosystem default | keep `Plugin` app-owned and minimal | document explicit anti-goal and route domain needs to small traits | no docs or first-party code teach `Plugin` as the universal model | Delete-ready | `crates/fret-app/src/plugins.rs`, `docs/adr/0016-plugin-and-panel-boundaries.md`, `docs/crate-usage-guide.md`, `DESIGN.md` |
| Command palette/catalog extension | shadcn recipe code used to own host-command collection and catalog shaping | `CommandCatalog` data contract in a reusable policy layer | move host-command collection/gating/shortcut derivation to `fret-ui-kit::command`, keep recipe crates as mapping/render layers | bootstrap and recipe crates consume, rather than own, the shared catalog contract | Migrated | `ecosystem/fret-ui-kit/src/command.rs`, `ecosystem/fret-ui-shadcn/src/command.rs`, `ecosystem/fret-bootstrap/src/ui_app_driver.rs`, `DESIGN.md` |
| Typed route integration | typed routes are possible, but there is no shared codec seam yet | `RouteCodec` | add a shared route encode/decode contract next to router core | official router examples stop teaching stringly route construction as the primary path | Migrated | `ecosystem/fret-router/src/codec.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `apps/fret-ui-gallery/src/spec.rs`, `apps/fret-ui-gallery/src/driver/router.rs`, `apps/fret-demo-web/src/wasm.rs`, `docs/workstreams/router-v1/router-v1.md`, `DESIGN.md` |
| Reusable dock panel contributions | app-owned registry service existed, but contribution-level factory was not standardized | `DockPanelFactory` aggregated into the dock registry story | add `DockPanelFactory` + `DockPanelRegistryBuilder`, keep `DockPanelRegistry` as the app-owned final aggregation seam | reusable panel packs no longer need bespoke registration shapes | Migrated | `ecosystem/fret-docking/src/dock/panel_registry.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `apps/fret-examples/src/docking_demo.rs`, `apps/fret-examples/src/container_queries_docking_demo.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`, `apps/fret-examples/src/docking_arbitration_demo.rs`, `docs/adr/0016-plugin-and-panel-boundaries.md`, `DESIGN.md` |
| Optional query-aware reusable kits | higher-level ecosystems can depend directly on `fret-query`, but there is no shared optional adapter contract | defer `QueryAdapter` in v1 and keep direct/local adapters explicit until a second real reusable consumer appears | audit real consumers first, then extract a small adapter only if multiple reusable consumers converge on the same contract | a second real reusable consumer appears with materially shared adapter pressure | Deferred | `ecosystem/fret-markdown/src/mermaid_svg_support.rs`, `ecosystem/fret-authoring/src/query.rs`, `ecosystem/fret-router/src/query_integration.rs`, `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`, `docs/workstreams/state-management-v1/state-management-v1-extension-contract.md`, `DESIGN.md` |
| Selector integration | grouped app data helpers already exist; no trait budget is documented | keep grouped app path, no shared selector trait in v1 | record the non-goal, keep state helpers opt-in, and stop speculative trait growth | selector remains trait-free unless a concrete new pressure appears | Migrated | `docs/workstreams/state-management-v1/state-management-v1-extension-contract.md`, `docs/workstreams/component-ecosystem-state-integration-v1/component-ecosystem-state-integration-v1.md`, `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`, `DESIGN.md` |
| Reusable helper conversion seams | several first-party helpers used to force callers onto `AnyElement` too early | keep typed inputs on reusable helpers and reserve raw `AnyElement` for explicit collection/landing seams only | audit helper signatures crate by crate; move generic slots/children to `IntoUiElement<H>` and document the intentional raw exceptions with source-policy tests | first-party helpers no longer force early `AnyElement` unless they truly aggregate heterogeneous values before an `ElementContext` exists | Migrated | `ecosystem/fret/src/workspace_shell.rs`, `ecosystem/fret-workspace/src/frame.rs`, `ecosystem/fret-workspace/src/command_scope.rs`, `ecosystem/fret-workspace/src/pane_content_focus.rs`, `ecosystem/fret-workspace/src/lib.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md` |

## Ecosystem Crate Lanes

| Crate / area | Current posture | Target posture | Migration notes | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| `fret-ui-shadcn` | curated facade already exists; `app`, `themes`, `raw` split is improving | keep curated facade plus explicit `app` and `advanced` seams, plus explicit raw escape hatches | do not add a shared component trait; keep raw access explicit and move environment / `UiServices` hooks off the default `app` lane | official first-party examples/docs consistently use curated root + explicit `raw`, while advanced hooks no longer sit on the default app setup surface | In progress | `docs/shadcn-declarative-progress.md`, `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`, `ecosystem/fret-ui-shadcn/src/lib.rs`, `ecosystem/fret-ui-shadcn/src/app.rs`, `ecosystem/fret-ui-shadcn/src/advanced.rs` |
| `fret-ui-kit` canvas compat re-exports | temporary `declarative::canvas_surface` and `recipes::canvas_*` modules forwarded canvas helpers from the wrong owner crate | canonical owner is `fret-canvas::ui` | migrate first-party consumers to `fret_canvas::ui::*`, then delete the compat modules and dependency edge from `fret-ui-kit` | first-party consumers and docs no longer depend on the ui-kit compat path | Deleted | `ecosystem/fret-chart/src/declarative/panel.rs`, `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs`, `ecosystem/fret-chart/src/declarative/legend_overlay.rs`, `ecosystem/fret-canvas/src/ui/mod.rs`, `ecosystem/fret-ui-kit/src/declarative/mod.rs`, `ecosystem/fret-ui-kit/src/recipes/mod.rs`, `docs/adr/IMPLEMENTATION_ALIGNMENT.md` |
| `fret-ui-assets` / `fret-icons-*` / `fret-node` / `fret-router-ui` app helpers | direct app-install helpers used to be re-exported from the crate root with mixed `install_app` / `install` naming | explicit `crate::app::*` submodules for default app wiring, plus `crate::advanced::*` seams for UI-services hooks | move first-party callers/docs/templates to `crate::app::install(...)`, move UI-services-boundary wrappers under `crate::advanced::*`, and add surface-policy tests per crate; keep low-level command registration on `fret-router-ui` root for advanced/manual callers while the default app lane uses `app::install(...)` | official docs/templates/examples no longer teach root-level app helper re-exports or advanced UI-services helpers on the default app lane | Migrated | `ecosystem/fret-ui-assets/src/lib.rs`, `ecosystem/fret-ui-assets/src/app.rs`, `ecosystem/fret-ui-assets/src/advanced.rs`, `ecosystem/fret-icons-lucide/src/lib.rs`, `ecosystem/fret-icons-lucide/src/app.rs`, `ecosystem/fret-icons-lucide/src/advanced.rs`, `ecosystem/fret-icons-radix/src/lib.rs`, `ecosystem/fret-icons-radix/src/app.rs`, `ecosystem/fret-icons-radix/src/advanced.rs`, `ecosystem/fret-node/src/lib.rs`, `ecosystem/fret-node/src/app.rs`, `ecosystem/fret-node/src/advanced.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `ecosystem/fret-router-ui/src/app.rs`, `ecosystem/fret-bootstrap/src/lib.rs`, `apps/fretboard/src/scaffold/templates.rs`, `docs/crate-usage-guide.md`, `docs/examples/todo-app-golden-path.md`, `apps/fret-cookbook/examples/router_basics.rs` |
| `fret-ui-magic` renderer/material helper | renderer-backed material registration lived under an ambiguous `app_integration` module | explicit `advanced::*` seam for renderer/material-service hooks | move the helper under `advanced`, keep the component surface root clean, and migrate first-party call sites to the new advanced path | first-party code no longer teaches `app_integration` for renderer/material hooks | Migrated | `ecosystem/fret-ui-magic/src/lib.rs`, `ecosystem/fret-ui-magic/src/advanced.rs`, `apps/fret-ui-gallery/src/driver/runtime_driver.rs` |
| `fret-docking` | registry service exists and now has a shared contribution seam | explicit panel-factory contribution model feeding the registry | keep app-owned aggregation and stable `PanelKind` identity; migrate bespoke registries case by case | reusable panel packs use the shared contribution seam | Migrated | `ecosystem/fret-docking/src/dock/panel_registry.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `apps/fret-examples/src/docking_demo.rs`, `apps/fret-examples/src/container_queries_docking_demo.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`, `apps/fret-examples/src/docking_arbitration_demo.rs`, `docs/adr/0013-docking-ops-and-persistence.md` |
| `fret-router` | strong core/location/history contract; typed route seam not standardized | typed route integration via `RouteCodec` | keep history/canonicalization router-owned | official route examples use typed codec-driven patterns | Migrated | `ecosystem/fret-router/src/lib.rs`, `ecosystem/fret-router/src/codec.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `apps/fret-ui-gallery/src/spec.rs`, `apps/fret-ui-gallery/src/driver/router.rs`, `apps/fret-demo-web/src/wasm.rs`, `docs/workstreams/router-v1/router-v1.md`, `docs/workstreams/router-ui-v1/router-ui-v1.md` |
| `fret-query` | grouped app helpers exist; reusable adapter story is intentionally still local/policy guidance only | keep grouped app path and defer a shared adapter until multiple reusable consumers need it | avoid pushing query types into primitive public APIs and keep crate-local wrappers explicit | a second real reusable consumer forces a stable shared adapter contract | Deferred | `ecosystem/fret-query/src/lib.rs`, `ecosystem/fret-authoring/src/query.rs`, `ecosystem/fret-markdown/src/mermaid_svg_support.rs`, `docs/workstreams/query-lifecycle-v1/query-lifecycle-v1.md` |
| `fret-selector` | explicit deps/signature story already exists | no shared selector trait in v1 | keep selector integration data-first and isolate opt-in selector helpers to explicit component-layer seams | docs continue to teach `cx.data().selector(...)` and not a new trait | Migrated | `docs/workstreams/state-management-v1/state-management-v1-extension-contract.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `ecosystem/fret-ui-kit/src/lib.rs`, `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` |
| future third-party kits | no single published rubric today | choose app/component/advanced tier explicitly and adopt only the small traits they truly need | publish the tier rules and trait budget together | third-party guidance is documented and linked from the docs index | Not started | `TARGET_INTERFACE_STATE.md`, `DESIGN.md` |

## Old Postures to Delete or Keep Out

| Old posture | Replacement / target posture | Delete when | Status |
| --- | --- | --- | --- |
| treating `fret-app::Plugin` as the future answer for every ecosystem integration problem | small domain traits plus registries/codecs/factories | official docs and first-party code stop implying otherwise | Delete-ready |
| adding policy traits to `crates/fret-ui` | keep ecosystem traits in ecosystem layers | docs and implementation guardrails exist | In progress |
| direct root-level raw access for recipe crates as a recommended path | curated facade plus explicit `raw::*` | official examples/docs no longer teach raw-through-root posture | In progress |
| speculative selector trait growth | keep selector data-first and trait-free in v1 | written non-goal is stable and no code depends on a selector trait | In progress |
| ad-hoc panel registration logic per app | `DockPanelFactory` plus registry aggregation | reusable panel packs and first-party demos migrate; dynamic-kind examples are refactored to stable `PanelKind + instance` identity | Delete-ready |
| stringly route construction in official routing guidance | typed route codec + canonical location helpers | cookbook, UI Gallery, and demo-web are migrated; remaining string parsing is compatibility-only fallback logic | Delete-ready |

## Recommended Execution Order

| Order | Track | Why |
| --- | --- | --- |
| 1 | lock the budget in docs | prevents trait sprawl while implementation is still fluid |
| 2 | normalize app installer posture | this is the most visible app-author seam |
| 3 | extract domain-specific traits where justified | command/router/docking need different shapes |
| 4 | validate query/selector boundaries | avoid adding speculative state traits |
| 5 | add guardrails and delete mixed guidance | keeps the cleaned surface from drifting |

## Completion Rule

This matrix is complete when every tracked old posture is either:

- deleted,
- intentionally retained as an explicit advanced/internal seam, or
- explicitly deferred with written rationale.
