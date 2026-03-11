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

- [ ] Decide the canonical home for `InstallIntoApp`.
  - Preferred outcome: ecosystem-level integration module, not `crates/fret-ui`.
- [ ] Decide whether `InstallIntoApp` should be defined in `ecosystem/fret` first and re-exported
  later, or introduced directly under a stable `fret::integration` module.
- [ ] Lock the canonical owner for `CommandCatalog`.
  - Preferred outcome: `fret-ui-kit::command`, with recipe crates consuming it.
- [ ] Audit existing router code and decide whether `RouteCodec` should be implemented directly by
  route-table builders or by app-defined codec types.
- [ ] Audit docking registry usage and decide the precise relationship between the current
  `DockPanelRegistry` and the target `DockPanelFactory`.
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

- [ ] Introduce a thin adapter story for installer functions so bundles can compose existing
  `fn(&mut App)` installers without forcing app authors onto traits.
- [ ] Decide whether tuple/slice/vec bundle composition should be supported directly or left to
  small app-owned wrapper types.
- [ ] Migrate first-party "app integration pack" examples to use one consistent bundle story.
- [ ] Keep advanced builder-only install paths explicit; do not let them leak onto the default app
  path.

## 4. `CommandCatalog` Adoption

- [ ] Audit current command palette code in `fret-ui-shadcn` and split reusable catalog logic from
  shadcn-specific recipe/UI glue.
- [ ] Define `CommandCatalogEntry` and `CommandCatalogCx` as data-oriented types.
- [ ] Keep catalog entries execution-free; route activation back through typed actions or
  registered `CommandId`s.
- [ ] Document when a plain `CommandMeta` is sufficient and when a catalog source is warranted.

## 5. `RouteCodec` Adoption

- [ ] Add a single target note explaining how typed routes, canonical `RouteLocation`, and query
  integration fit together.
- [ ] Pick one in-tree app as the first codec-based migration target.
  - Preferred candidates: gallery routing or cookbook router basics.
- [ ] Remove remaining stringly route construction in official examples after the typed route path
  is ready.
- [ ] Keep browser/history adapters independent from codec ownership.

## 6. `DockPanelFactory` Adoption

- [ ] Audit current `DockPanelRegistry` call sites and classify them into:
  - app-owned bespoke registries,
  - reusable panel packs,
  - runtime-only helpers.
- [ ] Define a contribution-level factory shape that can aggregate into the existing registry
  service.
- [ ] Keep persistent panel identity anchored on `PanelKind` / `PanelKey`.
- [ ] Migrate at least one reusable panel set to the new contribution model before deleting old
  bespoke wiring guidance.

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
