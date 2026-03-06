# Fret Docs — Start Here

This repository is intentionally documentation-driven: the goal is to lock in “hard-to-change” editor-grade UI
contracts early to avoid large rewrites later.

New to the repo? Start with:

- First hour onboarding (native): [docs/first-hour.md](./first-hour.md)
- Setup (native: toolchain + OS deps + fast builds): [docs/setup.md](./setup.md)
- Examples index (templates + cookbook + gallery + labs): [docs/examples/README.md](./examples/README.md)

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
- Action-first authoring + view runtime refactor: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- Examples redesign (Flutter-like ladder + cookbook + labs + gates): `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
- Open source readiness (README + examples + defaults polish): [docs/workstreams/open-source-readiness-fearless-refactor-v1/DESIGN.md](./workstreams/open-source-readiness-fearless-refactor-v1/DESIGN.md)
- Framework modularity (Bevy-like consumption profiles): `docs/workstreams/framework-modularity-fearless-refactor-v1/design.md`
- Launch/app public surface refactor (`fret-launch` + `fret` facade + GPUI comparison): `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/EXPORT_INVENTORY.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/CONFIG_INVENTORY.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md`, `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md`, and `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/MILESTONES.md`
- Overlay + input arbitration v2: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- Overlay + pointer occlusion v2 progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- UI focus + overlay focus containment (fearless refactor v1): `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/DESIGN.md`
- Foreground style context refactor (inheritance is context, not a wrapper fragment): `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- Foundation closure (P0, cross-workstream milestones): `docs/workstreams/foundation-closure-p0.md` and `docs/workstreams/foundation-closure-p0-todo.md`
- Headless table engine parity (TanStack Table core): `docs/workstreams/headless-table-tanstack-parity.md` and `docs/workstreams/headless-table-tanstack-parity-todo.md`
- Theme token alignment (semantic vs named vs component-derived): `docs/workstreams/theme-token-alignment-v1/design.md`, `docs/workstreams/theme-token-alignment-v1/todo.md`, and `docs/workstreams/theme-token-alignment-v1/milestones.md`
- Charts (ECharts alignment): `docs/audits/echarts-alignment.md` and `docs/delinea-echarts-alignment.md`
- Text system v2 tracker: `docs/workstreams/text-system-v2-parley.md`
- UI typography presets (stable control text line boxes): `docs/workstreams/ui-typography-presets-v1.md`
- Font system audit + roadmap: `docs/workstreams/font-system-v1.md`
- Input dispatch v2 tracker: `docs/workstreams/input-dispatch-v2.md`
- Mobile bring-up v1 (scroll + IME + keyboard avoidance): `docs/workstreams/mobile-bringup-v1.md`, `docs/workstreams/mobile-bringup-v1-todo.md`, and `docs/workstreams/mobile-bringup-v1-milestones.md`
- Mobile graphics backend selection v1 (Vulkan/Metal-first + override + diagnostics): `docs/workstreams/mobile-gfx-backend-v1/design.md`, `docs/workstreams/mobile-gfx-backend-v1/todo.md`, and `docs/workstreams/mobile-gfx-backend-v1/milestones.md`
- Gesture recognizers v1 (component-layer policy): `docs/workstreams/gesture-recognizers-v1.md`, `docs/workstreams/gesture-recognizers-v1-todo.md`, and `docs/workstreams/gesture-recognizers-v1-milestones.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- Layout engine refactor: `docs/layout-engine-refactor-roadmap.md`
- Percent sizing semantics v1 (percent/fraction closure): `docs/workstreams/length-percentage-semantics-v1.md` and `docs/workstreams/length-percentage-semantics-v1-todo.md`
- Renderer refactor: `docs/renderer-refactor-roadmap.md`
- Renderer contract surface summary: `docs/renderer-contracts.md`
- GPU debugging (RenderDoc): `docs/renderdoc-inspection.md`
- Debugging playbook: `docs/debugging-playbook.md`
- CPU timeline profiling (Tracy): `docs/tracy.md`
- UI gallery profiling report (native): `docs/perf/ui-gallery-profile-report.md`
- UI Gallery docs-style component pages tracker: `docs/workstreams/ui-gallery-docs-page-layout-refactor.md`
- UI diagnostics + scripted repros: `docs/ui-diagnostics-and-scripted-tests.md`
- Inspect workflow (picker + scripts): `docs/debugging-ui-with-inspector-and-scripts.md`
- Window style profiles (ecosystem recipes): `docs/window-style-profiles.md`
- Viewport panels (engine/video): `docs/viewport-panels.md`
- Gizmo + viewport integration: `docs/gizmo-viewport-integration.md`
- Docking multi-window parity (ImGui-style tear-off): `docs/workstreams/docking-multiwindow-imgui-parity.md` (macOS: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`)
- Docking diagnostics hardening (multi-window arbitration scripts + bounded evidence): `docs/workstreams/docking-arbitration-diag-hardening-v1/`
- UI diagnostics timebase decoupling v1 (no-frame liveness + `reason_code=timeout.no_frames`): `docs/workstreams/ui-diagnostics-timebase-decoupling-v1/README.md`
- Localization/i18n v1 tracker: `docs/workstreams/localization-i18n-v1.md` and `docs/workstreams/localization-i18n-v1-todo.md`
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

Fret’s kernel primitives are intentionally small (`Model<T>`, explicit invalidation, driver-boundary inbox draining),
so the default authoring story lives in ecosystem crates.

- Workstream: `docs/workstreams/state-management-v1.md` and `docs/workstreams/state-management-v1-todo.md`
- Action-first authoring + view runtime (v1, available now):
  - Workstream: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
  - ADRs: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`, `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
  - Template entry points: `cargo run -p fretboard -- new hello`, `cargo run -p fretboard -- new todo`, `cargo run -p fretboard -- new simple-todo`
- Recommended building blocks:
  - View runtime + hooks + typed unit actions (golden path): `ecosystem/fret` (`View`, `ViewCx`, `fret::actions!`)
  - Derived state (selectors/computed): `ecosystem/fret-selector`
  - Async resources (loading/error/cache/invalidation): `ecosystem/fret-query`
  - Legacy/dynamic routing (per-item payloads): `fret::mvu::MessageRouter<M>` (compat; avoid in new templates)
- Default entrypoints (recommended mental model):
  - `cx.on_action_notify_models::<A>(|models| ...)` - default for most typed UI actions.
  - `cx.on_action_notify_transient::<A>(...)` - default when the real work must happen with `&mut App` in `render()`.
  - `on_activate(...)` / `on_activate_notify(...)` - local pressable/widget glue only; do not treat these as the default replacement for typed action handlers.
  - Treat raw `on_action` / `on_action_notify` and single-model aliases as advanced shorthands; keep first-contact docs and templates focused on the three entrypoints above.
- Upgrade guidance (app authors): `docs/fearless-refactoring.md`
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
13. `docs/known-issues.md` — common diagnostics and current platform limitations.
14. Archived MVP planning docs (historical): `docs/archive/mvp.md`, `docs/archive/mvp/active-plan.md`, `docs/archive/mvp-archive.md`
15. ADR deep dives (pick by subsystem):
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
