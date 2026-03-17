# Fret Docs — Start Here

This repository is intentionally documentation-driven: the goal is to lock in “hard-to-change” editor-grade UI
contracts early to avoid large rewrites later.

New to the repo? Start with:

- First hour onboarding (native): [docs/first-hour.md](./first-hour.md)
- Setup (native: toolchain + OS deps + fast builds): [docs/setup.md](./setup.md)
- Examples index (templates + cookbook + gallery + labs): [docs/examples/README.md](./examples/README.md)

Default onboarding ladder:

- **Default**: `hello` → `simple-todo` → `todo`
- **Comparison**: `simple_todo_v2_target` when you want to compare authoring density or local-state/list tradeoffs
- **Advanced**: gallery, interop, docking, renderer, and maintainer harnesses

Default app-author surface to keep in your head:

- `use fret::app::prelude::*;`
- `FretApp::new(...).window(...).view::<MyView>()?.run()`
- `impl View for MyView { fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui { ... } }`
- grouped defaults first: `cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`

## Command conventions (docs)

Unless a document says otherwise:

- Run commands from the repository root.
- Prefer the workspace runner: `cargo run -p fretboard -- ...`
  - Example (cookbook): `cargo run -p fretboard -- dev native --example simple_todo`
  - Example (native demo bin): `cargo run -p fretboard -- dev native --bin todo_demo`
- Some maintainer/labs docs reference the broad harness app directly:
  - `cargo run -p fret-demo --bin <name>`
  - This is not the first-hour onboarding path; start from [docs/first-hour.md](./first-hour.md) and
    [docs/examples/README.md](./examples/README.md) instead.

## Sources of Truth

