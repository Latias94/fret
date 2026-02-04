# Fret Docs — Start Here

This repository is intentionally documentation-driven: the goal is to lock in “hard-to-change” editor-grade UI
contracts early to avoid large rewrites later.

## Sources of Truth

- Progress: `docs/roadmap.md` and `docs/shadcn-declarative-progress.md`
- Overlay + input arbitration v2: `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- Overlay + pointer occlusion v2 progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- Foundation closure (P0, cross-workstream milestones): `docs/workstreams/foundation-closure-p0.md` and `docs/workstreams/foundation-closure-p0-todo.md`
- Headless table engine parity (TanStack Table core): `docs/workstreams/headless-table-tanstack-parity.md` and `docs/workstreams/headless-table-tanstack-parity-todo.md`
- Charts (ECharts alignment): `docs/audits/echarts-alignment.md` and `docs/delinea-echarts-alignment.md`
- Text system v2 tracker: `docs/workstreams/text-system-v2-parley.md`
- Input dispatch v2 tracker: `docs/workstreams/input-dispatch-v2.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- Layout engine refactor: `docs/layout-engine-refactor-roadmap.md`
- Renderer refactor: `docs/renderer-refactor-roadmap.md`
- GPU debugging (RenderDoc): `docs/renderdoc-inspection.md`
- Debugging playbook: `docs/debugging-playbook.md`
- CPU timeline profiling (Tracy): `docs/tracy.md`
- UI gallery profiling report (native): `docs/perf/ui-gallery-profile-report.md`
- UI diagnostics + scripted repros: `docs/ui-diagnostics-and-scripted-tests.md`
- Inspect workflow (picker + scripts): `docs/debugging-ui-with-inspector-and-scripts.md`
- Viewport panels (engine/video): `docs/viewport-panels.md`
- Gizmo + viewport integration: `docs/gizmo-viewport-integration.md`
- Docking multi-window parity (ImGui-style tear-off): `docs/workstreams/docking-multiwindow-imgui-parity.md` (macOS: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`)
- Contracts: `docs/adr/`
- Workstream notes (non-authoritative): `docs/workstreams/`
- Historical planning docs: `docs/archive/`

## Public crate surfaces (what to remember)

We intentionally keep the *user-facing* story to a small set of crate names:
these are the only crate names we treat as stable entry points; internal crates may be renamed or reshuffled.

- `fret-kit`: desktop-first batteries-included app entry points.
- `fret-ui-shadcn`: default component surface (shadcn/ui-aligned taxonomy + recipes).
- `fret-ui-kit`: component authoring glue (policies + headless primitives + declarative helpers).
- `fret`: framework facade for advanced/manual assembly and integrations.
- `fretboard`: dev tooling (templates + native/web demo runner).

Web/wasm runs through tooling (not through `fret-kit`):

- `cargo run -p fretboard -- dev web --demo ui_gallery`

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

- End-to-end demo wiring (effects → runner → render): `apps/fret-examples/src/components_gallery.rs`
- Todo app “golden path” (shadcn + bootstrap): `apps/fret-examples/src/todo_demo.rs` (or `fretboard dev native --bin todo_demo`)
- Starter todo template generator: `fretboard new todo --name my-todo` (uses `fret-kit`; see `docs/examples/todo-app-golden-path.md`)
- shadcn surface smoke test (components + overlays): `apps/fret-examples/src/components_gallery.rs` (or `cargo run -p fret-demo --bin components_gallery`)
- Docking + viewport + overlays conformance harness (ADR 0072): `apps/fret-examples/src/docking_arbitration_demo.rs` (or `cargo run -p fret-demo --bin docking_arbitration_demo`; checklist: `docs/docking-arbitration-checklist.md`)
- Plot demos (2D): `apps/fret-examples/src/plot_demo.rs` (or `cargo run -p fret-demo --bin plot_demo`; web: `apps/fret-demo-web` + `?demo=plot_demo`)
- Plot stress harness (desktop-only): `apps/fret-examples/src/plot_stress_demo.rs` (or `cargo run -p fret-demo --bin plot_stress_demo`)
- A11y manual acceptance checklist (overlays + demo): `docs/a11y-acceptance-checklist.md`
- App runtime (effects + models + commands): `crates/fret-app/src/app.rs`
- Desktop runner (integrated example; winit window lifecycle + scheduling): `crates/fret-launch/src/runner/mod.rs`
   - Note: crate boundary direction is “core vs backends vs apps” (ADR 0093): `docs/adr/0093-crate-structure-core-backends-apps.md`
- UI runtime substrate (UiTree + declarative bridge): `crates/fret-ui/src/tree/mod.rs` and `crates/fret-ui/src/declarative/`
- Docking UI (`DockSpace`, policy-heavy): `ecosystem/fret-docking/src/dock/space.rs`
- Renderer (display list → wgpu pipelines; SDF AA lives here): `crates/fret-render/src/renderer/mod.rs`

## Repository references

- `repo-ref/zed` is a local reference checkout used to study GPUI patterns.
- Optional (clone locally when needed): `repo-ref/gpui-component` is used to study component ergonomics and theme schema patterns (see `docs/repo-ref.md`).
- Optional (clone locally when needed): `repo-ref/godot` is used to study editor workflows (docking, multi-window, viewport patterns).

These references are not required to build Fret, but they are helpful when validating architectural decisions.
