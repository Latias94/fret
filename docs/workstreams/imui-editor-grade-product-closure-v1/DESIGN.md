# ImUi Editor-Grade Product Closure v1

Status: active execution lane
Last updated: 2026-04-12

Related:

- `M0_BASELINE_AUDIT_2026-04-12.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_DEMOTE_DELETE_PLAN_2026-04-12.md`
- `P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `P0_ROOT_HOSTING_RULE_2026-04-12.md`
- `P0_STABLE_IDENTITY_RULE_2026-04-12.md`
- `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/diagnostics-first-open.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

This follow-on owns one narrow question:

> after the in-tree `imui` stack reset and helper-closure lanes are closed, how do we close the
> remaining editor-grade maturity gap against Dear ImGui without undoing Fret's
> mechanism-vs-policy layering?

## Context

The earlier `imui` lanes already answered the ownership questions that used to justify broad stack
work:

- `fret-authoring` owns the minimal shared authoring contract,
- `fret-imui` stays a minimal frontend,
- `fret-ui-kit::imui` owns generic immediate vocabulary,
- `fret-ui-editor::imui` owns thin editor composites and editor-specific nouns,
- and `crates/fret-ui` stays the runtime substrate rather than a policy/component layer.

That means the remaining gap is no longer "missing immediate primitives" or "wrong stack
ownership." The remaining gap is product closure:

- one low-friction default authoring lane,
- one coherent editor workbench shell,
- one discoverable diagnostics/devtools loop,
- and one runner-grade multi-window hand-feel story.

## Goals

1. Keep the closed `imui` stack and helper lanes closed unless fresh evidence exceeds their
   closeout records.
2. Turn the remaining maturity gap into a staged execution plan with explicit owners.
3. Keep each phase tied to a real proof surface and a bounded gate package.
4. Preserve the runtime/policy split while making the golden path feel like one coherent system.

## Non-goals

- Reopening generic helper growth in `fret-imui` or `fret-ui-kit::imui` by default.
- Moving editor/shell/dialog/tooltip/dismiss policy into `crates/fret-ui`.
- Reproducing Dear ImGui API or flag vocabulary one-for-one.
- Treating this lane as a permanent umbrella backlog.
  Once a phase becomes implementation-heavy, start a narrower follow-on instead of expanding this
  directory indefinitely.

## Problem statement

The current repo already contains the right ingredients, but they are not yet experienced as a
single default product path.

### A) Authoring lane friction

Fret has a healthy layered stack, but the author still sees multiple entry surfaces and still needs
to think explicitly about layout/identity/plumbing more often than Dear ImGui users do on the
golden path.

The problem to solve is not "remove layering." It is "make the layered golden path obvious,
teachable, and low-footgun."

### B) Editor shell incompleteness

The repo has real shell pieces across workspace, docking, editor composites, and proof demos, but
not yet a single default editor workbench story that makes those surfaces feel pre-composed.

### C) Diagnostics/devtools fragmentation

The diagnostics foundation is already strong: bundles, scripts, inspect/pick, sidecars, compare,
and DevTools experiments all exist. What is missing is a single first-open developer loop that ties
those pieces together.

### D) Multi-window hand-feel closure

The repo already tracks docking tear-off and multi-window parity, but this remains a runner/backend
closure problem rather than an `imui` API problem. We need the default owner split to stay clear so
future work does not leak platform heuristics back into the runtime or immediate facade.

## Owner split by phase

### P0 - Default authoring lane closure

Primary owners:

- `ecosystem/fret-authoring`
- `ecosystem/fret-imui`
- `ecosystem/fret-ui-kit::imui`
- `ecosystem/fret-ui-editor::imui`
- first-party proof/demo surfaces in `apps/fret-examples`

Expected outcome:

- one taught default path for immediate-style authors,
- explicit stable-identity and layout guidance,
- and a delete/demote plan for first-party teaching surfaces that still imply the wrong layer.

### P1 - Editor workbench shell closure

Primary owners:

- `ecosystem/fret-workspace`
- `ecosystem/fret-docking`
- `ecosystem/fret-ui-editor`
- selected proof shells in `apps/fret-examples`

Expected outcome:

- one coherent default workbench proof,
- explicit owner boundaries between shell, docking, and editor composites,
- and a reviewable shell primitive matrix instead of scattered examples.

### P2 - Unified diagnostics/devtools surface

Primary owners:

- `crates/fret-diag-protocol`
- `crates/fret-diag`
- `ecosystem/fret-bootstrap`
- `apps/fret-devtools`
- `apps/fret-devtools-mcp`

Expected outcome:

- one first-open developer workflow for inspect -> selector -> script -> bundle -> compare,
- shared contract surfaces across GUI, CLI, and MCP,
- and a bounded devtools smoke/gate package.

### P3 - Multi-window hand-feel closure

Primary owners:

- `crates/fret-launch`
- runner/backend integrations
- `ecosystem/fret-docking`

Expected outcome:

- hovered-window, peek-behind, transparent payload, and mixed-DPI drag/follow behavior remain
  clearly runner-owned,
- and the parity matrix/gates stop this from regressing into `imui` helper growth.

## Execution rules

1. Do not widen `crates/fret-ui` just to make the immediate path feel easier.
   If the missing behavior is dialog/menu/tooltip/editor policy, keep it above the runtime layer.
2. Do not reopen the closed helper-growth lanes unless new evidence names all of:
   - the missing noun/helper shape,
   - at least two real first-party proof surfaces that cannot reasonably stay explicit,
   - and one bounded gate/evidence package.
   The current minimum proof budget is the frozen golden pair:
   - `apps/fret-cookbook/examples/imui_action_basics.rs`
   - `apps/fret-examples/src/imui_editor_proof_demo.rs`
   Reference, advanced, and compatibility-only surfaces may strengthen the case, but they do not
   justify widening on their own.
3. Prefer thin but strong top-level facades over a new monolithic context.
   The goal is "feels unified," not "implemented as one giant owner."
4. Every phase must name:
   - one proof surface,
   - one gate package,
   - and one evidence set
   before broadening public guidance.

## Current proof surfaces

- Immediate generic/default proof:
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
- Immediate editor-grade proof:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Supporting immediate implementation evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
- Editor shell proof:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Supporting shell/docking evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- Diagnostics/devtools proof:
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `docs/workstreams/diag-fearless-refactor-v2/README.md`
  - `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
  - `apps/fret-devtools/src/main.rs`
- Multi-window parity proof:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`

## Success condition

This lane succeeds when the repo can teach one coherent answer to the following question:

> "How do I build and debug an editor-grade Fret UI that feels as integrated as a Dear ImGui tool,
> without collapsing policy back into the runtime?"

That does not require one giant crate or one giant API. It requires:

- a clear golden path,
- explicit owner splits,
- bounded proof surfaces,
- and gates that make the path durable.