- Progress: `docs/roadmap.md` and `docs/shadcn-declarative-progress.md`
- Action-first authoring + view runtime refactor (closed lane): `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
  - Closeout read: `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- Post-v1 authoring density reduction (closed closeout lane): `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/TODO.md`, and `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Dataflow authoring surface (closed closeout lane for selector/query + ecosystem/router boundary conclusions): `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TODO.md`, and `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`
- Action write surface (closed closeout lane for the default app-lane write budget): `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/action-write-surface-fearless-refactor-v1/TODO.md`, and `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- App composition density follow-on (closeout / maintenance lane: M1 closed on a no-new-API verdict, M2 closed on grouped query invalidation, router excluded): `docs/workstreams/app-composition-density-follow-on-v1/DESIGN.md`, `docs/workstreams/app-composition-density-follow-on-v1/TARGET_INTERFACE_STATE.md`, `docs/workstreams/app-composition-density-follow-on-v1/MILESTONES.md`, and `docs/workstreams/app-composition-density-follow-on-v1/TODO.md`
- Local-state architecture follow-on (closed decision lane; reopen only with fresh cross-surface evidence): `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/local-state-architecture-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/local-state-architecture-fearless-refactor-v1/TODO.md`, and `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Local-state facade boundary hardening (closed maintenance lane): `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/MILESTONES.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/TODO.md`, `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`, and `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Examples redesign (Flutter-like ladder + cookbook + labs + gates): `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
- Open source readiness (README + examples + defaults polish): [docs/workstreams/open-source-readiness-fearless-refactor-v1/DESIGN.md](./workstreams/open-source-readiness-fearless-refactor-v1/DESIGN.md)
- Framework modularity (Bevy-like consumption profiles): `docs/workstreams/framework-modularity-fearless-refactor-v1/design.md`
- Launch/app public surface refactor (`fret-launch` + `fret` facade + GPUI comparison): `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/MILESTONES.md`, and `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/FINAL_STATUS.md`
- App entry builder v1 (`fret::FretApp` onboarding + builder/hook story): `docs/workstreams/app-entry-builder-v1/DESIGN.md`, `docs/workstreams/app-entry-builder-v1/TODO.md`, and `docs/workstreams/app-entry-builder-v1/MILESTONES.md`
- Authoring surface + ecosystem reset (pre-release, no-compat cleanup): `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TODO.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, and `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/MIGRATION_MATRIX.md`
- Ecosystem integration traits budget (install/router/docking/query/catalog seams): `docs/workstreams/ecosystem-integration-traits-v1/DESIGN.md`, `docs/workstreams/ecosystem-integration-traits-v1/TODO.md`, `docs/workstreams/ecosystem-integration-traits-v1/MILESTONES.md`, `docs/workstreams/ecosystem-integration-traits-v1/TARGET_INTERFACE_STATE.md`, and `docs/workstreams/ecosystem-integration-traits-v1/MIGRATION_MATRIX.md`
- Into-element surface cleanup (follow-on to the authoring reset; collapse public conversion vocabulary): `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/MILESTONES.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`, and `docs/workstreams/into-element-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`
- Overlay + input arbitration v2: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- Overlay + pointer occlusion v2 progress: `docs/workstreams/overlay-input-arbitration-v2/overlay-input-arbitration-v2.md`
- UI focus + overlay focus containment (fearless refactor v1): `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/DESIGN.md`
- Foreground style context refactor (inheritance is context, not a wrapper fragment): `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- Foundation closure (P0, cross-workstream milestones): `docs/workstreams/foundation-closure-p0/foundation-closure-p0.md` and `docs/workstreams/foundation-closure-p0/foundation-closure-p0-todo.md`
- Headless table engine parity (TanStack Table core): `docs/workstreams/headless-table-tanstack-parity/headless-table-tanstack-parity.md` and `docs/workstreams/headless-table-tanstack-parity/headless-table-tanstack-parity-todo.md`
- Theme token alignment (semantic vs named vs component-derived): `docs/workstreams/theme-token-alignment-v1/design.md`, `docs/workstreams/theme-token-alignment-v1/todo.md`, and `docs/workstreams/theme-token-alignment-v1/milestones.md`
- Charts (ECharts alignment): `docs/audits/echarts-alignment.md` and `docs/delinea-echarts-alignment.md`
- Text system v2 tracker: `docs/workstreams/standalone/text-system-v2-parley.md`
- UI typography presets (stable control text line boxes): `docs/workstreams/ui-typography-presets-v1/ui-typography-presets-v1.md`
- Text style cascade fearless refactor (GPUI-style subtree text refinement for passive text): `docs/workstreams/text-style-cascade-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/text-style-cascade-fearless-refactor-v1/TODO.md`, and `docs/workstreams/text-style-cascade-fearless-refactor-v1/MILESTONES.md`
- Font system background roadmap + contract rationale (historical background; active execution moved to the fearless-refactor lane): `docs/workstreams/standalone/font-system-v1.md`
- Font system fearless refactor (active execution tracker for the pre-release hard reset around publication order, bundled profiles, and rescan semantics): `docs/workstreams/font-system-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/font-system-fearless-refactor-v1/TODO.md`, and `docs/workstreams/font-system-fearless-refactor-v1/MILESTONES.md`
- Input dispatch v2 tracker: `docs/workstreams/input-dispatch-v2/input-dispatch-v2.md`
- Mobile bring-up v1 (scroll + IME + keyboard avoidance): `docs/workstreams/mobile-bringup-v1/mobile-bringup-v1.md`, `docs/workstreams/mobile-bringup-v1/mobile-bringup-v1-todo.md`, and `docs/workstreams/mobile-bringup-v1/mobile-bringup-v1-milestones.md`
- Mobile graphics backend selection v1 (Vulkan/Metal-first + override + diagnostics): `docs/workstreams/mobile-gfx-backend-v1/design.md`, `docs/workstreams/mobile-gfx-backend-v1/todo.md`, and `docs/workstreams/mobile-gfx-backend-v1/milestones.md`
- Gesture recognizers v1 (component-layer policy): `docs/workstreams/gesture-recognizers-v1/gesture-recognizers-v1.md`, `docs/workstreams/gesture-recognizers-v1/gesture-recognizers-v1-todo.md`, and `docs/workstreams/gesture-recognizers-v1/gesture-recognizers-v1-milestones.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- Layout engine refactor: `docs/layout-engine-refactor-roadmap.md`
- Percent sizing semantics v1 (percent/fraction closure): `docs/workstreams/length-percentage-semantics-v1/length-percentage-semantics-v1.md` and `docs/workstreams/length-percentage-semantics-v1/length-percentage-semantics-v1-todo.md`
- Renderer refactor: `docs/renderer-refactor-roadmap.md`
- Renderer contract surface summary: `docs/renderer-contracts.md`
- GPU debugging (RenderDoc): `docs/renderdoc-inspection.md`
- Debugging playbook: `docs/debugging-playbook.md`
- CPU timeline profiling (Tracy): `docs/tracy.md`
- UI gallery profiling report (native): `docs/perf/ui-gallery-profile-report.md`
- UI Gallery docs-style component pages tracker: `docs/workstreams/standalone/ui-gallery-docs-page-layout-refactor.md`
- AI Elements port + selector surface alignment: `docs/workstreams/ai-elements-port/ai-elements-port.md`, `docs/workstreams/ai-elements-port/ai-elements-port-todo.md`, and `docs/workstreams/standalone/ai-elements-upstream-alignment.md`
- UI diagnostics + scripted repros: `docs/ui-diagnostics-and-scripted-tests.md`
- Diag artifact + evidence model (M2): `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- Inspect workflow (picker + scripts): `docs/debugging-ui-with-inspector-and-scripts.md`
- Window style profiles (ecosystem recipes): `docs/window-style-profiles.md`
- Viewport panels (engine/video): `docs/viewport-panels.md`
- Gizmo + viewport integration: `docs/gizmo-viewport-integration.md`
- Docking multi-window parity (ImGui-style tear-off): `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md` (macOS: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`)
- Docking diagnostics hardening (multi-window arbitration scripts + bounded evidence): `docs/workstreams/docking-arbitration-diag-hardening-v1/`
- UI diagnostics timebase decoupling v1 (no-frame liveness + `reason_code=timeout.no_frames`): `docs/workstreams/ui-diagnostics-timebase-decoupling-v1/README.md`
- Localization/i18n v1 tracker: `docs/workstreams/localization-i18n-v1/localization-i18n-v1.md` and `docs/workstreams/localization-i18n-v1/localization-i18n-v1-todo.md`
- Contracts: `docs/adr/`
- Audit notes index (non-authoritative): `docs/audits/README.md`
- Workstream notes (non-authoritative): `docs/workstreams/`
- Historical planning docs: `docs/archive/`

