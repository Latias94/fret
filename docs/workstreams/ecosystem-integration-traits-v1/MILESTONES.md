# Ecosystem Integration Traits v1 — Milestones

Status: Maintenance closeout
Last updated: 2026-03-15

Related:

- `DESIGN.md`
- `TODO.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

## Current execution stance (2026-03-15)

This workstream is now a **maintenance closeout lane**.

Current reading:

- the core justified seams are landed (`InstallIntoApp`, the `CommandCatalog` data contract,
  `RouteCodec`, `DockPanelFactory`),
- the `QueryAdapter` decision is now closed for v1: defer until a second real reusable consumer
  appears,
- the remaining work is narrow: first-party docs/export cleanup and a final mixed-posture audit,
- conversion-surface redesign remains owned by
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`.

Recommended closeout order:

1. keep ecosystem docs/examples aligned with the now-closed conversion story,
2. finish the first-party installer/export audit and delete-ready cleanup,
3. archive the lane with the v1 `QueryAdapter` defer note kept explicit in docs.

Progress note on 2026-03-12:

- the canonical todo/scaffold evidence set has now moved onto the new app-facing posture,
- the next reopen point for this lane is after `into-element` M2 finishes builder/child-pipeline
  cleanup, not before.

Current status on 2026-03-11:

- `InstallIntoApp` is now landed on the explicit `fret::integration` surface.
- `FretApp::setup(...)` and `UiAppBuilder::setup(...)` both accept `InstallIntoApp`.
- Small tuple composition is now intentionally supported for app-local setup bundles; slice/vec
  composition remains deferred.
- The broad `InstallIntoApp` impl is now explicitly treated as a Rust ergonomics accommodation:
  first-party docs/examples keep `.setup(...)` on named installers/tuples/bundles and reserve
  `UiAppBuilder::setup_with(...)` for one-off inline closures or captured runtime values.
- The first in-tree bundle example is landed in
  `apps/fret-cookbook/examples/docking_basics.rs`.
- `CommandCatalog` ownership is now locked to `fret-ui-kit::command`.
- Host command registry collection/gating/shortcut derivation now lives in
  `ecosystem/fret-ui-kit/src/command.rs`, while `fret-ui-shadcn::command` only maps catalog data
  into shadcn recipe entries and `fret-bootstrap` consumes the shared collector directly.
- `DockPanelFactory` is now landed as a contribution-level seam in `fret-docking`.
- `DockPanelRegistryBuilder` now aggregates `PanelKind`-keyed factories into the existing
  app-owned `DockPanelRegistry` story, and the cookbook docking example has migrated to the new
  contribution model.
- The baseline first-party docking demos that already use stable panel identities now also install
  their panels through `DockPanelRegistryBuilder`:
  `docking_demo`, `container_queries_docking_demo`, and `imui_editor_proof_demo`.
- `docking_arbitration_demo` now also uses `DockPanelRegistryBuilder`, with the old
  `demo.viewport.extra.{ix}` dynamic kinds replaced by stable `demo.viewport.extra + instance`
  identity.
- No remaining first-party app/example `DockPanelRegistry` implementations remain outside
  `fret-docking` test harnesses; the old monolithic app-registry teaching posture is now in
  delete-ready territory.
- `RouteCodec` is now landed in `fret-router` as the shared typed-route seam.
- `fret-router-ui` now exposes typed-route authoring helpers on top of the codec contract.
- The first codec-based migration target is landed in
  `apps/fret-cookbook/examples/router_basics.rs`.
- `apps/fret-ui-gallery` now also routes its gallery page URL encode/decode through
  `UiGalleryRouteCodec`, including canonical page navigation and legacy `start_page` fallback.
- `apps/fret-demo-web/src/wasm.rs` now resolves demo selection through a codec-backed helper for
  the canonical `?demo=...` entry path while retaining legacy hash-token fallback compatibility.
- First-party web README surfaces now also teach canonical query routes first and treat hash/query
  aliases as compatibility-only migration baggage.
- `fret-ui-assets`, `fret-icons-lucide`, `fret-icons-radix`, `fret-node`, and `fret-router-ui`
  now expose default
  app wiring under explicit `crate::app::install(...)` seams instead of root-level
  `install_app(...)` exports; their UI-services-boundary wrappers now live under explicit
  `crate::advanced::*` seams.
- `fret-ui-magic` now exposes its renderer/material helper on an explicit
  `fret_ui_magic::advanced::ensure_materials(...)` seam instead of the ambiguous
  `app_integration` module.
- `fret-ui-shadcn` now keeps default theme installation on `fret_ui_shadcn::app::*`, while
  environment-sync and `UiServices` helpers live on the explicit
  `fret_ui_shadcn::advanced::*` seam.
- First-party callers and docs that used those root-level helpers (`fret-bootstrap`,
  `fret-examples`, scaffold templates, crate usage docs, and the todo golden-path doc) have been
  migrated to the explicit `crate::app::*` posture.
