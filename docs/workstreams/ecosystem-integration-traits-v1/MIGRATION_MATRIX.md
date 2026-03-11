# Ecosystem Integration Traits v1 — Migration Matrix

Status: Execution tracker
Last updated: 2026-03-11

This matrix keeps the ecosystem integration cleanup concrete:

- what the current posture is,
- what the target seam should be,
- what still blocks deletion,
- and which old guidance must disappear.

This workstream is pre-release and does not optimize for compatibility.

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
| App installer composition | mixed `install_app`, `install`, and ad-hoc bundle patterns | canonical installer functions plus `InstallIntoApp` for composition | add a thin trait adapter over existing installer functions; allow small tuple composition but keep reusable packs named | official examples use one consistent app wiring story | In progress | `ecosystem/fret/src/integration.rs`, `ecosystem/fret/src/app_entry.rs`, `ecosystem/fret/src/lib.rs`, `ecosystem/fret/README.md`, `apps/fret-cookbook/examples/docking_basics.rs`, `DESIGN.md` |
| Universal plugin pressure | temptation to widen `fret-app::Plugin` into the ecosystem default | keep `Plugin` app-owned and minimal | document explicit anti-goal and route domain needs to small traits | no docs or first-party code teach `Plugin` as the universal model | In progress | `crates/fret-app/src/plugins.rs`, `docs/adr/0016-plugin-and-panel-boundaries.md`, `DESIGN.md` |
| Command palette/catalog extension | shadcn recipe code used to own host-command collection and catalog shaping | `CommandCatalog` data contract in a reusable policy layer | move host-command collection/gating/shortcut derivation to `fret-ui-kit::command`, keep recipe crates as mapping/render layers | bootstrap and recipe crates consume, rather than own, the shared catalog contract | In progress | `ecosystem/fret-ui-kit/src/command.rs`, `ecosystem/fret-ui-shadcn/src/command.rs`, `ecosystem/fret-bootstrap/src/ui_app_driver.rs`, `DESIGN.md` |
| Typed route integration | typed routes are possible, but there is no shared codec seam yet | `RouteCodec` | add a shared route encode/decode contract next to router core | official router examples stop teaching stringly route construction as the primary path | In progress | `ecosystem/fret-router/src/codec.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `docs/workstreams/router-v1.md`, `DESIGN.md` |
| Reusable dock panel contributions | app-owned registry service existed, but contribution-level factory was not standardized | `DockPanelFactory` aggregated into the dock registry story | add `DockPanelFactory` + `DockPanelRegistryBuilder`, keep `DockPanelRegistry` as the app-owned final aggregation seam | reusable panel packs no longer need bespoke registration shapes | Migrated | `ecosystem/fret-docking/src/dock/panel_registry.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `apps/fret-examples/src/docking_demo.rs`, `apps/fret-examples/src/container_queries_docking_demo.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`, `apps/fret-examples/src/docking_arbitration_demo.rs`, `docs/adr/0016-plugin-and-panel-boundaries.md`, `DESIGN.md` |
| Optional query-aware reusable kits | higher-level ecosystems can depend directly on `fret-query`, but there is no shared optional adapter contract | `QueryAdapter` for higher-level libraries only | audit real consumers first, then extract a small adapter if justified | at least one real reusable consumer validates the seam or the trait is explicitly deferred | Not started | `docs/workstreams/component-ecosystem-state-integration-v1.md`, `docs/workstreams/state-management-v1-extension-contract.md`, `DESIGN.md` |
| Selector integration | grouped app data helpers already exist; no trait budget is documented | keep grouped app path, no shared selector trait in v1 | record the non-goal and stop speculative trait growth | selector remains trait-free unless a concrete new pressure appears | In progress | `docs/workstreams/state-management-v1-extension-contract.md`, `docs/workstreams/component-ecosystem-state-integration-v1.md`, `DESIGN.md` |

## Ecosystem Crate Lanes

| Crate / area | Current posture | Target posture | Migration notes | Delete trigger | Status | Evidence anchors |
| --- | --- | --- | --- | --- | --- | --- |
| `fret-ui-shadcn` | curated facade already exists; `app`, `themes`, `raw` split is improving | keep curated facade plus explicit raw seams; consume shared catalog/install contracts where appropriate | do not add a shared component trait; keep raw access explicit | official first-party examples/docs consistently use curated root + explicit `raw` | In progress | `docs/shadcn-declarative-progress.md`, `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`, `ecosystem/fret-ui-shadcn/src/lib.rs` |
| `fret-docking` | registry service exists and now has a shared contribution seam | explicit panel-factory contribution model feeding the registry | keep app-owned aggregation and stable `PanelKind` identity; migrate bespoke registries case by case | reusable panel packs use the shared contribution seam | Migrated | `ecosystem/fret-docking/src/dock/panel_registry.rs`, `apps/fret-cookbook/examples/docking_basics.rs`, `apps/fret-examples/src/docking_demo.rs`, `apps/fret-examples/src/container_queries_docking_demo.rs`, `apps/fret-examples/src/imui_editor_proof_demo.rs`, `apps/fret-examples/src/docking_arbitration_demo.rs`, `docs/adr/0013-docking-ops-and-persistence.md` |
| `fret-router` | strong core/location/history contract; typed route seam not standardized | typed route integration via `RouteCodec` | keep history/canonicalization router-owned | official route examples use typed codec-driven patterns | In progress | `ecosystem/fret-router/src/lib.rs`, `ecosystem/fret-router/src/codec.rs`, `ecosystem/fret-router-ui/src/lib.rs`, `apps/fret-cookbook/examples/router_basics.rs`, `docs/workstreams/router-v1.md`, `docs/workstreams/router-ui-v1.md` |
| `fret-query` | grouped app helpers exist; reusable adapter story is still policy guidance only | keep grouped app path and add optional adapter only if a real consumer needs it | avoid pushing query types into primitive public APIs | either a real adapter lands or the trait is explicitly deferred | Not started | `ecosystem/fret-query/src/lib.rs`, `docs/workstreams/query-lifecycle-v1.md` |
| `fret-selector` | explicit deps/signature story already exists | no shared selector trait in v1 | keep selector integration data-first | docs continue to teach `cx.data().selector(...)` and not a new trait | In progress | `docs/workstreams/state-management-v1-extension-contract.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md` |
| future third-party kits | no single published rubric today | choose app/component/advanced tier explicitly and adopt only the small traits they truly need | publish the tier rules and trait budget together | third-party guidance is documented and linked from the docs index | Not started | `TARGET_INTERFACE_STATE.md`, `DESIGN.md` |

## Old Postures to Delete or Keep Out

| Old posture | Replacement / target posture | Delete when | Status |
| --- | --- | --- | --- |
| treating `fret-app::Plugin` as the future answer for every ecosystem integration problem | small domain traits plus registries/codecs/factories | official docs and first-party code stop implying otherwise | In progress |
| adding policy traits to `crates/fret-ui` | keep ecosystem traits in ecosystem layers | docs and implementation guardrails exist | In progress |
| direct root-level raw access for recipe crates as a recommended path | curated facade plus explicit `raw::*` | official examples/docs no longer teach raw-through-root posture | In progress |
| speculative selector trait growth | keep selector data-first and trait-free in v1 | written non-goal is stable and no code depends on a selector trait | In progress |
| ad-hoc panel registration logic per app | `DockPanelFactory` plus registry aggregation | reusable panel packs and first-party demos migrate; dynamic-kind examples are refactored to stable `PanelKind + instance` identity | Delete-ready |
| stringly route construction in official routing guidance | typed route codec + canonical location helpers | first official router examples migrate | In progress |

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