## Public crate surfaces (what to remember)

We intentionally keep the *user-facing* story to a small set of crate names:
these are the only crate names we treat as stable entry points; internal crates may be renamed or reshuffled.

- `fret`: desktop-first batteries-included app entry points (recommended).
- `fret-ui-shadcn`: default component surface (shadcn/ui-aligned taxonomy + recipes).
- `fret-ui-kit`: component authoring glue (policies + headless primitives + declarative helpers).
- `fret-framework`: framework facade for advanced/manual assembly and integrations.
- `fretboard`: dev tooling (templates + native/web demo runner).

Web/wasm runs through tooling (not through `fret`):

- `cargo run -p fretboard -- dev web --demo ui_gallery`

## Consumption profiles (modularity)

Fret is designed to be consumed modularly. If you want a stable “pick only what you need” story, track:

- Design: `docs/workstreams/framework-modularity-fearless-refactor-v1/design.md`
- TODO: `docs/workstreams/framework-modularity-fearless-refactor-v1/todo.md`
- Milestones: `docs/workstreams/framework-modularity-fearless-refactor-v1/milestones.md`

Portable profiles we treat as regression gates:

- Contracts-only: `fret-core` (+ `fret-runtime`, `fret-platform`, `fret-render-core`)
- UI substrate: `fret-ui`
- Manual assembly facade (portable): `fret-framework` with `--no-default-features --features core,runtime,ui`

## State management (authoring ergonomics)

Fret’s kernel/runtime primitives are intentionally small (`Model<T>`, tracked invalidation,
driver-boundary inbox draining), so the default app-authoring story lives in ecosystem crates and is
now taught as `LocalState` + view runtime + typed actions.

- Workstream: `docs/workstreams/state-management-v1/state-management-v1.md` and `docs/workstreams/state-management-v1/state-management-v1-todo.md`
- Action-first authoring + view runtime (v1, available now):
  - Workstream: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
  - Closeout read: `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
  - ADRs: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`, `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
  - Template entry points: `cargo run -p fretboard -- new hello`, `cargo run -p fretboard -- new simple-todo`, `cargo run -p fretboard -- new todo`
- Dataflow authoring surface follow-on (closed selector/query closeout lane):
  - Workstream: `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
  - Target state: `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
  - Scope note: the selector/query posture and ecosystem/router boundary are closed there; write-side follow-on moved out
- Action write surface follow-on (closed closeout lane for the default app-lane write budget):
  - Workstream: `docs/workstreams/action-write-surface-fearless-refactor-v1/DESIGN.md`
  - Target state: `docs/workstreams/action-write-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
  - Closeout: `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
  - Scope note: the default app-lane `cx.actions()` write budget is now frozen there; selector/query/router stay out unless a new narrower follow-on reopens them separately
  - Post-closeout retained-seam audit: `docs/workstreams/action-write-surface-fearless-refactor-v1/RETAINED_PAYLOAD_SURFACE_AUDIT_2026-03-17.md`
  - Scope note: keep payload retained-seam dedup/delete-ready notes inside the original action-write folder unless the question expands beyond payload-surface residue
- App composition density follow-on (closeout / maintenance lane):
  - Workstream: `docs/workstreams/app-composition-density-follow-on-v1/DESIGN.md`
  - Target state: `docs/workstreams/app-composition-density-follow-on-v1/TARGET_INTERFACE_STATE.md`
  - Scope note: M1 is now closed on a no-new-API composition verdict, M2 is now closed on grouped `cx.data().invalidate_query*`, and remaining work is closeout / maintenance only
- Local-state architecture (closed contract-decision lane):
  - Workstream: `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`
  - Closeout: `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
  - Purpose: record why the long-term `LocalState<T>` contract stays model-backed in v1, keep
    `use_state` as the explicit raw-model seam, and name the evidence required before reopening a
    narrower storage-model lane.
