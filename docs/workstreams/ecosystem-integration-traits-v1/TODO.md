# Ecosystem Integration Traits v1 — TODO

Status: Maintenance closeout tracker
Last updated: 2026-03-15

Related:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Execution note on 2026-03-12:

- do not treat this as the first place to solve authoring-surface feel,
- finish core trait closure after `into-element` M0/M1 and the canonical todo/scaffold evidence
  set move onto the new conversion vocabulary,
- keep `QueryAdapter` explicitly behind a later keep/defer decision instead of expanding trait
  budget in parallel with the conversion-surface refactor.

Status note on 2026-03-12:

- the canonical todo/scaffold evidence set is now on the new app-facing posture,
- keep this workstream focused on follow-up deletion/alignment while `into-element` M2 rewires the
  remaining builder and child pipelines.

Closeout note on 2026-03-15:

- treat this file as a narrow maintenance tracker, not an active trait-expansion backlog,
- the remaining high-value items are the first-party closeout audit and delete-ready cleanup,
- the `QueryAdapter` v1 decision is now explicit: defer rather than spend trait budget
  speculatively.

This file tracks the execution work needed to turn the trait budget into a clean pre-release
surface.

## 1. Contract Lock

- [x] Decide the canonical home for `InstallIntoApp`.
  - Landed on 2026-03-11 under `ecosystem/fret/src/integration.rs` and exposed as the explicit
    `fret::integration::InstallIntoApp` surface, not in `crates/fret-ui`.
- [x] Decide whether `InstallIntoApp` should be defined in `ecosystem/fret` first and re-exported
  later, or introduced directly under a stable `fret::integration` module.
  - Current decision: define it directly on the stable `fret::integration` module and keep the
    default app prelude free of integration-trait leakage.
- [x] Lock the canonical owner for `CommandCatalog`.
  - Landed on 2026-03-11: host-command catalog ownership now lives in
    `fret-ui-kit::command`, with recipe crates consuming and mapping the data-only entries.
- [x] Audit existing router code and decide whether `RouteCodec` should be implemented directly by
  route-table builders or by app-defined codec types.
  - Landed on 2026-03-11: the shared seam is app-defined codec types in `fret-router`, while
    `RouteTree` / `Router` continue to own route matching, guards, and history semantics
    independently.
- [x] Audit docking registry usage and decide the precise relationship between the current
  `DockPanelRegistry` and the target `DockPanelFactory`.
  - Landed on 2026-03-11: `DockPanelRegistry` remains the app-owned final aggregation seam,
    while `DockPanelFactory` + `DockPanelRegistryBuilder` provide the reusable contribution-level
    path.
- [x] Decide whether `QueryAdapter` is needed in v1 or should stay "design locked, implementation
  deferred" until a second real consumer appears.
  - Decision on 2026-03-15: defer in v1.
  - Audit result: there is still no in-tree `QueryAdapter` implementation, no second reusable
    consumer, and no materially shared adapter contract across the current query-touching surfaces
    (`ecosystem/fret-markdown`, `ecosystem/fret-authoring`, `ecosystem/fret-router` query-key
    helpers).
- [x] Record an explicit "do not add" policy for universal `Component` / giant ecosystem `Plugin`
  traits in the relevant docs.
  - Landed on 2026-03-11:
    `DESIGN.md`, `TARGET_INTERFACE_STATE.md`, and `docs/crate-usage-guide.md` now all repeat the
    same anti-goal so the rule is visible outside the workstream folder too.

## 2. First-Party Surface Normalization

- [x] Ensure first-party ecosystem crates expose module boundaries consistent with the target tier
  rules:
  - `app`
  - `themes` / `tokens` where applicable
  - `raw`
  - `core` / `headless` / `ui` where applicable
  - Audit result on 2026-03-15: the currently in-scope first-party crates now expose the expected
    explicit seams where applicable:
    `fret-ui-shadcn` (`app`, `advanced`, `themes`, `raw`),
    `fret-ui-assets` / `fret-icons-lucide` / `fret-icons-radix` / `fret-node`
    (`app`, `advanced`),
    `fret-router-ui` (`app`),
    and `fret-ui-magic` (`advanced`).
