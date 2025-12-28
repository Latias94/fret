# Fret Docs — Start Here

This repository is intentionally documentation-driven: the goal is to lock in “hard-to-change” editor-grade UI
contracts early to avoid large rewrites later.

## Recommended reading order (for a new contributor or AI agent)

1. `docs/architecture.md` — what Fret is, what it is not, and how crates fit together.
2. `docs/runtime-contract-matrix.md` — a compact map of the `fret-ui` runtime contract surface and references.
3. `docs/roadmap.md` — priorities, decision gates, and milestones (what to do next, and what must be decided early).
4. `docs/adr/README.md` — module-oriented ADR index (where to find the relevant contracts fast).
   - Tip: ADRs marked `Status: Proposed` are “decision gates” and should be treated as changeable until accepted.
5. `docs/repo-ref.md` — pinned local reference sources (where to read upstream code without version drift).
6. `docs/dependency-policy.md` — dependency and MSRV policy (how we keep contracts portable).
7. `docs/todo-tracker.md` — review-driven TODO list (action items linked back to ADRs).
8. `docs/known-issues.md` — common diagnostics and current platform limitations.
9. Archived MVP planning docs (historical): `docs/archive/mvp.md`, `docs/archive/mvp/active-plan.md`, `docs/archive/mvp-archive.md`
10. ADR deep dives (pick by subsystem):
   - UI execution model: `docs/adr/0028-declarative-elements-and-element-state.md`
   - Component authoring: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
   - Ownership/data flow: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
   - Scheduling/observability: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
   - Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`
   - Drag & drop / clipboard: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
   - Text + SDF: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
   - Docking + multi-window: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0017-multi-window-display-and-dpi.md`

## Code Entry Points (After You Read The Docs)

- End-to-end demo wiring (effects → runner → render): `crates/fret-demo/src/components_gallery.rs`
- UI kit harness (widgets/components + overlays): `crates/fret-demo/src/ui_kit.rs` (or `cargo run -p fret-demo --bin ui_kit`)
- A11y manual acceptance checklist (overlays + demo): `docs/a11y-acceptance-checklist.md`
- App runtime (effects + models + commands): `crates/fret-app/src/app.rs`
- Desktop runner (winit window lifecycle + scheduling): `crates/fret-runner-winit-wgpu/src/runner.rs`
- UI runtime (retained tree prototype) + docking widget: `crates/fret-ui/src/tree.rs`, `crates/fret-ui/src/dock.rs`
- Renderer (display list → wgpu pipelines; SDF AA lives here): `crates/fret-render/src/renderer.rs`

## Repository references

- `repo-ref/zed` and `repo-ref/gpui-component` are local reference checkouts used to study GPUI patterns.
- `repo-ref/godot` is used to study editor workflows (docking, multi-window, viewport patterns).

These references are not required to build Fret, but they are helpful when validating architectural decisions.