- Local-state facade boundary hardening (closed narrow follow-on):
  - Workstream: `docs/workstreams/local-state-facade-boundary-hardening-v1/DESIGN.md`
  - Inventory: `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`
  - Closeout: `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`
  - Purpose: translate the O1 decision into a hardened public facade by classifying `use_state`,
    `LocalState::{model, clone_model}`, `LocalState::*_in(...)`, and helper-context bridge
    surfaces without changing the storage model.
- Recommended building blocks:
  - View runtime + hooks + typed unit actions (golden path): `ecosystem/fret` (`View`, `AppUi`, `fret::actions!`)
  - Derived state (selectors/computed): `ecosystem/fret-selector`
  - Async resources (loading/error/cache/invalidation): `ecosystem/fret-query`
- Canonical startup/import reminder:
  - app-facing imports live under `use fret::app::prelude::*;`
  - default native startup uses `FretApp::new(...).window(...).view::<MyView>()?.run()`
- Keyed row payloads (default when rows are view-owned): `fret::payload_actions!` + `ui::for_each_keyed(...)` + `.action_payload(...)`, with `payload_local_update_if::<A>(...)` as the default row write path.
- Default entrypoints (recommended mental model):
  - `LocalState<T>` / `LocalState<Vec<_>>` - default for view-owned state, including starter keyed lists.
  - `cx.actions().locals::<A>(|tx| ...)` - default for most typed UI actions that coordinate view-owned `LocalState<T>` slots.
  - `cx.actions().transient::<A>(...)` - default when the real work must happen with `&mut App` in `render()`.
  - `cx.actions().models::<A>(|models| ...)` - drop down when coordinating shared `Model<T>` graphs (cross-view ownership).
- widget-local `.action(...)` / `.action_payload(...)` / `.listen(...)` - use this only when a control exposes activation glue rather than a narrower widget-owned app-facing helper. Import `use fret::app::AppActivateExt as _;` explicitly for that bridge; raw `on_activate*` helper families stay component/advanced-oriented.
  - Treat raw `on_action_notify` / `on_payload_action_notify` as advanced shorthands; `on_action` and the former single-model aliases are deleted, so keep first-contact docs and templates focused on the default entrypoints above plus keyed-row payload binding. The remaining in-tree examples are cookbook-only host-side categories (toasts, background scheduling, RAF effects).
- Surface taxonomy:
  - **Default**: `hello`, `simple-todo`, `todo`, plus stable cookbook lessons
  - **Comparison**: `simple_todo_v2_target` and other evidence-oriented side-by-side samples
  - **Advanced**: gallery, viewport/interop, docking, renderer, maintainer harnesses
- Upgrade guidance (app authors): `docs/fearless-refactoring.md`
- Authoring golden path (v2): `docs/authoring-golden-path-v2.md`
- Integration guidance:
  - Async fetch (tokio/wasm): `docs/integrating-tokio-and-reqwest.md`
  - Persistence (sqlite/sqlx): `docs/integrating-sqlite-and-sqlx.md`
  - Service injection + subtree overrides: `docs/service-injection-and-overrides.md`

## Recommended reading order (for a new contributor or AI agent)

1. `docs/architecture.md` — what Fret is, what it is not, and how crates fit together.
   - Repository layout (core vs ecosystem): `docs/repo-structure.md`
2. `docs/golden-architecture.md` — module closure index (docs/ADRs/code/tests entry points).
3. `docs/ui-closure-map.md` — closure-oriented UI subsystem map (overlays/transform/clip/a11y/docking).
4. `docs/runtime-contract-matrix.md` — a compact map of the `fret-ui` runtime contract surface and references.
5. `docs/roadmap.md` — priorities, decision gates, and milestones (what to do next, and what must be decided early).
6. `docs/shadcn-declarative-progress.md` — shadcn/ui v4 parity and the declarative-only migration tracker.
7. `docs/action-hooks.md` — how we keep `fret-ui` mechanism-only by moving interaction policy to components (ADR 0074).
8. `docs/component-authoring-contracts.md` — the public APIs and “gotchas” component authors should rely on (living checklist).
9. `docs/adr/README.md` — module-oriented ADR index (where to find the relevant contracts fast).
   - Tip: ADRs marked `Status: Proposed` are “decision gates” and should be treated as changeable until accepted.