- [x] Normalize app integration naming where it is still mixed between `install_app`, `install`,
  and `install_into`.
  - Landed first-pass migration on 2026-03-11:
    `fret-ui-assets`, `fret-icons-lucide`, `fret-icons-radix`, and `fret-node` now expose
    default app wiring under explicit `crate::app::install(...)` seams, while UI-services-boundary
    helpers live under explicit `crate::advanced::*` seams.
  - Extended on 2026-03-12:
    `fret-router-ui` now exposes command-registration setup through `crate::app::install(...)`,
    and the `fret::router` facade mirrors the same `app::install` seam instead of a root
    `install_app(...)` exception.
  - Audit result on 2026-03-15:
    there is no remaining root-level `pub fn install_app(...)` export across the targeted
    first-party ecosystem crates.
    Remaining `install_app` spellings are now app-local helper function names or
    `fret-bootstrap::BootstrapBuilder::install_app(...)`, not mixed ecosystem export posture.
- [x] Keep `FretApp::setup(...)` as the canonical app authoring story in docs and templates.
  - Extended on 2026-03-12: first-party docs/examples now document `.setup(...)` for named
    installers/tuples/bundles, keep inline closures on `UiAppBuilder::setup_with(...)`, and gate
    against `.setup(|app| ...)` on the default app-author path across both
    `apps/fret-examples` and `apps/fret-cookbook`.
- [x] Audit first-party crates for root-level exports that bypass curated facades.
  - Landed first-pass migration on 2026-03-11 for
    `fret-ui-assets`, `fret-icons-lucide`, `fret-icons-radix`, and `fret-node`; keep auditing the
    remaining ecosystem crates rather than treating this checklist item as globally done yet.
  - Audit result on 2026-03-15:
    the targeted first-party crates now carry source-policy assertions preventing root-level
    `advanced::*` shortcut re-exports or root-level `install_app(...)` drift.
- [x] Normalize advanced service/material helpers so they live on explicit advanced seams instead
  of ambiguous integration modules.
  - Landed first-pass migration on 2026-03-11:
    `fret-ui-magic` now exposes its renderer/material helper as
    `fret_ui_magic::advanced::ensure_materials(...)` rather than the old ambiguous
    `app_integration`-named path.
  - Extended on 2026-03-11:
    `fret-ui-shadcn` now keeps environment sync and `UiServices` helpers on
    `fret_ui_shadcn::advanced::*`, leaving `fret_ui_shadcn::app::*` for default theme setup only.
- [x] Audit first-party crates for ad-hoc panel/route/query integration helpers that should move to
  one of the target seams.
  - Audit result on 2026-03-15:
    first-party panel surfaces now route through `DockPanelFactory` /
    `DockPanelRegistryBuilder`,
    typed route integration stays on `RouteCodec`,
    `ecosystem/fret-router/src/query_integration.rs` remains a route-key helper surface rather
    than a new adapter contract,
    and `ecosystem/fret-authoring/src/query.rs::UiWriterQueryExt` remains an authoring-local query
    wrapper rather than a shared ecosystem integration seam.
- [ ] Audit reusable helper signatures in first-party ecosystem crates so each public helper is
  classified by lane:
  - app-facing teaching helpers use `Ui` / `UiChild`,
  - reusable generic helpers move toward the unified component conversion trait tracked by
    `docs/workstreams/into-element-surface-fearless-refactor-v1/`,
  - raw `AnyElement` stays explicit for advanced/internal seams only.

## 3. `InstallIntoApp` Adoption

- [x] Introduce a thin adapter story for installer functions so bundles can compose existing
  `fn(&mut App)` installers without forcing app authors onto traits.
  - Landed on 2026-03-11: `InstallIntoApp` has a blanket implementation for installer functions
    and is accepted by both `FretApp::setup(...)` and `UiAppBuilder::setup(...)`.
  - Clarified on 2026-03-12: the blanket implementation stays in place because Rust would
    otherwise require explicit casts for plain function items under a trait bound. First-party docs
    and source gates now keep `.setup(...)` on named installers/tuples/bundles and reserve
    `setup_with(...)` for explicit inline closures.
- [x] Decide whether tuple/slice/vec bundle composition should be supported directly or left to
  small app-owned wrapper types.
  - Decision on 2026-03-11:
    support small tuple composition directly for app-local wiring (`(a, b)`, `(a, b, c)`,
    `(a, b, c, d)`), keep reusable/published packs on named `InstallIntoApp` bundle types, and
    defer slice/vec-style dynamic composition until a concrete need appears.
- [x] Migrate first-party "app integration pack" examples to use one consistent bundle story.
  - First landed example on 2026-03-11: `apps/fret-cookbook/examples/docking_basics.rs` now uses
    `DockingBasicsBundle`.
  - Extended on 2026-03-15:
    the cookbook examples
    `chart_interactions_basics.rs`,
    `embedded_viewport_basics.rs`,
    `external_texture_import_basics.rs`,
    `gizmo_basics.rs`,
    and `docking_basics.rs`
    now all teach app-local installer composition through a single `.setup((..., ...))` pack
    story instead of stacking multiple adjacent `.setup(...)` calls for the same local wiring.
