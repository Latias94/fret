# Ecosystem Integration Traits v1 — TODO

Status: Active planning tracker
Last updated: 2026-03-11

Related:

- `DESIGN.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

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
- [ ] Decide whether `QueryAdapter` is needed in v1 or should stay "design locked, implementation
  deferred" until a second real consumer appears.
- [ ] Record an explicit "do not add" policy for universal `Component` / giant ecosystem `Plugin`
  traits in the relevant docs.

## 2. First-Party Surface Normalization

- [ ] Ensure first-party ecosystem crates expose module boundaries consistent with the target tier
  rules:
  - `app`
  - `themes` / `tokens` where applicable
  - `raw`
  - `core` / `headless` / `ui` where applicable
- [ ] Normalize app integration naming where it is still mixed between `install_app`, `install`,
  and `install_into`.
- [ ] Keep `FretApp::setup(...)` as the canonical app authoring story in docs and templates.
- [ ] Audit first-party crates for root-level exports that bypass curated facades.
- [ ] Audit first-party crates for ad-hoc panel/route/query integration helpers that should move to
  one of the target seams.

## 3. `InstallIntoApp` Adoption

- [x] Introduce a thin adapter story for installer functions so bundles can compose existing
  `fn(&mut App)` installers without forcing app authors onto traits.
  - Landed on 2026-03-11: `InstallIntoApp` has a blanket implementation for installer functions
    and is accepted by both `FretApp::setup(...)` and `UiAppBuilder::setup(...)`.
- [x] Decide whether tuple/slice/vec bundle composition should be supported directly or left to
  small app-owned wrapper types.
  - Decision on 2026-03-11:
    support small tuple composition directly for app-local wiring (`(a, b)`, `(a, b, c)`,
    `(a, b, c, d)`), keep reusable/published packs on named `InstallIntoApp` bundle types, and
    defer slice/vec-style dynamic composition until a concrete need appears.
- [ ] Migrate first-party "app integration pack" examples to use one consistent bundle story.
  - First landed example on 2026-03-11: `apps/fret-cookbook/examples/docking_basics.rs` now uses
    `DockingBasicsBundle` via `.setup(DockingBasicsBundle)`.
- [ ] Keep advanced builder-only install paths explicit; do not let them leak onto the default app
  path.

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
- [ ] Document when a plain `CommandMeta` is sufficient and when a catalog source is warranted.
  - Follow-up: this should be written once command palette, menu, and any future non-shadcn
    command surfaces share the same guidance.

## 5. `RouteCodec` Adoption

- [x] Add a single target note explaining how typed routes, canonical `RouteLocation`, and query
  integration fit together.
  - Landed on 2026-03-11 in `docs/workstreams/router-v1.md`.
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

- [ ] Audit higher-level component ecosystems for real query-integration pressure.
  - Candidate areas: markdown, chart/plot, data-table, future router-aware data surfaces.
- [ ] Keep primitives and base recipes state-stack agnostic while the adapter design lands.
- [ ] Do not add a selector trait unless a concrete multi-crate need appears.
- [ ] Keep official docs teaching grouped `cx.data().selector(...)` / `cx.data().query(...)` on the
  app path.

## 8. Documentation and Guardrails

- [ ] Add the workstream to the canonical docs index and roadmap tracker.
- [ ] Add a short ecosystem author checklist that points to this trait budget.
- [ ] Add a docs/gate check that first-party guidance does not regress toward:
  - direct raw crate-root teaching,
  - giant plugin abstractions,
  - universal component traits.
- [ ] Add evidence anchors for the first in-tree implementation of each accepted trait.

## 9. Hard Deletion Work

- [ ] Delete mixed or redundant first-party installer naming once official call sites are migrated.
- [ ] Delete legacy docs that imply every ecosystem crate should expose the same plugin shape.
- [ ] Delete any temporary adapters that survive after official examples and templates no longer
  need them.
- [ ] Keep the migration matrix updated until all tracked old postures are either deleted or
  intentionally retained as explicit advanced seams.