- `docs/crate-usage-guide.md` now includes a short ecosystem author checklist that points back to
  this trait budget and repeats the "no universal component / giant plugin trait" rule.
- `ecosystem/fret` authoring-surface policy tests now gate that guidance.
- This workstream now explicitly hands off the remaining conversion-vocabulary cleanup to
  `docs/workstreams/into-element-surface-fearless-refactor-v1/` so trait adoption does not create
  a second, crate-local authoring language.
- The 2026-03-15 audit closes `QueryAdapter` for v1:
  there is still no in-tree `QueryAdapter` implementation or second reusable consumer.
  `ecosystem/fret-markdown` currently uses `fret_query::ui::QueryElementContextExt` directly for a
  local feature seam, `ecosystem/fret-authoring/src/query.rs` provides an authoring-surface
  wrapper, and `ecosystem/fret-router/src/query_integration.rs` only contributes query-key helpers.
  Those signals do not yet justify freezing a shared adapter trait.
- Free installer functions remain the default story; first-party bundle examples are still a
  follow-up cleanup item rather than a trait-budget blocker.

Milestone readout on 2026-03-11:

| Milestone | State | Notes |
| --- | --- | --- |
| M0 | Done | budget, owners, rejected shapes, and docs index/roadmap links are all recorded |
| M1 | In progress | `InstallIntoApp` is landed, the first app-helper crates now use explicit `crate::app::*` seams, `fret-ui-shadcn` split its advanced hooks off the default app lane, first-party installer naming cleanup is effectively closed, and one advanced material helper has moved to `advanced::*`; the remaining tail here is mainly broader bundle-example/docs closeout |
| M2 | Done | the `CommandCatalog` data contract, `RouteCodec`, and `DockPanelFactory` now have the intended ownership story |
| M3 | Done | `QueryAdapter` is now explicitly deferred for v1 because the 2026-03-15 audit found no in-tree implementation and no second reusable consumer with a materially shared adapter contract; selector remains intentionally trait-free |
| M4 | In progress | checklist/gate work has started, but hard deletions and full docs/template cleanup remain |

## M0 — Trait Budget Lock

Goal:

- agree on the integration vocabulary before more first-party ecosystems drift.

Exit criteria:

- `DESIGN.md` is accepted as the working plan,
- the target ownership for `InstallIntoApp`, `CommandCatalog`, `RouteCodec`,
  `DockPanelFactory`, and `QueryAdapter` is recorded,
- rejected/deferred traits are explicitly documented,
- docs index and roadmap link to this workstream.

## M1 — App Integration Normalization

Goal:

- make app-level ecosystem wiring boring and consistent.

Exit criteria:

- `FretApp::setup(...)` remains the canonical default story,
- first-party app integration modules converge on one naming posture,
- `InstallIntoApp` (or its final equivalent) is implemented or intentionally deferred with a clear
  reason,
- at least one first-party bundle/composition example exists.

## M2 — Domain Trait Extraction

Goal:

- land the domain-specific seams where traits are actually justified.

Exit criteria:

- command catalog abstraction is separated from shadcn recipe code,
- router typed route encoding has one shared codec seam,
- docking panel contributions have a contribution-level factory seam,
- each extracted trait is owned by the correct ecosystem layer.

## M3 — State/Query Integration Closure

Goal:

- clarify the boundary between app-facing grouped data helpers and optional reusable adapters.

Exit criteria:

- grouped `cx.data().selector_layout(...)` for LocalState-first selectors, raw
  `cx.data().selector(...)` for explicit signatures, and `cx.data().query(...)` remain the
  official app-path story,
- a decision is made on `QueryAdapter`:
  - implemented for a real consumer, or
  - explicitly deferred with written rationale,
- selector remains trait-free unless concrete pressure proves otherwise.

## M4 — Docs, Gates, and Deletion

Goal:

- lock the cleaned surface and prevent drift back toward mixed integration stories.

Exit criteria:

- official docs/examples/templates point to the new ecosystem integration story,
- guardrails exist for the highest-risk regressions,
- legacy mixed postures tracked in `MIGRATION_MATRIX.md` are either deleted or intentionally marked
  as advanced/internal,
- first-party router docs stop teaching compatibility-only hash-token routing as a primary path,
- this workstream can be downgraded from active refactor planning to maintenance.

## Post-v1 handoff — Conversion-surface alignment

Goal:

- keep ecosystem integration contracts and authoring conversion contracts moving in lockstep.

Exit criteria:

- curated ecosystem docs/examples stop teaching legacy split conversion traits while trait adoption continues,
- shadcn and UI Gallery exemplars track the active authoring target state,
- the remaining conversion-surface cleanup is clearly owned by
  `docs/workstreams/into-element-surface-fearless-refactor-v1/` rather than ad-hoc crate docs.