- [x] Keep advanced builder-only install paths explicit; do not let them leak onto the default app
  path.
  - Evidence on 2026-03-15:
    `ecosystem/fret/src/lib.rs` and `docs/crate-usage-guide.md` explicitly reserve
    `UiAppBuilder::setup_with(...)` for one-off inline closures or captured runtime values,
    while `apps/fret-cookbook/src/lib.rs` source gates reject `.setup_with(...)` on the default
    app-author teaching surface.

## 4. `CommandCatalog` Adoption

- [x] Audit current command palette code in `fret-ui-shadcn` and split reusable catalog logic from
  shadcn-specific recipe/UI glue.
  - Landed on 2026-03-11: host command registry collection, gating, and shortcut derivation moved
    to `fret-ui-kit::command`; `fret-ui-shadcn::command` now maps catalog data into recipe
    `CommandEntry` values.
- [x] Define data-oriented `CommandCatalogEntry` / `CommandCatalogItem` / `CommandCatalogGroup`
  types and collector helpers.
  - Landed on 2026-03-11 in `ecosystem/fret-ui-kit/src/command.rs`.
- [x] Keep catalog entries execution-free; route activation back through typed actions or
  registered `CommandId`s.
  - Current shape stores `CommandId` on data-only catalog items and lets recipe crates decide how
    to surface activation UI.
- [x] Document when a plain `CommandMeta` is sufficient and when a catalog source is warranted.
  - Landed on 2026-03-15 in `TARGET_INTERFACE_STATE.md` and `docs/crate-usage-guide.md`.
  - Current rule:
    plain `CommandMeta` is enough for normal registration, keybindings, menus, and shared command
    identity; `CommandCatalog` is for grouped or enriched discovery surfaces that need more than a
    flat registry listing.

## 5. `RouteCodec` Adoption

- [x] Add a single target note explaining how typed routes, canonical `RouteLocation`, and query
  integration fit together.
  - Landed on 2026-03-11 in `docs/workstreams/router-v1/router-v1.md`.
- [x] Pick one in-tree app as the first codec-based migration target.
  - Landed on 2026-03-11: `apps/fret-cookbook/examples/router_basics.rs` now teaches an
    app-defined `RouteCodec` plus typed-route router link helpers.
- [x] Remove remaining stringly route construction in official examples after the typed route path
  is ready.
  - Landed on 2026-03-11: `apps/fret-cookbook/examples/router_basics.rs`,
    `apps/fret-ui-gallery`, and `apps/fret-demo-web/src/wasm.rs` now route official first-party
    entry paths through codec-backed helpers; remaining string parsing is compatibility-only
    fallback logic.
- [x] Keep browser/history adapters independent from codec ownership.
  - Current implementation keeps `RouteCodec` in `fret-router` core while history remains owned by
    `MemoryHistory` / web adapters and is not referenced by the codec contract.

## 6. `DockPanelFactory` Adoption

- [x] Audit current `DockPanelRegistry` call sites and classify them into:
  - app-owned bespoke registries,
  - reusable panel packs,
  - runtime-only helpers.
  - Landed on 2026-03-11: existing call sites are now explicitly split between app-owned final
    registries (`DockPanelRegistry`), contribution-level reusable panel factories, and runtime-only
    `render_and_bind_dock_panels(...)` helpers.
- [x] Define a contribution-level factory shape that can aggregate into the existing registry
  service.
  - Landed on 2026-03-11 in `ecosystem/fret-docking/src/dock/panel_registry.rs` as
    `DockPanelFactory`, `DockPanelFactoryCx`, `DockPanelRegistryBuilder`, and
    `DockPanelFactoryRegistry`.
- [x] Keep persistent panel identity anchored on `PanelKind` / `PanelKey`.
  - The builder dispatches by stable `PanelKind`, while each factory still receives the full
    `PanelKey` for singleton or multi-instance panels.
- [x] Migrate at least one reusable panel set to the new contribution model before deleting old
  bespoke wiring guidance.
  - Landed on 2026-03-11: `apps/fret-cookbook/examples/docking_basics.rs` now registers its panel
    set through `DockPanelFactory` contributions rather than a monolithic bespoke registry.
- [x] Migrate the baseline first-party docking demos that already use stable panel kinds to the
  registry-builder path.
  - Landed on 2026-03-11: `apps/fret-examples/src/docking_demo.rs`,
    `apps/fret-examples/src/container_queries_docking_demo.rs`, and
    `apps/fret-examples/src/imui_editor_proof_demo.rs` now install panel factories through
    `DockPanelRegistryBuilder`.
