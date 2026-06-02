# Fret Architecture Lanes

Status: planner routing registry
Last updated: 2026-05-30

ADRs remain the source of truth for contracts. `docs/architecture.md` and
`docs/golden-architecture.md` remain the architecture overview and module closure index. This file
only maps active workstreams to planner/lane ownership so multi-worktree Codex sessions can avoid
scope collisions.

## Rules

- Keep workstreams in `docs/workstreams/<slug>/`; do not move them into lane folders.
- Every active `WORKSTREAM.json` should have `lane_slug` matching one slug below.
- Keep each lane's active queue short. Split or close stale active workstreams before opening more.
- Lane terminals own capability areas, not global sequencing. Planner owns shared-scope conflicts.
- Shared contract changes, ADR changes, workspace dependency changes, and cross-lane file scopes
  must return to planner coordination.

## Lanes

### `imui-docking`

Scope: immediate-mode compatibility, Dear ImGui gap closure, docking, multi-window, and editor-grade
IMUI proof surfaces.

Primary docs:

- `docs/workstreams/imui-imgui-gap-closure-v1/`
- `docs/docking-imgui-parity-matrix.md`
- `docs/golden-architecture.md`

Active queue:

- `docs/workstreams/docking-multiwindow-imgui-parity/`
- `docs/workstreams/imui-imgui-gap-closure-v1/`

Shared scopes: `ecosystem/fret-imui`, `ecosystem/fret-ui-kit/src/imui*`, docking examples, and
workspace shell demos.

### `shadcn-parity`

Scope: shadcn/ui parity, parity discovery harnesses, fixture promotion, and UI gallery parity gates.

Primary docs:

- `docs/shadcn-declarative-progress.md`
- `docs/shadcn-conformance-matrix.md`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/`

Active queue:

- `docs/workstreams/shadcn-component-parity-matrix-v1/`
- `docs/workstreams/shadcn-parity-discovery-harness-v1/`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/`

Shared scopes: `ecosystem/fret-ui-shadcn`, `apps/fret-ui-gallery`, `tools/parity-discovery`, and
`tools/diag-scripts/ui-gallery`.

### `editor-canvas`

Scope: code-editor public API, row/fragment replay, canvas paint replay, and editor gallery proof
surfaces.

Primary docs:

- `docs/code-editor.md`
- `docs/workstreams/code-editor-public-api-and-architecture-v1/`
- `docs/workstreams/editor-canvas-paint-replay-canvas-exclusive-v1/`

Active queue:

- `docs/workstreams/code-editor-public-api-and-architecture-v1/`
- `docs/workstreams/code-editor-row-fragment-replay-contract-v1/`
- `docs/workstreams/editor-canvas-paint-replay-canvas-exclusive-v1/`

Shared scopes: code-editor gallery surfaces, canvas paint/replay paths, and editor diagnostics.

### `diagnostics-performance`

Scope: diagnostics infrastructure, performance attribution, profiling, scroll optimization, and
fretboard public diagnostics.

Primary docs:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/debugging-ui-with-inspector-and-scripts.md`
- `docs/perf/`

Active queue:

- `docs/workstreams/diag-fearless-refactor-v2/`
- `docs/workstreams/diag-perf-attribution-v1/`
- `docs/workstreams/diag-perf-profiling-infra-v1/`
- `docs/workstreams/fretboard-public-diag-implementation-v1/`
- `docs/workstreams/scroll-optimization-v1/`
- `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1/`
- `docs/workstreams/ui-perf-zed-smoothness-v1/`

Shared scopes: `apps/fretboard`, diagnostics scripts, `target/fret-diag` evidence conventions, and
performance instrumentation.

### `framework-boundaries`

Scope: public surfaces, root launch/facade boundaries, crate/package split, retained bridge exit,
mechanism harnesses, and cross-ecosystem architecture convergence.

Primary docs:

- `docs/architecture.md`
- `docs/golden-architecture.md`
- `docs/repo-structure.md`
- `docs/dependency-policy.md`

Active queue:

- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/`
- `docs/workstreams/fearless-architecture-convergence-v1/`
- `docs/workstreams/font-bundle-release-boundary-v1/`
- `docs/workstreams/fret-launch-root-surface-convergence-v1/`
- `docs/workstreams/fret-mechanism-harness-v1/`
- `docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1/`
- `docs/workstreams/jellyflow-package-split-v1/`
- `docs/workstreams/retained-public-surface-exit-v1/`

Shared scopes: workspace `Cargo.toml`, public facade crates, `ecosystem/fret`, `crates/fret-launch`,
and dependency layering policy.

### `fret-node-mechanisms`

Scope: `fret-node` declarative-first surface, node graph controller/binding APIs, and graph-edit
transaction seams.

Primary docs:

- `docs/node-graph-roadmap.md`
- `docs/node-graph-how-to-build-like-xyflow.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`

Active queue:

- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`

Shared scopes: `ecosystem/fret-node`, node graph diagnostics, and graph editor examples.

### `ui-frame-overlay`

Scope: frame pipeline phase contracts, overlay/focus dismissal policy, and command/action
availability publication.

Primary docs:

- `docs/runtime-contract-matrix.md`
- `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`
- `docs/action-hooks.md`

Active queue:

- `docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1/`
- `docs/workstreams/ui-overlay-focus-dismissal-oracle-v1/`
- `docs/workstreams/window-command-availability-snapshot-v2/`

Shared scopes: `crates/fret-ui`, action/command routing, focus/overlay policy, and frame pipeline
diagnostics.

## Planner Notes

Use local `.codex/planner-state.local.json` to record actual worktree paths, branch heads, dirty
state, current task IDs, and related repositories. Commit only examples or lane names, not personal
absolute paths.