10. `docs/repo-ref.md` — pinned local reference sources (where to read upstream code without version drift).
11. `docs/dependency-policy.md` — dependency and MSRV policy (how we keep contracts portable).
12. `docs/todo-tracker.md` — review-driven TODO list (action items linked back to ADRs).
13. `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md` — one-page status view for what is done, what is maintenance mode, what is still architectural, and what remains on the hard-delete track.
14. `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md` and `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md` — use these when the question is no longer default-path sugar, but the long-term storage/ownership contract behind `LocalState<T>` and why v1 closes on the current model-backed contract.
15. `docs/known-issues.md` — common diagnostics and current platform limitations.
16. Archived MVP planning docs (historical): `docs/archive/mvp.md`, `docs/archive/mvp/active-plan.md`, `docs/archive/mvp-archive.md`
17. ADR deep dives (pick by subsystem):
   - UI execution model: `docs/adr/0028-declarative-elements-and-element-state.md`
   - Component authoring: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
   - Ownership/data flow: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
   - Scheduling/observability: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
   - Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
   - Drag & drop / clipboard: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
   - Text + SDF: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
   - Docking + multi-window: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0017-multi-window-display-and-dpi.md`

## Code Entry Points (After You Read The Docs)

- End-to-end demo wiring (effects → runner → render): [apps/fret-examples/src/components_gallery.rs](../apps/fret-examples/src/components_gallery.rs)
  - Run: `cargo run -p fretboard -- dev native --bin components_gallery`
- Todo app “golden path” (shadcn + bootstrap): [apps/fret-examples/src/todo_demo.rs](../apps/fret-examples/src/todo_demo.rs)
  - Run: `cargo run -p fretboard -- dev native --bin todo_demo`
- Starter todo template generator: `cargo run -p fretboard -- new todo --name my-todo`
  - Guide: [docs/examples/todo-app-golden-path.md](./examples/todo-app-golden-path.md)
- Windows build speed note: prefer `fretboard dev native ...` (defaults to `--profile dev-fast` on Windows).
- Docking + viewport + overlays conformance harness (ADR 0072): [apps/fret-examples/src/docking_arbitration_demo.rs](../apps/fret-examples/src/docking_arbitration_demo.rs)
  - Run: `cargo run -p fretboard -- dev native --bin docking_arbitration_demo`
  - Checklist: [docs/docking-arbitration-checklist.md](./docking-arbitration-checklist.md)
- Plot demos (2D): [apps/fret-examples/src/plot_demo.rs](../apps/fret-examples/src/plot_demo.rs)
- Plot stress harness (desktop-only): [apps/fret-examples/src/plot_stress_demo.rs](../apps/fret-examples/src/plot_stress_demo.rs)
- A11y manual acceptance checklist (overlays + demo): [docs/a11y-acceptance-checklist.md](./a11y-acceptance-checklist.md)
- App runtime (effects + models + commands): [crates/fret-app/src/app.rs](../crates/fret-app/src/app.rs)
- Desktop runner (integrated example; winit window lifecycle + scheduling): [crates/fret-launch/src/runner/mod.rs](../crates/fret-launch/src/runner/mod.rs)
  - Crate boundaries (ADR 0092): [docs/adr/0092-crate-structure-core-backends-apps.md](./adr/0092-crate-structure-core-backends-apps.md)
- UI runtime substrate (UiTree + declarative bridge): [crates/fret-ui/src/tree/mod.rs](../crates/fret-ui/src/tree/mod.rs) and [crates/fret-ui/src/declarative/](../crates/fret-ui/src/declarative/)
- Docking UI (`DockSpace`, policy-heavy): [ecosystem/fret-docking/src/dock/space.rs](../ecosystem/fret-docking/src/dock/space.rs)
- Renderer (display list → wgpu pipelines; SDF AA lives here): [crates/fret-render-wgpu/src/renderer/mod.rs](../crates/fret-render-wgpu/src/renderer/mod.rs)

## Repository references

These upstream repositories are used as non-normative reference sources (design + implementation vocabulary):

- Zed/GPUI: https://github.com/zed-industries/zed
- GPUI component experiments: https://github.com/zed-industries/zed
- Godot editor workflows: https://github.com/godotengine/godot

If you want local, pinned snapshots for alignment work, see `docs/repo-ref.md`.

These references are not required to build Fret, but they are helpful when validating architectural decisions.