- [x] Rework the first dynamic-kind extra-viewport demo onto stable `PanelKind + instance`
  semantics and move it onto the factory/builder path.
  - Landed on 2026-03-11: `apps/fret-examples/src/docking_arbitration_demo.rs` now models extra
    viewports as `demo.viewport.extra + instance`, and the demo installs all panels through
    `DockPanelRegistryBuilder`.
- [x] Confirm whether any first-party app/example code still teaches monolithic app-owned panel
  registries.
  - Result on 2026-03-11: no remaining first-party app/example `DockPanelRegistry` implementations
    remain outside `fret-docking` test harnesses.

## 7. `QueryAdapter` and Selector Boundaries

- [x] Audit higher-level component ecosystems for real query-integration pressure.
  - Audit result on 2026-03-15:
    `ecosystem/fret-markdown` uses direct `fret_query` context helpers for Mermaid/MathJax
    resource loading,
    `ecosystem/fret-authoring/src/query.rs` exposes an authoring-specific `UiWriterQueryExt`,
    and
    `ecosystem/fret-router/src/query_integration.rs` only provides route-key helpers.
    This is not yet a second reusable consumer pair with a materially shared adapter contract.
- [ ] Keep primitives and base recipes state-stack agnostic while the adapter design lands.
- [x] Do not add a selector trait unless a concrete multi-crate need appears.
  - Current v1 posture remains explicit non-adoption.
- [x] Keep official docs teaching grouped `cx.data().selector(...)` / `cx.data().query(...)` on the
  app path.
  - Evidence: first-party docs/tests now gate the grouped app data surface and reject
    `cx.use_query*` as the default teaching path.
- [x] Record a final keep/defer note for `QueryAdapter` before this workstream is archived.
  - Final direction on 2026-03-15: defer in v1 unless a second real reusable consumer appears
    with a materially shared adapter contract.

## 8. Documentation and Guardrails

- [x] Add the workstream to the canonical docs index and roadmap tracker.
  - Landed on 2026-03-11: both `docs/README.md` and `docs/roadmap.md` link to the full
    ecosystem-integration-traits-v1 workstream set.
- [x] Add a short ecosystem author checklist that points to this trait budget.
  - Landed on 2026-03-11 in `docs/crate-usage-guide.md`.
- [x] Add a docs/gate check that first-party guidance does not regress toward:
  - direct raw crate-root teaching,
  - giant plugin abstractions,
  - universal component traits.
  - Landed on 2026-03-11 in `ecosystem/fret/src/lib.rs` authoring-surface policy tests.
  - Extended on 2026-03-12 with first-party source-policy coverage in
    `apps/fret-examples/src/lib.rs` and `apps/fret-cookbook/src/lib.rs`.
- [x] Add evidence anchors for the first in-tree implementation of each accepted trait.
  - `TARGET_INTERFACE_STATE.md` and `MIGRATION_MATRIX.md` now record the concrete first-party
    evidence set for `InstallIntoApp`, `CommandCatalog`, `RouteCodec`, and `DockPanelFactory`,
    plus the v1 defer evidence for `QueryAdapter`.
- [x] Keep first-party ecosystem docs/examples aligned with the follow-on conversion-surface
  tracker so trait adoption does not re-teach the legacy split conversion traits.
  - Evidence on 2026-03-15:
    `docs/crate-usage-guide.md`,
    `docs/roadmap.md`,
    and `docs/shadcn-declarative-progress.md`
    all point first-party ecosystem guidance at the unified `IntoUiElement<H>` conversion story
    and the `into-element-surface-fearless-refactor-v1` target state instead of re-teaching the
    legacy split conversion vocabulary.

## 9. Hard Deletion Work

- [x] Delete mixed or redundant first-party installer naming once official call sites are migrated.
  - Audit result on 2026-03-15:
    root-level ecosystem `install_app(...)` exports are gone on the targeted first-party crates;
    remaining `install_app` spellings are local app helper functions or the app-owned bootstrap
    builder method, so they are no longer part of the mixed ecosystem naming story.
- [x] Delete legacy docs that imply every ecosystem crate should expose the same plugin shape.
  - Audit result on 2026-03-15:
    no targeted first-party guidance now implies a universal ecosystem plugin model.
    Remaining `Plugin` references are app-owned (`fret-app::Plugin`) or domain-local
    (`GizmoPlugin`) and are explicitly not treated as the default ecosystem extension template.
- [ ] Delete any temporary adapters that survive after official examples and templates no longer
  need them.
- [ ] Keep the migration matrix updated until all tracked old postures are either deleted or
  intentionally retained as explicit advanced seams.
