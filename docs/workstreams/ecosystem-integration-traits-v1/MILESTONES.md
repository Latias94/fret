# Ecosystem Integration Traits v1 — Milestones

Status: Planning
Last updated: 2026-03-11

Related:

- `DESIGN.md`
- `TODO.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`

Current status on 2026-03-11:

- `InstallIntoApp` is now landed on the explicit `fret::integration` surface.
- `FretApp::setup(...)` and `UiAppBuilder::setup(...)` both accept `InstallIntoApp`.
- Small tuple composition is now intentionally supported for app-local setup bundles; slice/vec
  composition remains deferred.
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
- Free installer functions remain the default story; first-party bundle examples and the remaining
  trait-budget decisions are still open.

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

- grouped `cx.data().selector(...)` / `cx.data().query(...)` remains the official app-path story,
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
- this workstream can be downgraded from active refactor planning to maintenance.
