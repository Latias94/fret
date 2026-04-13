# ImUi Workbench Shell Closure v1

Status: active execution lane
Last updated: 2026-04-13

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

This lane exists because the umbrella immediate-mode product-closure workstream already froze the
P1 proof roster and shell diagnostics floor.

The remaining question is now narrower:

> how do we close the default editor workbench shell gap without reopening generic `imui` helper
> growth and without collapsing shell composition back into the older tabstrip parity lanes?

## Context

The current umbrella lane already made three important decisions:

1. `workspace_shell_demo` is the primary P1 coherent workbench-shell proof.
2. `editor_notes_demo` is the minimal secondary proof for shell-mounted rails.
3. `diag-hardening-smoke-workspace` is the promoted launched shell smoke floor.

Those decisions mean the remaining work is not "find another shell proof."
The remaining work is implementation-heavy shell closure around:

- `WorkspaceFrame` composition,
- pane tree and shell slot posture,
- shell command scope and focus restore,
- dirty-close and keep-alive behavior on the default workbench path,
- and the owner split between shell chrome, editor composites, docking, and app-local center
  content.

## Why this is a new lane

This should not be forced back into the umbrella folder because P1 is now implementation-heavy.

It also should not be forced into the existing `workspace-tabstrip*` lanes because those lanes
primarily own tabstrip kernel, overflow, reorder, split-drop, and parity behavior.

This lane is broader than tabstrip behavior but narrower than the umbrella:

- broader than tabstrip because it owns shell composition and proof posture,
- narrower than the umbrella because it owns only P1 shell closure.

## Assumptions-first baseline

### 1) The P1 proof roster is already frozen strongly enough to start implementation work

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would start implementation work without a stable first-open proof order.

### 2) The missing closure is primarily shell composition, not generic immediate vocabulary

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift back into generic helper growth instead of shell closure.

### 3) Existing `workspace-tabstrip*` lanes remain reference lanes, not the owner of this whole problem

- Evidence:
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
  - `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/DESIGN.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - shell closure work would be split across multiple lane types with no single default workbench
    owner.

### 4) Multi-window hand-feel still belongs elsewhere

- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would mix shell composition work with runner/backend parity closure.

## Goals

1. Keep one default workbench shell answer for the immediate-mode product path.
2. Keep the owner split explicit between `fret-workspace`, `fret-ui-editor`, `fret-docking`, and
   app/example composition.
3. Turn the broad P1 product question into reviewable shell slices with one repro, one gate
   package, and one evidence set.
4. Keep shell closure discoverable without reopening the umbrella or tabstrip lanes every time.

## Non-goals

- Reopening generic `fret-imui` or `fret-ui-kit::imui` helper growth.
- Replacing the existing `workspace-tabstrip*` or docking tab-bar parity lanes.
- Solving runner/backend multi-window parity here.
- Widening `crates/fret-ui` to make the shell path feel easier.

## Default owner split

### `ecosystem/fret-workspace`

Owns:

- `WorkspaceFrame` shell slots and pane tree composition,
- shell-level tab strip posture and command scope,
- shell focus restore and dirty-close shell policy,
- the default workbench shell proof posture.

### `ecosystem/fret-ui-editor`

Owns:

- reusable editor rails, inspectors, and property/editor composites,
- editor-specific content mounted inside shell slots,
- editor composites that should remain reusable outside the main shell proof.

### `ecosystem/fret-docking`

Owns:

- dock-space choreography when the shell mounts docking,
- re-dock/move semantics,
- docking-owned shell interaction boundaries.

### app/example composition

Owns:

- center-scene product content,
- app-local selection/domain state,
- product-local panel recipes that are not generic shell policy.

## Execution rules

1. Use the umbrella lane for phase ordering and cross-phase status.
2. Use this lane for implementation-heavy P1 shell closure only.
3. If a gap reduces to tabstrip parity, continue the relevant `workspace-tabstrip*` lane instead of
   widening this folder.
4. If a gap reduces to multi-window hand-feel, continue the docking parity lane instead of
   widening this folder.
5. Every slice in this lane must name:
   - one shell proof surface,
   - one focused gate package,
   - and one evidence set.

## Current first-open proof order

1. `apps/fret-examples/src/workspace_shell_demo.rs`
2. `apps/fret-examples/src/editor_notes_demo.rs`
3. `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
4. `apps/fret-examples/src/imui_editor_proof_demo.rs`
5. `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

## Success condition

This lane succeeds when the repo can point to one narrow, durable answer for:

> what is the default editor workbench shell proof in Fret, which owner closes the remaining
> shell gaps, and which gates keep that answer from regressing?

That does not require a new giant shell framework.
It requires one coherent proof order, one owner split, and implementation slices that stay
reviewable.
