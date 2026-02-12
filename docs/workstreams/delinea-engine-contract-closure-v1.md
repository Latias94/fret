# Delinea Engine Contract Closure v1 (ECharts-Class Foundations)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- TODO tracker: `docs/workstreams/delinea-engine-contract-closure-v1-todo.md`
- Milestone board: `docs/workstreams/delinea-engine-contract-closure-v1-milestones.md`

## 0) Why this workstream exists

`ecosystem/delinea` + `ecosystem/fret-chart` already form a usable ECharts-inspired chart stack:

- portable headless engine (no `wgpu`/`winit`),
- explicit, revision-driven state transitions,
- budgeted incremental stages and caches,
- and a UI adapter that can run on native + wasm.

The remaining “hard-to-change” gaps are **engine-level contracts**, not UI polish:

1. **Single-engine multi-grid** (per-grid viewports/layout/routing) instead of “split spec into multiple engines”.
2. **ECharts-class transform lineage** (derived datasets/columns with stable raw-index identity).
3. **Incremental data mutation semantics** (append/update boundaries + cache invalidation rules).
4. **A stable participation/output contract** that downstream consumers can rely on (marks, hit test, axisPointer,
   tooltips, brush export, virtualization).
5. **Stable linking contracts** for cross-chart coordination (cursor/axisPointer + domain windows), so hosts can build
   dashboard-grade synchronization without UI-specific hacks (ADRs 0249/0250/0251).

This workstream tracks those closures as a sequence of milestones with explicit “done” gates (tests + demos).

## 0.1) Worktree posture (fearless refactors are OK)

This workstream is currently developed in a dedicated worktree/branch and is not expected to have
stable external callers yet. That means we prefer the “correct shape” over compatibility hacks:

- write/refresh ADRs first when behavior contracts change,
- refactor aggressively to keep contract surfaces small and coherent,
- delete redundant legacy paths once the v1 posture is locked by tests + goldens.

## 1) Invariants (do not break)

1. **Headless portability**
   - `delinea` stays renderer-agnostic and portable (desktop + wasm).
2. **Budgeted work**
   - Any indices/masks/scans must be budgeted (`WorkBudget`) and cacheable by revision + parameters.
3. **Deterministic outputs**
   - Behavior must remain deterministic under the same inputs (seedless, order-stable).
4. **Clear separation of concerns**
   - Engine owns semantics and contracts.
   - UI adapters own gesture mapping and presentation.
5. **No silent contract drift**
   - If a milestone changes a stable behavior, update the relevant ADR first (or add an amendment).

## 2) Current architecture snapshot (what exists today)

Engine-level anchors:

- Filter composition scaffold: per-grid step plan in `FilterProcessorStage` (order-sensitive, X-before-Y).
- Unified participation contract: `ParticipationState` produced by the filter processor and consumed by downstream
  stages (marks, axisPointer, brush export).
- Indices carriers and append-only reuse: `TransformGraph::data_views` (`DataViewStage`) with prefix reuse rules.
- Percent-window surface: `Action::SetAxisWindowPercent` → derived value windows pre-view rebuild.

Adapter-level anchors:

- Multi-axis conformance demo (native + wasm): `apps/fret-examples/src/chart_multi_axis_demo.rs`.
- Multi-grid UI (retained) hosts a **single** engine instance and supplies per-grid plot viewports:
  - multi-canvas layout: `ecosystem/fret-chart/src/retained/multi_grid.rs`
  - per-grid plot viewport patching: `ecosystem/fret-chart/src/retained/canvas.rs` (`grid_override`)
  - global controllers (single legend + tooltip/axisPointer overlay):
    `ecosystem/fret-chart/src/retained/multi_grid.rs` (`FillStack`) +
    `ecosystem/fret-chart/src/retained/canvas.rs` (`ChartCanvas::new_overlay`)
  - demo: `apps/fret-examples/src/echarts_multi_grid_demo.rs`
