---
title: DevTools GUI Dogfood Workflow
status: draft
date: 2026-03-06
scope: diagnostics, devtools, regression, workflow
---

# DevTools GUI Dogfood Workflow

This note captures one end-to-end dogfood workflow for `apps/fret-devtools`.

The goal is not to define a GUI-only run model. The goal is to prove that DevTools GUI can drive and inspect
the same diagnostics contracts already used by CLI and MCP:

- selector capture from live inspect,
- script choice/editing,
- run output under one artifacts root,
- aggregate summarization,
- summary drill-down,
- evidence packing for handoff.

Related notes:

- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`
- `docs/ui-diagnostics-and-scripted-tests.md`

## Scope and boundary

This workflow intentionally treats the GUI as a thin diagnostics consumer.

It should reuse:

- the same artifacts root as CLI/tooling,
- the same `regression.summary.json` and `regression.index.json` contracts,
- the same pack outputs under `.fret/diag/packs/`,
- the same script library and `test_id`-driven selectors.

It should not introduce:

- a GUI-only campaign store,
- a GUI-only summary schema,
- a second interpretation of what a regression run means.

## Pre-conditions

Recommended setup:

1. start one target app or demo with diagnostics enabled,
2. keep one explicit diagnostics artifacts root for the session,
3. run `apps/fret-devtools` against that same target/session,
4. prefer one active user/agent per artifacts root to avoid transport races.

For native or local workflows, the practical invariant is simple: the GUI must read and write against the same
artifacts root that the run/summarize/pack flow will use.

## End-to-end workflow

### 1. Pick selector from live UI

Use DevTools GUI to:

- enable inspect,
- arm pick,
- click the target UI element,
- inspect the selector JSON that comes back.

Success criteria:

- the chosen selector is stable and semantics-oriented,
- `test_id` is preferred over brittle geometry assumptions,
- the captured selector is ready to be pasted or applied into a script step.

### 2. Patch or choose script

Use Script Studio to either:

- choose an existing script from the workspace library, or
- fork/patch a script and apply the captured selector into the intended step.

Success criteria:

- the script stays in the existing diagnostics script model,
- the GUI edits JSON/script content but does not define a separate script representation,
- the resulting script is something CLI automation could run as well.

### 3. Run against the shared artifacts root

Run the script from DevTools GUI.

Expected outcome:

- the target session executes the script,
- run outputs land under the shared diagnostics artifacts root,
- failures and bundles remain consumable by the same tooling paths used outside the GUI.

This is the key alignment check: GUI-triggered work must still leave normal diagnostics artifacts behind.

### 4. Summarize aggregate results

Open the `Regression` tab and use `Summarize`.

Expected outcome:

- the GUI invokes the existing summarize flow against the current artifacts root,
- `regression.summary.json` and `regression.index.json` are refreshed,
- the GUI then reloads those aggregate artifacts as a reader.

This step proves that aggregation semantics stay outside the GUI.

### 5. Inspect aggregate and selected summary details

Use the `Regression` tab to:

- inspect the aggregate dashboard/index content,
- browse `failing_summaries`,
- select one failing summary row,
- review its `regression.summary.json` drill-down,
- copy the selected path or bundle dirs when needed.

Success criteria:

- the drill-down is sourced from the existing summary file on disk,
- evidence paths remain explicit and portable,
- a human can move from aggregate status to one concrete failing evidence payload quickly.

### 6. Pack and share selected evidence

If one selected summary has a failing bundle dir, use:

- `Copy first bundle dir` for quick handoff, or
- `Pack selected evidence` to create a shareable zip from the first failing bundle dir.

Expected outcome:

- packing reuses the existing diagnostics pack flow,
- the resulting zip lands under `.fret/diag/packs/`,
- the same artifact can be handed to another maintainer, MCP consumer, or offline viewer workflow.

## Why this workflow matters

This workflow is a dogfood proof for three architectural claims:

1. DevTools GUI can be productive without becoming the architecture center.
2. Aggregate regression contracts are reusable across GUI, CLI, and MCP.
3. Evidence handoff remains artifact-based instead of depending on live GUI state.

## What is still intentionally missing

This workflow is intentionally thin and does not yet imply:

- a full GUI campaign planner,
- matrix/perf/suite orchestration UX,
- a dedicated artifact browser for every file under the diagnostics root,
- platform-specific "open folder" integration.

Those may come later, but they should still be built on the same artifact and command seams.

## Maintainer checklist

If this workflow stops feeling smooth, check the failure against these questions first:

1. Did GUI start inventing a parallel diagnostics state model?
2. Did a GUI action stop writing normal diagnostics artifacts?
3. Did a selector/script path become GUI-only instead of CLI-compatible?
4. Did pack/summarize behavior drift from the shared tooling contract?

If the answer to any of those is "yes", fix the seam before adding more GUI polish.