- Cross-chart linking (v1):
  - engine emits `LinkEvent` streams (`AxisPointerChanged`, `DomainWindowChanged`) when a link group is set.
  - host/adapter code coordinates shared key-space models via `LinkAxisKey` (`ecosystem/fret-chart/src/linking.rs`).
  - ambiguous axis mappings can be overridden explicitly via `ChartCanvas::link_axis_map(...)`.

## 7) UI adapter consolidation plan (TanStack-style posture)

The long-term “correct” shape is:

1. **One headless engine, many views**
   - `ChartEngine` owns semantics + deterministic outputs.
   - UI hosts provide layout inputs (per-grid plot viewports) and render marks.
2. **Global controllers (B)**
   - One shared legend + one shared tooltip/axisPointer overlay for the whole multi-grid surface.
   - Individual grid views are “plot-only” surfaces (no per-grid legend duplication).
   - v1 retained status: implemented (grid views suppress legend/tooltip; overlay canvas draws once).
3. **Unambiguous routing in outputs**
   - Engine outputs that are tied to a specific grid (e.g. `axisPointer`) must carry `GridId` so
     adapters do not guess and do not draw duplicates.
4. **Delete legacy paths**
   - No adapter-side “spec splitting into multiple engines” once single-engine multi-grid is in place.

## 3) Scope (what this workstream is and is not)

In scope:

- Engine contracts for multi-grid, transforms/lineage, participation/output, and incremental data updates.
- Small adapter changes that are strictly required by engine contract closure (routing and wiring).
- Regression gates (Rust tests + headless goldens; optionally `fretboard diag` scripts for interactive semantics).

Out of scope (for v1):

- Full Apache ECharts option-schema parity (translator breadth is tracked elsewhere).
- Non-cartesian coordinate systems (polar, geo, etc.).
- Rendering backends and GPU-level perf work (separate workstreams).

## 4) Milestone map (high-level)

See the one-screen board: `docs/workstreams/delinea-engine-contract-closure-v1-milestones.md`.

The default sequencing is:

1. M0 — Documentation + audit closure (keep docs honest; align ADRs with current v1 subset behavior).
2. M1 — Single-engine multi-grid viewport/layout contract.
3. M2 — Transform lineage contract (derived datasets/columns with stable identity).
4. M3 — Incremental mutation semantics (append/update) + cache invalidation boundaries.
5. M4 — Conformance harnesses (headless + interactive) that keep refactors safe.

Current posture highlight:

- M3 append-only behavior under `WorkBudget` is now regression-gated (multi-series, unfinished-step continuity).
- M3 update semantics are explicit (no silent column mutation) via `DataTable` update APIs and an engine-level invalidation gate.
- M4 interactive domain-window linking is gated via `fretboard diag` pixels-changed checks (`tools/diag-scripts/chart-multi-axis-linking-domain-window-pixels-changed.json`).

## 5) Reference posture (what we borrow, not what we copy)

Local snapshots under `repo-ref/` are non-normative, but they help avoid reinventing semantics:

- Apache ECharts (`repo-ref/echarts`): `dataZoomProcessor`, AxisProxy, dataset transforms, filter modes.
- Zed/GPUI (`repo-ref/zed`): contract-driven incremental pipelines and caching posture.

## 6) Definition of done (workstream-level)

We consider this workstream “done” when:

1. Multi-grid is supported in a **single** `ChartEngine` instance with per-grid layout and deterministic routing.
2. Transform lineage exists as a first-class engine contract (no adapter-side “eager table cloning” required for core
   transforms).
3. Incremental data update semantics are explicit and regression-gated.
4. The participation/output contract is stable, documented, and consumed consistently by all downstream stages.

## 8) Current plan (2026-02-10)

1. Keep the worktree green while `main` evolves (materials + `Paint`/`SceneOp::Quad` API changes).
   - Evidence: see the “Main sync” section in the TODO tracker.
2. Finish M4 conformance harness closure:
   - ensure headless goldens remain stable after refactors,
   - keep at least one interactive/scripted linking gate runnable (`fretboard diag`),
   - document a minimal “fast” nextest suite for local + CI use.
